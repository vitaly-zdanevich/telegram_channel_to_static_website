//! Configuration: an optional TOML file plus CLI overrides, resolved into
//! a single [`Settings`] used by the rest of the pipeline.

use anyhow::{Context, Result};
use serde::Deserialize;
use std::path::{Path, PathBuf};

/// Values that may be supplied via a `--config <file>.toml`. Everything is
/// optional; CLI flags take precedence over these.
#[derive(Debug, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct FileConfig {
    pub channel: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub base_url: Option<String>,
    pub site: Option<PathBuf>,
    /// Repository URL shown on the About page (defaults to the tg2zola repo;
    /// CI sets it to the deployment repo).
    pub repo_url: Option<String>,
    /// Custom HTML for the About page body. When set, replaces the default
    /// (channel + repo links).
    pub about: Option<String>,
    /// Show the per-post tag list footer (tags are clickable in the body anyway).
    pub tags_footer: Option<bool>,
    /// Next/Prev navigation buttons on posts (default on).
    pub next_prev: Option<bool>,
    /// "View on Telegram" link on posts (default on).
    pub telegram_link: Option<bool>,
    /// Generate an RSS feed at /rss.xml (default on).
    pub rss: Option<bool>,
    /// Extra pages, each starting with a `# Title` Markdown heading.
    pub pages: Option<String>,
    /// Number of full posts per page on the home feed (default 20).
    pub posts_per_page: Option<usize>,
    /// Background colors (any CSS color). Defaults: dark `#000000`, light `#ffffff`.
    pub background_dark: Option<String>,
    pub background_light: Option<String>,
    /// Extra CSS appended to the built-in stylesheet.
    pub css: Option<String>,
    /// Optional theme name (a directory under `site/themes/`). When set, the
    /// built-in templates are not written and the theme drives the look.
    pub theme: Option<String>,
    pub max_pages: Option<usize>,
    pub page_delay_ms: Option<u64>,
    pub concurrency: Option<usize>,
    pub group_window_secs: Option<i64>,
    pub download_media: Option<bool>,
}

impl FileConfig {
    pub fn load(path: &Path) -> Result<Self> {
        let s = std::fs::read_to_string(path)
            .with_context(|| format!("reading config {}", path.display()))?;
        toml::from_str(&s).with_context(|| format!("parsing config {}", path.display()))
    }
}

/// Fully resolved settings used to run a generation.
#[derive(Debug, Clone)]
pub struct Settings {
    pub channel: String,
    pub title: String,
    pub description: String,
    pub base_url: String,
    pub site: PathBuf,
    pub repo_url: String,
    pub about: Option<String>,
    pub tags_footer: bool,
    pub next_prev: bool,
    pub telegram_link: bool,
    pub rss: bool,
    pub pages: Option<String>,
    pub posts_per_page: usize,
    pub background_dark: String,
    pub background_light: String,
    pub css: Option<String>,
    pub theme: Option<String>,
    pub max_pages: Option<usize>,
    pub page_delay_ms: u64,
    pub concurrency: usize,
    pub group_window_secs: i64,
    pub download_media: bool,
}
