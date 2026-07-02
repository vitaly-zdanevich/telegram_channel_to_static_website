//! Scaffold the Zola site (config + templates) and reconcile generated posts on
//! every run. Config/templates are regenerated deterministically; when a theme
//! is configured the built-in templates are removed so the theme drives the
//! look. Media lives in the post bundles — the bundle (committed to the `blog`
//! branch) is the cache.

use anyhow::{Context, Result};
use std::collections::HashSet;
use std::fs;
use std::path::Path;

use crate::config::{Search, Settings};
use crate::model::ChannelInfo;
use crate::render::RenderedPost;

/// (Re)write config + content indexes, and either install our built-in
/// templates (default) or clear them so an external theme takes over.
pub fn scaffold(
    s: &Settings,
    info: Option<&ChannelInfo>,
    tags: &[(String, usize)],
    page_nav: &[(String, String)],
    days: &[DayMeta],
) -> Result<()> {
    let site = &s.site;
    fs::create_dir_all(site.join("templates/shortcodes"))?;
    fs::create_dir_all(site.join("content/posts"))?;
    fs::create_dir_all(site.join("static"))?;
    fs::create_dir_all(site.join("themes"))?;

    let pages = parse_pages(s.pages.as_deref());

    // Remove stray top-level content files from older layouts (About and custom
    // pages now live under content/pages/), which would otherwise collide.
    if let Ok(entries) = fs::read_dir(site.join("content")) {
        for e in entries.flatten() {
            let p = e.path();
            let is_md = p.extension().is_some_and(|x| x == "md");
            let keep = p.file_name().and_then(|n| n.to_str()) == Some("_index.md");
            if is_md && !keep {
                let _ = fs::remove_file(p);
            }
        }
    }

    // Regenerated every run (deterministic) so theme switches take effect.
    write_file(&site.join("config.toml"), &config_toml(s, &pages, tags, page_nav, days))?;
    write_file(&site.join("content/_index.md"), &root_index_md(s))?;
    write_file(&site.join("content/posts/_index.md"), &posts_index_md(s))?;
    // Static pages live in a non-rendered subsection so they don't appear in the
    // homepage post feed; `path` keeps them at /about/, /<slug>/.
    fs::create_dir_all(site.join("content/pages"))?;
    write_file(&site.join("content/pages/_index.md"), "+++\nrender = false\n+++\n")?;
    write_file(&site.join("content/pages/about.md"), &about_md(s, info))?;
    // The calendar page (a year/month grid linking to the /day/<date>/ pages).
    if s.theme.is_none() && !days.is_empty() {
        write_file(&site.join("content/pages/calendar.md"), &calendar_md(s, days))?;
    }
    for p in &pages {
        write_file(&site.join(format!("content/pages/{}.md", p.slug)), &page_md(s, p))?;
    }
    // Per-tag "full posts" pages at /tags/<slug>/full/ (the number on the Tags
    // page links here; the tag name links to the titles-only term page). Only
    // with the built-in templates (a theme has no tag_full.html).
    let tags_full = site.join("content/tags-full");
    let _ = fs::remove_dir_all(&tags_full);
    if s.theme.is_none() && !tags.is_empty() {
        write_file(&tags_full.join("_index.md"), "+++\nrender = false\n+++\n")?;
        for (name, _) in tags {
            let slug = slugify(name);
            let md = format!(
                "+++\ntitle = \"#{}\"\npath = \"/tags/{}/full/\"\ntemplate = \"tag_full.html\"\n\n[extra]\ntag = \"{}\"\n+++\n",
                toml_escape(name),
                slug,
                toml_escape(name)
            );
            write_file(&tags_full.join(&slug).join("index.md"), &md)?;
        }
    }
    // Per-day "full posts" pages at /day/<date>/ — every post's date links here.
    // They render that day's posts in full, with prev/next-day navigation. `days`
    // is sorted ascending, so the next entry is the newer day. Built-in only.
    let days_full = site.join("content/days-full");
    let _ = fs::remove_dir_all(&days_full);
    if s.theme.is_none() && !days.is_empty() {
        write_file(&days_full.join("_index.md"), "+++\nrender = false\n+++\n")?;
        for (i, d) in days.iter().enumerate() {
            let day = &d.day;
            let mut extra = format!("day = \"{day}\"\n");
            if let Some(n) = days.get(i + 1) {
                extra.push_str(&format!("newer_day = \"{}\"\n", n.day));
            }
            if let Some(o) = i.checked_sub(1).and_then(|j| days.get(j)) {
                extra.push_str(&format!("older_day = \"{}\"\n", o.day));
            }
            let md = format!(
                "+++\ntitle = \"{day}\"\npath = \"/day/{day}/\"\ntemplate = \"day_full.html\"\n\n[extra]\n{extra}+++\n"
            );
            write_file(&days_full.join(day).join("index.md"), &md)?;
        }
    }
    // Always provide our YouTube shortcode (project shortcodes override the
    // theme's), so generated `{{ youtube(...) }}` always resolves.
    write_file(&site.join("templates/shortcodes/youtube.html"), YOUTUBE_SHORTCODE)?;
    write_file(
        &site.join("templates/shortcodes/apple_podcast.html"),
        APPLE_PODCAST_SHORTCODE,
    )?;
    write_file(
        &site.join("templates/shortcodes/yandex_music.html"),
        YANDEX_MUSIC_SHORTCODE,
    )?;
    write_file(
        &site.join("templates/shortcodes/instagram.html"),
        INSTAGRAM_SHORTCODE,
    )?;
    write_file(&site.join("templates/shortcodes/video.html"), VIDEO_SHORTCODE)?;
    write_file(&site.join("templates/shortcodes/audio.html"), AUDIO_SHORTCODE)?;
    write_file(&site.join("templates/shortcodes/tag.html"), TAG_SHORTCODE)?;
    write_file(&site.join("templates/shortcodes/avatar.html"), AVATAR_SHORTCODE)?;

    let builtins = [
        ("templates/base.html", BASE_HTML),
        ("templates/index.html", INDEX_HTML),
        ("templates/section.html", SECTION_HTML),
        ("templates/page.html", PAGE_HTML),
        ("templates/tags/single.html", TAGS_SINGLE),
        ("templates/tags/list.html", TAGS_LIST),
        ("templates/tag_full.html", TAG_FULL_HTML),
        ("templates/day_full.html", DAY_FULL_HTML),
        ("templates/calendar.html", CALENDAR_HTML),
    ];
    if s.theme.is_none() {
        fs::create_dir_all(site.join("templates/tags"))?;
        for (path, content) in builtins {
            write_file(&site.join(path), content)?;
        }
        write_file(&site.join("static/style.css"), &style_css(s))?;
    } else {
        // Theme mode: remove our built-ins so they don't shadow the theme.
        for (path, _) in builtins {
            let _ = fs::remove_file(site.join(path));
        }
        let _ = fs::remove_dir_all(site.join("templates/tags"));
        let _ = fs::remove_file(site.join("static/style.css"));
    }
    Ok(())
}

/// Reconcile `content/posts`: (re)write every `index.md`, keep already-present
/// media (the cache), and prune posts that no longer exist plus stale media
/// left from a previous version of a post. Media is downloaded afterwards.
pub fn write_site(s: &Settings, rendered: &[RenderedPost]) -> Result<()> {
    let posts_dir = s.site.join("content/posts");
    fs::create_dir_all(&posts_dir)?;
    write_if_absent(&posts_dir.join("_index.md"), &posts_index_md(s))?;

    // Prune post directories that no longer exist on the channel.
    let current: HashSet<&str> = rendered.iter().map(|r| r.slug.as_str()).collect();
    for entry in fs::read_dir(&posts_dir)? {
        let entry = entry?;
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if name == "_index.md" {
            continue;
        }
        if entry.path().is_dir() && !current.contains(name.as_ref()) {
            fs::remove_dir_all(entry.path())?;
        }
    }

    for r in rendered {
        let dir = posts_dir.join(&r.slug);
        fs::create_dir_all(&dir)?;
        fs::write(dir.join("index.md"), &r.index_md)
            .with_context(|| format!("writing {}", dir.join("index.md").display()))?;

        // Keep index.md + the media this post currently references; remove the
        // rest (e.g. media left over from a previous edit of the post).
        let mut keep: HashSet<String> = r.downloads.iter().map(|d| d.filename.clone()).collect();
        keep.insert("index.md".to_string());
        for e in fs::read_dir(&dir)? {
            let e = e?;
            let n = e.file_name().to_string_lossy().into_owned();
            if !keep.contains(&n) {
                let _ = fs::remove_file(e.path());
            }
        }
    }
    tracing::info!("wrote {} post bundles", rendered.len());
    Ok(())
}

