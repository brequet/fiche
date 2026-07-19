use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::llm::LlmError;

const GROQ_API_URL: &str = "https://api.groq.com/openai/v1/chat/completions";
const GROQ_MODEL: &str = "openai/gpt-oss-120b";

#[derive(Debug, Serialize)]
struct GroqCompletionsRequest {
    messages: Vec<GroqMessage>,
    model: String,
    stream: bool,
    response_format: GroqRequestResponseFormat,
}

#[derive(Debug, Serialize, Deserialize)]
struct GroqMessage {
    role: GroqRole,
    content: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum GroqRole {
    System,
    User,
    Assistant,
}

const GROQ_RESPONSE_FORMAT_JSON_SCHEMA: &str = "json_schema";

#[derive(Debug, Serialize)]
struct GroqRequestResponseFormat {
    #[serde(rename = "type")]
    response_type: String, // always "json_schema"
    json_schema: GroqRequestResponseFormatJsonSchema,
}

#[derive(Debug, Serialize)]
struct GroqRequestResponseFormatJsonSchema {
    name: String,
    schema: Option<serde_json::Value>,
    strict: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct GroqCompletionsResponse {
    choices: Vec<GroqCompletionsChoice>,
}

#[derive(Debug, Deserialize)]
struct GroqCompletionsChoice {
    message: GroqMessage,
}

#[derive(Debug)]
pub struct GroqClient {
    api_key: String,
    http_client: Client,
}

impl GroqClient {
    pub fn new(api_key: String, http_client: Client) -> Self {
        Self {
            api_key,
            http_client,
        }
    }

    async fn send_completion_request(
        &self,
        request: GroqCompletionsRequest,
    ) -> Result<String, LlmError> {
        let response = self
            .http_client
            .post(GROQ_API_URL)
            .bearer_auth(&self.api_key)
            .json(&request)
            .send()
            .await
            .map_err(|err| LlmError::HttpRequestError(err.to_string()))?;

        if !response.status().is_success() {
            let err_body = response.text().await.unwrap_or_default();
            return Err(LlmError::ApiError(err_body));
        }

        let groq_res: GroqCompletionsResponse = response
            .json()
            .await
            .map_err(|err| LlmError::ResponseParseError(err.to_string()))?;

        let content = groq_res
            .choices
            .first()
            .and_then(|choice| choice.message.content.clone())
            .ok_or_else(|| LlmError::ApiError("No response content returned".to_string()))?;

        Ok(content)
    }
}

impl super::LlmClient for GroqClient {
    async fn generate_article_summary(
        &self,
        article_page_content: &str,
    ) -> Result<super::ArticleSummary, super::LlmError> {
        let schema = json!({
            "type": "object",
            "properties": {
                "summary": { "type": "string" },
                "tags": {
                    "type": "array",
                    "items": { "type": "string" }
                }
            },
            "required": ["summary", "tags"],
            "additionalProperties": false
        });

        let request = GroqCompletionsRequest {
            model: GROQ_MODEL.to_string(),
            stream: false,
            messages: vec![
                GroqMessage {
                    role: GroqRole::System,
                    content: Some(
                        "You are a helpful assistant that summarizes articles into structured JSON. Be pragmatic, brief and concise. Add keypoints if appropriated. Between 1 and 3 tags max. No spaces in tags.".into(),
                    ),
                },
                GroqMessage {
                    role: GroqRole::User,
                    content: Some(article_page_content.into()),
                },
            ],
            response_format: GroqRequestResponseFormat {
                response_type: GROQ_RESPONSE_FORMAT_JSON_SCHEMA.to_string(),
                json_schema: GroqRequestResponseFormatJsonSchema {
                    name: "article_summary".to_string(),
                    schema: Some(schema),
                    strict: Some(true),
                },
            },
        };

        let raw_json = self.send_completion_request(request).await?;
        serde_json::from_str(&raw_json).map_err(|err| LlmError::ResponseParseError(err.to_string()))
    }

    async fn generate_tool_summary(
        &self,
        tool_page_content: &str,
    ) -> Result<super::ToolSummary, super::LlmError> {
        let schema = json!({
            "type": "object",
            "properties": {
                "summary": { "type": "string" },
            },
            "required": ["summary"],
            "additionalProperties": false
        });

        let request = GroqCompletionsRequest {
            model: GROQ_MODEL.to_string(),
            stream: false,
            messages: vec![
                GroqMessage {
                    role: GroqRole::System,
                    content: Some(
                        "You are a helpful assistant that summarizes tool presentation page into structured JSON. Be pragmatic, brief and concise. Add keypoints if appropriated.".into(),
                    ),
                },
                GroqMessage {
                    role: GroqRole::User,
                    content: Some(tool_page_content.into()),
                },
            ],
            response_format: GroqRequestResponseFormat {
                response_type: GROQ_RESPONSE_FORMAT_JSON_SCHEMA.to_string(),
                json_schema: GroqRequestResponseFormatJsonSchema {
                    name: "article_summary".to_string(),
                    schema: Some(schema),
                    strict: Some(true),
                },
            },
        };

        let raw_json = self.send_completion_request(request).await?;
        serde_json::from_str(&raw_json).map_err(|err| LlmError::ResponseParseError(err.to_string()))
    }
}
