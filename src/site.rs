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
        &site.join("templates/shortcodes/youtube_link.html"),
        YOUTUBE_LINK_SHORTCODE,
    )?;
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
    write_file(
        &site.join("templates/shortcodes/spotify.html"),
        SPOTIFY_SHORTCODE,
    )?;
    write_file(
        &site.join("templates/shortcodes/pinterest.html"),
        PINTEREST_SHORTCODE,
    )?;
    write_file(
        &site.join("templates/shortcodes/bandcamp.html"),
        BANDCAMP_SHORTCODE,
    )?;
    write_file(
        &site.join("templates/shortcodes/vk_playlist.html"),
        VK_PLAYLIST_SHORTCODE,
    )?;
    write_file(&site.join("templates/shortcodes/video.html"), VIDEO_SHORTCODE)?;
    write_file(
        &site.join("templates/shortcodes/video_ext.html"),
        VIDEO_EXT_SHORTCODE,
    )?;
    write_file(
        &site.join("templates/shortcodes/aboutme_photo.html"),
        ABOUTME_PHOTO_SHORTCODE,
    )?;
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
        ("templates/404.html", NOT_FOUND_HTML),
    ];
    if s.theme.is_none() {
        fs::create_dir_all(site.join("templates/tags"))?;
        for (path, content) in builtins {
            write_file(&site.join(path), content)?;
        }
        write_file(&site.join("static/style.css"), &style_css(s))?;
        // Bundle the Elasticlunr library + wiring only when that search is on;
        // otherwise clean up any copy left by a previous run.
        if matches!(s.search, Search::Elasticlunr) {
            write_file(&site.join("static/elasticlunr.min.js"), ELASTICLUNR_JS)?;
            write_file(&site.join("static/search.js"), SEARCH_JS)?;
        } else {
            let _ = fs::remove_file(site.join("static/elasticlunr.min.js"));
            let _ = fs::remove_file(site.join("static/search.js"));
        }
        // Carousel enhancement script (opt-in); the swipe itself is CSS-only.
        if s.carousel {
            write_file(&site.join("static/carousel.js"), CAROUSEL_JS)?;
        } else {
            let _ = fs::remove_file(site.join("static/carousel.js"));
        }
        // Iframe auto-resize helper (opt-in): reports height to the host page.
        if s.embed {
            write_file(&site.join("static/embed.js"), EMBED_JS)?;
        } else {
            let _ = fs::remove_file(site.join("static/embed.js"));
        }
        // Service worker: for the installable PWA and/or offline precaching. The
        // precache list (asset-manifest.json) is written post-build by `tg2zola pwa`.
        if s.pwa || s.offline {
            write_file(&site.join("static/sw.js"), crate::pwa::SW_JS)?;
        } else {
            let _ = fs::remove_file(site.join("static/sw.js"));
        }
        // Web app manifest: makes the site an installable PWA.
        if s.pwa {
            let has_avatar = site.join("static/channel-avatar.jpg").exists();
            let manifest = crate::pwa::manifest_json(&s.title, &s.background_dark, has_avatar);
            write_file(&site.join("static/manifest.webmanifest"), &manifest)?;
        } else {
            let _ = fs::remove_file(site.join("static/manifest.webmanifest"));
        }
    } else {
        // Theme mode: remove our built-ins so they don't shadow the theme.
        for (path, _) in builtins {
            let _ = fs::remove_file(site.join(path));
        }
        let _ = fs::remove_dir_all(site.join("templates/tags"));
        let _ = fs::remove_file(site.join("static/style.css"));
        let _ = fs::remove_file(site.join("static/elasticlunr.min.js"));
        let _ = fs::remove_file(site.join("static/search.js"));
        let _ = fs::remove_file(site.join("static/sw.js"));
        let _ = fs::remove_file(site.join("static/manifest.webmanifest"));
        let _ = fs::remove_file(site.join("static/carousel.js"));
        let _ = fs::remove_file(site.join("static/embed.js"));
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
        Search::Elasticlunr => String::from("search_elasticlunr = true"),
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
        ("not_found", u.not_found),
        ("posts", u.posts),
        ("related", u.related),
    ]
    .iter()
    .map(|&(k, v)| format!("{k} = \"{}\"", toml_escape(v)))
    .collect::<Vec<_>>()
    .join("\n");

    // Per-tag post counts, looked up by the tag shortcode + nav for a hover
    // tooltip ("17 posts"). Localized here so templates need no number logic.
    let tag_counts_block = if tags.is_empty() {
        String::new()
    } else {
        let word = crate::i18n::ui(&s.language).posts;
        let rows = tags
            .iter()
            .map(|(name, n)| format!("\"{}\" = \"{n} {word}\"", toml_escape(name)))
            .collect::<Vec<_>>()
            .join("\n");
        format!("\n[extra.tag_counts]\n{rows}")
    };

    CONFIG_TOML
        .replace("__TAG_COUNTS__", &tag_counts_block)
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
        .replace("__CAROUSEL__", if s.carousel { "true" } else { "false" })
        .replace("__EMBED__", if s.embed { "true" } else { "false" })
        .replace("__HIDE_NAV__", if s.hide_nav { "true" } else { "false" })
        .replace(
            "__PINTEREST_SAVE__",
            if s.pinterest_save { "true" } else { "false" },
        )
        .replace("__PWA__", if s.pwa { "true" } else { "false" })
        .replace("__OFFLINE__", if s.offline { "true" } else { "false" })
        .replace("__CALENDAR__", if days.is_empty() { "false" } else { "true" })
        .replace(
            "__GOOGLE_FONT__",
            &match google_font_href(s.google_font.as_deref()) {
                Some(href) => format!("google_font_href = \"{href}\""),
                None => String::new(),
            },
        )
        .replace(
            "__GOOGLE_ANALYTICS__",
            &analytics_toml("google_analytics", s.google_analytics.as_deref()),
        )
        .replace(
            "__YANDEX_METRICA__",
            &analytics_toml("yandex_metrica", s.yandex_metrica.as_deref()),
        )
        .replace("__FEDI__", &fedi)
        .replace("__SEARCH__", &search)
        .replace(
            "__BUILD_SEARCH__",
            if matches!(s.search, Search::Elasticlunr) { "true" } else { "false" },
        )
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
                "jpg" | "jpeg" | "png" | "webp" | "avif" | "gif" | "svg" | "bmp" => &mut b.images,
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

