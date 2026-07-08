//! tg2zola — back up a public Telegram channel into a self-contained Zola site.
//!
//! Pipeline: scrape `t.me/s/<channel>` → group messages into posts → download
//! media into a cache → regenerate every Zola page bundle. The generated site
//! references only local files and YouTube — never Telegram — so it survives the
//! channel being removed.

mod aboutme;
mod config;
mod bandcamp;
mod dedup;
mod genius;
mod group;
mod html2md;
mod i18n;
mod liveness;
mod media;
mod model;
#[cfg(feature = "mtproto")]
mod mtproto;
mod offline;
mod pagespeed;
mod parse;
mod podcast;
mod pwa;
mod render;
mod linktitles;
mod scrape;
mod site;
mod wikidata;

#[cfg(test)]
mod e2e_tests;

use anyhow::{Context, Result};
use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;
use tracing::info;

use config::{FileConfig, Search, Settings};

#[derive(Parser, Debug)]
#[command(
    name = "tg2zola",
    version,
    about = "Back up a public Telegram channel into a self-contained Zola static website",
    args_conflicts_with_subcommands = true
)]
struct Cli {
    #[command(flatten)]
    generate: GenerateArgs,
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Rewrite a built site (the Zola `public/` directory) into relative links
    /// so it opens directly via file:// with no web server.
    Offline {
        /// The built site directory, e.g. site/public
        dir: PathBuf,
    },
    /// Write `asset-manifest.json` (the service-worker precache list) for a built
    /// site. Run after `zola build` when `--offline` was used.
    Pwa {
        /// The built site directory, e.g. site/public
        dir: PathBuf,
    },
    /// One-time MTProto login: create a reusable user session. Writes
    /// `tg2zola.session` and prints a base64 `TG_SESSION` for a CI secret.
    /// Needs `TG_API_ID` / `TG_API_HASH` in the environment.
    #[cfg(feature = "mtproto")]
    Login,
}

/// Scrape + regenerate the Zola site (the default action when no subcommand).
#[derive(Args, Debug)]
struct GenerateArgs {
    /// Optional TOML config file (CLI flags override its values).
    #[arg(long)]
    config: Option<PathBuf>,

    /// Public channel username, e.g. `durov` (without @).
    #[arg(long)]
    channel: Option<String>,

    /// Output Zola site directory.
    #[arg(long)]
    site: Option<PathBuf>,

    /// Theme name (a directory under site/themes/) to use instead of the
    /// built-in templates. Usually set by CI from the THEME_REPO variable.
    #[arg(long)]
    theme: Option<String>,

    /// Site title (defaults to the channel name).
    #[arg(long)]
    title: Option<String>,

    /// Site description.
    #[arg(long)]
    description: Option<String>,

    /// Zola base_url written into config.toml.
    #[arg(long)]
    base_url: Option<String>,

    /// Repository URL shown on the About page (defaults to the tg2zola repo).
    #[arg(long)]
    repo_url: Option<String>,

    /// Custom HTML for the About page body (overrides the default channel/repo
    /// links). In CI this comes from the ABOUT repository variable.
    #[arg(long)]
    about: Option<String>,

    /// Show the per-post tag footer (off by default — tags are clickable in the
    /// post body anyway).
    #[arg(long)]
    tags_footer: bool,

    /// Disable the Next/Prev post navigation buttons.
    #[arg(long)]
    no_next_prev: bool,

    /// Disable the "View on Telegram" link on posts.
    #[arg(long)]
    no_telegram_link: bool,

    /// Disable RSS feed generation (on by default).
    #[arg(long)]
    no_rss: bool,

    /// Mastodon handle (`@user@instance`) for the `fediverse:creator` byline on
    /// link previews and a `rel="me"` profile-verification link.
    #[arg(long)]
    fediverse_creator: Option<String>,

    /// Header search box engine: google | duckduckgo | yandex | bing (scoped to
    /// this site). Adds a tiny inline Enter handler; omit for no search box.
    #[arg(long)]
    search_engine: Option<String>,

    /// Custom search URL prefix; the typed query is appended on Enter. Overrides
    /// --search-engine (e.g. for a self-hosted or unlisted engine).
    #[arg(long)]
    search_url: Option<String>,

    /// Footer content (plain text, Markdown or HTML). In CI set the FOOTER
    /// variable. Empty = no footer.
    #[arg(long)]
    footer: Option<String>,

    /// Static host for the About-page size limit: `github` | `gitlab` | `none`.
    /// Auto-detected from the base URL (github.io / gitlab.io) when unset.
    #[arg(long)]
    pages_host: Option<String>,

    /// strftime format for displayed dates (default `%Y %B %d`, e.g.
    /// "2025 October 28"; use `%Y` for year only).
    #[arg(long)]
    date_format: Option<String>,

    /// UI language for the site chrome (Newer/Older/Tags/About/…): one of
    /// en/be/uk/ru/de/fr/zh/ja/pl/es/ko/ka (default `en`). In CI set LANGUAGE.
    #[arg(long)]
    language: Option<String>,

    /// Derive a post title from its first sentence. Default off: each post is
    /// shown by a clickable #id on its date/views line instead of a title.
    #[arg(long)]
    derive_titles: bool,

    /// With --derive-titles, also remove that first sentence from the body so it
    /// isn't shown twice.
    #[arg(long)]
    strip_title: bool,

    /// Underline links (default: no underline).
    #[arg(long)]
    link_underline: bool,

    /// Enable the CSS click-to-load YouTube facade (default: direct iframe).
    #[arg(long)]
    youtube_facade: bool,

    /// Show multi-image posts as a swipeable carousel instead of a vertical
    /// stack (opt-in; the swipe is CSS scroll-snap, arrows/dots need JavaScript).
    #[arg(long)]
    carousel: bool,

    /// Ship a small script that posts the page height to a host page so the site
    /// auto-resizes when embedded in a cross-origin iframe (opt-in).
    #[arg(long)]
    embed: bool,

    /// Hide the header navigation links (tags/calendar/about/custom), leaving
    /// only the search box (opt-in).
    #[arg(long)]
    hide_nav: bool,

    /// Keep (download + show) attached media — video and audio — even when the
    /// post also links YouTube / Apple Podcasts (default: the embed replaces it).
    #[arg(long)]
    keep_media: bool,

    /// Don't resolve genius.com links (skip fetching their pages for a YouTube
    /// video / lyrics widget).
    #[arg(long)]
    no_genius: bool,

