//! tg2zola — back up a public Telegram channel into a self-contained Zola site.
//!
//! Pipeline: scrape `t.me/s/<channel>` → group messages into posts → download
//! media into a cache → regenerate every Zola page bundle. The generated site
//! references only local files and YouTube — never Telegram — so it survives the
//! channel being removed.

mod config;
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

use config::{FileConfig, Settings};

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

    /// Extra pages as Markdown, each section starting with a `# Title` heading
    /// (becomes a page + nav entry). In CI this comes from the PAGES variable.
    #[arg(long)]
    pages: Option<String>,

    /// Full posts per page on the home feed (default 20).
    #[arg(long)]
    posts_per_page: Option<usize>,

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

    Ok(Settings {
        title: g.title.clone().or(fc.title).unwrap_or_else(|| channel.clone()),
        description: g.description.clone().or(fc.description).unwrap_or_default(),
        base_url: g
            .base_url
            .clone()
            .or(fc.base_url)
            .unwrap_or_else(|| "/".to_string()),
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
        pages: g.pages.clone().or(fc.pages).filter(|s| !s.trim().is_empty()),
        posts_per_page: g
            .posts_per_page
            .or(fc.posts_per_page)
            .filter(|&n| n > 0)
            .unwrap_or(20),
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

async fn run(mut s: Settings, init_site: bool) -> Result<()> {
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

    if init_site {
        site::scaffold(&s, channel_info.as_ref(), &tag_counts)?;
    }

    let rewriter = render::LinkRewriter::new(&s.channel, &posts);
    // Posts are id-ascending; neighbour id+title drive the Next/Prev nav.
    let titles: Vec<String> = posts.iter().map(render::post_title).collect();
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
            render::render_post(p, &rewriter, newer, older)
        })
        .collect();

    // Write bundles (dirs + index.md) and prune removed posts / stale files
    // first, so downloads land in existing dirs and reuse already-cached files.
    site::write_site(&s, &rendered)?;

    if s.download_media {
        let posts_dir = s.site.join("content/posts");
        let jobs: Vec<media::Job> = rendered
            .iter()
            .flat_map(|r| {
                let dir = posts_dir.join(&r.slug);
                r.downloads.iter().map(move |d| media::Job {
                    url: d.url.clone(),
                    dest: dir.join(&d.filename),
                    force: d.force,
                })
            })
            .collect();
        info!("{} media references across posts", jobs.len());
        media::download_all(&client, &jobs, s.concurrency).await?;
    } else {
        info!("--no-media: skipping downloads");
    }
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
