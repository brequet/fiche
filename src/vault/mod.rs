use std::path::{Path, PathBuf};

use chrono::Local;
use minijinja::{Environment, context};

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
read: false
---

# {{ title }}

{{ summary }}
"#;

struct ArticleContext {
    url: String,
    title: Option<String>,
    summary: String,
    tags: Vec<String>,
    read: bool,
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
}

impl Vault {
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
        }
    }

    pub fn write_article_report(
        &self,
        url: &str,
        title: Option<String>,
        summary: &str,
        tags: &[String],
    ) -> Result<PathBuf, VaultError> {
        let report_dir = self.path.join(ARTICLES_REPORT_PATH);
        std::fs::create_dir_all(&report_dir)?;

        let resolved_title = get_file_name(title);
        let file_path = report_dir.join(&resolved_title).with_extension("md");

        let today = Local::now().format("%Y-%m-%d").to_string();

        let mut env = Environment::new();
        env.add_template("article", ARTICLE_TEMPLATE)?;

        let template = env.get_template("article")?;

        let rendered = template.render(context! {
            title => resolved_title,
                        url => url,
                        date => today,
                        summary => summary,
                        tags => tags,
        })?;

        std::fs::write(&file_path, rendered)?;

        Ok(file_path)
    }
}

fn get_file_name(parsed_name: Option<String>) -> String {
    match parsed_name {
        Some(name) => name, // TODO: sanitize name to be a valid Windows file name
        None => "untitled_article".to_string(), // TODO: what if already taken at path?
    }
}