    /// Replace a Spotify link with the Spotify player (opt-in; note it plays a
    /// ~30s preview for non-Premium listeners).
    #[arg(long)]
    spotify: bool,

    /// Embed a live Instagram post in place of an attached video (opt-in; the
    /// widget needs JavaScript and loads from instagram.com).
    #[arg(long)]
    instagram: bool,

    /// Don't replace a Pinterest pin link with the embedded pin (default: embed).
    #[arg(long)]
    no_pinterest: bool,

    /// Don't replace a Bandcamp album/track link with the Bandcamp player
    /// (default: fetch the page and embed the player).
    #[arg(long)]
    no_bandcamp: bool,

    /// Replace a VK music playlist link with the VK playlist widget (opt-in).
    /// Note: VK music playback needs a logged-in VK session and is blocked in
    /// some regions; a fallback "Open on VK" link is always shown.
    #[arg(long)]
    vk: bool,

    /// Don't append a "Related" list to each post (default: append the posts
    /// that share the most tags with it).
    #[arg(long)]
    no_related: bool,

    /// Don't merge identical media across posts into a shared /media/ store
    /// (default: dedup so the published site serves one copy of a reposted file).
    #[arg(long)]
    no_dedup: bool,

    /// Add a Pinterest "Save" hover button to the site's own images so visitors
    /// can pin them to their boards (opt-in; needs JavaScript).
    #[arg(long)]
    pinterest_save: bool,

    /// Don't fetch Google Lighthouse scores (default: fetch them for the About
    /// page + shields.io badge endpoints, when base_url is an http(s) URL).
    #[arg(long)]
    no_pagespeed: bool,

    /// Don't make the site an installable PWA (default: emit a web app manifest
    /// + a service worker so it can be installed / hides the address bar).
    #[arg(long)]
    no_pwa: bool,

    /// Offline mode: the service worker precaches the whole archive (on any
    /// non-cellular connection) so the site works fully offline (opt-in; needs
    /// JavaScript). Run `tg2zola pwa <public>` after `zola build` for the list.
    #[arg(long)]
    offline: bool,

    /// Don't offload videos to GitHub Releases (default: for a github.com repo,
    /// videos are staged for upload to a `media` release and played from there,
    /// keeping them off the Pages 1 GB quota).
    #[arg(long)]
    no_video_releases: bool,

    /// Skip the YouTube liveness check (a removed video otherwise keeps its local
    /// media instead of being replaced by a dead embed).
    #[arg(long)]
    no_liveness: bool,

    /// Don't show per-emoji reaction counts (they're fetched by the MTProto
    /// backend when it runs; the web preview never exposes reactions).
    #[arg(long)]
    no_reactions: bool,

    /// Don't enrich the About page from an about.me link in the channel
    /// description (default: pull its social links + a contact button).
    #[arg(long)]
    no_about_me: bool,

    /// Also copy the about.me bio text onto the About page (default: skip it;
    /// only the social links, photo and contact button are used).
    #[arg(long)]
    aboutme_bio: bool,

    /// Show both the Telegram channel avatar and the about.me photo (default:
    /// when an about.me photo is present, the channel avatar is dropped).
    #[arg(long)]
    aboutme_both_images: bool,

    /// Wikidata item id (e.g. `Q42`) to render as a statements table on the
    /// About page. Also, any `wikidata.org/wiki/Q…` link in a post gets its own
    /// table appended (this flag only controls the About-page entity).
    #[arg(long)]
    wikidata: Option<String>,

    /// Collapse in-post Wikidata tables behind a click-to-expand emoji (the
    /// About-page table is always shown expanded).
    #[arg(long)]
    wikidata_spoiler: bool,

    /// Don't add hover tooltips (a `title=`) to Wikipedia / MediaWiki / Wikimedia
    /// Commons / YouTube links from the linked page's intro (default: add them).
    #[arg(long)]
    no_link_titles: bool,

    /// Generate a podcast feed (audio posts) at /podcast.xml (opt-in). Cover from
    /// the about.me photo, else a post tagged `podcast_description`; that post's
    /// text (or the channel description) is the podcast description.
    #[arg(long)]
    podcast: bool,

    /// Restrict the podcast feed to audio posts tagged `podcast` (default: all
    /// audio posts). Only meaningful with `--podcast`.
    #[arg(long)]
    podcast_tagged: bool,

    /// Comma-separated tags to surface as `#tag` links in the top nav.
    #[arg(long)]
    tags_to_pages: Option<String>,

    /// Extra pages as Markdown, each section starting with a `# Title` heading
    /// (becomes a page + nav entry). In CI this comes from the PAGES variable.
    #[arg(long)]
    pages: Option<String>,

    /// Full posts per page on the home feed (default 20).
    #[arg(long)]
    posts_per_page: Option<usize>,

    /// Max post-title length in characters before truncation (default 200).
    /// When a title is truncated its full first sentence is kept in the body.
    #[arg(long)]
    title_max_len: Option<usize>,

    /// Dark-mode background color (any CSS color). Default #000000.
    #[arg(long)]
    background_dark_color: Option<String>,

    /// Light-mode background color (any CSS color). Default #ffffff.
    #[arg(long)]
    background_light_color: Option<String>,

    /// Extra CSS appended to the built-in stylesheet.
    #[arg(long)]
    css: Option<String>,

    /// Body font-family CSS — a local/system font stack, e.g. "Georgia, serif".
    #[arg(long)]
    font: Option<String>,

    /// A Google Fonts family to load and use for the body text, e.g. "Inter".
    /// Loads it from fonts.googleapis.com (an external request).
    #[arg(long)]
    google_font: Option<String>,

    /// Google Analytics measurement ID (e.g. "G-XXXXXXX") — injects gtag.js.
    #[arg(long)]
    google_analytics: Option<String>,

    /// Yandex Metrica counter ID (a number) — injects the Metrica tag.
    #[arg(long)]
    yandex_metrica: Option<String>,

    /// Stop after N pages (~20 messages each). Omit to fetch the whole history.
    #[arg(long)]
    max_pages: Option<usize>,

    /// Delay between page fetches, milliseconds.
    #[arg(long)]
    page_delay_ms: Option<u64>,

    /// Concurrent media downloads.
    #[arg(long)]
    concurrency: Option<usize>,

    /// Merge consecutive same-author messages within this many seconds into one
    /// post (handles bursts of forwarded messages).
    #[arg(long)]
    group_window_secs: Option<i64>,

