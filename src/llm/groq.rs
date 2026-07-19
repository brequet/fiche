use reqwest::Client;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::json;

use crate::llm::LlmError;

const GROQ_API_URL: &str = "https://api.groq.com/openai/v1/chat/completions";
const GROQ_MODEL: &str = "openai/gpt-oss-120b";

#[derive(Debug, Serialize)]
struct GroqCompletionsRequest {
    model: String,
    stream: bool,
    messages: Vec<GroqMessage>,
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

    async fn generate_structured<T: DeserializeOwned>(
        &self,
        system_prompt: &str,
        user_content: &str,
        schema_name: &str,
        schema: serde_json::Value,
    ) -> Result<T, LlmError> {
        let request = GroqCompletionsRequest {
            model: GROQ_MODEL.to_string(),
            stream: false,
            messages: vec![
                GroqMessage {
                    role: GroqRole::System,
                    content: Some(system_prompt.to_string()),
                },
                GroqMessage {
                    role: GroqRole::User,
                    content: Some(user_content.to_string()),
                },
            ],
            response_format: GroqRequestResponseFormat {
                response_type: GROQ_RESPONSE_FORMAT_JSON_SCHEMA.to_string(),
                json_schema: GroqRequestResponseFormatJsonSchema {
                    name: schema_name.to_string(),
                    schema: Some(schema),
                    strict: Some(true),
                },
            },
        };

        let response = self
            .http_client
            .post(GROQ_API_URL)
            .bearer_auth(&self.api_key)
            .json(&request)
            .send()
            .await
            .map_err(|err| LlmError::HttpRequest(err.to_string()))?;

        if !response.status().is_success() {
            let err_body = response.text().await.unwrap_or_default();
            return Err(LlmError::Api(err_body));
        }

        let body: GroqCompletionsResponse = response
            .json()
            .await
            .map_err(|err| LlmError::ResponseParse(err.to_string()))?;

        let content = body
            .choices
            .into_iter()
            .next()
            .and_then(|choice| choice.message.content)
            .ok_or_else(|| LlmError::Api("No response content returned".to_string()))?;

        serde_json::from_str(&content).map_err(|err| LlmError::ResponseParse(err.to_string()))
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

        self.generate_structured(
            "You are a helpful assistant that summarizes articles into structured JSON. Be pragmatic, brief and concise. Add keypoints if appropriated. Between 1 and 3 tags max. No spaces in tags.",
            article_page_content,
            "article_summary",
            schema,
        )
        .await
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

        self.generate_structured(
            "You are a helpful assistant that summarizes tool presentation page into structured JSON. Be pragmatic, brief and concise. Add keypoints if appropriated.",
            tool_page_content,
            "tool_summary",
            schema,
        )
        .await
    }
}
