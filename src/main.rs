use std::io;

use reqwest::Client;
use tracing::{debug, error, level_filters::LevelFilter};

use crate::{cli::Args, error::AppError};

mod cli;
mod commands;
mod config;
mod context;
mod error;
mod llm;
mod scrapper;
mod vault;

#[tokio::main]
async fn main() {
    let args = Args::parse_args();

    init_tracing(args.verbose);

    if let Err(e) = run(args).await {
        error!("{}", e);
        std::process::exit(1);
    }
}

async fn run(args: Args) -> Result<(), AppError> {
    let config = config::Config::load()?;

    let http_client = Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .user_agent("fiche-cli/1.0")
        .build()
        .map_err(|e| AppError::InitializationError(e.to_string()))?;

    let scrapper = scrapper::Scrapper::new(http_client.clone());
    let groq_client = llm::groq::GroqClient::new(config.groq_api_key.clone(), http_client);
    let vault = vault::Vault::new(config.vault_path);

    let ctx = context::AppContext::new(scrapper, groq_client, vault);

    match args.command {
        cli::Commands::Article(cmd) => commands::article::article_clip(&cmd.url, ctx).await,
        cli::Commands::Tool(cmd) => commands::tool::tool_clip(&cmd.url, ctx).await,
    }
}

fn init_tracing(verbose: bool) {
    let log_level = if verbose {
        LevelFilter::DEBUG
    } else {
        LevelFilter::INFO
    };

    tracing_subscriber::fmt()
        .with_target(false)
        .without_time()
        .with_writer(io::stderr)
        .with_max_level(log_level)
        .init();

    debug!("Verbose logging enabled.");
}