    /// Skip downloading media (text-only run, useful for quick tests).
    #[arg(long)]
    no_media: bool,

    /// Create config.toml + templates if missing (idempotent).
    #[arg(long)]
    init_site: bool,

    /// Log level: error|warn|info|debug|trace (or set RUST_LOG).
    #[arg(long, default_value = "info")]
    log: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Some(Commands::Offline { dir }) => {
            init_tracing("info");
            offline::relativize(&dir)
        }
        Some(Commands::Pwa { dir }) => {
            init_tracing("info");
            let n = pwa::write_asset_manifest(&dir)?;
            info!("pwa: wrote asset-manifest.json with {n} URL(s) to precache");
            Ok(())
        }
        #[cfg(feature = "mtproto")]
        Some(Commands::Login) => {
            init_tracing("info");
            mtproto::login().await
        }
        None => {
            let g = cli.generate;
            init_tracing(&g.log);
            let file_config = match &g.config {
                Some(p) => FileConfig::load(p)?,
                None => FileConfig::default(),
            };
            let settings = resolve(&g, file_config)?;
            run(settings, g.init_site).await
        }
    }
}

fn resolve(g: &GenerateArgs, fc: FileConfig) -> Result<Settings> {
    let channel = g
        .channel
        .clone()
        .or(fc.channel)
        .context("channel is required (pass --channel or set channel = \"...\" in --config)")?;
    let channel = channel.trim().trim_start_matches('@').to_string();
    anyhow::ensure!(!channel.is_empty(), "channel must not be empty");

    let base_url = g
        .base_url
        .clone()
        .or(fc.base_url)
        .unwrap_or_else(|| "/".to_string());

    let raw_language = g.language.clone().or(fc.language).unwrap_or_default();
    let language_base = raw_language
        .trim()
        .split(['-', '_'])
        .next()
        .unwrap_or("")
        .to_ascii_lowercase();
    if !language_base.is_empty() && !i18n::SUPPORTED.contains(&language_base.as_str()) {
        tracing::warn!(
            "unsupported language {:?}; using English (supported: {})",
            raw_language,
            i18n::SUPPORTED.join(", ")
        );
    }
    let language = i18n::normalize(&raw_language).to_string();

    Ok(Settings {
        title: g.title.clone().or(fc.title).unwrap_or_else(|| channel.clone()),
        description: g.description.clone().or(fc.description).unwrap_or_default(),
        base_url: base_url.clone(),
        repo_url: g.repo_url.clone().or(fc.repo_url).unwrap_or_else(|| {
            "https://github.com/vitaly-zdanevich/telegram_channel_to_static_website".to_string()
        }),
        about: g
            .about
            .clone()
            .or(fc.about)
            .filter(|s| !s.trim().is_empty()),
        tags_footer: g.tags_footer || fc.tags_footer.unwrap_or(false),
        next_prev: if g.no_next_prev {
            false
        } else {
            fc.next_prev.unwrap_or(true)
        },
        telegram_link: if g.no_telegram_link {
            false
        } else {
            fc.telegram_link.unwrap_or(true)
        },
        rss: if g.no_rss {
            false
        } else {
            fc.rss.unwrap_or(true)
        },
        fediverse_creator: g
            .fediverse_creator
            .clone()
            .or(fc.fediverse_creator)
            .filter(|s| !s.trim().is_empty()),
        pages: g.pages.clone().or(fc.pages).filter(|s| !s.trim().is_empty()),
        posts_per_page: g
            .posts_per_page
            .or(fc.posts_per_page)
            .filter(|&n| n > 0)
            .unwrap_or(20),
        title_max_len: g
            .title_max_len
            .or(fc.title_max_len)
            .filter(|&n| n > 0)
            .unwrap_or(200),
        search: resolve_search(
            g.search_engine.clone().or(fc.search_engine),
            g.search_url.clone().or(fc.search_url),
            &base_url,
        ),
        footer: g.footer.clone().or(fc.footer).filter(|s| !s.trim().is_empty()),
        pages_host: g
            .pages_host
            .clone()
            .or(fc.pages_host)
            .filter(|s| !s.trim().is_empty()),
        date_format: g
            .date_format
            .clone()
            .or(fc.date_format)
            .filter(|s| !s.trim().is_empty())
            .unwrap_or_else(|| "%Y %B %d".to_string()),
        language,
        derive_titles: g.derive_titles || fc.derive_titles.unwrap_or(false),
        strip_title: g.strip_title || fc.strip_title.unwrap_or(false),
        link_underline: g.link_underline || fc.link_underline.unwrap_or(false),
        youtube_facade: g.youtube_facade || fc.youtube_facade.unwrap_or(false),
        carousel: g.carousel || fc.carousel.unwrap_or(false),
        embed: g.embed || fc.embed.unwrap_or(false),
        hide_nav: g.hide_nav || fc.hide_nav.unwrap_or(false),
        // Default flips by environment: on CI (GitHub Actions / GitLab) prefer
        // embeds over downloads to fit the static-host budget; on a local machine
        // download everything for a complete backup. An explicit --keep-media or
        // a config `keep_media` value still wins.
        keep_media: g.keep_media || fc.keep_media.unwrap_or_else(|| !running_in_ci()),
        genius: if g.no_genius {
            false
        } else {
            fc.genius.unwrap_or(true)
        },
        spotify: g.spotify || fc.spotify.unwrap_or(false),
        instagram: g.instagram || fc.instagram.unwrap_or(false),
        pinterest: if g.no_pinterest {
            false
        } else {
            fc.pinterest.unwrap_or(true)
        },
        bandcamp: if g.no_bandcamp {
            false
        } else {
            fc.bandcamp.unwrap_or(true)
        },
        vk: g.vk || fc.vk.unwrap_or(false),
        related: if g.no_related {
            false
        } else {
            fc.related.unwrap_or(true)
        },
        dedup: if g.no_dedup {
            false
        } else {
            fc.dedup.unwrap_or(true)
        },
        pinterest_save: g.pinterest_save || fc.pinterest_save.unwrap_or(false),
        pagespeed: if g.no_pagespeed {
            false
        } else {
            fc.pagespeed.unwrap_or(true)
        },
        pwa: if g.no_pwa { false } else { fc.pwa.unwrap_or(true) },
        offline: g.offline || fc.offline.unwrap_or(false),
        video_releases: if g.no_video_releases {
            false
        } else {
            fc.video_releases.unwrap_or(true)
        },
        liveness: if g.no_liveness {
            false
        } else {
            fc.liveness.unwrap_or(true)
        },
        reactions: if g.no_reactions {
            false
        } else {
            fc.reactions.unwrap_or(true)
        },
        about_me: if g.no_about_me {
            false
        } else {
            fc.about_me.unwrap_or(true)
        },
        aboutme_bio: g.aboutme_bio || fc.aboutme_bio.unwrap_or(false),
        aboutme_both_images: g.aboutme_both_images || fc.aboutme_both_images.unwrap_or(false),
        wikidata: g.wikidata.clone().or(fc.wikidata.clone()),
        wikidata_spoiler: g.wikidata_spoiler || fc.wikidata_spoiler.unwrap_or(false),
        link_titles: if g.no_link_titles {
            false
        } else {
            fc.link_titles.unwrap_or(true)
        },
        podcast: g.podcast || fc.podcast.unwrap_or(false),
        podcast_tagged: g.podcast_tagged || fc.podcast_tagged.unwrap_or(false),
        tags_to_pages: g
            .tags_to_pages
            .clone()
            .or(fc.tags_to_pages)
            .filter(|s| !s.trim().is_empty()),
        background_dark: g
            .background_dark_color
            .clone()
            .or(fc.background_dark)
            .filter(|s| !s.trim().is_empty())
            .unwrap_or_else(|| "#000000".to_string()),
        background_light: g
            .background_light_color
            .clone()
            .or(fc.background_light)
            .filter(|s| !s.trim().is_empty())
            .unwrap_or_else(|| "#ffffff".to_string()),
        css: g.css.clone().or(fc.css).filter(|s| !s.trim().is_empty()),
        font: g.font.clone().or(fc.font).filter(|s| !s.trim().is_empty()),
        google_font: g
            .google_font
            .clone()
            .or(fc.google_font)
            .filter(|s| !s.trim().is_empty()),
        google_analytics: g
            .google_analytics
            .clone()
            .or(fc.google_analytics)
            .filter(|s| !s.trim().is_empty()),
        yandex_metrica: g
            .yandex_metrica
            .clone()
            .or(fc.yandex_metrica)
            .filter(|s| !s.trim().is_empty()),
        site: g
            .site
            .clone()
            .or(fc.site)
            .unwrap_or_else(|| PathBuf::from("site")),
        theme: g
            .theme
            .clone()
            .or(fc.theme)
            .filter(|t| !t.trim().is_empty()),
        max_pages: g.max_pages.or(fc.max_pages),
        page_delay_ms: g.page_delay_ms.or(fc.page_delay_ms).unwrap_or(600),
        concurrency: g.concurrency.or(fc.concurrency).unwrap_or(4).max(1),
        group_window_secs: g.group_window_secs.or(fc.group_window_secs).unwrap_or(1),
        download_media: if g.no_media {
            false
        } else {
            fc.download_media.unwrap_or(true)
        },
        channel,
    })
}

