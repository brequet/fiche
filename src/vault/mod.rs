use std::path::{Path, PathBuf};

use chrono::Local;
use minijinja::Environment;
use serde::Serialize;
use std::io::Write;

const ARTICLES_REPORT_PATH: &str = "10_Personal/tech-radar/reports";
const TOOLS_REPORT_PATH: &str = "10_Personal/tech-radar/tools";

const ARTICLE_TEMPLATE: &str = r#"---
type: radar/report
url: {{ url }}
date_created: {{ date }}
tags:
{%- for tag in tags %}
  - {{ tag }}
{%- endfor %}
read: {{ read }}
---
# {{ title }}

{{ summary }}
"#;

const TOOL_TEMPLATE: &str = r#"---
type: radar/tool
url:  {{ url }}
date_created: {{ date }}
---
# {{ title }}

{{ summary }}
"#;

#[derive(Debug, Clone, Copy)]
pub enum TemplateKind {
    Article,
    Tool,
}

impl TemplateKind {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Article => "article",
            Self::Tool => "tool",
        }
    }
}

trait ReportPayload: Serialize {
    fn template_kind(&self) -> TemplateKind;
    fn dir_path(&self) -> &str;
    fn title(&self) -> &str;
}

#[derive(Serialize)]
struct ArticleReportContext {
    pub url: String,
    pub title: String,
    pub summary: String,
    pub date: String,
    pub tags: Vec<String>,
    pub read: bool,
}

impl ReportPayload for ArticleReportContext {
    fn template_kind(&self) -> TemplateKind {
        TemplateKind::Article
    }

    fn dir_path(&self) -> &str {
        ARTICLES_REPORT_PATH
    }

    fn title(&self) -> &str {
        &self.title
    }
}

#[derive(Serialize)]
struct ToolReportContext {
    pub url: String,
    pub title: String,
    pub summary: String,
    pub date: String,
}

impl ReportPayload for ToolReportContext {
    fn template_kind(&self) -> TemplateKind {
        TemplateKind::Tool
    }

    fn dir_path(&self) -> &str {
        TOOLS_REPORT_PATH
    }

    fn title(&self) -> &str {
        &self.title
    }
}

#[derive(Debug, thiserror::Error)]
pub enum VaultError {
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Template rendering error: {0}")]
    TemplateError(#[from] minijinja::Error),
}

#[derive(Debug)]
pub struct Vault {
    path: PathBuf,
    templates_env: Environment<'static>,
}

impl Vault {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, VaultError> {
        let mut templates_env = Environment::new();
        templates_env.add_template(TemplateKind::Article.as_str(), ARTICLE_TEMPLATE)?;
        templates_env.add_template(TemplateKind::Tool.as_str(), TOOL_TEMPLATE)?;

        Ok(Self {
            path: path.as_ref().to_path_buf(),
            templates_env,
        })
    }

    fn write_report<T: ReportPayload>(&self, payload: &T) -> Result<PathBuf, VaultError> {
        let report_dir = self.path.join(payload.dir_path());
        std::fs::create_dir_all(&report_dir)?;

        let file_path = report_dir.join(format!("{}.md", payload.title()));
        if file_path.exists() {
            return Err(VaultError::IoError(std::io::Error::new(
                std::io::ErrorKind::AlreadyExists,
                format!("File already exists: {}", file_path.display()),
            )));
        }

        let template = self
            .templates_env
            .get_template(payload.template_kind().as_str())?;
        let rendered = template.render(payload)?;

        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&file_path)?;
        file.write_all(rendered.as_bytes())?;

        Ok(file_path)
    }

    pub fn write_article_report(
        &self,
        url: &str,
        title: Option<String>,
        summary: &str,
        tags: &[String],
    ) -> Result<PathBuf, VaultError> {
        let ctx = ArticleReportContext {
            url: url.to_string(),
            title: build_file_name(title),
            summary: summary.to_string(),
            tags: tags.to_vec(),
            date: today(),
            read: false,
        };
        self.write_report(&ctx)
    }

    pub fn write_tool_report(
        &self,
        url: &str,
        title: Option<String>,
        summary: &str,
    ) -> Result<PathBuf, VaultError> {
        let ctx = ToolReportContext {
            url: url.to_string(),
            title: build_file_name(title),
            summary: summary.to_string(),
            date: today(),
        };
        self.write_report(&ctx)
    }
}

fn build_file_name(parsed_name: Option<String>) -> String {
    match parsed_name {
        Some(name) => sanitize_file_name(&name),
        None => "untitled_article".to_string(),
    }
}

fn sanitize_file_name(name: &str) -> String {
    let mut sanitized: String = name
        .chars()
        .map(|c| match c {
            '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*' => '-',
            c if (c as u32) < 0x20 => '_',
            c => c,
        })
        .collect();

    sanitized = sanitized.trim_end_matches(['.', ' ']).to_string();

    if sanitized.is_empty() {
        sanitized = "untitled".to_string();
    }

    let upper = sanitized.to_uppercase();
    let reserved = [
        "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8",
        "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
    ];
    let base = upper.split('.').next().unwrap_or("");
    if reserved.contains(&base) {
        sanitized = format!("_{sanitized}");
    }

    if sanitized.len() > 255 {
        let mut idx = 255;
        while !sanitized.is_char_boundary(idx) {
            idx -= 1;
        }
        sanitized.truncate(idx);
    }

    sanitized
}

fn today() -> String {
    Local::now().format("%Y-%m-%d").to_string()
}