/// Write PAGE-marked posts as page bundles under content/pages/<slug>/ (media
/// alongside). Other content/pages entries (About, custom pages) are untouched.
pub fn write_pages(s: &Settings, pages: &[RenderedPost]) -> Result<()> {
    let dir = s.site.join("content/pages");
    for p in pages {
        let bundle = dir.join(&p.slug);
        fs::create_dir_all(&bundle)?;
        fs::write(bundle.join("index.md"), &p.index_md)
            .with_context(|| format!("writing {}", bundle.join("index.md").display()))?;
        let mut keep: HashSet<String> = p.downloads.iter().map(|d| d.filename.clone()).collect();
        keep.insert("index.md".to_string());
        if let Ok(rd) = fs::read_dir(&bundle) {
            for e in rd.flatten() {
                let n = e.file_name().to_string_lossy().into_owned();
                if !keep.contains(&n) {
                    let _ = fs::remove_file(e.path());
                }
            }
        }
    }
    if !pages.is_empty() {
        tracing::info!("wrote {} PAGE bundle(s)", pages.len());
    }
    Ok(())
}

fn write_file(path: &Path, contents: &str) -> Result<()> {
    if let Some(p) = path.parent() {
        fs::create_dir_all(p)?;
    }
    fs::write(path, contents).with_context(|| format!("writing {}", path.display()))
}

fn write_if_absent(path: &Path, contents: &str) -> Result<()> {
    if path.exists() {
        return Ok(());
    }
    write_file(path, contents)
}

fn config_toml(
    s: &Settings,
    pages: &[Page],
    tags: &[(String, usize)],
    page_nav: &[(String, String)],
    days: &[DayMeta],
) -> String {
    let theme_line = match &s.theme {
        Some(t) => format!("theme = \"{}\"\n", toml_escape(t)),
        None => String::new(),
    };
    // Zola's built-in RSS 2.0 feed. No feed_limit: include every post, full
    // text — the feed is a complete archive, not just a recent-items teaser.
    let feeds = if s.rss {
        "generate_feeds = true\nfeed_filenames = [\"rss.xml\"]"
    } else {
        "generate_feeds = false"
    };
    let tags_toml = if tags.is_empty() {
        String::new()
    } else {
        let items: Vec<String> = tags
            .iter()
            .map(|(n, c)| {
                format!(
                    "{{ name = \"{}\", count = {}, slug = \"{}\" }}",
                    toml_escape(n),
                    c,
                    toml_escape(&slugify(n))
                )
            })
            .collect();
        format!("tags = [{}]", items.join(", "))
    };
    let avatar = if s.site.join("static/channel-avatar.jpg").exists() {
        "avatar = \"channel-avatar.jpg\""
    } else {
        ""
    };
    // Mastodon: the creator handle drives `fediverse:creator` (author byline on
    // link previews) and a derived `rel="me"` profile link (profile verification).
    let fedi = match &s.fediverse_creator {
        Some(h) => {
            let mut out = format!("fediverse_creator = \"{}\"", toml_escape(h));
            if let Some(url) = fediverse_profile_url(h) {
                out.push_str(&format!("\nfediverse_profile = \"{}\"", toml_escape(&url)));
            }
            out
        }
        None => String::new(),
    };
    let search = match &s.search {
        Search::None => String::new(),
        Search::Google { site } => {
            let mut o = String::from("search_google = true");
            if let Some(h) = site {
                o.push_str(&format!("\nsearch_site = \"{}\"", toml_escape(h)));
            }
            o
        }
        Search::Custom { url } => format!("search_url = \"{}\"", toml_escape(url)),
    };
    // Footer may be multi-line Markdown/HTML, so escape newlines for the TOML
    // basic string; the template renders it via the `markdown` filter.
    let footer = match &s.footer {
        Some(f) => format!(
            "footer = \"{}\"",
            toml_escape(f).replace('\n', "\\n").replace('\r', "")
        ),
        None => String::new(),
    };
    let mut nav_items: Vec<String> = pages
        .iter()
        .map(|p| (p.title.as_str(), p.slug.as_str()))
        .chain(page_nav.iter().map(|(t, s)| (t.as_str(), s.as_str())))
        .map(|(title, slug)| format!("{{ title = \"{}\", path = \"/{}/\" }}", toml_escape(title), slug))
        .collect();
    let nav = if nav_items.is_empty() {
        String::new()
    } else {
        nav_items.dedup();
        format!("nav = [{}]", nav_items.join(", "))
    };
    // Tags surfaced in the top nav (TAGS_TO_PAGES), restricted to tags that
    // actually exist (case-insensitively) so get_taxonomy_url can't fail.
    let nav_tags = match &s.tags_to_pages {
        Some(input) => {
            let canon: std::collections::HashMap<String, &str> =
                tags.iter().map(|(n, _)| (n.to_lowercase(), n.as_str())).collect();
            let items: Vec<String> = input
                .split(',')
                .map(|t| t.trim().trim_start_matches('#').trim().to_lowercase())
                .filter(|t| !t.is_empty())
                .filter_map(|t| canon.get(&t).copied())
                .map(|t| format!("\"{}\"", toml_escape(t)))
                .collect();
            if items.is_empty() {
                String::new()
            } else {
                format!("nav_tags = [{}]", items.join(", "))
            }
        }
        None => String::new(),
    };
    // Localized UI strings for the chrome, injected as an [extra.i18n] sub-table.
    let u = crate::i18n::ui(&s.language);
    let i18n_block = [
        ("newer", u.newer),
        ("older", u.older),
        ("tags", u.tags),
        ("about", u.about),
        ("archive", u.archive),
        ("search", u.search),
        ("search_aria", u.search_aria),
        ("views", u.views),
        ("view_on_telegram", u.view_on_telegram),
        ("forwarded_from", u.forwarded_from),
        ("full_posts", u.full_posts),
        ("titles", u.titles),
        ("not_archived", u.not_archived),
        ("video", u.video),
        ("calendar", u.calendar),
        ("newer_day", u.newer_day),
        ("older_day", u.older_day),
    ]
    .iter()
    .map(|&(k, v)| format!("{k} = \"{}\"", toml_escape(v)))
    .collect::<Vec<_>>()
    .join("\n");

    CONFIG_TOML
        .replace("__LANGUAGE__", &toml_escape(&s.language))
        .replace("__DATE_LOCALE__", crate::i18n::date_locale(&s.language))
        .replace("__I18N__", &i18n_block)
        .replace("__BASE_URL__", &toml_escape(&s.base_url))
        .replace("__TITLE__", &toml_escape(&s.title))
        .replace("__DESC__", &toml_escape(&s.description))
        .replace("__THEME__", &theme_line)
        .replace("__FEEDS__", feeds)
        .replace("__RSS__", if s.rss { "true" } else { "false" })
        .replace("__CHANNEL__", &toml_escape(&s.channel))
        .replace("__DATE_FORMAT__", &toml_escape(&s.date_format))
        .replace("__TAGS_FOOTER__", if s.tags_footer { "true" } else { "false" })
        .replace("__NEXT_PREV__", if s.next_prev { "true" } else { "false" })
        .replace(
            "__TELEGRAM_LINK__",
            if s.telegram_link { "true" } else { "false" },
        )
        .replace("__YT_FACADE__", if s.youtube_facade { "true" } else { "false" })
        .replace("__CALENDAR__", if days.is_empty() { "false" } else { "true" })
        .replace("__FEDI__", &fedi)
        .replace("__SEARCH__", &search)
        .replace("__FOOTER__", &footer)
        .replace("__AVATAR__", avatar)
        .replace("__NAV__", &nav)
        .replace("__NAV_TAGS__", &nav_tags)
        .replace("__TAGS__", &tags_toml)
}

/// Derive a Mastodon profile URL from an `@user@instance.tld` handle, for the
/// `rel="me"` verification link. Returns `None` for a malformed handle.
fn fediverse_profile_url(handle: &str) -> Option<String> {
    let h = handle.trim().trim_start_matches('@');
    let (user, instance) = h.split_once('@')?;
    if user.is_empty() || !instance.contains('.') || instance.contains('/') {
        return None;
    }
    Some(format!("https://{instance}/@{user}"))
}

