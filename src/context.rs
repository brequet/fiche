use reqwest::Client;
use std::sync::Arc;
use thiserror::Error;

use crate::{config::Config, llm::groq::GroqClient, scrapper::Scrapper, vault::Vault};

#[derive(Debug, Clone)]
pub struct AppContext {
    pub scrapper: Arc<Scrapper>,
    pub llm_client: Arc<GroqClient>,
    pub vault: Arc<Vault>,
}

impl AppContext {
    pub fn new(scrapper: Scrapper, llm_client: GroqClient, vault: Vault) -> Self {
        AppContext {
            scrapper: Arc::new(scrapper),
            llm_client: Arc::new(llm_client),
            vault: Arc::new(vault),
        }
    }
}
