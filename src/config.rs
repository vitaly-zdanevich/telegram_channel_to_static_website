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
    /// Mastodon `@user@instance` handle for `fediverse:creator` + `rel="me"`.
    pub fediverse_creator: Option<String>,
    /// Search-engine name for the header search box (google/duckduckgo/yandex/bing).
    pub search_engine: Option<String>,
    /// Custom search URL prefix; the query is appended on Enter (overrides engine).
    pub search_url: Option<String>,
    /// Footer content (plain text, Markdown or HTML). Empty = no footer.
    pub footer: Option<String>,
    /// Static host for the About-page size limit (`github`/`gitlab`/`none`).
    /// Auto-detected from base_url when unset.
    pub pages_host: Option<String>,
    /// strftime format for displayed dates (default `%Y %B %d` → "2025 October 28";
    /// use `%Y` for year only).
    pub date_format: Option<String>,
    /// UI language for the generated site chrome (Newer/Older/Tags/About/…):
    /// one of en/be/uk/ru/de/fr/zh/ja/pl/es/ko/ka (default `en`). Post content
    /// keeps the channel's own language.
    pub language: Option<String>,
    /// Derive a post title from its first sentence (default off → each post is
    /// identified by a clickable `#id` on its date/views line instead).
    pub derive_titles: Option<bool>,
    /// When deriving titles, also remove that first sentence from the body so it
    /// isn't shown twice (only applies with `derive_titles`).
    pub strip_title: Option<bool>,
    /// Underline links (default false → `text-decoration: none`).
    pub link_underline: Option<bool>,
    /// CSS-only click-to-load YouTube facade (default true); false = direct iframe.
    pub youtube_facade: Option<bool>,
    /// Download + show an attached video even when the post also links YouTube
    /// (default off → the YouTube embed replaces the attached video).
    pub keep_media: Option<bool>,
    /// Resolve genius.com links (fetch the page for its YouTube + lyrics widget).
    pub genius: Option<bool>,
    /// Check a post's YouTube link is still live (oEmbed); a removed video keeps
    /// its local media instead of a dead embed. Default on.
    pub liveness: Option<bool>,
    /// Comma-separated tags to surface as links in the top nav.
    pub tags_to_pages: Option<String>,
    /// Extra pages, each starting with a `# Title` Markdown heading.
    pub pages: Option<String>,
    /// Number of full posts per page on the home feed (default 20).
    pub posts_per_page: Option<usize>,
    /// Max post-title length in characters before it's truncated (default 200).
    pub title_max_len: Option<usize>,
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

/// Header search box behaviour. Google uses a plain (JS-free) form scoped with
/// the `sitesearch` param; the others need a tiny Enter handler to fold
/// `site:<host>` into the query, so they add one small inline script.
#[derive(Debug, Clone)]
pub enum Search {
    /// No search box (and no script).
    None,
    /// Google, JS-free `<form>`. `site` is the host for `sitesearch=` (None
    /// under base_url "/", e.g. offline builds).
    Google { site: Option<String> },
    /// Any other engine / custom prefix: the typed query is appended to `url`
    /// on Enter via a small inline handler.
    Custom { url: String },
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
    pub fediverse_creator: Option<String>,
    /// Resolved header search box behaviour.
    pub search: Search,
    /// Footer content (text/Markdown/HTML); None = no footer.
    pub footer: Option<String>,
    /// Static host for the About-page size limit (github/gitlab); auto-detected
    /// from base_url when None.
    pub pages_host: Option<String>,
    /// strftime format for displayed dates.
    pub date_format: String,
    /// UI language for the site chrome (one of the supported codes; `en` default).
    pub language: String,
    /// Derive post titles from the first sentence (default false → `#id`).
    pub derive_titles: bool,
    /// When deriving titles, strip that sentence from the body.
    pub strip_title: bool,
    /// Underline links (default false).
    pub link_underline: bool,
    /// CSS-only click-to-load YouTube facade (default true).
    pub youtube_facade: bool,
    /// Keep an attached video even alongside a YouTube link (default false).
    pub keep_media: bool,
    /// Resolve genius.com links (default true).
    pub genius: bool,
    /// YouTube liveness check (default true).
    pub liveness: bool,
    /// Comma-separated tags to surface as links in the top nav.
    pub tags_to_pages: Option<String>,
    pub pages: Option<String>,
    pub posts_per_page: usize,
    pub title_max_len: usize,
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