/// On-disk size split by file kind, for the About page.
#[derive(Default)]
pub struct SizeBreakdown {
    pub text: u64,
    pub images: u64,
    pub videos: u64,
    pub audio: u64,
    pub other: u64,
}

impl SizeBreakdown {
    pub fn total(&self) -> u64 {
        self.text + self.images + self.videos + self.audio + self.other
    }
}

/// Walk directory trees and total file sizes per kind (by extension).
pub fn size_breakdown(roots: &[&Path]) -> SizeBreakdown {
    let mut b = SizeBreakdown::default();
    for root in roots {
        accumulate(root, &mut b);
    }
    b
}

fn accumulate(path: &Path, b: &mut SizeBreakdown) {
    let Ok(entries) = fs::read_dir(path) else {
        return;
    };
    for e in entries.flatten() {
        let p = e.path();
        if p.is_dir() {
            accumulate(&p, b);
        } else if let Ok(m) = e.metadata() {
            let ext = p
                .extension()
                .and_then(|x| x.to_str())
                .unwrap_or("")
                .to_lowercase();
            let bucket = match ext.as_str() {
                "md" | "txt" => &mut b.text,
                "jpg" | "jpeg" | "png" | "webp" | "gif" | "svg" | "bmp" => &mut b.images,
                "mp4" | "webm" | "mov" | "mkv" | "avi" | "m4v" => &mut b.videos,
                "mp3" | "ogg" | "oga" | "opus" | "m4a" | "wav" | "flac" => &mut b.audio,
                _ => &mut b.other,
            };
            *bucket += m.len();
        }
    }
}

/// Human-readable byte size, e.g. `928 MB` / `1.4 GB`.
/// The `n` largest files under `roots`, descending by size — printed at the end
/// of a run so you can see what dominates the (size-limited) hosting budget.
pub fn largest_files(roots: &[&Path], n: usize) -> Vec<(std::path::PathBuf, u64)> {
    let mut files: Vec<(std::path::PathBuf, u64)> = Vec::new();
    for root in roots {
        collect_files(root, &mut files);
    }
    files.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    files.truncate(n);
    files
}

fn collect_files(path: &Path, out: &mut Vec<(std::path::PathBuf, u64)>) {
    let Ok(entries) = fs::read_dir(path) else {
        return;
    };
    for e in entries.flatten() {
        let p = e.path();
        if p.is_dir() {
            collect_files(&p, out);
        } else if let Ok(m) = e.metadata() {
            out.push((p, m.len()));
        }
    }
}

pub fn human_size(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
    let mut v = bytes as f64;
    let mut i = 0;
    while v >= 1024.0 && i < UNITS.len() - 1 {
        v /= 1024.0;
        i += 1;
    }
    if i == 0 {
        format!("{bytes} B")
    } else if v >= 100.0 {
        format!("{v:.0} {}", UNITS[i])
    } else {
        format!("{v:.1} {}", UNITS[i])
    }
}

/// A static host's published-site size limit, for the About page.
pub struct PagesLimit {
    pub name: &'static str,
    pub display: &'static str,
    pub bytes: u64,
    pub doc: &'static str,
}

/// Detect the static host (for the About-page size limit) from an explicit
/// override or the base URL (github.io / gitlab.io). Both GitHub and GitLab
/// Pages cap a published site at ~1 GB. None = unknown host (no limit shown).
pub fn pages_limit(base_url: &str, explicit: Option<&str>) -> Option<PagesLimit> {
    const GIB: u64 = 1024 * 1024 * 1024;
    let key = match explicit.map(|s| s.trim().to_lowercase()).filter(|s| !s.is_empty()) {
        Some(e) if matches!(e.as_str(), "none" | "off" | "no") => return None,
        Some(e) => e,
        None => base_url.to_lowercase(),
    };
    if key.contains("github") {
        Some(PagesLimit {
            name: "GitHub Pages",
            display: "1 GB",
            bytes: GIB,
            doc: "https://docs.github.com/en/pages/getting-started-with-github-pages/about-github-pages#usage-limits",
        })
    } else if key.contains("gitlab") {
        Some(PagesLimit {
            name: "GitLab Pages",
            display: "1 GB",
            bytes: GIB,
            doc: "https://docs.gitlab.com/ee/user/project/pages/",
        })
    } else {
        None
    }
}

/// Fill the About page's size placeholders: `__TOTAL_SIZE__` (total),
/// `__PERCENT__` (share of the host limit, if known) and `__SIZE_BREAKDOWN__`
/// (per-kind sizes). Computed after media download, so it's the real footprint.
pub fn set_about_size(
    site: &Path,
    b: &SizeBreakdown,
    limit: Option<u64>,
    elapsed: std::time::Duration,
    about: &crate::i18n::About,
    largest: &[(std::path::PathBuf, u64)],
) {
    let about_path = site.join("content/pages/about.md");
    let Ok(s) = fs::read_to_string(&about_path) else {
        return;
    };
    if !s.contains("__TOTAL_SIZE__") {
        return;
    }
    // The 10 biggest files as a list linking to each owning post.
    let largest_block = if largest.is_empty() {
        String::new()
    } else {
        let items = largest
            .iter()
            .map(|(p, sz)| about_largest_link(p.strip_prefix(site).unwrap_or(p), *sz))
            .collect::<Vec<_>>()
            .join("\n");
        format!("{}\n\n{items}", about.largest_files)
    };
    let total = b.total();
    let percent = limit
        .filter(|&m| m > 0)
        .map(|m| format!("{:.0}%", total as f64 / m as f64 * 100.0))
        .unwrap_or_default();
    // Per-kind sizes as a list, biggest first.
    let mut kinds: Vec<(&str, u64)> = [
        (about.kind_text, b.text),
        (about.kind_images, b.images),
        (about.kind_videos, b.videos),
        (about.kind_audio, b.audio),
        (about.kind_other, b.other),
    ]
    .into_iter()
    .filter(|(_, v)| *v > 0)
    .collect();
    kinds.sort_by(|a, b| b.1.cmp(&a.1));
    let breakdown = kinds
        .iter()
        .map(|(n, v)| format!("- **{n}** {}", human_size(*v)))
        .collect::<Vec<_>>()
        .join("\n");
    let out = s
        .replace("__TOTAL_SIZE__", &human_size(total))
        .replace("__PERCENT__", &percent)
        .replace("__SIZE_BREAKDOWN__", &breakdown)
        .replace("__LARGEST_FILES__", &largest_block)
        .replace("__BUILD_TIME__", &human_duration(elapsed));
    let _ = fs::write(&about_path, out);
}

/// A markdown list item for a largest-file entry, linking to the owning post
/// (`content/posts/<slug>/…` → `@/posts/<slug>/index.md`, base_url-aware). Files
/// outside a post bundle are shown without a link.
fn about_largest_link(rel: &Path, size: u64) -> String {
    let comps: Vec<&str> = rel
        .components()
        .filter_map(|c| c.as_os_str().to_str())
        .collect();
    let fname = comps.last().copied().unwrap_or("file");
    if comps.len() >= 3 && comps[0] == "content" && comps[1] == "posts" {
        format!(
            "- [{} — {fname}](@/posts/{}/index.md)",
            human_size(size),
            comps[2]
        )
    } else {
        format!("- {} — {fname}", human_size(size))
    }
}

/// Human-readable duration, e.g. `2m 30s` / `45s`.
fn human_duration(d: std::time::Duration) -> String {
    let s = d.as_secs();
    if s >= 60 {
        format!("{}m {}s", s / 60, s % 60)
    } else {
        format!("{s}s")
    }
}

/// Homepage = the paginated full-posts feed (posts bubble up from the
/// `transparent` posts section). Zola emits a `page/1/` redirect stub with a
/// <script>; the offline pass strips it so the output stays JS-free.
fn root_index_md(s: &Settings) -> String {
    let mut o = format!("+++\nsort_by = \"date\"\npaginate_by = {}\n", s.posts_per_page);
    if s.theme.is_none() {
        o.push_str("template = \"index.html\"\n");
    }
    o.push_str("+++\n");
    o
}

