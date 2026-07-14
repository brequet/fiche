use tracing::info;

use crate::{context::AppContext, error::AppError, llm::LlmClient};

pub async fn article_clip(url: &str, ctx: AppContext) -> Result<(), AppError> {
    info!("Scrapping article from URL: {}", url);
    let article = ctx.scrapper.scrap(url).await?;

    let summary = ctx
        .llm_client
        .generate_article_summary(&article.content)
        .await?;

    let article_path =
        ctx.vault
            .write_article_report(url, article.title, &summary.summary, &summary.tags)?;

    info!("Article report written to: {}", article_path.display());

    Ok(())
}
