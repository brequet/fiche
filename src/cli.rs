use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Enable verbose logging.
    #[arg(short, long, global = true, default_value_t = false)]
    pub verbose: bool,

    #[command(subcommand)]
    pub command: Commands,
}

impl Args {
    pub fn parse_args() -> Self {
        Parser::parse()
    }
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    Article(ArticleClipCommand),
    Tool(ToolClipCommand),
}

#[derive(Parser, Debug)]
pub struct ArticleClipCommand {
    #[arg(required = true, help = "The URL")]
    pub url: String,
}

#[derive(Parser, Debug)]
pub struct ToolClipCommand {
    #[arg(required = true, help = "The URL")]
    pub url: String,
}
