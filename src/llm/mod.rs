use serde::{Deserialize, Serialize};

pub mod groq;

#[derive(Debug, thiserror::Error)]
pub enum LlmError {
    #[error("HTTP request failed: {0}")]
    HttpRequest(String),

    #[error("Failed to parse response: {0}")]
    ResponseParse(String),

    #[error("API returned an error: {0}")]
    Api(String),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ArticleSummary {
    pub summary: String,
    pub tags: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ToolSummary {
    pub summary: String,
}

pub trait LlmClient {
    async fn generate_article_summary(
        &self,
        article_page_content: &str,
    ) -> Result<ArticleSummary, LlmError>;

    async fn generate_tool_summary(&self, tool_page_content: &str)
    -> Result<ToolSummary, LlmError>;
}