/// Per-day data for the day pages + calendar: the date, how many posts, and the
/// union of those posts' tags (unique, sorted).
pub struct DayMeta {
    pub day: String,
    pub count: usize,
    pub tags: Vec<String>,
}

/// The `/calendar/` page: one month grid per month that has posts, grouped by
/// year (newest first). Days with posts link to their `/day/<date>/` page; month
/// and weekday names follow LANGUAGE. `days` is "YYYY-MM-DD" sorted ascending.
fn calendar_md(s: &Settings, days: &[DayMeta]) -> String {
    use chrono::{Datelike, NaiveDate};
    let locale =
        chrono::Locale::try_from(crate::i18n::date_locale(&s.language)).unwrap_or(chrono::Locale::en_US);
    let present: std::collections::HashMap<&str, &DayMeta> =
        days.iter().map(|d| (d.day.as_str(), d)).collect();
    let dates: Vec<NaiveDate> = days
        .iter()
        .filter_map(|d| NaiveDate::parse_from_str(&d.day, "%Y-%m-%d").ok())
        .collect();
    let mut years: Vec<i32> = dates.iter().map(|d| d.year()).collect();
    years.sort();
    years.dedup();
    years.reverse();

    // Localized Mon..Sun headers (2024-01-06 is a Monday).
    let monday = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
    let weekdays: Vec<String> = (0..7)
        .map(|i| {
            (monday + chrono::Duration::days(i))
                .format_localized("%a", locale)
                .to_string()
        })
        .collect();

    let mut b = String::new();
    b.push_str("<nav class=\"cal-years\">");
    for y in &years {
        b.push_str(&format!("<a href=\"#y{y}\">{y}</a>"));
    }
    b.push_str("</nav>\n\n");

    for y in &years {
        b.push_str(&format!(
            "<h2 id=\"y{y}\" class=\"cal-year\">{y}</h2>\n<div class=\"cal-months\">\n"
        ));
        for m in 1..=12u32 {
            if !dates.iter().any(|d| d.year() == *y && d.month() == m) {
                continue;
            }
            let first = NaiveDate::from_ymd_opt(*y, m, 1).unwrap();
            let name = first.format_localized("%B", locale).to_string();
            b.push_str(&format!("<table class=\"cal\"><caption>{name}</caption><thead><tr>"));
            for wd in &weekdays {
                b.push_str(&format!("<th>{wd}</th>"));
            }
            b.push_str("</tr></thead><tbody><tr>");
            let lead = first.weekday().num_days_from_monday();
            for _ in 0..lead {
                b.push_str("<td class=\"pad\"></td>");
            }
            let mut col = lead;
            let mut day = first;
            while day.month() == m {
                let key = day.format("%Y-%m-%d").to_string();
                let dd = day.day();
                if let Some(meta) = present.get(key.as_str()) {
                    // Busier days get a larger number; the day's tags go in the
                    // hover title.
                    let cls = if meta.count >= 3 {
                        "on c3"
                    } else if meta.count == 2 {
                        "on c2"
                    } else {
                        "on"
                    };
                    let title = if meta.tags.is_empty() {
                        String::new()
                    } else {
                        // One tag per line in the tooltip (&#10; = newline; a real
                        // newline would break the raw-HTML block inside Markdown).
                        let tags = meta
                            .tags
                            .iter()
                            .map(|t| html_escape(&format!("#{t}")))
                            .collect::<Vec<_>>()
                            .join("&#10;");
                        format!(" title=\"{tags}\"")
                    };
                    b.push_str(&format!(
                        "<td class=\"{cls}\"{title}><a href=\"{}day/{key}/\">{dd}</a></td>",
                        s.base_url
                    ));
                } else {
                    b.push_str(&format!("<td class=\"off\">{dd}</td>"));
                }
                col += 1;
                day = day.succ_opt().unwrap();
                if col == 7 && day.month() == m {
                    b.push_str("</tr><tr>");
                    col = 0;
                }
            }
            while col % 7 != 0 {
                b.push_str("<td class=\"pad\"></td>");
                col += 1;
            }
            b.push_str("</tr></tbody></table>\n");
        }
        b.push_str("</div>\n");
    }

    let template = if s.theme.is_none() {
        "template = \"calendar.html\"\n"
    } else {
        ""
    };
    let label = crate::i18n::ui(&s.language).calendar;
    format!(
        "+++\ntitle = \"{}\"\npath = \"calendar\"\n{template}+++\n\n{b}\n",
        toml_escape(label)
    )
}

/// The last 10 commits of the repo tg2zola runs in, for the About page: a
/// clickable short hash (→ the repo's commit page), the subject and date, with
/// the full commit body as a hover tooltip. `None` if git isn't available.
fn recent_commits(repo_url: &str) -> Option<String> {
    // %x1f = field separator within a commit, %x1e = record separator between
    // commits (so multi-line bodies don't break parsing).
    let out = std::process::Command::new("git")
        .args(["log", "-10", "--format=%H%x1f%h%x1f%s%x1f%cs%x1f%b%x1e"])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&out.stdout);
    let repo = repo_url.trim_end_matches('/');
    let mut items = String::new();
    for record in text.split('\u{1e}') {
        let record = record.trim_start_matches(['\n', '\r']);
        if record.trim().is_empty() {
            continue;
        }
        let fields: Vec<&str> = record.splitn(5, '\u{1f}').collect();
        if fields.len() < 4 {
            continue;
        }
        let (full, short, subject, date) = (fields[0], fields[1], fields[2], fields[3]);
        let body = fields.get(4).copied().unwrap_or("").trim();
        // Encode newlines as &#10; so the (multi-paragraph) body keeps the whole
        // <ul> on one line — a blank line would otherwise end the raw-HTML block
        // inside Markdown. The tooltip still shows the line breaks.
        let title = if body.is_empty() {
            String::new()
        } else {
            let b = html_escape(body).replace('\r', "").replace('\n', "&#10;");
            format!(" title=\"{b}\"")
        };
        items.push_str(&format!(
            "<li{title}><a href=\"{repo}/commit/{full}\"><code>{short}</code></a> {} <span class=\"cdate\">{date}</span></li>",
            html_escape(subject)
        ));
    }
    if items.is_empty() {
        None
    } else {
        Some(format!("<ul class=\"commits\">{items}</ul>"))
    }
}

/// Minimal HTML escaping for text and attribute values.
fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// About page. The channel link is built from the configured channel (never
/// hardcoded), so the project works for any channel; the repo link is
/// configurable. In theme mode the template line is omitted.
fn about_md(s: &Settings, info: Option<&ChannelInfo>) -> String {
    let template = if s.theme.is_none() {
        "template = \"page.html\"\n"
    } else {
        ""
    };
    let body = match &s.about {
        // Custom HTML (from --about / ABOUT) becomes the page body verbatim.
        Some(html) => html.trim().to_string(),
        None => {
            let mut b = String::new();
            let about = crate::i18n::about(&s.language);
            // Channel avatar at its original size (base_url-aware via shortcode).
            if s.site.join("static/channel-avatar.jpg").exists() {
                b.push_str("{{ avatar() }}\n\n");
            }
            let channel_link = format!("[@{ch}](https://t.me/{ch})", ch = s.channel);
            let tool_link =
                "[tg2zola](https://github.com/vitaly-zdanevich/telegram_channel_to_static_website)";
            b.push_str(&format!(
                "*{}*\n\n",
                about
                    .intro
                    .replace("{channel}", &channel_link)
                    .replace("{tool}", tool_link)
            ));
            if let Some(info) = info {
                if let Some(desc) = &info.description_md {
                    b.push_str(desc.trim());
                    b.push_str("\n\n");
                }
                if !info.counters.is_empty() {
                    let stats: Vec<String> = info
                        .counters
                        .iter()
                        .map(|(v, t)| format!("**{v}** {t}"))
                        .collect();
                    b.push_str(&stats.join(" · "));
                    b.push_str("\n\n");
                }
            }
            let size_line = match pages_limit(&s.base_url, s.pages_host.as_deref()) {
                Some(l) => {
                    let phrase = about
                        .limit_phrase
                        .replace("{display}", l.display)
                        .replace("{name}", l.name);
                    let limit_link = format!("[{}]({})", phrase, l.doc);
                    format!("{}\n\n", about.size_limit.replace("{limit_link}", &limit_link))
                }
                None => format!("{}\n\n", about.size_plain),
            };
            b.push_str(&size_line);
            b.push_str(&format!("{}\n\n__SIZE_BREAKDOWN__\n\n", about.by_kind));
            b.push_str("__LARGEST_FILES__\n\n");
            b.push_str(&format!("{}\n\n", about.generated_in));
            b.push_str(&format!(
                "{} [{repo}]({repo})\n\n{no_api}",
                about.source_repo,
                repo = s.repo_url,
                no_api = about.no_api,
            ));
            if let Some(commits) = recent_commits(&s.repo_url) {
                b.push_str("\n\n");
                b.push_str(&commits);
            }
            b
        }
    };
    let about_label = crate::i18n::ui(&s.language).about;
    format!(
        "+++\ntitle = \"{}\"\npath = \"about\"\n{template}+++\n\n{body}\n",
        toml_escape(about_label)
    )
}

