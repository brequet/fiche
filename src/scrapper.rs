use html_to_markdown_rs::HtmlMetadata;
use reqwest::Client;
use tracing::warn;

#[derive(Debug, thiserror::Error)]
pub enum ScrapError {
    #[error("Failed to fetch the URL: {0}")]
    FetchError(#[from] reqwest::Error),

    #[error("Failed to parse the HTML content: {0}")]
    ParseError(String),

    #[error("Failed to convert HTML to Markdown: {0}")]
    ConversionError(#[from] html_to_markdown_rs::ConversionError),
}

pub struct ScrapResult {
    pub title: Option<String>,
    pub content: String,
    pub url: String,
}

#[derive(Debug)]
pub struct Scrapper {
    http_client: Client,
}

impl Scrapper {
    pub fn new(http_client: reqwest::Client) -> Self {
        Scrapper { http_client }
    }

    pub async fn scrap(&self, url: &str) -> Result<ScrapResult, ScrapError> {
        let raw_html = self
            .http_client
            .get(url)
            .send()
            .await
            .map_err(ScrapError::FetchError)?
            .text()
            .await
            .map_err(|e| ScrapError::ParseError(e.to_string()))?;

        // TODO: Consider extracting the <article> tag content if present, or the main content of the page, instead of converting the entire HTML to Markdown.

        let md_conversion = html_to_markdown_rs::convert(raw_html.as_str(), None)?;

        if md_conversion.warnings.len() > 0 {
            warn!(
                "HTML to Markdown conversion generated {} warnings",
                md_conversion.warnings.len()
            );
            md_conversion.warnings.iter().for_each(|w| {
                warn!("[{:#?}] {}", w.kind, w.message);
            });
        }

        let content = md_conversion.content.ok_or_else(|| {
            ScrapError::ParseError("Failed to extract content from HTML".to_string())
        })?;

        Ok(ScrapResult {
            title: extract_title(md_conversion.metadata),
            content,
            url: url.to_string(),
        })
    }
}

fn extract_title(html_metadata: HtmlMetadata) -> Option<String> {
    html_metadata.document.title.or(html_metadata
        .headers
        .iter()
        .find(|header| header.level == 1)
        .map(|header| header.text.clone()))
}
