#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Configuration error: {0}")]
    Config(#[from] crate::config::ConfigError),

    #[error("Failed to initialize application context: {0}")]
    InitializationError(String),

    #[error("Scraping error: {0}")]
    Scrap(#[from] crate::scrapper::ScrapError),

    #[error("LLM error: {0}")]
    Llm(#[from] crate::llm::LlmError),

    #[error("Vault error: {0}")]
    Vault(#[from] crate::vault::VaultError),
}
