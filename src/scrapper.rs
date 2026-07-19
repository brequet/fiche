use html_to_markdown_rs::HtmlMetadata;
use reqwest::{Client, Url};
use tracing::warn;

#[derive(Debug, thiserror::Error)]
pub enum ScrapError {
    #[error("Failed to fetch the URL: {0}")]
    Fetch(#[from] reqwest::Error),

    #[error("Failed to parse the HTML content: {0}")]
    Parse(String),

    #[error("Failed to convert HTML to Markdown: {0}")]
    Conversion(#[from] html_to_markdown_rs::ConversionError),
}

pub struct ScrapResult {
    pub title: Option<String>,
    pub content: String,
}

enum PageType {
    GitHub { user: String, repository: String },
    GenericWeb,
}

impl PageType {
    fn from_url(url_str: &str) -> Self {
        if let Ok(url) = Url::parse(url_str)
            && let Some(host) = url.host_str()
            && (host == "github.com" || host == "www.github.com")
        {
            let segments: Vec<&str> = url.path_segments().map(|c| c.collect()).unwrap_or_default();

            // Paths like /user/repo
            if segments.len() >= 2 {
                return PageType::GitHub {
                    user: segments[0].to_string(),
                    repository: segments[1].to_string(),
                };
            }
        }
        PageType::GenericWeb
    }
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
        match PageType::from_url(url) {
            PageType::GitHub { user, repository } => self.scrap_github(&user, &repository).await,
            PageType::GenericWeb => self.scrap_generic(url).await,
        }
    }

    async fn scrap_github(&self, user: &str, repo: &str) -> Result<ScrapResult, ScrapError> {
        let variants = ["README.md", "readme.md"];

        for filename in variants.iter() {
            let raw_url = format!(
                "https://raw.githubusercontent.com/{}/{}/HEAD/{}",
                user, repo, filename
            );

            let response = self.http_client.get(&raw_url).send().await;

            if let Ok(res) = response
                && res.status().is_success()
            {
                let content = res.text().await.map_err(ScrapError::Fetch)?;
                return Ok(ScrapResult {
                    title: Some(format!("{} ({})", repo, user)),
                    content,
                });
            }
        }

        // Fallback to generic scraping if raw assets aren't located safely
        let fallback_url = format!("https://github.com/{}/{}", user, repo);
        self.scrap_generic(&fallback_url).await
    }

    async fn scrap_generic(&self, url: &str) -> Result<ScrapResult, ScrapError> {
        let raw_html = self
            .http_client
            .get(url)
            .send()
            .await
            .map_err(ScrapError::Fetch)?
            .text()
            .await
            .map_err(|e| ScrapError::Parse(e.to_string()))?;

        let html_to_convert = extract_main_content(&raw_html).unwrap_or(raw_html);

        let md_conversion = html_to_markdown_rs::convert(html_to_convert.as_str(), None)?;

        if !md_conversion.warnings.is_empty() {
            warn!(
                "HTML to Markdown conversion generated {} warnings",
                md_conversion.warnings.len()
            );
            md_conversion.warnings.iter().for_each(|w| {
                warn!("[{:#?}] {}", w.kind, w.message);
            });
        }

        let content = md_conversion
            .content
            .ok_or_else(|| ScrapError::Parse("Failed to extract content from HTML".to_string()))?;

        Ok(ScrapResult {
            title: extract_title(md_conversion.metadata),
            content: truncate_to_token_budget(content),
        })
    }
}

fn extract_main_content(raw_html: &str) -> Option<String> {
    let dom = tl::parse(raw_html, tl::ParserOptions::default()).ok()?;
    let parser = dom.parser();

    // Strategy 1: Look for standard semantic <article> tags first
    if let Some(article_handle) = dom
        .query_selector("article")
        .and_then(|mut iter| iter.next())
    {
        if let Some(node) = article_handle.get(parser) {
            return Some(node.inner_html(parser).into_owned());
        }
    }

    // Strategy 2: Look for generic <body> fallback to ditch raw script/head tags
    if let Some(body_handle) = dom.query_selector("body").and_then(|mut iter| iter.next()) {
        if let Some(node) = body_handle.get(parser) {
            return Some(node.inner_html(parser).into_owned());
        }
    }

    None
}

fn extract_title(html_metadata: HtmlMetadata) -> Option<String> {
    html_metadata.document.title.or(html_metadata
        .headers
        .iter()
        .find(|header| header.level == 1)
        .map(|header| header.text.clone()))
}

fn truncate_to_token_budget(mut content: String) -> String {
    const MAX_CHARS: usize = 20_000;

    if content.len() > MAX_CHARS {
        let cut_index = content
            .char_indices()
            .map(|(idx, _)| idx)
            .find(|&idx| idx >= MAX_CHARS)
            .unwrap_or(content.len());

        content.truncate(cut_index);
        content.push_str("\n\n[... Content truncated due to context size limits ...]");
    }
    content
}
