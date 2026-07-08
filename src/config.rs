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
    /// Generate a podcast feed (audio posts) at /podcast.xml (opt-in, default off).
    pub podcast: Option<bool>,
    /// Podcast feed includes only posts tagged `podcast` (default off → all audio).
    pub podcast_tagged: Option<bool>,
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
    /// Show multi-image posts as a swipeable carousel instead of a stack (opt-in).
    pub carousel: Option<bool>,
    /// Ship a tiny script that reports page height to a host page so the site
    /// auto-resizes when embedded in a cross-origin iframe (opt-in).
    pub embed: Option<bool>,
    /// Hide the header navigation links (tags/calendar/about/custom), leaving
    /// only the search box (opt-in).
    pub hide_nav: Option<bool>,
    /// Download + show an attached video even when the post also links YouTube
    /// (default off → the YouTube embed replaces the attached video).
    pub keep_media: Option<bool>,
    /// Resolve genius.com links (fetch the page for its YouTube + lyrics widget).
    pub genius: Option<bool>,
    /// Replace a Spotify link with the Spotify player (opt-in, default off).
    pub spotify: Option<bool>,
    /// Embed a live Instagram post replacing an attached video (opt-in, default off).
    pub instagram: Option<bool>,
    /// Replace a Pinterest pin link with the embedded pin (default on).
    pub pinterest: Option<bool>,
    /// Replace a Bandcamp album/track link with the Bandcamp player (default on).
    pub bandcamp: Option<bool>,
    /// Replace a VK music playlist link with the VK playlist widget (opt-in;
    /// login/region-gated, so a fallback link is always shown).
    pub vk: Option<bool>,
    /// Append a "Related" list to each post, ranked by shared-tag overlap (default on).
    pub related: Option<bool>,
    /// Merge identical media across posts into a shared store (default on).
    pub dedup: Option<bool>,
    /// Check outbound links and log the dead ones (opt-in diagnostic).
    pub dead_links: Option<bool>,
    /// Add a Pinterest "Save" hover button to the site's own images (opt-in).
    pub pinterest_save: Option<bool>,
    /// Fetch Google PageSpeed/Lighthouse scores for the deployed site (opt-in).
    pub pagespeed: Option<bool>,
    /// Installable PWA: web app manifest + a service worker (default on).
    pub pwa: Option<bool>,
    /// Offline mode: the service worker precaches the whole archive (opt-in).
    pub offline: Option<bool>,
    /// Offload videos to this repo's GitHub Releases (kept off the Pages quota).
    pub video_releases: Option<bool>,
    /// Check a post's YouTube link is still live (oEmbed); a removed video keeps
    /// its local media instead of a dead embed. Default on.
    pub liveness: Option<bool>,
    /// Show per-emoji reaction counts (MTProto backend only). Default on.
    pub reactions: Option<bool>,
    /// Enrich the About page from an about.me link in the channel description. Default on.
    pub about_me: Option<bool>,
    /// Copy the about.me bio text onto the About page (opt-in; default off).
    pub aboutme_bio: Option<bool>,
    /// Keep the Telegram channel avatar even when an about.me photo is shown
    /// (opt-in; default drops the avatar to avoid two portraits).
    pub aboutme_both_images: Option<bool>,
    /// Wikidata QID (e.g. `Q42`); renders a statements table on the About page.
    pub wikidata: Option<String>,
    /// Collapse in-post Wikidata tables behind a click-to-expand emoji (opt-in).
    pub wikidata_spoiler: Option<bool>,
    /// Add hover tooltips (a `title=`) to Wikipedia/MediaWiki/YouTube links from
    /// the linked page's intro. Default on.
    pub link_titles: Option<bool>,
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
    /// Body font-family CSS (a local/system font stack, e.g. `Georgia, serif`).
    pub font: Option<String>,
    /// A Google Fonts family to load and use for the body (e.g. `Inter`).
    pub google_font: Option<String>,
    /// Google Analytics measurement ID (e.g. `G-XXXXXXX`) — loads gtag.js.
    pub google_analytics: Option<String>,
    /// Yandex Metrica counter ID (a number) — loads the Metrica tag.
    pub yandex_metrica: Option<String>,
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
    /// Zola's built-in Elasticlunr client-side full-text search over the post
    /// content. Needs JavaScript (bundled, no CDN), so it's opt-in and the
    /// offline pass strips it — it works on the deployed site, not file://.
    Elasticlunr,
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
    /// Generate a podcast feed at /podcast.xml (opt-in, default false).
    pub podcast: bool,
    /// Podcast feed includes only posts tagged `podcast` (default false → all audio).
    pub podcast_tagged: bool,
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
    /// Show multi-image posts as a swipeable carousel (opt-in, default false).
    pub carousel: bool,
    /// Report page height to a host page for iframe auto-resize (opt-in, default false).
    pub embed: bool,
    /// Hide the header nav links, leaving only search (opt-in, default false).
    pub hide_nav: bool,
    /// Keep an attached video even alongside a YouTube link (default false).
    pub keep_media: bool,
    /// Resolve genius.com links (default true).
    pub genius: bool,
    /// Replace Spotify links with the Spotify player (opt-in, default false).
    pub spotify: bool,
    /// Embed a live Instagram post replacing an attached video (opt-in, default false).
    pub instagram: bool,
    /// Replace Pinterest pin links with the embedded pin (default true).
    pub pinterest: bool,
    /// Replace Bandcamp album/track links with the Bandcamp player (default true).
    pub bandcamp: bool,
    /// Replace VK music playlist links with the VK playlist widget (opt-in, default false).
    pub vk: bool,
    /// Append a shared-tag "Related" list to each post (default true).
    pub related: bool,
    /// Merge identical media across posts into a shared store (default true).
    pub dedup: bool,
    /// Check outbound links and log the dead ones (opt-in, default false).
    pub dead_links: bool,
    /// Add a Pinterest "Save" hover button to the site's own images (opt-in, default false).
    pub pinterest_save: bool,
    /// Fetch Google Lighthouse scores for the About page + README badges (opt-in, default false).
    pub pagespeed: bool,
    /// Installable PWA: web app manifest + service worker (default true).
    pub pwa: bool,
    /// Offline mode: the service worker precaches the whole archive (opt-in, default false).
    pub offline: bool,
    /// Offload videos to this repo's GitHub Releases (default true; needs a
    /// github.com repo_url + the CI upload step). Falls back to inline otherwise.
    pub video_releases: bool,
    /// YouTube liveness check (default true).
    pub liveness: bool,
    /// Show per-emoji reaction counts, via MTProto (default true). Only read by
    /// the MTProto backend, which is the only source of reactions.
    #[cfg_attr(not(feature = "mtproto"), allow(dead_code))]
    pub reactions: bool,
    /// Enrich the About page from an about.me link in the description (default true).
    pub about_me: bool,
    /// Copy the about.me bio text onto the About page (opt-in, default false).
    pub aboutme_bio: bool,
    /// Keep the channel avatar alongside the about.me photo (opt-in, default false).
    pub aboutme_both_images: bool,
    /// Wikidata QID for the About-page statements table (`None` = disabled).
    pub wikidata: Option<String>,
    /// Collapse in-post Wikidata tables behind a click-to-expand emoji (default false).
    pub wikidata_spoiler: bool,
    /// Hover tooltips on Wikipedia/MediaWiki/Commons/YouTube links (default true).
    pub link_titles: bool,
    /// Comma-separated tags to surface as links in the top nav.
    pub tags_to_pages: Option<String>,
    pub pages: Option<String>,
    pub posts_per_page: usize,
    pub title_max_len: usize,
    pub background_dark: String,
    pub background_light: String,
    pub css: Option<String>,
    /// Body font-family (local/system font stack). Empty → the default stack.
    pub font: Option<String>,
    /// A Google Fonts family name to load + use for the body. Empty → none.
    pub google_font: Option<String>,
    /// Google Analytics measurement ID (`G-…`); loads gtag.js when set.
    pub google_analytics: Option<String>,
    /// Yandex Metrica counter ID (numeric); loads the Metrica tag when set.
    pub yandex_metrica: Option<String>,
    pub theme: Option<String>,
    pub max_pages: Option<usize>,
    pub page_delay_ms: u64,
    pub concurrency: usize,
    pub group_window_secs: i64,
    pub download_media: bool,
}