/// Resolve the header search box. A custom URL wins (Enter handler appends the
/// query). Otherwise the engine name (default `elasticlunr` — Zola's built-in
/// client-side full-text index, self-contained but JS) selects a built-in:
/// `google` is a JS-free form, the other web engines get an Enter handler with
/// `site:<host>` folded into the query, `none`/`off` disables it.
fn resolve_search(engine: Option<String>, custom: Option<String>, base_url: &str) -> Search {
    if let Some(u) = custom.map(|s| s.trim().to_string()).filter(|s| !s.is_empty()) {
        return Search::Custom { url: u };
    }
    let host = host_of(base_url);
    // `site:<host>` filter, URL-encoded, so non-Google results stay on this site.
    let scope = host
        .as_ref()
        .map(|h| format!("site%3A{h}%20"))
        .unwrap_or_default();
    let engine = engine
        .map(|s| s.trim().to_lowercase())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "elasticlunr".to_string());
    match engine.as_str() {
        "none" | "off" | "no" | "false" => Search::None,
        "google" | "g" => Search::Google { site: host },
        "duckduckgo" | "duckduck" | "duck" | "ddg" => Search::Custom {
            url: format!("https://duckduckgo.com/?q={scope}"),
        },
        "yandex" | "ya" => Search::Custom {
            url: format!("https://yandex.com/search/?text={scope}"),
        },
        "bing" | "bind" => Search::Custom {
            url: format!("https://www.bing.com/search?q={scope}"),
        },
        "elasticlunr" | "lunr" | "local" => Search::Elasticlunr,
        other => {
            tracing::warn!("unknown search engine '{other}' — disabling the search box");
            Search::None
        }
    }
}

/// True on a CI runner (GitHub Actions, GitLab CI, or a generic `CI=true`). There
/// the `keep_media` default flips to space-saving — a live YouTube/Instagram link
/// replaces the attached video instead of downloading it, to fit the ~1 GB
/// static-host budget. Off CI (a local machine) the default is to download
/// everything for a complete backup.
fn running_in_ci() -> bool {
    ["GITHUB_ACTIONS", "GITLAB_CI", "CI"]
        .iter()
        .any(|k| std::env::var(k).is_ok_and(|v| matches!(v.trim(), "true" | "1" | "yes")))
}

/// Host of a base URL (`https://host/path` → `host`); None for "/" (offline
/// builds), where domain-scoped search doesn't apply.
fn host_of(base_url: &str) -> Option<String> {
    let s = base_url.trim();
    let s = s
        .strip_prefix("https://")
        .or_else(|| s.strip_prefix("http://"))
        .unwrap_or(s);
    let host = s.split('/').next().unwrap_or("").trim();
    (!host.is_empty() && host.contains('.')).then(|| host.to_string())
}

