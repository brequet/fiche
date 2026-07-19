use tracing::info;

use crate::{context::AppContext, error::AppError, llm::LlmClient};

pub async fn tool_clip(url: &str, ctx: AppContext) -> Result<(), AppError> {
    info!("Scrapping tool from URL: {}", url);
    let tool = ctx.scrapper.scrap(url).await?;

    // TODO: leaner, more focus content, often overflowing (ex github readme, etc)
    let summary = ctx.llm_client.generate_tool_summary(&tool.content).await?;

    let article_path = ctx
        .vault
        .write_tool_report(url, tool.title, &summary.summary)?;

    info!("Tool report written to: {}", article_path.display());

    Ok(())
}
