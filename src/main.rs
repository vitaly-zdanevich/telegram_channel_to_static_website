//! tg2zola — back up a public Telegram channel into a self-contained Zola site.
//!
//! Pipeline: scrape `t.me/s/<channel>` → group messages into posts → download
//! media into a cache → regenerate every Zola page bundle. The generated site
//! references only local files and YouTube — never Telegram — so it survives the
//! channel being removed.

mod config;
mod genius;
mod group;
mod html2md;
mod media;
mod model;
mod offline;
mod parse;
mod render;
mod scrape;
mod site;

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
}

/// Scrape + regenerate the Zola site (the default action when no subcommand).
#[derive(Args, Debug)]
struct GenerateArgs {
    /// Optional TOML config file (CLI flags override its values).
    #[arg(long)]
    config: Option<PathBuf>,

    /// Public channel username, e.g. `vitaly_zdanevich_chan` (without @).
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

    /// Underline links (default: no underline).
    #[arg(long)]
    link_underline: bool,

    /// Enable the CSS click-to-load YouTube facade (default: direct iframe).
    #[arg(long)]
    youtube_facade: bool,

    /// Don't resolve genius.com links (skip fetching their pages for a YouTube
    /// video / lyrics widget).
    #[arg(long)]
    no_genius: bool,

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
        link_underline: g.link_underline || fc.link_underline.unwrap_or(false),
        youtube_facade: g.youtube_facade || fc.youtube_facade.unwrap_or(false),
        genius: if g.no_genius {
            false
        } else {
            fc.genius.unwrap_or(true)
        },
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
/// query). Otherwise the engine name (default `google`) selects a built-in:
/// Google is a JS-free form; the rest get an Enter handler with `site:<host>`
/// folded into the query. `none`/`off` disables it.
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
        .unwrap_or_else(|| "google".to_string());
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
        other => {
            tracing::warn!("unknown search engine '{other}' — disabling the search box");
            Search::None
        }
    }
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

    // Drop posts with nothing worth showing (e.g. a lone non-downloadable file).
    let before = posts.len();
    posts.retain(|p| !render::is_empty_post(p));
    if before != posts.len() {
        info!("skipped {} empty post(s)", before - posts.len());
    }

    // Resolve genius.com links into the YouTube video they reference (+ song id).
    if s.genius {
        genius::enrich(&client, &mut posts, s.concurrency).await;
    }

    // Auto-tag posts that have a playable (downloadable) video with #video,
    // unless the author already tagged it.
    for p in &mut posts {
        let has_video = p
            .media
            .iter()
            .any(|m| matches!(m, model::Media::Video { .. }));
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
            };
            media::download_all(&client, &[job], 1).await?;
        }
    }

    let rewriter = render::LinkRewriter::new(&s.channel, &posts);

    // Render PAGE posts first so their nav entries are ready for scaffolding.
    let rendered_pages: Vec<render::RenderedPost> = page_posts
        .iter()
        .map(|p| render::render_post(p, &rewriter, s.title_max_len, true, None, None))
        .collect();
    let page_nav: Vec<(String, String)> = rendered_pages
        .iter()
        .map(|r| (r.title.clone(), r.slug.clone()))
        .collect();

    if init_site {
        site::scaffold(&s, channel_info.as_ref(), &tag_counts, &page_nav)?;
    }

    // Posts are id-ascending; neighbour id+title drive the Next/Prev nav.
    let titles: Vec<String> = posts
        .iter()
        .map(|p| render::post_title(p, s.title_max_len))
        .collect();
    let rendered: Vec<render::RenderedPost> = posts
        .iter()
        .enumerate()
        .map(|(i, p)| {
            let newer = posts
                .get(i + 1)
                .map(|q| (q.primary_id, titles[i + 1].as_str()));
            let older = i
                .checked_sub(1)
                .map(|j| (posts[j].primary_id, titles[j].as_str()));
            render::render_post(p, &rewriter, s.title_max_len, false, newer, older)
        })
        .collect();

    // Write bundles (dirs + index.md) and prune removed posts / stale files
    // first, so downloads land in existing dirs and reuse already-cached files.
    site::write_site(&s, &rendered)?;
    site::write_pages(&s, &rendered_pages)?;

    if s.download_media {
        let job_for = |dir: std::path::PathBuf| {
            move |d: &render::Download| media::Job {
                url: d.url.clone(),
                dest: dir.join(&d.filename),
                force: d.force,
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
    } else {
        info!("--no-media: skipping downloads");
    }

    // Record the on-disk footprint (total + per-kind + share of the host limit)
    // on the About page, after downloads so it's the real size.
    let breakdown = site::size_breakdown(&[&s.site.join("content"), &s.site.join("static")]);
    let limit = site::pages_limit(&s.base_url, s.pages_host.as_deref()).map(|l| l.bytes);
    site::set_about_size(&s.site, &breakdown, limit, started.elapsed());

    info!("done — Zola site at {}", s.site.display());
    info!("build it with:  zola --root {} build", s.site.display());
    Ok(())
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