/// One extra page parsed from the `PAGES` input.
struct Page {
    title: String,
    slug: String,
    body: String,
}

/// Split the `pages` input into pages at each top-level `# Title` heading.
fn parse_pages(src: Option<&str>) -> Vec<Page> {
    let src = match src {
        Some(s) if !s.trim().is_empty() => s,
        _ => return Vec::new(),
    };
    let mut pages = Vec::new();
    let mut cur: Option<(String, String)> = None;
    let flush = |cur: &mut Option<(String, String)>, pages: &mut Vec<Page>| {
        if let Some((title, body)) = cur.take() {
            let slug = slugify(&title);
            if !slug.is_empty() {
                pages.push(Page {
                    title,
                    slug,
                    body: body.trim().to_string(),
                });
            }
        }
    };
    for line in src.lines() {
        if let Some(title) = line.strip_prefix("# ") {
            flush(&mut cur, &mut pages);
            cur = Some((title.trim().to_string(), String::new()));
        } else if let Some((_, body)) = cur.as_mut() {
            body.push_str(line);
            body.push('\n');
        }
    }
    flush(&mut cur, &mut pages);
    pages
}

pub fn slugify(s: &str) -> String {
    let mut out = String::new();
    let mut dash = false;
    for c in s.chars() {
        if c.is_alphanumeric() {
            out.extend(c.to_lowercase());
            dash = false;
        } else if !out.is_empty() && !dash {
            out.push('-');
            dash = true;
        }
    }
    out.trim_matches('-').to_string()
}

fn page_md(s: &Settings, p: &Page) -> String {
    let template = if s.theme.is_none() {
        "template = \"page.html\"\n"
    } else {
        ""
    };
    format!(
        "+++\ntitle = \"{}\"\npath = \"{}\"\n{}+++\n\n{}\n",
        toml_escape(&p.title),
        p.slug,
        template,
        p.body
    )
}

/// The posts live here but are `transparent`, so they appear in the homepage
/// feed while keeping their `/posts/<id>/` URLs.
fn posts_index_md(s: &Settings) -> String {
    let mut o = String::from("+++\ntransparent = true\nrender = false\nsort_by = \"date\"\n");
    if s.theme.is_none() {
        o.push_str("page_template = \"page.html\"\n");
    }
    o.push_str("+++\n");
    o
}

fn style_css(s: &Settings) -> String {
    // Strip characters that could break out of the CSS value.
    let clean = |c: &str| -> String {
        c.chars()
            .filter(|ch| !matches!(ch, ';' | '{' | '}' | '"' | '\n' | '\r'))
            .collect()
    };
    let mut css = STYLE_CSS
        .replace("__BG_LIGHT__", &clean(&s.background_light))
        .replace("__BG_DARK__", &clean(&s.background_dark))
        .replace(
            "__LINK_DECO__",
            if s.link_underline { "underline" } else { "none" },
        );
    if let Some(extra) = &s.css {
        css.push_str("\n/* custom CSS (from --css / CSS) */\n");
        css.push_str(extra.trim());
        css.push('\n');
    }
    css
}