async fn run(mut s: Settings, init_site: bool) -> Result<()> {
    let started = std::time::Instant::now();
    let client = http_client()?;

    info!(
        "running {} — media: {}",
        if running_in_ci() { "in CI" } else { "locally" },
        if s.keep_media {
            "downloading all media (full backup)"
        } else {
            "embedding live YouTube/Instagram to save space"
        },
    );
    info!("scraping https://t.me/s/{}", s.channel);
    let scraper = scrape::Scraper::new(client.clone(), s.channel.clone(), s.page_delay_ms);
    let (messages, channel_info) = scraper.fetch_all(s.max_pages).await?;
    anyhow::ensure!(
        !messages.is_empty(),
        "no messages found — is '{}' a public channel with the web preview enabled?",
        s.channel
    );
    info!("fetched {} messages", messages.len());

    // Use the channel's display name as the blog title unless one was set.
    if s.title == s.channel {
        if let Some(t) = channel_info.as_ref().and_then(|i| i.title.as_deref()) {
            s.title = t.to_string();
        }
    }

    let posts = group::group(messages, s.group_window_secs);
    info!("grouped into {} posts", posts.len());

    // PAGE-marked posts become standalone pages (in the nav), not feed posts.
    let (page_posts, mut posts): (Vec<_>, Vec<_>) =
        posts.into_iter().partition(render::is_page);
    if !page_posts.is_empty() {
        info!("{} PAGE post(s) → pages", page_posts.len());
    }

    // Resolve genius.com links into the YouTube video they reference (+ song id)
    // *before* the liveness check below, so a genius-resolved video is verified
    // for embeddability too — not just direct YouTube links.
    if s.genius {
        genius::enrich(&client, &mut posts, s.concurrency).await;
    }

    // Resolve Bandcamp album/track links to their embeddable player (page fetch).
    if s.bandcamp {
        bandcamp::enrich(&client, &mut posts, s.concurrency).await;
    }

    // VK music playlist links → the opt-in VK widget (pure URL parse, no fetch).
    if s.vk {
        for p in &mut posts {
            p.vk_playlist = media::vk_playlist_url(&p.links);
        }
    }

    // Related posts (opt-in): rank each post's neighbours by shared-tag overlap.
    if s.related {
        let related = render::compute_related(&posts, 5);
        for (p, r) in posts.iter_mut().zip(related) {
            p.related = r;
        }
    }

    // Check YouTube links for liveness so a *removed* video keeps its local
    // media instead of being replaced by a dead embed (before the MTProto
    // audio-skip + render, which both consult it).
    if s.liveness {
        liveness::check_youtube(&client, &mut posts, s.concurrency).await;
        liveness::check_apple(&client, &mut posts, s.concurrency).await;
        liveness::check_yandex(&client, &mut posts, s.concurrency).await;
        if s.instagram {
            liveness::check_instagram(&client, &mut posts, &s.base_url).await;
        }
        if s.spotify {
            liveness::check_spotify(&client, &mut posts, s.concurrency).await;
        }
        if s.pinterest {
            liveness::check_pinterest(&client, &mut posts, s.concurrency).await;
        }
    }

    // Optional MTProto backend: pull audio/voice (and, with MTPROTO_IMAGES=1,
    // original-quality photos) the web preview omits. Runs before the empty-post
    // filter so an audio-only post survives. No-op without the feature or creds.
    #[cfg(feature = "mtproto")]
    let mtproto_used = mtproto::maybe_enrich(&mut posts, &s).await;
    #[cfg(not(feature = "mtproto"))]
    let mtproto_used = false;

    // Drop posts with nothing worth showing (e.g. a lone non-downloadable file).
    let before = posts.len();
    posts.retain(|p| !render::is_empty_post(p));
    if before != posts.len() {
        info!("skipped {} empty post(s)", before - posts.len());
    }

    // Hover tooltips: attach the linked page's intro as a `title=` on Wikipedia /
    // MediaWiki / Wikimedia Commons / YouTube links (build-time fetch, no key).
    if s.link_titles {
        linktitles::enrich(&client, &mut posts, s.concurrency).await;
    }

    // Wikidata: render a statements table for every wikidata.org item linked in a
    // post (and the About-page entity). Each entity is fetched once via the public
    // API (no key); the result is static HTML that also works offline.
    let mut about_wikidata_html: Option<String> = None;
    {
        use futures::StreamExt;
        let mut qids: std::collections::BTreeSet<String> =
            posts.iter().flat_map(|p| media::wikidata_qids(&p.links)).collect();
        if let Some(q) = &s.wikidata {
            qids.insert(q.trim().to_ascii_uppercase());
        }
        if !qids.is_empty() {
            info!("wikidata: resolving {} entit(ies)", qids.len());
            let fetched: Vec<(String, Option<wikidata::Table>)> = futures::stream::iter(
                qids.into_iter().map(|q| async {
                    let t = wikidata::fetch(&client, &q, &s.language).await;
                    (q, t)
                }),
            )
            .buffer_unordered(s.concurrency.max(1))
            .collect()
            .await;
            let tables: std::collections::HashMap<String, wikidata::Table> =
                fetched.into_iter().filter_map(|(q, t)| t.map(|t| (q, t))).collect();
            let about = i18n::about(&s.language);
            for p in &mut posts {
                for q in media::wikidata_qids(&p.links) {
                    if let Some(t) = tables.get(&q) {
                        let html = t.to_html(about.wd_property, about.wd_value);
                        p.wikidata_html.push(if s.wikidata_spoiler {
                            wikidata::spoiler(&html)
                        } else {
                            html
                        });
                    }
                }
            }
            if let Some(q) = &s.wikidata {
                if let Some(t) = tables.get(&q.trim().to_ascii_uppercase()) {
                    about_wikidata_html = Some(t.to_html(about.wd_property, about.wd_value));
                }
            }
        }
    }

    // Auto-tag posts that have a playable (downloadable) video with #video,
    // unless the author already tagged it.
    for p in &mut posts {
        let has_video = p
            .media
            .iter()
            .any(|m| matches!(m, model::Media::Video { .. } | model::Media::LocalVideo { .. }));
        if has_video && !p.tags.iter().any(|t| t == "video") {
            p.tags.push("video".to_string());
        }
    }

    let tag_counts = count_tags(&posts);

    // Download the channel avatar (for the header) before scaffolding so the
    // config can reference it.
    if s.download_media {
        if let Some(url) = channel_info.as_ref().and_then(|i| i.avatar_url.as_deref()) {
            let job = media::Job {
                url: url.to_string(),
                dest: s.site.join("static/channel-avatar.jpg"),
                force: false,
                local: None,
            };
            media::download_all(&client, &[job], 1).await?;
        }
    }

    let rewriter = render::LinkRewriter::new(&s.channel, &posts);

    // Localized UI strings for rendered post bodies (e.g. the "not archived"
    // attachment note). Template chrome is localized via config.extra.i18n.
    let ui = i18n::ui(&s.language);
    // GitHub Releases base URL for offloaded videos — only for a github.com repo
    // (the URL scheme is GitHub-specific); otherwise videos stay inline.
    let video_release_base = (s.video_releases && s.repo_url.contains("github.com"))
        .then(|| format!("{}/releases/download/media", s.repo_url.trim_end_matches('/')));
    if let Some(base) = &video_release_base {
        info!("videos offloaded to GitHub Releases: {base}/…");
    }
    let render_opts = render::RenderOpts {
        ui: &ui,
        title_max: s.title_max_len,
        derive_titles: s.derive_titles,
        strip_title: s.strip_title,
        keep_media: s.keep_media,
        spotify: s.spotify,
        instagram: s.instagram,
        pinterest: s.pinterest,
        video_releases: video_release_base.as_deref(),
        carousel: s.carousel,
    };

    // Render PAGE posts first so their nav entries are ready for scaffolding.
    let rendered_pages: Vec<render::RenderedPost> = page_posts
        .iter()
        .map(|p| render::render_post(p, &rewriter, true, None, None, &render_opts))
        .collect();
    let page_nav: Vec<(String, String)> = rendered_pages
        .iter()
        .map(|r| (r.title.clone(), r.slug.clone()))
        .collect();

    // Per-day post count + tags (ascending by day) drive the /day/<date>/ pages
    // and the calendar (cell size by count, the day's tags in the hover title).
    let mut by_day: std::collections::BTreeMap<String, (usize, std::collections::BTreeSet<String>)> =
        std::collections::BTreeMap::new();
    for p in &posts {
        let e = by_day
            .entry(p.date.format("%Y-%m-%d").to_string())
            .or_default();
        e.0 += 1;
        for t in &p.tags {
            e.1.insert(t.clone());
        }
    }
    let days: Vec<site::DayMeta> = by_day
        .into_iter()
        .map(|(day, (count, tags))| site::DayMeta {
            day,
            count,
            tags: tags.into_iter().collect(),
        })
        .collect();

    if init_site {
        site::scaffold(&s, channel_info.as_ref(), &tag_counts, &page_nav, &days)?;
    }

    // Posts are id-ascending; neighbour id+title drive the Next/Prev nav.
    let titles: Vec<String> = posts
        .iter()
        .map(|p| render::post_title(p, s.title_max_len, s.derive_titles))
        .collect();
    // Full plain-text of each post, for the Newer/Older hover tooltip.
    let previews: Vec<String> = posts.iter().map(render::post_preview).collect();
    let rendered: Vec<render::RenderedPost> = posts
        .iter()
        .enumerate()
        .map(|(i, p)| {
            let newer = posts
                .get(i + 1)
                .map(|q| (q.primary_id, titles[i + 1].as_str(), previews[i + 1].as_str()));
            let older = i
                .checked_sub(1)
                .map(|j| (posts[j].primary_id, titles[j].as_str(), previews[j].as_str()));
            render::render_post(p, &rewriter, false, newer, older, &render_opts)
        })
        .collect();

    // Write bundles (dirs + index.md) and prune removed posts / stale files
    // first, so downloads land in existing dirs and reuse already-cached files.
    site::write_site(&s, &rendered)?;
    site::write_pages(&s, &rendered_pages)?;

    if s.download_media {
        // Videos flagged for GitHub Releases are staged outside the published
        // tree (the CI step uploads them); everything else lands in its bundle.
        let staging = s.site.join(".video-releases");
        let job_for = |dir: std::path::PathBuf| {
            let staging = staging.clone();
            move |d: &render::Download| media::Job {
                url: d.url.clone(),
                dest: if d.release {
                    staging.join(&d.filename)
                } else {
                    dir.join(&d.filename)
                },
                force: d.force,
                local: d.local.clone(),
            }
        };
        let posts_dir = s.site.join("content/posts");
        let pages_dir = s.site.join("content/pages");
        let mut jobs: Vec<media::Job> = rendered
            .iter()
            .flat_map(|r| r.downloads.iter().map(job_for(posts_dir.join(&r.slug))))
            .collect();
        jobs.extend(
            rendered_pages
                .iter()
                .flat_map(|r| r.downloads.iter().map(job_for(pages_dir.join(&r.slug)))),
        );
        info!("{} media references across posts/pages", jobs.len());
        media::download_all(&client, &jobs, s.concurrency).await?;

        // Deduplicate identical media across bundles into a shared store, so the
        // published site serves one copy of a reposted file. Runs before the size
        // report so the About page reflects the deduped footprint.
        if s.dedup {
            match dedup::run(&s.site, &s.base_url) {
                Ok(st) if st.files_removed > 0 => info!(
                    "dedup: merged {} duplicate file(s) across {} group(s), saved {}",
                    st.files_removed,
                    st.groups,
                    site::human_size(st.bytes_saved)
                ),
                Ok(_) => {}
                Err(e) => info!("dedup: skipped ({e})"),
            }
        }
    } else {
        info!("--no-media: skipping downloads");
    }

    // Record the on-disk footprint (total + per-kind + share of the host limit)
    // on the About page, after downloads so it's the real size.
    let breakdown = site::size_breakdown(&[&s.site.join("content"), &s.site.join("static")]);
    let limit = site::pages_limit(&s.base_url, s.pages_host.as_deref()).map(|l| l.bytes);
    let biggest = site::largest_files(&[&s.site.join("content"), &s.site.join("static")], 10);
    // Post text keyed by slug → the hover tooltip on each largest-file link.
    let preview_by_slug: std::collections::HashMap<String, String> = posts
        .iter()
        .map(|p| (render::slug_for(p), render::post_preview(p)))
        .collect();
    // Video offloaded to GitHub Releases lives outside the published tree and
    // doesn't count against the Pages quota — measured separately so the About
    // page can report it on its own line.
    let releases = site::size_breakdown(&[&s.site.join(".video-releases")]).total();
    let now = chrono::Utc::now().format("%Y-%m-%d %H:%M UTC").to_string();
    let ci_url = ci_job_url();
    site::set_about_size(
        &s.site,
        &breakdown,
        limit,
        started.elapsed(),
        &i18n::about(&s.language),
        &site::LargestFiles { files: &biggest, previews: &preview_by_slug },
        mtproto_used,
        &now,
        ci_url.as_deref(),
        releases,
    );

    // Optional Google Lighthouse scores for the deployed site → About page +
    // shields.io badge endpoints under static/. Best-effort; it measures the
    // currently-live deploy, so scores lag a run behind. The placeholder is
    // always cleared, so a disabled/failed fetch never shows literal text.
    let pagespeed_scores = if s.pagespeed && s.base_url.starts_with("http") {
        info!("PageSpeed: fetching Lighthouse scores for {}", s.base_url);
        pagespeed::fetch(&client, &s.base_url).await
    } else {
        None
    };
    if let Some(sc) = &pagespeed_scores {
        let summary = sc
            .entries()
            .iter()
            .map(|(n, v)| format!("{n} {v}"))
            .collect::<Vec<_>>()
            .join(", ");
        info!("PageSpeed (mobile): {summary}");
        if let Err(e) = site::write_pagespeed_badges(&s.site, sc) {
            tracing::warn!("PageSpeed: writing badge endpoints failed: {e:#}");
        }
    }
    site::set_about_pagespeed(&s.site, pagespeed_scores, &i18n::about(&s.language));

    // Enrich the About page from an about.me link in the channel description
    // (bio + social links + a contact button). Best-effort; clears the placeholder
    // when disabled / absent / unreachable.
    let about_me = match s.about_me.then(|| {
        channel_info
            .as_ref()
            .and_then(|i| i.description_md.as_deref())
            .and_then(aboutme::url_in)
    }) {
        Some(Some(url)) => {
            info!("about.me: enriching the About page from {url}");
            aboutme::fetch(&client, &url).await
        }
        _ => None,
    };
    // Download the full profile photo into the bundle (base-aware static path).
    let about_me_photo = match about_me.as_ref().and_then(|a| a.image.as_deref()) {
        Some(img) if s.download_media => {
            let file = format!("aboutme.{}", media::ext_from_url(img, "jpg"));
            let job = media::Job {
                url: img.to_string(),
                dest: s.site.join("static").join(&file),
                force: false,
                local: None,
            };
            media::download_all(&client, &[job], 1).await.ok().map(|()| file)
        }
        _ => None,
    };
    site::set_about_me(
        &s.site,
        about_me.as_ref(),
        about_me_photo.as_deref(),
        s.aboutme_bio,
        s.aboutme_both_images,
    );
    site::set_about_wikidata(&s.site, about_wikidata_html.as_deref());

    // Podcast feed (opt-in): the channel's audio posts as an iTunes RSS feed.
    // Needs absolute URLs, so it's skipped for a relative base_url.
    if s.podcast && s.base_url.starts_with("http") {
        let episodes = podcast_episodes(&posts, &rendered, &s);
        if episodes.is_empty() {
            info!("podcast: no matching audio posts — skipping podcast.xml");
        } else {
            let cover = about_me
                .as_ref()
                .and_then(|a| a.image.clone())
                .or_else(|| podcast_cover(&posts, &rendered, &s.base_url));
            let description = posts
                .iter()
                .find(|p| p.tags.iter().any(|t| t == "podcast_description"))
                .map(render::post_text_plain)
                .filter(|d| !d.is_empty())
                .or_else(|| channel_info.as_ref().and_then(|i| i.description_md.clone()))
                .unwrap_or_default();
            let title = channel_info
                .as_ref()
                .and_then(|i| i.title.clone())
                .unwrap_or_else(|| s.title.clone());
            let ch = podcast::Channel {
                author: title.clone(),
                title,
                description,
                link: s.base_url.clone(),
                cover: cover.unwrap_or_default(),
                language: s.language.clone(),
                category: "Personal Journals".into(),
            };
            match std::fs::write(s.site.join("static/podcast.xml"), podcast::feed(&ch, &episodes)) {
                Ok(()) => info!("podcast: wrote podcast.xml with {} episode(s)", episodes.len()),
                Err(e) => tracing::warn!("podcast: writing podcast.xml failed: {e}"),
            }
        }
    }

    // Total on-disk footprint + per-kind breakdown (also shown on the About page).
    info!(
        "total size: {} — images {}, videos {}, audio {}, text {}, other {}",
        site::human_size(breakdown.total()),
        site::human_size(breakdown.images),
        site::human_size(breakdown.videos),
        site::human_size(breakdown.audio),
        site::human_size(breakdown.text),
        site::human_size(breakdown.other),
    );

    // Show the 10 biggest files (descending) — what's eating the hosting budget.
    // Each is a file:// link to the post's built page so it opens in a browser
    // (the HTML exists after `zola build`), with the owning post's full text
    // printed beneath it so the log alone says what the file is.
    if !biggest.is_empty() {
        let site_abs = std::fs::canonicalize(&s.site).unwrap_or_else(|_| s.site.clone());
        let text_by_slug: std::collections::HashMap<String, String> = posts
            .iter()
            .map(|p| (render::slug_for(p), render::post_text_plain(p)))
            .collect();
        info!("10 largest files:");
        for (p, sz) in &biggest {
            let rel = p.strip_prefix(&s.site).unwrap_or(p);
            info!("  {:>9}  {}", site::human_size(*sz), largest_link(&site_abs, rel));
            if let Some(text) = largest_slug(rel).and_then(|slug| text_by_slug.get(slug)) {
                if !text.is_empty() {
                    let indented = text
                        .lines()
                        .map(|l| format!("             {l}"))
                        .collect::<Vec<_>>()
                        .join("\n");
                    info!("{indented}");
                }
            }
        }
    }

    info!("done — Zola site at {}", s.site.display());
    info!("build it with:  zola --root {} build", s.site.display());
    Ok(())
}