/// The largest files (path, size) plus per-post preview text keyed by slug (for
/// their hover tooltips) — bundled so `set_about_size` stays within the argument
/// limit.
pub struct LargestFiles<'a> {
    pub files: &'a [(std::path::PathBuf, u64)],
    pub previews: &'a std::collections::HashMap<String, String>,
}

/// Fill the About page's size placeholders: `__TOTAL_SIZE__` (total),
/// `__PERCENT__` (share of the host limit, if known) and `__SIZE_BREAKDOWN__`
/// (per-kind sizes). Computed after media download, so it's the real footprint.
#[allow(clippy::too_many_arguments)]
pub fn set_about_size(
    site: &Path,
    b: &SizeBreakdown,
    limit: Option<u64>,
    elapsed: std::time::Duration,
    about: &crate::i18n::About,
    largest: &LargestFiles,
    mtproto_used: bool,
    now: &str,
    ci_url: Option<&str>,
    releases: u64,
    repo_url: &str,
) {
    let about_path = site.join("content/pages/about.md");
    let Ok(s) = fs::read_to_string(&about_path) else {
        return;
    };
    if !s.contains("__TOTAL_SIZE__") {
        return;
    }
    // The 10 biggest files as a list linking to each owning post (with the post's
    // text as a hover tooltip).
    let largest_block = if largest.files.is_empty() {
        String::new()
    } else {
        let items = largest
            .files
            .iter()
            .map(|(p, sz)| about_largest_link(p.strip_prefix(site).unwrap_or(p), *sz, largest.previews))
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
    kinds.sort_by_key(|k| std::cmp::Reverse(k.1));
    let breakdown = kinds
        .iter()
        .map(|(n, v)| format!("- **{n}** {}", human_size(*v)))
        .collect::<Vec<_>>()
        .join("\n");
    // Link the first "MTProto" mention to grammers, the library that implements
    // the backend (https://github.com/Lonami/grammers).
    let mtproto_line = if mtproto_used { about.mtproto_on } else { about.mtproto_off }
        .replacen("MTProto", "[MTProto](https://github.com/Lonami/grammers)", 1);
    // "Last updated <time>" with an optional link to the CI job that built it.
    let mut last_build = format!("{} **{now}**", about.last_updated);
    if let Some(url) = ci_url {
        last_build.push_str(&format!(" · [{}]({url})", about.build_log));
    }
    // Video offloaded to GitHub Releases doesn't count against the Pages quota,
    // so it's reported on its own line (omitted when there's none).
    let releases_line = if releases > 0 {
        let repo = repo_url.trim_end_matches('/');
        about
            .releases_size
            .replace("__RELEASES_SIZE__", &human_size(releases))
            .replace("GitHub Releases", &format!("[GitHub Releases]({repo}/releases)"))
    } else {
        String::new()
    };
    let out = s
        .replace("__TOTAL_SIZE__", &human_size(total))
        .replace("__PERCENT__", &percent)
        .replace("__RELEASES__", &releases_line)
        .replace("__SIZE_BREAKDOWN__", &breakdown)
        .replace("__LARGEST_FILES__", &largest_block)
        .replace("__BUILD_TIME__", &human_duration(elapsed))
        .replace("__LAST_BUILD__", &last_build)
        .replace("__MTPROTO__", &mtproto_line);
    let _ = fs::write(&about_path, out);
}

/// Fill the About page's `__ABOUT_ME__` placeholder with the scraped about.me
/// bio, social links and a contact button (about.me blocks iframing, so the
/// button links to the profile), or clear it.
pub fn set_about_me(
    site: &Path,
    am: Option<&crate::aboutme::AboutMe>,
    photo: Option<&str>,
    include_bio: bool,
    both_images: bool,
) {
    let about_path = site.join("content/pages/about.md");
    let Ok(mut s) = fs::read_to_string(&about_path) else {
        return;
    };
    if !s.contains("__ABOUT_ME__") {
        return;
    }
    // When an about.me photo is shown, drop the channel avatar by default so the
    // page doesn't lead with two portraits (unless the user asked for both).
    if photo.is_some() && !both_images {
        s = s.replace("{{ avatar() }}\n\n", "");
    }
    let block = match am {
        Some(am) if !am.is_empty() => {
            let mut b = String::new();
            if let Some(photo) = photo {
                b.push_str(&format!("{{{{ aboutme_photo(src=\"{photo}\") }}}}\n\n"));
            }
            if include_bio && !am.bio.is_empty() {
                b.push_str(am.bio.trim());
                b.push_str("\n\n");
            }
            // Skip any Telegram link — the channel is already linked at the top.
            let links: Vec<_> = am.links.iter().filter(|(_, url)| !is_telegram_url(url)).collect();
            if !links.is_empty() {
                let row = links
                    .iter()
                    .map(|(label, url)| format!("[{}]({url})", link_label(label)))
                    .collect::<Vec<_>>()
                    .join(" · ");
                b.push_str(&row);
                b.push_str("\n\n");
            }
            if !am.url.is_empty() {
                b.push_str(&format!(
                    "<a class=\"contact-btn\" href=\"{}\">✉ Message me on about.me</a>",
                    am.url
                ));
            }
            b.trim_end().to_string()
        }
        _ => String::new(),
    };
    let out = s.replace("__ABOUT_ME__", &block);
    let _ = fs::write(&about_path, out);
}

/// Fill the About page's `__WIKIDATA__` placeholder with a Wikidata statements
/// table (raw HTML from [`crate::wikidata::Table::to_html`]), or clear it.
pub fn set_about_wikidata(site: &Path, html: Option<&str>) {
    let about_path = site.join("content/pages/about.md");
    let Ok(s) = fs::read_to_string(&about_path) else {
        return;
    };
    if !s.contains("__WIKIDATA__") {
        return;
    }
    let _ = fs::write(&about_path, s.replace("__WIKIDATA__", html.unwrap_or("")));
}

/// True for a Telegram URL (t.me / telegram.me / telegram.org). Used to drop a
/// redundant Telegram social link from about.me — the channel is already at top.
fn is_telegram_url(url: &str) -> bool {
    let host = url
        .split_once("://")
        .map_or(url, |(_, rest)| rest)
        .split(['/', '?', '#'])
        .next()
        .unwrap_or("")
        .trim_start_matches("www.")
        .to_ascii_lowercase();
    matches!(host.as_str(), "t.me" | "telegram.me" | "telegram.org" | "telegram.dog")
}

/// A social-link label safe to drop into a Markdown `[label](url)` (drops the
/// brackets/parens that would break the link).
fn link_label(label: &str) -> String {
    label.chars().filter(|c| !matches!(c, '[' | ']' | '(' | ')')).collect()
}

/// Fill the About page's `__PAGESPEED__` placeholder with the Lighthouse scores,
/// or clear it when scoring is disabled / unavailable. Runs after
/// `set_about_size` (which leaves this placeholder untouched).
pub fn set_about_pagespeed(
    site: &Path,
    scores: Option<crate::pagespeed::Scores>,
    about: &crate::i18n::About,
    report_url: Option<&str>,
) {
    let about_path = site.join("content/pages/about.md");
    let Ok(s) = fs::read_to_string(&about_path) else {
        return;
    };
    if !s.contains("__PAGESPEED__") {
        return;
    }
    let block = match scores {
        Some(sc) if !sc.entries().is_empty() => {
            let items = sc
                .entries()
                .iter()
                .map(|(name, v)| format!("- **{name}** {v}"))
                .collect::<Vec<_>>()
                .join("\n");
            // Linkify the heading to the full PageSpeed report (mobile + desktop).
            let heading = match report_url {
                Some(u) => format!("[{}]({u})", about.pagespeed),
                None => about.pagespeed.to_string(),
            };
            format!("{heading}\n\n{items}")
        }
        _ => String::new(),
    };
    let out = s.replace("__PAGESPEED__", &block);
    let _ = fs::write(&about_path, out);
}

/// Write a shields.io endpoint JSON for each Lighthouse score under `static/`
/// (published at `<site>/lighthouse-<metric>.json`), returning how many were
/// written. README badges point at these via `img.shields.io/endpoint`.
pub fn write_pagespeed_badges(site: &Path, scores: &crate::pagespeed::Scores) -> Result<usize> {
    let dir = site.join("static");
    fs::create_dir_all(&dir)?;
    let metrics = [
        ("performance", "Performance", scores.performance),
        ("accessibility", "Accessibility", scores.accessibility),
        ("best-practices", "Best Practices", scores.best_practices),
        ("seo", "SEO", scores.seo),
    ];
    let mut written = 0;
    for (slug, label, score) in metrics {
        let Some(score) = score else { continue };
        let json = format!(
            r#"{{"schemaVersion":1,"label":"{label}","message":"{score}","color":"{}"}}"#,
            crate::pagespeed::badge_color(score)
        );
        write_file(&dir.join(format!("lighthouse-{slug}.json")), &json)?;
        written += 1;
    }
    Ok(written)
}

/// A markdown list item for a largest-file entry, linking to the owning post
/// (`content/posts/<slug>/…` → `@/posts/<slug>/index.md`, base_url-aware) with
/// that post's text as a hover `title` tooltip. Files outside a post bundle are
/// shown without a link.
fn about_largest_link(
    rel: &Path,
    size: u64,
    previews: &std::collections::HashMap<String, String>,
) -> String {
    let comps: Vec<&str> = rel
        .components()
        .filter_map(|c| c.as_os_str().to_str())
        .collect();
    let fname = comps.last().copied().unwrap_or("file");
    if comps.len() >= 3 && comps[0] == "content" && comps[1] == "posts" {
        let title = previews.get(comps[2]).map(|t| md_title_attr(t)).unwrap_or_default();
        format!(
            "- [{} — {fname}](@/posts/{}/index.md{title})",
            human_size(size),
            comps[2]
        )
    } else {
        format!("- {} — {fname}", human_size(size))
    }
}

/// A CommonMark link-title suffix ` "…"` (rendered as a hover `title` tooltip)
/// from a post's plain-text preview: whitespace collapsed to a single line,
/// capped, with `\` and `"` backslash-escaped so the title parses. Empty when
/// there's nothing to show.
fn md_title_attr(text: &str) -> String {
    let one_line = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if one_line.is_empty() {
        return String::new();
    }
    let mut capped: String = one_line.chars().take(300).collect();
    if one_line.chars().count() > 300 {
        capped.push('…');
    }
    let escaped = capped.replace('\\', "\\\\").replace('"', "\\\"");
    format!(" \"{escaped}\"")
}

/// Human-readable duration, e.g. `2m 30s` / `45s`.
fn human_duration(d: std::time::Duration) -> String {
    let s = d.as_secs();
    let (h, m, sec) = (s / 3600, (s % 3600) / 60, s % 60);
    if h > 0 {
        format!("{h}h {m}m {sec}s")
    } else if m > 0 {
        format!("{m}m {sec}s")
    } else {
        format!("{sec}s")
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
/// The `/day/<date>/` URL for the calendar, joined so a base_url without a
/// trailing slash doesn't glue into `…websiteday/` (a 404).
fn day_url(base_url: &str, day: &str) -> String {
    format!("{}/day/{day}/", base_url.trim_end_matches('/'))
}

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
                        "<td class=\"{cls}\"{title}><a href=\"{}\">{dd}</a></td>",
                        day_url(&s.base_url, &key)
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
            while !col.is_multiple_of(7) {
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
            b.push_str(&format!(
                "*{}*\n\n",
                about.intro.replace("{channel}", &channel_link)
            ));
            if let Some(info) = info {
                if let Some(desc) = &info.description_md {
                    b.push_str(desc.trim());
                    b.push_str("\n\n");
                }
                if !info.counters.is_empty() {
                    // Telegram labels its counter "photos"; call it "images".
                    let stats: Vec<String> = info
                        .counters
                        .iter()
                        .map(|(v, t)| {
                            let label = match t.as_str() {
                                "photos" => "images",
                                "photo" => "image",
                                other => other,
                            };
                            format!("**{v}** {label}")
                        })
                        .collect();
                    b.push_str(&stats.join(" · "));
                    b.push_str("\n\n");
                }
            }
            // Enrichment from an about.me link in the description (filled later).
            b.push_str("__ABOUT_ME__\n\n");
            // Optional Wikidata statements table for a configured QID.
            b.push_str("__WIKIDATA__\n\n");
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
            b.push_str("__RELEASES__\n\n");
            b.push_str(&format!("{}\n\n__SIZE_BREAKDOWN__\n\n", about.by_kind));
            b.push_str("__LARGEST_FILES__\n\n");
            b.push_str(&format!("{}\n\n", about.generated_in));
            b.push_str("__LAST_BUILD__\n\n");
            b.push_str("__PAGESPEED__\n\n");
            b.push_str("__MTPROTO__\n\n");
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

/// The body font-family CSS value: a Google font (quoted, with a system
/// fallback), a local/system stack from `font`, or the built-in default. User
/// input is stripped of CSS-structural characters so it can't break the rule.
fn font_family(google: Option<&str>, font: Option<&str>) -> String {
    let strip = |v: &str, drop_quotes: bool| -> String {
        v.chars()
            .filter(|c| {
                !(matches!(c, ';' | '{' | '}' | '\n' | '\r') || (drop_quotes && *c == '"'))
            })
            .collect::<String>()
            .trim()
            .to_string()
    };
    match (
        google.map(str::trim).filter(|g| !g.is_empty()),
        font.map(str::trim).filter(|f| !f.is_empty()),
    ) {
        (Some(g), _) => format!("\"{}\", system-ui, sans-serif", strip(g, true)),
        (None, Some(f)) => strip(f, false),
        (None, None) => "system-ui, -apple-system, sans-serif".to_string(),
    }
}

/// A `[extra]` line `key = "id"` for an analytics ID — sanitized to
/// alphanumeric / `-` / `_` so it can't break out of the TOML string or the
/// injected script — or empty when unset.
fn analytics_toml(key: &str, id: Option<&str>) -> String {
    let clean: String = id
        .unwrap_or("")
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_'))
        .collect();
    if clean.is_empty() {
        String::new()
    } else {
        format!("{key} = \"{clean}\"")
    }
}

/// The `fonts.googleapis.com` stylesheet URL for the configured Google font, if any.
fn google_font_href(google: Option<&str>) -> Option<String> {
    google.map(str::trim).filter(|g| !g.is_empty()).map(|g| {
        let fam: String = g
            .chars()
            .filter(|c| c.is_alphanumeric() || matches!(c, ' ' | '-'))
            .collect::<String>()
            .replace(' ', "+");
        format!("https://fonts.googleapis.com/css2?family={fam}:wght@400;600;700&display=swap")
    })
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
            "__FONT_FAMILY__",
            &font_family(s.google_font.as_deref(), s.font.as_deref()),
        )
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
build_search_index = __BUILD_SEARCH__
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
carousel = __CAROUSEL__
embed = __EMBED__
hide_nav = __HIDE_NAV__
pinterest_save = __PINTEREST_SAVE__
pwa = __PWA__
offline = __OFFLINE__
calendar = __CALENDAR__
__GOOGLE_FONT__
__FEDI__
__SEARCH__
__FOOTER__
__AVATAR__
__NAV__
__NAV_TAGS__
__TAGS__
__GOOGLE_ANALYTICS__
__YANDEX_METRICA__

[extra.i18n]
__I18N__
__TAG_COUNTS__
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
  {% if page.extra.og_image %}{% if page.extra.og_image is starting_with("http") %}{% set_global og_image = page.extra.og_image %}{% else %}{% set_global og_image = page.permalink ~ page.extra.og_image %}{% endif %}
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
  {% if config.extra.google_font_href %}<link rel="preconnect" href="https://fonts.googleapis.com"><link rel="preconnect" href="https://fonts.gstatic.com" crossorigin><link rel="stylesheet" href="{{ config.extra.google_font_href | safe }}">{% endif %}
  {% if config.extra.pwa %}<link rel="manifest" href="{{ get_url(path='manifest.webmanifest') | safe }}">{% endif %}
  <link rel="stylesheet" href="{{ get_url(path='style.css', cachebust=true) }}">
  {% if config.extra.google_analytics %}<script async src="https://www.googletagmanager.com/gtag/js?id={{ config.extra.google_analytics }}"></script><script>window.dataLayer=window.dataLayer||[];function gtag(){dataLayer.push(arguments);}gtag('js',new Date());gtag('config','{{ config.extra.google_analytics }}');</script>{% endif %}
  {% if config.extra.yandex_metrica %}<script>(function(m,e,t,r,i,k,a){m[i]=m[i]||function(){(m[i].a=m[i].a||[]).push(arguments)};m[i].l=1*new Date();for(var j=0;j<document.scripts.length;j++){if(document.scripts[j].src===r){return}}k=e.createElement(t),a=e.getElementsByTagName(t)[0],k.async=1,k.src=r,a.parentNode.insertBefore(k,a)})(window,document,'script','https://mc.yandex.ru/metrika/tag.js','ym');ym({{ config.extra.yandex_metrica }},'init',{clickmap:true,trackLinks:true,accurateTrackBounce:true});</script><noscript><div><img src="https://mc.yandex.ru/watch/{{ config.extra.yandex_metrica }}" style="position:absolute;left:-9999px" alt=""></div></noscript>{% endif %}
</head>
<body>
  {# current_path is undefined on the 404 page; default it so the nav's path
     tests below don't fail there. #}
  {% set current_path = current_path | default(value="") %}
  <header class="site-header">
    {% if config.extra.avatar %}<a href="{{ config.base_url | safe }}"><img class="site-avatar" src="{{ get_url(path=config.extra.avatar) }}" alt=""></a>{% endif %}
    <a class="site-title" href="{{ config.base_url | safe }}">{{ config.title }}</a>
    {% if not config.extra.hide_nav %}<nav>
      {% for t in config.extra.nav_tags | default(value=[]) %}<a class="tag" href="{{ get_taxonomy_url(kind='tags', name=t) | safe }}" title="{{ config.extra.tag_counts | get(key=t, default='') }}">#{{ t }}</a>{% endfor %}
      {% if current_path is matching("/tags/$") %}<span class="here">{{ config.extra.i18n.tags }}</span>{% else %}<a href="{{ get_url(path='/tags/') }}">{{ config.extra.i18n.tags }}</a>{% endif %}
      {% if config.extra.calendar %}{% if current_path is containing("/calendar/") %}<span class="here">{{ config.extra.i18n.calendar }}</span>{% else %}<a href="{{ get_url(path='/calendar/') }}">{{ config.extra.i18n.calendar }}</a>{% endif %}{% endif %}
      {% if current_path is containing("/about/") %}<span class="here">{{ config.extra.i18n.about }}</span>{% else %}<a href="{{ get_url(path='/about/') }}">{{ config.extra.i18n.about }}</a>{% endif %}
      {% for p in config.extra.nav | default(value=[]) %}<a href="{{ get_url(path=p.path) }}">{{ p.title }}</a>{% endfor %}
    </nav>{% endif %}
    {% if config.extra.search_google %}<form class="site-search" action="https://www.google.com/search" method="get" role="search"><input type="search" name="q" placeholder="{{ config.extra.i18n.search }}" aria-label="{{ config.extra.i18n.search_aria }}" autocomplete="off">{% if config.extra.search_site %}<input type="hidden" name="sitesearch" value="{{ config.extra.search_site }}">{% endif %}</form>{% elif config.extra.search_url %}<input type="search" id="site-search" class="site-search" placeholder="{{ config.extra.i18n.search }}" aria-label="{{ config.extra.i18n.search_aria }}" data-url="{{ config.extra.search_url | safe }}" autocomplete="off">{% elif config.extra.search_elasticlunr %}<span class="site-search els"><input type="search" id="site-search" placeholder="{{ config.extra.i18n.search }}" aria-label="{{ config.extra.i18n.search_aria }}" autocomplete="off"><ul id="search-results" class="search-results" hidden></ul></span>{% endif %}
  </header>
  <main>{% block content %}{% endblock content %}</main>
  {% if config.extra.footer %}<footer class="site-footer">{{ config.extra.footer | markdown(inline=true) | safe }}</footer>{% endif %}
  {% if config.extra.search_url %}<script>el=document.getElementById('site-search');el.addEventListener('keydown',function(e){if(e.key==='Enter'&&el.value)location.href=el.dataset.url+encodeURIComponent(el.value);});</script>{% endif %}
  {% if config.extra.search_elasticlunr %}<script src="{{ get_url(path='elasticlunr.min.js') | safe }}"></script><script src="{{ get_url(path='search_index.' ~ config.default_language ~ '.js') | safe }}"></script><script src="{{ get_url(path='search.js') | safe }}"></script>{% endif %}
  {% if config.extra.carousel %}<script src="{{ get_url(path='carousel.js') | safe }}"></script>{% endif %}
  {% if config.extra.embed %}<script src="{{ get_url(path='embed.js') | safe }}" defer></script>{% endif %}
  {% if config.extra.pinterest_save %}<script async defer src="//assets.pinterest.com/js/pinit.js" data-pin-hover="true"></script>{% endif %}
  {% if config.extra.pwa or config.extra.offline %}<script>if('serviceWorker' in navigator){addEventListener('load',function(){navigator.serviceWorker.register('{{ get_url(path='sw.js') | safe }}');});}</script>{% endif %}
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
  {% if page.extra.related %}
  <nav class="related">
    <strong>{{ config.extra.i18n.related }}:</strong>
    <ul>{% for r in page.extra.related %}<li><a href="{{ get_url(path=r.path) | safe }}">{{ r.label }}</a></li>{% endfor %}</ul>
  </nav>
  {% endif %}
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
    <li><a href="{{ get_taxonomy_url(kind='tags', name=t.name) | safe }}" title="{{ t.count }} {{ config.extra.i18n.posts }}">#{{ t.name }}</a> <a class="count" href="{{ get_url(path='/tags/' ~ t.slug ~ '/full/') | safe }}" title="{{ config.extra.i18n.full_posts }}">{{ t.count }}</a></li>
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

// Custom 404. Extends base.html, so it inherits style.css (and its
// prefers-color-scheme dark/light rules) — the page follows the OS theme like
// the rest of the site instead of falling back to the host's default 404.
const NOT_FOUND_HTML: &str = r#"{% extends "base.html" %}
{% block title %}404 · {{ config.title }}{% endblock title %}
{% block content %}
  <article class="not-found">
    <h1>404</h1>
    <p>{{ config.extra.i18n.not_found }}</p>
    <p><a href="{{ config.base_url | safe }}">{{ config.title }}</a></p>
  </article>
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

// A video that plays on YouTube but has embedding disabled: a thumbnail facade
// linking out to the watch page (no iframe), plus a labelled caption. Static
// HTML, so it survives the offline pass and needs no JavaScript.
const YOUTUBE_LINK_SHORTCODE: &str = r#"<div class="yt-embed"><a class="yt-facade" href="https://www.youtube.com/watch?v={{ id }}" target="_blank" rel="noopener"><img src="https://i.ytimg.com/vi/{{ id }}/hqdefault.jpg" alt="Watch on YouTube" loading="lazy"><span class="yt-btn" aria-hidden="true">▶</span></a></div>
<a class="yt-link" href="https://www.youtube.com/watch?v={{ id }}" target="_blank" rel="noopener">▶ Watch on YouTube</a>
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

// Spotify player (iframe). Over file:// it degrades to the "Open in Spotify"
// link. Plays a ~30s preview for non-Premium listeners.
const SPOTIFY_SHORTCODE: &str = r#"<div class="sp-embed"><iframe src="{{ url }}" height="152" loading="lazy" frameborder="0" allow="autoplay; clipboard-write; encrypted-media; fullscreen; picture-in-picture" sandbox="allow-forms allow-popups allow-same-origin allow-scripts allow-storage-access-by-user-activation allow-top-navigation-by-user-activation"></iframe><a class="sp-link" href="{{ url }}">Open in Spotify</a></div>
"#;

// Pinterest embedded pin (pinit.js turns the <a> into the pin). The offline pass
// strips the script, leaving the "View on Pinterest" link.
const PINTEREST_SHORTCODE: &str = r#"<a data-pin-do="embedPin" data-pin-width="large" href="{{ url }}">View on Pinterest</a>{% if not config.extra.pinterest_save %}<script async defer src="//assets.pinterest.com/js/pinit.js"></script>{% endif %}
"#;

// Bandcamp player. The EmbeddedPlayer iframe is a plain, self-contained iframe
// (no JS shim), so it works offline once cached and needs no fallback swap.
const BANDCAMP_SHORTCODE: &str = r#"<div class="bc-embed"><iframe src="{{ url }}" seamless loading="lazy" frameborder="0"></iframe></div>
"#;

// VK playlist widget (openapi.js turns the div into the player). Login/region
// gated, so the div holds a fallback "Open on VK" link that stays if the widget
// can't load; the offline pass strips the scripts, leaving just that link.
const VK_PLAYLIST_SHORTCODE: &str = r#"<div class="vk-playlist"><div id="{{ elid }}"><a href="{{ url | safe }}">▶ Open playlist on VK</a></div><script src="https://vk.com/js/api/openapi.js?169" async></script><script>(function(){function go(){if(window.VK&&VK.Widgets&&VK.Widgets.Playlist){VK.Widgets.Playlist("{{ elid }}",{{ owner }},{{ id }},"{{ key }}");}else{setTimeout(go,300);}}go();})();</script></div>
"#;

// Resolve colocated media against the post's permalink so it works both on the
// post page and when the post is shown in full on the homepage feed (a relative
// src would otherwise break off the post's own page).
const VIDEO_SHORTCODE: &str =
    "<video controls preload=\"metadata\" src=\"{{ page.permalink | safe }}{{ src }}\"></video>\n";
// Video hosted off-site (a GitHub Release asset) — the src is the absolute URL.
const VIDEO_EXT_SHORTCODE: &str =
    "<video controls preload=\"metadata\" src=\"{{ url | safe }}\"></video>\n";
// The full about.me profile photo (a base-aware static file).
const ABOUTME_PHOTO_SHORTCODE: &str =
    "<img class=\"about-photo\" src=\"{{ get_url(path=src) }}\" alt=\"\" loading=\"lazy\">\n";
const AUDIO_SHORTCODE: &str =
    "<audio controls src=\"{{ page.permalink | safe }}{{ src }}\"></audio>\n";

// Inline clickable hashtag → its taxonomy page (base_url-aware).
const TAG_SHORTCODE: &str =
    "<a class=\"tag\" href=\"{{ get_taxonomy_url(kind='tags', name=t) | safe }}\" title=\"{{ config.extra.tag_counts | get(key=t, default='') }}\">#{{ t }}</a>";

/// The bundled Elasticlunr library (0.9.5, http://elasticlunr.com/), written to
/// static/ only when Elasticlunr search is enabled (SEARCH_ENGINE=elasticlunr).
const ELASTICLUNR_JS: &str = include_str!("elasticlunr.min.js");

/// Client-side search wiring: load the Zola-generated index, query on input, and
/// show up to 10 result links. Runs after elasticlunr.min.js + search_index.<lang>.js
/// (the header emits the three scripts in that order).
const SEARCH_JS: &str = include_str!("search.js");

/// Carousel enhancement (prev/next arrows + dots) for `--carousel`. The swipe is
/// CSS scroll-snap, so touch works without this; kept in a real .js file.
const CAROUSEL_JS: &str = include_str!("carousel.js");

/// Iframe auto-resize helper for `--embed`: posts page height to the host page.
const EMBED_JS: &str = include_str!("embed.js");

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
  font-family: __FONT_FAMILY__; line-height: 1.6;
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
.yt-link { display: block; font-size: .9em; margin-top: .25rem; }
.contact-btn { display: inline-block; padding: .45rem .9rem; border-radius: 6px; background: var(--code-bg); color: var(--fg); border: 1px solid var(--border); text-decoration: none; font-size: .9em; transition: background .15s ease, border-color .15s ease, transform .1s ease; }
.contact-btn:hover { background: var(--input-bg); border-color: var(--muted); transform: translateY(-1px); }
.contact-btn:active { background: var(--border); transform: translateY(0); }
@media (prefers-reduced-motion: reduce) { .contact-btn { transition: none; } .contact-btn:hover { transform: none; } }
.about-photo { max-width: 320px; width: 100%; height: auto; border-radius: 10px; display: block; margin: .5rem 0; }
/* Wikidata statements table (About page + in-post). */
.wd { margin: 1rem 0; overflow-x: auto; }
.wd figcaption { font-weight: 600; margin-bottom: .3rem; }
.wd .wd-qid { color: var(--muted); font-weight: 400; font-size: .85em; }
.wd table { border-collapse: collapse; width: 100%; font-size: .9em; }
.wd th, .wd td { text-align: left; padding: .3rem .6rem; border-bottom: 1px solid var(--border); vertical-align: top; }
.wd th { color: var(--muted); font-weight: 500; }
.wd-spoiler { margin: 1rem 0; }
.wd-spoiler > summary { cursor: pointer; user-select: none; color: var(--link); }
.wd-spoiler > summary:hover { text-decoration: underline; }
.wd-spoiler .wd { margin-top: .5rem; }
/* Image carousel (opt-in) — swipe is native CSS scroll-snap; JS adds arrows/dots. */
.carousel { position: relative; margin: 1rem 0; }
.carousel-track { display: flex; overflow-x: auto; scroll-snap-type: x mandatory; scrollbar-width: none; }
.carousel-track::-webkit-scrollbar { display: none; }
.carousel-track img { scroll-snap-align: center; flex: 0 0 100%; width: 100%; height: auto; }
.carousel-prev, .carousel-next { position: absolute; top: 50%; transform: translateY(-50%); width: 2.2rem; height: 2.2rem; border: 0; border-radius: 50%; background: rgba(0,0,0,.55); color: #fff; font-size: 1.3rem; line-height: 1; cursor: pointer; }
.carousel-prev { left: .4rem; }
.carousel-next { right: .4rem; }
.carousel-dots { display: flex; gap: .35rem; justify-content: center; margin-top: .4rem; }
.carousel-dots button { width: .5rem; height: .5rem; padding: 0; border: 0; border-radius: 50%; background: var(--muted); cursor: pointer; }
.carousel-dots button.active { background: var(--fg); }
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
/* Genius (and, if ever embedded, Reddit) ship a light-theme widget with no dark
   option; under a dark OS theme approximate one by inverting luminance and
   rotating hues back so colours stay roughly true. The Genius song embed injects
   an inline `.rg_embed` card (not always an iframe), so target both — but not
   `.rg_embed_link`, the fallback link left after the offline pass strips the JS. */
@media (prefers-color-scheme: dark) {
  .rg_embed,
  iframe[src*="genius.com"],
  iframe[src*="redditmedia.com"],
  iframe[src*="embed.reddit.com"] { filter: invert(1) hue-rotate(180deg); }
}
.ym-embed .ym-link { display: block; font-size: .85rem; margin-top: .3rem; }
blockquote.instagram-media { max-width: 540px; margin: 1rem 0; padding: .5rem 1rem; }
.sp-embed { margin: 1rem 0; }
.sp-embed iframe { width: 100%; max-width: 660px; height: 152px; border: 0; border-radius: 12px; }
.sp-embed .sp-link { display: block; font-size: .85rem; margin-top: .3rem; }
.bc-embed { margin: 1rem 0; }
.bc-embed iframe { width: 100%; max-width: 400px; height: 120px; border: 0; }
.related { margin: 1rem 0 0; font-size: .9em; }
.related ul { margin: .2rem 0 0; padding-left: 1.2rem; }
/* Poll → static result bars (no JS). */
.tg-poll { margin: 1rem 0; padding: .6rem .8rem; border: 1px solid var(--border); border-radius: 8px; }
.tg-poll-q { font-weight: 600; margin-bottom: .5rem; }
.tg-poll-opts { list-style: none; margin: 0; padding: 0; }
.tg-poll-opts li { margin: .45rem 0; }
.tg-poll-row { display: flex; justify-content: space-between; gap: .5rem; font-size: .9em; }
.tg-poll-pct { color: var(--muted); flex: none; }
.tg-poll-bar { height: 6px; margin-top: .25rem; border-radius: 3px; background: var(--link); min-width: 2px; }
.tg-poll-voters { color: var(--muted); font-size: .85em; margin-top: .5rem; }
.vk-playlist { margin: 1rem 0; }
/* VK's widget is light-only; approximate a dark variant under a dark OS theme. */
@media (prefers-color-scheme: dark) { .vk-playlist iframe { filter: invert(1) hue-rotate(180deg); } }
.yt-embed .yt-toggle:checked ~ .yt-facade { display: none; }
.yt-embed .yt-toggle:checked ~ .yt-frame { display: block; }
.tag { white-space: nowrap; }
pre { background: var(--code-bg); padding: .75rem; border-radius: 4px; overflow-x: auto; }
code { background: var(--code-bg); padding: .1rem .3rem; border-radius: 4px; }
pre code { padding: 0; background: none; }
.site-header { display: flex; gap: .6rem; align-items: center; padding-bottom: .5rem; margin-bottom: 1rem; flex-wrap: wrap; }
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
.site-search.els { position: relative; margin-left: auto; }
.search-results {
  position: absolute; right: 0; top: 100%; z-index: 30;
  margin: .25rem 0 0; padding: 0; list-style: none;
  min-width: 14rem; max-width: min(28rem, 80vw); max-height: 60vh; overflow: auto;
  background: var(--bg); border: 1px solid var(--border); border-radius: 6px;
}
.search-results[hidden] { display: none; }
.search-results li { margin: 0; padding: 0; }
.search-results a { display: block; padding: .4rem .6rem; text-decoration: none; color: var(--fg); }
.search-results a:hover, .search-results a:focus { background: var(--input-bg); }
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::path::PathBuf;

    #[test]
    fn day_url_keeps_the_slash_after_base() {
        // A base_url with no trailing slash must not glue into "…websiteday/".
        assert_eq!(day_url("https://x.io/repo", "2025-12-28"), "https://x.io/repo/day/2025-12-28/");
        assert_eq!(day_url("https://x.io/repo/", "2025-12-28"), "https://x.io/repo/day/2025-12-28/");
        assert_eq!(day_url("/", "2025-12-28"), "/day/2025-12-28/");
    }

    #[test]
    fn telegram_urls_are_recognized() {
        assert!(is_telegram_url("https://t.me/some_channel"));
        assert!(is_telegram_url("http://www.telegram.me/x"));
        assert!(is_telegram_url("https://telegram.org"));
        assert!(!is_telegram_url("https://github.com/t.me"));
        assert!(!is_telegram_url("https://example.com/telegram"));
    }

    #[test]
    fn md_title_collapses_escapes_and_caps() {
        assert_eq!(md_title_attr(""), "");
        assert_eq!(md_title_attr("   \n\t "), "");
        // Newlines / runs of whitespace collapse to single spaces.
        assert_eq!(md_title_attr("hello\n\n  world"), " \"hello world\"");
        // `"` and `\` are backslash-escaped so the CommonMark title parses.
        assert_eq!(
            md_title_attr(r#"a "quote" and \ back"#),
            r#" "a \"quote\" and \\ back""#
        );
        // Over-long text is capped (300 chars) with an ellipsis.
        let out = md_title_attr(&"x".repeat(400));
        assert!(out.ends_with("…\""), "{out}");
        assert_eq!(out.chars().filter(|&c| c == 'x').count(), 300);
    }

    #[test]
    fn analytics_toml_sanitizes_and_omits() {
        assert_eq!(
            analytics_toml("google_analytics", Some("G-ABC123")),
            "google_analytics = \"G-ABC123\""
        );
        assert_eq!(analytics_toml("yandex_metrica", Some("12345678")), "yandex_metrica = \"12345678\"");
        // Injection characters are stripped.
        assert_eq!(analytics_toml("k", Some("a\"; alert(1)//")), "k = \"aalert1\"");
        // Empty / unset → nothing emitted.
        assert_eq!(analytics_toml("k", None), "");
        assert_eq!(analytics_toml("k", Some("  ")), "");
    }

    #[test]
    fn font_family_and_google_href() {
        // Default when neither is set.
        assert_eq!(font_family(None, None), "system-ui, -apple-system, sans-serif");
        // A local stack is used verbatim (sans CSS-structural chars).
        assert_eq!(font_family(None, Some("Georgia, serif")), "Georgia, serif");
        // A Google font is quoted with a fallback and wins over `font`.
        assert_eq!(
            font_family(Some("Open Sans"), Some("Georgia")),
            "\"Open Sans\", system-ui, sans-serif"
        );
        // Injection attempts have their CSS-structural characters stripped.
        let injected = font_family(None, Some("x; } body{display:none"));
        assert!(
            !injected.contains([';', '{', '}']),
            "structural chars not stripped: {injected}"
        );
        assert!(!font_family(Some("a\"}"), None).contains('}'));

        // The Google Fonts URL encodes spaces and is None when unset.
        assert_eq!(
            google_font_href(Some("Open Sans")),
            Some("https://fonts.googleapis.com/css2?family=Open+Sans:wght@400;600;700&display=swap".into())
        );
        assert_eq!(google_font_href(None), None);
        assert_eq!(google_font_href(Some("  ")), None);
    }

    #[test]
    fn largest_link_tooltip_and_linking() {
        let mut previews = HashMap::new();
        previews.insert("2024-01-02-7".to_string(), "post body".to_string());

        // Known post → internal link carrying the body as a title tooltip.
        let link = about_largest_link(&PathBuf::from("content/posts/2024-01-02-7/photo.jpg"), 2048, &previews);
        assert!(link.contains("(@/posts/2024-01-02-7/index.md \"post body\")"), "{link}");
        assert!(link.contains("photo.jpg"), "{link}");

        // Post with no preview on record → link, but no title.
        let link2 = about_largest_link(&PathBuf::from("content/posts/1999-12-31-1/a.mp4"), 1024, &previews);
        assert!(link2.contains("(@/posts/1999-12-31-1/index.md)"), "{link2}");
        assert!(!link2.contains('"'), "no title expected: {link2}");

        // File outside a post bundle → plain text, no link.
        let link3 = about_largest_link(&PathBuf::from("static/search.js"), 512, &previews);
        assert!(!link3.contains("]("), "no link expected: {link3}");
        assert!(link3.contains("search.js"), "{link3}");
    }
}