fn toml_escape(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

const CONFIG_TOML: &str = r#"# Generated by tg2zola — regenerated every run. Configure via the tool, not here.
base_url = "__BASE_URL__"
title = "__TITLE__"
description = "__DESC__"
default_language = "__LANGUAGE__"
__THEME____FEEDS__
compile_sass = false
build_search_index = false
minify_html = true

taxonomies = [
  { name = "tags" },
  { name = "days", render = false },
]

[markdown]
render_emoji = false

[extra]
generator = "tg2zola"
channel = "__CHANNEL__"
date_format = "__DATE_FORMAT__"
date_locale = "__DATE_LOCALE__"
tags_footer = __TAGS_FOOTER__
next_prev = __NEXT_PREV__
telegram_link = __TELEGRAM_LINK__
rss = __RSS__
youtube_facade = __YT_FACADE__
calendar = __CALENDAR__
__FEDI__
__SEARCH__
__FOOTER__
__AVATAR__
__NAV__
__NAV_TAGS__
__TAGS__

[extra.i18n]
__I18N__
"#;

const BASE_HTML: &str = r#"<!DOCTYPE html>
<html lang="{{ config.default_language }}">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>{% block title %}{{ config.title }}{% endblock title %}</title>
  {% if config.extra.avatar %}<link rel="icon" type="image/jpeg" href="{{ get_url(path=config.extra.avatar) }}">{% endif %}
  {% if config.extra.rss %}<link rel="alternate" type="application/rss+xml" title="{{ config.title }}" href="{{ get_url(path='rss.xml', trailing_slash=false) | safe }}">{% endif %}
  {# Social cards (Open Graph + Twitter) and Mastodon attribution. `page` is
     only defined on post/page templates; sections fall back to site defaults. #}
  {% set_global og_title = config.title %}
  {% if page.title %}{% set_global og_title = page.title %}{% elif page.extra.id %}{% set_global og_title = page.extra.id %}{% endif %}
  {% set og_desc = page.description | default(value=config.description) %}
  {% set og_url = page.permalink | default(value=config.base_url) %}
  {% set_global og_image = "" %}
  {% if page.extra.og_image %}{% set_global og_image = page.permalink ~ page.extra.og_image %}
  {% elif config.extra.avatar %}{% set_global og_image = get_url(path=config.extra.avatar) %}{% endif %}
  {% if og_desc %}<meta name="description" content="{{ og_desc }}">{% endif %}
  <meta property="og:site_name" content="{{ config.title }}">
  <meta property="og:title" content="{{ og_title }}">
  {% if og_desc %}<meta property="og:description" content="{{ og_desc }}">{% endif %}
  <meta property="og:url" content="{{ og_url | safe }}">
  {% if page.date %}<meta property="og:type" content="article"><meta property="article:published_time" content="{{ page.date }}">{% else %}<meta property="og:type" content="website">{% endif %}
  {% if og_image %}<meta property="og:image" content="{{ og_image | safe }}"><meta name="twitter:card" content="summary_large_image">{% else %}<meta name="twitter:card" content="summary">{% endif %}
  <meta name="twitter:title" content="{{ og_title }}">
  {% if og_desc %}<meta name="twitter:description" content="{{ og_desc }}">{% endif %}
  {% if og_image %}<meta name="twitter:image" content="{{ og_image | safe }}">{% endif %}
  {% if config.extra.fediverse_creator %}<meta name="fediverse:creator" content="{{ config.extra.fediverse_creator }}">{% endif %}
  {% if config.extra.fediverse_profile %}<link rel="me" href="{{ config.extra.fediverse_profile | safe }}">{% endif %}
  <link rel="stylesheet" href="{{ get_url(path='style.css', cachebust=true) }}">
</head>
<body>
  <header class="site-header">
    {% if config.extra.avatar %}<a href="{{ config.base_url | safe }}"><img class="site-avatar" src="{{ get_url(path=config.extra.avatar) }}" alt=""></a>{% endif %}
    <a class="site-title" href="{{ config.base_url | safe }}">{{ config.title }}</a>
    <nav>
      {% for t in config.extra.nav_tags | default(value=[]) %}<a class="tag" href="{{ get_taxonomy_url(kind='tags', name=t) | safe }}">#{{ t }}</a>{% endfor %}
      {% if current_path is matching("/tags/$") %}<span class="here">{{ config.extra.i18n.tags }}</span>{% else %}<a href="{{ get_url(path='/tags/') }}">{{ config.extra.i18n.tags }}</a>{% endif %}
      {% if config.extra.calendar %}{% if current_path is containing("/calendar/") %}<span class="here">{{ config.extra.i18n.calendar }}</span>{% else %}<a href="{{ get_url(path='/calendar/') }}">{{ config.extra.i18n.calendar }}</a>{% endif %}{% endif %}
      {% if current_path is containing("/about/") %}<span class="here">{{ config.extra.i18n.about }}</span>{% else %}<a href="{{ get_url(path='/about/') }}">{{ config.extra.i18n.about }}</a>{% endif %}
      {% for p in config.extra.nav | default(value=[]) %}<a href="{{ get_url(path=p.path) }}">{{ p.title }}</a>{% endfor %}
    </nav>
    {% if config.extra.search_google %}<form class="site-search" action="https://www.google.com/search" method="get" role="search"><input type="search" name="q" placeholder="{{ config.extra.i18n.search }}" aria-label="{{ config.extra.i18n.search_aria }}" autocomplete="off">{% if config.extra.search_site %}<input type="hidden" name="sitesearch" value="{{ config.extra.search_site }}">{% endif %}</form>{% elif config.extra.search_url %}<input type="search" id="site-search" class="site-search" placeholder="{{ config.extra.i18n.search }}" aria-label="{{ config.extra.i18n.search_aria }}" data-url="{{ config.extra.search_url | safe }}" autocomplete="off">{% endif %}
  </header>
  <main>{% block content %}{% endblock content %}</main>
  {% if config.extra.footer %}<footer class="site-footer">{{ config.extra.footer | markdown(inline=true) | safe }}</footer>{% endif %}
  {% if config.extra.search_url %}<script>el=document.getElementById('site-search');el.addEventListener('keydown',function(e){if(e.key==='Enter'&&el.value)location.href=el.dataset.url+encodeURIComponent(el.value);});</script>{% endif %}
</body>
</html>
"#;

const INDEX_HTML: &str = r#"{% extends "base.html" %}
{% block content %}
  {% if config.description and paginator.current_index == 1 %}<p class="lead">{{ config.description }}</p>{% endif %}
  {% for page in paginator.pages %}
    <article class="post{% if page.extra.forwarded_from %} forwarded{% endif %}">
      {% if page.title %}<h2 class="post-title"><a href="{{ page.permalink | safe }}">{{ page.title }}</a></h2>{% endif %}
      <p class="meta">
        <a class="day" href="{{ get_url(path='/day/' ~ page.extra.day ~ '/') | safe }}"><time datetime="{{ page.date }}" title="{{ page.date | date(format='%A %H:%M', locale=config.extra.date_locale) }}">{{ page.date | date(format=config.extra.date_format, locale=config.extra.date_locale) }}</time></a>
        {% if page.extra.views %}· 👁 {{ page.extra.views }}{% endif %}
        {% if page.extra.forwarded_from %}· {{ config.extra.i18n.forwarded_from }} {% if page.extra.forwarded_from_url %}<a href="{{ page.extra.forwarded_from_url }}">{{ page.extra.forwarded_from }}</a>{% else %}{{ page.extra.forwarded_from }}{% endif %}{% endif %}
        {% if not page.title %}· <a class="pid" href="{{ page.permalink | safe }}">{{ page.extra.id }}</a>{% endif %}
      </p>
      <div class="content">{{ page.content | safe }}</div>
      {% if config.extra.tags_footer and page.taxonomies.tags %}
        <p class="tags">{% for t in page.taxonomies.tags %}<a href="{{ get_taxonomy_url(kind='tags', name=t) }}">#{{ t }}</a> {% endfor %}</p>
      {% endif %}
    </article>
  {% endfor %}
  <nav class="pager">
    {% if paginator.previous %}<a href="{{ paginator.previous | safe }}">← {{ config.extra.i18n.newer }}</a>{% else %}<span></span>{% endif %}
    <span>{{ paginator.current_index }} / {{ paginator.number_pagers }}</span>
    {% if paginator.next %}<a href="{{ paginator.next | safe }}">{{ config.extra.i18n.older }} →</a>{% else %}<span></span>{% endif %}
  </nav>
{% endblock content %}
"#;

const SECTION_HTML: &str = r#"{% extends "base.html" %}
{% block title %}{{ config.extra.i18n.archive }} · {{ config.title }}{% endblock title %}
{% block content %}
  <h1>{{ config.extra.i18n.archive }}</h1>
  <ul class="post-list">
  {% for page in paginator.pages %}
    <li>
      {% if page.title %}<a href="{{ page.permalink | safe }}">{{ page.title }}</a>{% endif %}
      <a class="day" href="{{ get_url(path='/day/' ~ page.extra.day ~ '/') | safe }}"><time datetime="{{ page.date }}" title="{{ page.date | date(format='%A %H:%M', locale=config.extra.date_locale) }}">{{ page.date | date(format=config.extra.date_format, locale=config.extra.date_locale) }}</time></a>
      {% if page.extra.views %}<span class="views">👁 {{ page.extra.views }}</span>{% endif %}
      {% if not page.title %}<a class="pid" href="{{ page.permalink | safe }}">{{ page.extra.id }}</a>{% endif %}
    </li>
  {% endfor %}
  </ul>
  <nav class="pager">
    {% if paginator.previous %}<a href="{{ paginator.previous | safe }}">← {{ config.extra.i18n.newer }}</a>{% else %}<span></span>{% endif %}
    <span>{{ paginator.current_index }} / {{ paginator.number_pagers }}</span>
    {% if paginator.next %}<a href="{{ paginator.next | safe }}">{{ config.extra.i18n.older }} →</a>{% else %}<span></span>{% endif %}
  </nav>
{% endblock content %}
"#;

const PAGE_HTML: &str = r#"{% extends "base.html" %}
{% block title %}{% if page.title %}{{ page.title }}{% else %}{{ page.extra.id }}{% endif %} · {{ config.title }}{% endblock title %}
{% block content %}
  <article class="post{% if page.extra.forwarded_from %} forwarded{% endif %}">
    {% if not (current_path is containing("/about/")) %}<h1>{% if page.title %}{{ page.title }}{% else %}{{ page.extra.id }}{% endif %}</h1>{% endif %}
    <p class="meta">
      {% if page.date %}<a class="day" href="{{ get_url(path='/day/' ~ page.extra.day ~ '/') | safe }}"><time datetime="{{ page.date }}" title="{{ page.date | date(format='%A %H:%M', locale=config.extra.date_locale) }}">{{ page.date | date(format=config.extra.date_format, locale=config.extra.date_locale) }}</time></a>{% endif %}
      {% if page.extra.views %}· 👁 {{ page.extra.views }} {{ config.extra.i18n.views }}{% endif %}
      {% if page.extra.forwarded_from %}· {{ config.extra.i18n.forwarded_from }} {% if page.extra.forwarded_from_url %}<a href="{{ page.extra.forwarded_from_url }}">{{ page.extra.forwarded_from }}</a>{% else %}{{ page.extra.forwarded_from }}{% endif %}{% endif %}
    </p>
    <div class="content">{{ page.content | safe }}</div>
    {% if config.extra.tags_footer and page.taxonomies.tags %}
      <p class="tags">
      {% for t in page.taxonomies.tags %}
        <a href="{{ get_taxonomy_url(kind='tags', name=t) }}">#{{ t }}</a>
      {% endfor %}
      </p>
    {% endif %}
    {% if config.extra.telegram_link and page.extra.tg_url %}<p class="tg-link"><a href="{{ page.extra.tg_url }}">{{ config.extra.i18n.view_on_telegram }} ↗</a></p>{% endif %}
  </article>
  {% if config.extra.next_prev %}
  <nav class="post-nav">
    <span>{% if page.extra.next_id %}<a href="{{ get_url(path='/posts/' ~ page.extra.next_id ~ '/') | safe }}" title="{{ page.extra.next_body }}" accesskey="n" rel="prev">← {{ config.extra.i18n.newer }}</a>{% endif %}</span>
    <span>{% if page.extra.prev_id %}<a href="{{ get_url(path='/posts/' ~ page.extra.prev_id ~ '/') | safe }}" title="{{ page.extra.prev_body }}" accesskey="o" rel="next">{{ config.extra.i18n.older }} →</a>{% endif %}</span>
  </nav>
  {% endif %}
{% endblock content %}
"#;

const TAGS_SINGLE: &str = r#"{% extends "base.html" %}
{% block title %}#{{ term.name }} · {{ config.title }}{% endblock title %}
{% block content %}
  <h1>#{{ term.name }}</h1>
  <ul class="post-list">
  {% for page in term.pages %}
    <li>
      {% if page.title %}<a href="{{ page.permalink | safe }}" title="{{ page.title }}">{{ page.title }}</a>{% endif %}
      <a class="day" href="{{ get_url(path='/day/' ~ page.extra.day ~ '/') | safe }}"><time datetime="{{ page.date }}" title="{{ page.date | date(format='%A %H:%M', locale=config.extra.date_locale) }}">{{ page.date | date(format=config.extra.date_format, locale=config.extra.date_locale) }}</time></a>
      {% if not page.title %}<a class="pid" href="{{ page.permalink | safe }}">{{ page.extra.id }}</a>{% endif %}
    </li>
  {% endfor %}
  </ul>
{% endblock content %}
"#;

const TAGS_LIST: &str = r#"{% extends "base.html" %}
{% block title %}{{ config.extra.i18n.tags }} · {{ config.title }}{% endblock title %}
{% block content %}
  <h1>{{ config.extra.i18n.tags }}</h1>
  <ul class="tag-cloud">
  {% for t in config.extra.tags | default(value=[]) %}
    <li><a href="{{ get_taxonomy_url(kind='tags', name=t.name) | safe }}">#{{ t.name }}</a> <a class="count" href="{{ get_url(path='/tags/' ~ t.slug ~ '/full/') | safe }}" title="{{ config.extra.i18n.full_posts }}">{{ t.count }}</a></li>
  {% endfor %}
  </ul>
{% endblock content %}
"#;

// Full-posts view for one tag (the clickable count on the Tags page links here),
// rendering every tagged post in full — the term name links to the titles list.
const TAG_FULL_HTML: &str = r#"{% extends "base.html" %}
{% block title %}#{{ page.extra.tag }} · {{ config.title }}{% endblock title %}
{% block content %}
  <h1>#{{ page.extra.tag }} <a class="count" href="{{ get_taxonomy_url(kind='tags', name=page.extra.tag) | safe }}">{{ config.extra.i18n.titles }} ↗</a></h1>
  {% set tax = get_taxonomy(kind="tags") %}
  {% for term in tax.items %}{% if term.name == page.extra.tag %}
  {% for p in term.pages %}
    <article class="post{% if p.extra.forwarded_from %} forwarded{% endif %}">
      {% if p.title %}<h2 class="post-title"><a href="{{ p.permalink | safe }}">{{ p.title }}</a></h2>{% endif %}
      <p class="meta"><a class="day" href="{{ get_url(path='/day/' ~ p.extra.day ~ '/') | safe }}"><time datetime="{{ p.date }}" title="{{ p.date | date(format='%A %H:%M', locale=config.extra.date_locale) }}">{{ p.date | date(format=config.extra.date_format, locale=config.extra.date_locale) }}</time></a>{% if p.extra.views %} · 👁 {{ p.extra.views }}{% endif %}{% if not p.title %} · <a class="pid" href="{{ p.permalink | safe }}">{{ p.extra.id }}</a>{% endif %}</p>
      <div class="content">{{ p.content | safe }}</div>
    </article>
  {% endfor %}
  {% endif %}{% endfor %}
{% endblock content %}
"#;

// All posts of one day, rendered in full (the date everywhere links here), with
// prev/next-day navigation. Posts come via the render=false `days` taxonomy.
const DAY_FULL_HTML: &str = r#"{% extends "base.html" %}
{% block title %}{{ page.extra.day }} · {{ config.title }}{% endblock title %}
{% block content %}
  <h1>{{ page.extra.day }}</h1>
  {% set tax = get_taxonomy(kind="days") %}
  {% for term in tax.items %}{% if term.name == page.extra.day %}
  {% for p in term.pages %}
    <article class="post{% if p.extra.forwarded_from %} forwarded{% endif %}">
      {% if p.title %}<h2 class="post-title"><a href="{{ p.permalink | safe }}">{{ p.title }}</a></h2>{% endif %}
      <p class="meta"><a class="day" href="{{ get_url(path='/day/' ~ p.extra.day ~ '/') | safe }}"><time datetime="{{ p.date }}" title="{{ p.date | date(format='%A %H:%M', locale=config.extra.date_locale) }}">{{ p.date | date(format=config.extra.date_format, locale=config.extra.date_locale) }}</time></a>{% if p.extra.views %} · 👁 {{ p.extra.views }}{% endif %}{% if not p.title %} · <a class="pid" href="{{ p.permalink | safe }}">{{ p.extra.id }}</a>{% endif %}</p>
      <div class="content">{{ p.content | safe }}</div>
    </article>
  {% endfor %}
  {% endif %}{% endfor %}
  <nav class="post-nav">
    <span>{% if page.extra.newer_day %}<a href="{{ get_url(path='/day/' ~ page.extra.newer_day ~ '/') | safe }}" accesskey="n" rel="prev">← {{ config.extra.i18n.newer_day }}</a>{% endif %}</span>
    <span>{% if page.extra.older_day %}<a href="{{ get_url(path='/day/' ~ page.extra.older_day ~ '/') | safe }}" accesskey="o" rel="next">{{ config.extra.i18n.older_day }} →</a>{% endif %}</span>
  </nav>
{% endblock content %}
"#;

// The /calendar/ page — the grid HTML is generated (calendar_md), this just
// wraps it. The year/month grid links into the /day/<date>/ pages.
const CALENDAR_HTML: &str = r#"{% extends "base.html" %}
{% block title %}{{ config.extra.i18n.calendar }} · {{ config.title }}{% endblock title %}
{% block content %}
  <h1>{{ config.extra.i18n.calendar }}</h1>
  {{ page.content | safe }}
{% endblock content %}
"#;

// YouTube shortcode (Zola no longer ships one). Uses the regular youtube.com
// host so a played video counts toward the viewer's history. Default is a
// direct iframe; with config.extra.youtube_facade it's a CSS-only click-to-load
// facade — a display:none + loading=lazy iframe doesn't fetch until the
// checkbox reveals it, so no JavaScript is involved.
const YOUTUBE_SHORTCODE: &str = r#"<div class="yt-embed">
{%- if config.extra.youtube_facade -%}
<input type="checkbox" id="yt-{{ id }}" class="yt-toggle"><label for="yt-{{ id }}" class="yt-facade"><img src="https://i.ytimg.com/vi/{{ id }}/hqdefault.jpg" alt="" loading="lazy"><span class="yt-btn" aria-hidden="true">▶</span></label><iframe class="yt-frame" src="https://www.youtube.com/embed/{{ id }}?autoplay=1" title="YouTube video" loading="lazy" allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture; web-share" referrerpolicy="strict-origin-when-cross-origin" allowfullscreen></iframe>
{%- else -%}
<iframe src="https://www.youtube.com/embed/{{ id }}" title="YouTube video" loading="lazy" allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture; web-share" referrerpolicy="strict-origin-when-cross-origin" allowfullscreen></iframe>
{%- endif -%}
</div>
"#;

// Apple Podcasts episode embed. The iframe needs an origin, so over file:// it
// won't load — the "Listen on Apple Podcasts" link below it is the fallback.
const APPLE_PODCAST_SHORTCODE: &str = r#"<div class="ap-embed"><iframe src="{{ url }}" height="175" loading="lazy" frameborder="0" allow="autoplay *; encrypted-media *;" sandbox="allow-forms allow-popups allow-same-origin allow-scripts allow-storage-access-by-user-activation allow-top-navigation-by-user-activation"></iframe><a class="ap-link" href="{{ url }}">Listen on Apple Podcasts</a></div>
"#;

// Yandex Music track player (iframe uses a #hash route, so it won't load over
// file:// — the "Yandex Music" link below it is the fallback).
const YANDEX_MUSIC_SHORTCODE: &str = r#"<div class="ym-embed"><iframe frameborder="0" width="100%" height="180" loading="lazy" src="{{ url }}"></iframe><a class="ym-link" href="{{ url }}">Yandex Music</a></div>
"#;

// Instagram post embed (official blockquote + embed.js). The offline pass strips
// the <script>, leaving the "View on Instagram" link fallback.
const INSTAGRAM_SHORTCODE: &str = r#"<blockquote class="instagram-media" data-instgrm-permalink="{{ url }}" data-instgrm-version="14" style="max-width:540px;margin:1rem auto"><a href="{{ url }}">View on Instagram</a></blockquote><script async src="//www.instagram.com/embed.js"></script>
"#;

// Resolve colocated media against the post's permalink so it works both on the
// post page and when the post is shown in full on the homepage feed (a relative
// src would otherwise break off the post's own page).
const VIDEO_SHORTCODE: &str =
    "<video controls preload=\"metadata\" src=\"{{ page.permalink | safe }}{{ src }}\"></video>\n";
const AUDIO_SHORTCODE: &str =
    "<audio controls src=\"{{ page.permalink | safe }}{{ src }}\"></audio>\n";

// Inline clickable hashtag → its taxonomy page (base_url-aware).
const TAG_SHORTCODE: &str =
    "<a class=\"tag\" href=\"{{ get_taxonomy_url(kind='tags', name=t) | safe }}\">#{{ t }}</a>";

// Channel avatar at its original size on the About page (base_url-aware).
const AVATAR_SHORTCODE: &str =
    "<img class=\"about-avatar\" src=\"{{ get_url(path='channel-avatar.jpg') | safe }}\" alt=\"channel avatar\">";

// Built-in look: light by default, true-black #000 in dark mode (OLED-friendly),
// following the OS via prefers-color-scheme.
const STYLE_CSS: &str = r#":root {
  color-scheme: light dark;
  --bg: __BG_LIGHT__; --fg: #1a1a1a; --muted: #6b6b6b;
  --link: #0a58ca; --border: #e6e6e6; --code-bg: #f4f4f4;
  --input-bg: #ffffff; --fwd-bg: #eef2fb;
}
@media (prefers-color-scheme: dark) {
  :root {
    --bg: __BG_DARK__; --fg: #e6e6e6; --muted: #8a8a8a;
    --link: #6cb6ff; --border: #1c1c1c; --code-bg: #0d0d0d;
    --input-bg: #2a2a2a; --fwd-bg: #0a1024;
  }
}
* { box-sizing: border-box; }
body {
  width: 95vw; margin: 0 auto; padding: 1rem;
  font-family: system-ui, -apple-system, sans-serif; line-height: 1.6;
  background: var(--bg); color: var(--fg);
  min-height: 100vh; display: flex; flex-direction: column;
}
main { flex: 1 0 auto; display: flex; flex-direction: column; }
a, a:visited { color: var(--link); text-decoration: __LINK_DECO__; }
::selection {
    background-color: #292;
    color: #000;
}
img, video { max-width: 100%; height: auto; }
video { display: block; }
audio { width: 100%; }
.yt-embed { position: relative; aspect-ratio: 16 / 9; margin: 1rem 0; }
.yt-embed iframe { position: absolute; inset: 0; width: 100%; height: 100%; border: 0; }
.yt-link { font-size: .9em; margin-top: .25rem; }
/* CSS-only click-to-load facade (no JS): the iframe is display:none until the
   hidden checkbox is checked, so loading=lazy defers its fetch until click. */
.yt-embed .yt-toggle { position: absolute; width: 0; height: 0; opacity: 0; }
.yt-embed .yt-facade { position: absolute; inset: 0; cursor: pointer; display: block; }
.yt-embed .yt-facade img { width: 100%; height: 100%; object-fit: cover; display: block; }
.yt-embed .yt-btn { position: absolute; inset: 0; margin: auto; width: 4.2rem; height: 3rem; display: flex; align-items: center; justify-content: center; color: #fff; background: rgba(0,0,0,.65); border-radius: 12px; }
.yt-embed .yt-frame { display: none; }
.ap-embed { margin: 1rem 0; }
.ap-embed iframe { width: 100%; max-width: 660px; height: 175px; border: 0; border-radius: 10px; }
.ap-embed .ap-link { display: block; font-size: .85rem; margin-top: .3rem; }
.ym-embed { margin: 1rem 0; }
.ym-embed iframe { width: 100%; max-width: 900px; height: 180px; border: 0; }
.ym-embed .ym-link { display: block; font-size: .85rem; margin-top: .3rem; }
blockquote.instagram-media { max-width: 540px; margin: 1rem 0; padding: .5rem 1rem; }
.yt-embed .yt-toggle:checked ~ .yt-facade { display: none; }
.yt-embed .yt-toggle:checked ~ .yt-frame { display: block; }
.tag { white-space: nowrap; }
pre { background: var(--code-bg); padding: .75rem; border-radius: 4px; overflow-x: auto; }
code { background: var(--code-bg); padding: .1rem .3rem; border-radius: 4px; }
pre code { padding: 0; background: none; }
.site-header { display: flex; gap: .6rem; align-items: center; border-bottom: 1px solid var(--border); padding-bottom: .5rem; margin-bottom: 1rem; flex-wrap: wrap; }
.site-avatar { width: 32px; height: 32px; border-radius: 50%; object-fit: cover; display: block; }
.site-title { font-weight: 700; text-decoration: none; color: var(--fg); }
.site-header nav a, .site-header nav .here { margin-left: 1.4rem; }
.site-search { margin-left: auto; }
.site-search input, input.site-search {
  background: var(--input-bg); color: var(--fg);
  border: 1px solid var(--border); border-radius: 6px;
  padding: .3rem .55rem; font: inherit; width: 9rem;
  transition: width .2s ease;
}
/* Expand on click/focus — CSS only, no JS. */
.site-search input:focus, input.site-search:focus { width: min(22rem, 60vw); }
input.site-search { margin-left: auto; }
.post-list { list-style: none; padding: 0; }
.post-list li { padding: .2rem 0; }
.post-list time { color: var(--muted); font-size: .85em; margin-left: .5rem; }
.post { margin: 0 0 2.5rem; }
.post.forwarded { background: var(--fwd-bg); border-radius: 8px; padding: .75rem 1rem; }
.about-avatar { float: left; margin: 0 1rem .5rem 0; }
.post-title { margin: 0 0 .25rem; font-size: 1.25rem; }
.more { margin: 2rem 0; }
.views, .meta { color: var(--muted); font-size: .85em; }
.meta { font-size: .9em; }
.tags a { margin-right: .5rem; white-space: nowrap; }
.pager { display: flex; justify-content: space-between; margin: 2rem 0; }
.post-nav { display: flex; justify-content: space-between; gap: 1rem; margin: 1.5rem 0 0; padding-top: 1rem; margin-top: auto; }
.tg-link { margin-top: 1rem; font-size: .9em; }
.spoiler { background: var(--fg); border-radius: 3px; transition: background .1s; }
.spoiler:hover { background: transparent; }
.site-footer { margin-top: 3rem; color: var(--muted); font-size: .8rem; border-top: 1px solid var(--border); padding-top: .5rem; }
blockquote { border-left: 3px solid var(--border); margin: .5rem 0; padding-left: .75rem; color: var(--muted); }
.here { font-weight: 700; }

/* Calendar */
.cal-years { margin: .5rem 0 1.5rem; line-height: 1.9; }
.cal-years a { margin-right: .7rem; }
.cal-year { margin: 1.5rem 0 .5rem; }
.cal-months { display: flex; flex-wrap: wrap; gap: 1.4rem; }
table.cal { border-collapse: collapse; font-size: .8rem; }
table.cal caption { text-align: left; font-weight: 700; padding-bottom: .3rem; }
table.cal th { font-weight: 400; color: var(--muted); padding: .1rem; text-align: center; }
table.cal td { width: 1.95rem; height: 1.7rem; text-align: center; padding: 0; }
table.cal td.on a { display: block; line-height: 1.7rem; border-radius: 4px; text-decoration: none; background: var(--code-bg); font-weight: 700; }
table.cal td.off { color: var(--muted); opacity: .45; }
table.cal td.c2 a { font-size: 1.05rem; }
table.cal td.c3 a { font-size: 1.35rem; font-weight: 800; }

/* Recent commits (About page) */
.commits { list-style: none; padding: 0; font-size: .9em; }
.commits li { padding: .15rem 0; }
.commits code { background: var(--code-bg); padding: .05rem .3rem; border-radius: 3px; }
.commits .cdate { color: var(--muted); font-size: .85em; margin-left: .4rem; }

/* Mobile: use more of a narrow screen — less left/right inset. */
@media (max-width: 640px) {
  body { width: auto; padding: 1rem .5rem; }
  .cal-months { gap: .9rem; }
}
"#;