/// URL of the CI job that produced this build, if running in a recognized CI, so
/// the About page's "last updated" line can link to the build log. GitLab
/// exposes the job URL directly; GitHub Actions assembles it from the run id.
fn ci_job_url() -> Option<String> {
    if let Ok(url) = std::env::var("CI_JOB_URL") {
        if !url.is_empty() {
            return Some(url);
        }
    }
    let server = std::env::var("GITHUB_SERVER_URL").ok().filter(|v| !v.is_empty())?;
    let repo = std::env::var("GITHUB_REPOSITORY").ok().filter(|v| !v.is_empty())?;
    let run = std::env::var("GITHUB_RUN_ID").ok().filter(|v| !v.is_empty())?;
    Some(format!("{server}/{repo}/actions/runs/{run}"))
}

/// Count how many posts use each tag, sorted by count (descending), then name.
fn count_tags(posts: &[model::Post]) -> Vec<(String, usize)> {
    let mut counts: std::collections::HashMap<&str, usize> = std::collections::HashMap::new();
    for p in posts {
        for t in &p.tags {
            *counts.entry(t.as_str()).or_insert(0) += 1;
        }
    }
    let mut v: Vec<(String, usize)> = counts.into_iter().map(|(k, n)| (k.to_string(), n)).collect();
    v.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    v
}

