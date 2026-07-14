#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Failed to load .env file: {0}")]
    DotEnv(#[from] dotenvy::Error),

    #[error("Environment variable '{0}' is required but not set")]
    MissingVar(String),
}

#[derive(Debug, Clone)]
pub struct Config {
    pub vault_path: String,
    pub groq_api_key: String,
}

impl Config {
    pub fn load() -> Result<Self, ConfigError> {
        dotenvy::dotenv()?;

        let vault_path = Self::load_mandatory_env_var("FICHE_VAULT_PATH")?;
        let groq_api_key = Self::load_mandatory_env_var("GROQ_API_KEY")?;

        Ok(Config {
            vault_path,
            groq_api_key,
        })
    }

    fn load_mandatory_env_var(var_name: &str) -> Result<String, ConfigError> {
        std::env::var(var_name).map_err(|_| ConfigError::MissingVar(var_name.to_string()))
    }
}
