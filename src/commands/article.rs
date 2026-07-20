use tracing::info;

use crate::{context::AppContext, error::AppError, llm::LlmClient};

pub async fn article_clip(url: &str, read: bool, ctx: AppContext) -> Result<(), AppError> {
    info!("Scrapping article from URL: {}", url);
    let article = ctx.scrapper.scrap(url).await?;

    // TODO: summary could be optional
    let summary = ctx
        .llm_client
        .generate_article_summary(&article.content)
        .await?;

    let article_path = ctx.vault.write_article_report(
        url,
        article.title,
        &summary.summary,
        &summary.tags,
        read,
    )?;

    // TODO: Auto open in obsidian "obsidian vault=vault open file='10_Personal/tech-radar/reports/The Kimi K3 Moment _ Stephen Bochinski.md' newtab"

    info!("Article report written to: {}", article_path.display());

    Ok(())
}