/// A `file://` link to the built page that owns a bundle file, so the
/// largest-files log is clickable. A post bundle `content/posts/<date>-<id>/…`
/// maps to `public/posts/<id>/index.html`; anything else links to the file.
fn largest_link(site_abs: &std::path::Path, rel: &std::path::Path) -> String {
    let comps: Vec<&str> = rel
        .components()
        .filter_map(|c| c.as_os_str().to_str())
        .collect();
    if comps.len() >= 3 && comps[0] == "content" && comps[1] == "posts" {
        if let Some(id) = comps[2].rsplit('-').next() {
            let html = site_abs.join("public/posts").join(id).join("index.html");
            return format!("file://{}", html.display());
        }
    }
    format!("file://{}", site_abs.join(rel).display())
}

/// The post slug owning a bundle file (`content/posts/<slug>/…`), if any — used
/// to look up the post's text for the largest-files log.
fn largest_slug(rel: &std::path::Path) -> Option<&str> {
    let comps: Vec<&str> = rel.components().filter_map(|c| c.as_os_str().to_str()).collect();
    if comps.len() >= 3 && comps[0] == "content" && comps[1] == "posts" {
        Some(comps[2])
    } else {
        None
    }
}

/// Ensure a base URL ends with `/` so `posts/…` appends cleanly.
fn ensure_slash(base: &str) -> String {
    if base.ends_with('/') {
        base.to_string()
    } else {
        format!("{base}/")
    }
}

fn is_audio_file(name: &str) -> bool {
    let n = name.to_ascii_lowercase();
    [".mp3", ".ogg", ".oga", ".m4a", ".opus", ".wav", ".flac", ".aac"].iter().any(|e| n.ends_with(e))
}

fn is_image_file(name: &str) -> bool {
    let n = name.to_ascii_lowercase();
    [".jpg", ".jpeg", ".png", ".webp", ".gif", ".avif"].iter().any(|e| n.ends_with(e))
}

fn audio_mime(name: &str) -> &'static str {
    let n = name.to_ascii_lowercase();
    if n.ends_with(".mp3") {
        "audio/mpeg"
    } else if n.ends_with(".m4a") || n.ends_with(".aac") {
        "audio/mp4"
    } else if n.ends_with(".wav") {
        "audio/wav"
    } else if n.ends_with(".flac") {
        "audio/flac"
    } else {
        "audio/ogg"
    }
}

/// The channel's audio posts as podcast episodes (base_url-aware enclosure URLs +
/// on-disk file size). With `podcast_tagged`, only posts tagged `podcast` qualify.
fn podcast_episodes(
    posts: &[model::Post],
    rendered: &[render::RenderedPost],
    s: &config::Settings,
) -> Vec<podcast::Episode> {
    let base = ensure_slash(&s.base_url);
    posts
        .iter()
        .zip(rendered)
        .filter_map(|(post, r)| {
            if s.podcast_tagged && !post.tags.iter().any(|t| t == "podcast") {
                return None;
            }
            let audio = r.downloads.iter().find(|d| is_audio_file(&d.filename))?;
            let id = post.primary_id;
            let path = s.site.join("content/posts").join(&r.slug).join(&audio.filename);
            let length = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
            let derived = render::post_title(post, s.title_max_len, true);
            let title = if derived.is_empty() { format!("#{id}") } else { derived };
            Some(podcast::Episode {
                title,
                description: render::post_text_plain(post),
                url: format!("{base}posts/{id}/{}", audio.filename),
                mime: audio_mime(&audio.filename).to_string(),
                length,
                date: post.date,
                guid: format!("{base}posts/{id}/"),
            })
        })
        .collect()
}

/// Cover URL from the first image of a post tagged `podcast_description`, if any.
fn podcast_cover(
    posts: &[model::Post],
    rendered: &[render::RenderedPost],
    base_url: &str,
) -> Option<String> {
    let base = ensure_slash(base_url);
    posts.iter().zip(rendered).find_map(|(post, r)| {
        if !post.tags.iter().any(|t| t == "podcast_description") {
            return None;
        }
        let img = r.downloads.iter().find(|d| is_image_file(&d.filename))?;
        Some(format!("{base}posts/{}/{}", post.primary_id, img.filename))
    })
}

fn http_client() -> Result<reqwest::Client> {
    reqwest::Client::builder()
        .user_agent(
            "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 \
             (KHTML, like Gecko) Chrome/120.0 Safari/537.36",
        )
        .timeout(std::time::Duration::from_secs(120))
        .build()
        .context("building HTTP client")
}

fn init_tracing(level: &str) {
    use tracing_subscriber::{fmt, EnvFilter};
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(format!("tg2zola={level},info")));
    fmt()
        .with_env_filter(filter)
        .with_target(false)
        .without_time()
        .init();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn search_default_is_elasticlunr() {
        // No engine and no custom URL → the self-contained default.
        assert!(matches!(resolve_search(None, None, "/"), Search::Elasticlunr));
        for name in ["elasticlunr", "lunr", "local"] {
            assert!(
                matches!(resolve_search(Some(name.into()), None, "/"), Search::Elasticlunr),
                "{name}"
            );
        }
    }

    #[test]
    fn search_engines_and_overrides() {
        // Google keeps the host for its `site:` filter.
        match resolve_search(Some("google".into()), None, "https://ex.github.io/") {
            Search::Google { site } => assert_eq!(site.as_deref(), Some("ex.github.io")),
            other => panic!("expected Google, got {other:?}"),
        }
        // A custom URL wins over any engine name.
        match resolve_search(Some("google".into()), Some("https://s/?q=".into()), "/") {
            Search::Custom { url } => assert_eq!(url, "https://s/?q="),
            other => panic!("expected Custom, got {other:?}"),
        }
        // Named web engines resolve to a Custom URL.
        assert!(matches!(
            resolve_search(Some("duckduckgo".into()), None, "/"),
            Search::Custom { .. }
        ));
        // Explicit off, and any unknown engine, disable the box.
        assert!(matches!(resolve_search(Some("none".into()), None, "/"), Search::None));
        assert!(matches!(resolve_search(Some("nonsense".into()), None, "/"), Search::None));
    }

    #[test]
    fn host_extraction() {
        assert_eq!(host_of("https://user.github.io/repo/").as_deref(), Some("user.github.io"));
        assert_eq!(host_of("http://example.com").as_deref(), Some("example.com"));
        assert_eq!(host_of("/"), None); // relative (offline build)
        assert_eq!(host_of("nothost"), None); // no dot → not a host
    }
}
