//! End-to-end test: drive the real render → scaffold → write_site pipeline, run
//! the actual `zola build`, and assert on the generated HTML (plus the offline
//! pass). Skipped when the `zola` binary isn't on `PATH`, so `cargo test` still
//! passes in environments without it.

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fs;
use std::path::PathBuf;
use std::process::Command;

use crate::config::{Search, Settings};
use crate::model::{Media, Post};
use crate::render::{self, RenderedPost};
use crate::site::{self, DayMeta};

fn zola_available() -> bool {
    Command::new("zola")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Whether the e2e test may proceed. In CI (where the workflow installs zola) a
/// missing binary is a hard failure — the HTML checks must not be silently
/// skipped; on a local machine without zola it just skips.
fn zola_ready() -> bool {
    if zola_available() {
        return true;
    }
    assert!(
        std::env::var("CI").is_err(),
        "zola is not on PATH in CI — the end-to-end HTML test must run (the workflow installs it)"
    );
    eprintln!("skipping e2e: `zola` not on PATH");
    false
}

fn settings(site: PathBuf) -> Settings {
    Settings {
        channel: "testchan".into(),
        title: "Test Channel".into(),
        description: "e2e".into(),
        base_url: "/".into(),
        site,
        repo_url: "https://github.com/x/y".into(),
        about: None,
        tags_footer: false,
        next_prev: true,
        telegram_link: true,
        rss: true,
        podcast: false,
        podcast_tagged: false,
        fediverse_creator: None,
        search: Search::None,
        footer: None,
        pages_host: None,
        date_format: "%Y %B %d".into(),
        language: "en".into(),
        derive_titles: false,
        strip_title: false,
        link_underline: false,
        youtube_facade: false,
        carousel: false,
        embed: false,
        hide_nav: false,
        aboutme_bio: false,
        aboutme_both_images: false,
        wikidata: None,
        wikidata_spoiler: false,
        link_titles: false,
        bandcamp: false,
        vk: false,
        related: false,
        dedup: false,
        keep_media: false,
        genius: false,
        spotify: false,
        instagram: false,
        pinterest: false,
        pinterest_save: false,
        pagespeed: false,
        pwa: false,
        offline: false,
        video_releases: false,
        liveness: false,
        reactions: false,
        about_me: false,
        tags_to_pages: None,
        pages: None,
        posts_per_page: 20,
        title_max_len: 200,
        background_dark: "#000000".into(),
        background_light: "#ffffff".into(),
        css: None,
        font: None,
        google_font: None,
        google_analytics: None,
        yandex_metrica: None,
        theme: None,
        max_pages: None,
        page_delay_ms: 0,
        concurrency: 1,
        group_window_secs: 1,
        download_media: false,
    }
}

fn post(id: u64, body: &str, tags: &[&str], media: Vec<Media>, youtube: Option<&str>, instagram: Option<&str>) -> Post {
    use chrono::TimeZone;
    Post {
        primary_id: id,
        ids: vec![id],
        channel: "testchan".into(),
        date: chrono::FixedOffset::east_opt(0)
            .unwrap()
            .timestamp_opt(1_700_000_000 + id as i64 * 86_400, 0)
            .unwrap(),
        author: None,
        forwarded_from: None,
        reply: None,
        poll: None,
        body_md: body.into(),
        tags: tags.iter().map(|t| t.to_string()).collect(),
        media,
        views: Some(42),
        edited: false,
        reactions: vec![],
        links: vec![],
        youtube: youtube.map(String::from),
        apple_podcast: None,
        yandex_music: None,
        instagram: instagram.map(String::from),
        spotify: None,
        pinterest: None,
        youtube_dead: false,
        youtube_watchable: false,
        apple_dead: false,
        yandex_dead: false,
        instagram_dead: false,
        spotify_dead: false,
        pinterest_dead: false,
        genius_song_id: None,
        bandcamp: None,
        vk_playlist: None,
        related: vec![],
        wikidata_html: vec![],
    }
}

#[test]
fn zola_build_produces_expected_html() {
    if !zola_ready() {
        return;
    }

    let dir = std::env::temp_dir().join(format!("tg2zola-e2e-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    let s = settings(dir.join("site"));

    let mut posts = vec![
        post(
            1,
            "Hello world with a photo.",
            // A non-ASCII tag guards the tag-count tooltip lookup: a `[t]` map
            // subscript on a Cyrillic key panics Tera mid-char (broke the daily
            // build); the `get(key=t)` filter must handle it.
            &["greeting", "александровщина"],
            vec![Media::Photo {
                url: "https://example.com/a.jpg".into(),
                key: Some("k1".into()),
            }],
            None,
            None,
        ),
        // Attached video + a live YouTube link → the embed replaces the video.
        post(2, "Watch this clip.", &[], vec![Media::Video { url: "https://example.com/v.mp4".into() }], Some("dQw4w9WgXcQ"), None),
        // Attached video + an Instagram link → the video is kept (IG embed is
        // opt-in and off by default).
        post(
            3,
            "A reel.",
            &[],
            vec![Media::Video { url: "https://example.com/r.mp4".into() }],
            None,
            Some("https://www.instagram.com/reel/ABC123/"),
        ),
        // A downloaded MTProto video (opt-in MTPROTO_VIDEOS): no link, so it plays.
        post(
            4,
            "MTProto video.",
            &[],
            vec![Media::LocalVideo { path: PathBuf::from("/nonexistent/4.mp4") }],
            None,
            None,
        ),
    ];
    // Auto-tag posts with a playable video #video (mirrors main::run) so the
    // {{ tag(t="video") }} their body emits resolves against the taxonomy.
    for p in &mut posts {
        let has_video = p
            .media
            .iter()
            .any(|m| matches!(m, Media::Video { .. } | Media::LocalVideo { .. }));
        if has_video && !p.tags.iter().any(|t| t == "video") {
            p.tags.push("video".to_string());
        }
    }

    // Drive the real pipeline (mirrors main::run from grouping onward).
    let rewriter = render::LinkRewriter::new(&s.channel, &posts);
    let ui = crate::i18n::ui(&s.language);

    let mut tc: HashMap<String, usize> = HashMap::new();
    let mut by_day: BTreeMap<String, (usize, BTreeSet<String>)> = BTreeMap::new();
    for p in &posts {
        let e = by_day.entry(p.date.format("%Y-%m-%d").to_string()).or_default();
        e.0 += 1;
        for t in &p.tags {
            *tc.entry(t.clone()).or_default() += 1;
            e.1.insert(t.clone());
        }
    }
    let tag_counts: Vec<(String, usize)> = tc.into_iter().collect();
    let days: Vec<DayMeta> = by_day
        .into_iter()
        .map(|(day, (count, tags))| DayMeta { day, count, tags: tags.into_iter().collect() })
        .collect();

    site::scaffold(&s, None, &tag_counts, &[], &days).expect("scaffold");
    let rendered: Vec<RenderedPost> = posts
        .iter()
        .map(|p| {
            render::render_post(
                p,
                &rewriter,
                false,
                None,
                None,
                &render::RenderOpts {
                    ui: &ui,
                    title_max: s.title_max_len,
                    derive_titles: false,
                    strip_title: false,
                    keep_media: s.keep_media,
                    spotify: false,
                    instagram: false,
                    pinterest: false,
                    video_releases: None,
                    carousel: false,
                },
            )
        })
        .collect();
    site::write_site(&s, &rendered).expect("write_site");

    // The real static-site build.
    let out = Command::new("zola")
        .arg("--root")
        .arg(&s.site)
        .arg("build")
        .output()
        .expect("run zola");
    assert!(
        out.status.success(),
        "zola build failed:\n{}",
        String::from_utf8_lossy(&out.stderr)
    );

    let public = s.site.join("public");
    let read = |rel: &str| fs::read_to_string(public.join(rel)).unwrap_or_else(|e| panic!("read {rel}: {e}"));

    // Core pages exist.
    assert!(public.join("index.html").exists(), "home page missing");
    assert!(public.join("about/index.html").exists(), "about page missing");
    assert!(public.join("rss.xml").exists(), "rss feed missing");
    assert!(public.join("tags/greeting/index.html").exists(), "tag page missing");

    // Each tag link carries a post-count hover tooltip (localized "N posts").
    let tags_page = read("tags/index.html");
    assert!(tags_page.contains("title=\"1 posts\""), "tag count tooltip missing: {tags_page}");

    // Custom 404 exists and links the themed stylesheet (which carries the
    // prefers-color-scheme dark/light rules), so it follows the OS theme.
    assert!(public.join("404.html").exists(), "404 page missing");
    let notfound = read("404.html");
    assert!(notfound.contains("style.css"), "404 doesn't link the theme stylesheet");
    assert!(notfound.contains("404"), "404 page missing its heading");

    // Post 1: the photo became an <img>, no video machinery.
    let p1 = read("posts/1/index.html");
    assert!(p1.contains("<img"), "photo <img> missing:\n{p1}");

    // Post 2: the live YouTube link is a real iframe, and the attached video was
    // dropped in favour of it.
    let p2 = read("posts/2/index.html");
    assert!(
        p2.contains("https://www.youtube.com/embed/dQw4w9WgXcQ"),
        "youtube iframe missing:\n{p2}"
    );
    assert!(!p2.contains("<video"), "attached video not dropped for the embed:\n{p2}");

    // Post 3: Instagram embedding is off by default (opt-in), so the attached
    // video is kept and no Instagram embed is emitted.
    let p3 = read("posts/3/index.html");
    assert!(p3.contains("<video"), "attached video should be kept when IG is off:\n{p3}");
    assert!(!p3.contains("instagram-media"), "no IG embed expected (off by default):\n{p3}");
    assert!(!p3.contains("embed.js"), "no IG embed.js expected (off by default):\n{p3}");

    // Post 4: a downloaded MTProto video plays as a <video>, and is #video-tagged.
    let p4 = read("posts/4/index.html");
    assert!(p4.contains("<video"), "LocalVideo <video> missing:\n{p4}");
    assert!(public.join("tags/video/index.html").exists(), "video tag page missing");

    // The offline pass strips scripts and rewrites to relative links, so the
    // copy opens from file://.
    crate::offline::relativize(&public).expect("offline relativize");
    let p3o = read("posts/3/index.html");
    assert!(p3o.contains("<video"), "kept video should remain after offline:\n{p3o}");
    assert!(!p3o.contains("<script"), "offline pass should strip all <script>:\n{p3o}");

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn elasticlunr_search_builds() {
    if !zola_ready() {
        return;
    }
    let dir = std::env::temp_dir().join(format!("tg2zola-els-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    let mut s = settings(dir.join("site"));
    s.search = Search::Elasticlunr;

    let posts = vec![post(1, "a searchable body of text", &[], vec![], None, None)];
    let rewriter = render::LinkRewriter::new(&s.channel, &posts);
    let ui = crate::i18n::ui(&s.language);
    site::scaffold(&s, None, &[], &[], &[]).expect("scaffold");
    let rendered: Vec<RenderedPost> = posts
        .iter()
        .map(|p| {
            render::render_post(
                p,
                &rewriter,
                false,
                None,
                None,
                &render::RenderOpts {
                    ui: &ui,
                    title_max: s.title_max_len,
                    derive_titles: false,
                    strip_title: false,
                    keep_media: s.keep_media,
                    spotify: false,
                    instagram: false,
                    pinterest: false,
                    video_releases: None,
                    carousel: false,
                },
            )
        })
        .collect();
    site::write_site(&s, &rendered).expect("write_site");

    let out = Command::new("zola")
        .arg("--root")
        .arg(&s.site)
        .arg("build")
        .output()
        .expect("run zola");
    assert!(out.status.success(), "zola build failed:\n{}", String::from_utf8_lossy(&out.stderr));

    let public = s.site.join("public");
    // Zola built the Elasticlunr index (http://elasticlunr.com/), and we bundled
    // the library + wiring.
    assert!(public.join("search_index.en.js").exists(), "search index not generated");
    assert!(public.join("elasticlunr.min.js").exists(), "elasticlunr library not bundled");
    assert!(public.join("search.js").exists(), "search wiring not bundled");
    let home = fs::read_to_string(public.join("index.html")).expect("home html");
    assert!(home.contains("search-results"), "search UI missing:\n{home}");

    // The offline pass keeps the local search scripts (they run from local files),
    // so client-side search works over file:// too.
    crate::offline::relativize(&public).expect("offline relativize");
    let home_off = fs::read_to_string(public.join("index.html")).expect("home html");
    assert!(home_off.contains("elasticlunr.min.js"), "search library dropped offline:\n{home_off}");
    assert!(home_off.contains("search.js"), "search wiring dropped offline");

    let _ = fs::remove_dir_all(&dir);
}

/// The About page's dynamic fields (filled by `set_about_size` after the media
/// footprint is known): the largest-files list links each file to its owning
/// post with the post text as a hover `title` tooltip, and the "MTProto" mention
/// links to grammers. Exercises the Markdown-title survival through Zola's `@/`
/// internal-link resolution end to end.
#[test]
fn about_page_renders_tooltip_and_mtproto_link() {
    if !zola_ready() {
        return;
    }
    let dir = std::env::temp_dir().join(format!("tg2zola-about-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    let s = settings(dir.join("site"));

    // One real post so the largest-file link's `@/posts/…` target resolves.
    let posts = vec![post(1, "The body of the biggest post.", &[], vec![], None, None)];
    let slug = render::slug_for(&posts[0]);

    let rewriter = render::LinkRewriter::new(&s.channel, &posts);
    let ui = crate::i18n::ui(&s.language);
    site::scaffold(&s, None, &[], &[], &[]).expect("scaffold");
    let rendered: Vec<RenderedPost> = posts
        .iter()
        .map(|p| {
            render::render_post(
                p,
                &rewriter,
                false,
                None,
                None,
                &render::RenderOpts {
                    ui: &ui,
                    title_max: s.title_max_len,
                    derive_titles: false,
                    strip_title: false,
                    keep_media: s.keep_media,
                    spotify: false,
                    instagram: false,
                    pinterest: false,
                    video_releases: None,
                    carousel: false,
                },
            )
        })
        .collect();
    site::write_site(&s, &rendered).expect("write_site");

    // Fill the About page: one largest file owned by the post (with the post
    // text as a tooltip) and the MTProto line (used → links to grammers).
    let breakdown = site::size_breakdown(&[&s.site.join("content"), &s.site.join("static")]);
    let biggest = vec![(s.site.join(format!("content/posts/{slug}/photo.jpg")), 123_456u64)];
    let mut previews = HashMap::new();
    previews.insert(slug.clone(), "Tooltip body of the biggest post.".to_string());
    site::set_about_size(
        &s.site,
        &breakdown,
        Some(1_000_000_000),
        std::time::Duration::from_secs(3),
        &crate::i18n::about(&s.language),
        &site::LargestFiles { files: &biggest, previews: &previews },
        true,
        "2026-07-04 12:00 UTC",
        Some("https://github.com/x/y/actions/runs/42"),
        7_654_321,
    );
    site::set_about_pagespeed(
        &s.site,
        Some(crate::pagespeed::Scores {
            performance: Some(98),
            accessibility: Some(100),
            best_practices: Some(92),
            seo: Some(85),
        }),
        &crate::i18n::about(&s.language),
    );
    site::set_about_me(&s.site, None, None, false, false);

    let out = Command::new("zola")
        .arg("--root")
        .arg(&s.site)
        .arg("build")
        .output()
        .expect("run zola");
    assert!(out.status.success(), "zola build failed:\n{}", String::from_utf8_lossy(&out.stderr));

    let about = fs::read_to_string(s.site.join("public/about/index.html")).expect("about html");
    // Placeholders were substituted.
    assert!(!about.contains("__TOTAL_SIZE__"), "size placeholder left unfilled:\n{about}");
    assert!(!about.contains("__LARGEST_FILES__"), "largest-files placeholder left unfilled");
    assert!(!about.contains("__PAGESPEED__"), "pagespeed placeholder left unfilled");
    // The Lighthouse scores render on the About page.
    assert!(about.contains("Performance") && about.contains("98"), "lighthouse scores missing:\n{about}");
    // The largest-file entry links to its post and carries the body as a tooltip.
    assert!(
        about.contains("title=\"Tooltip body of the biggest post.\""),
        "largest-file hover tooltip missing:\n{about}"
    );
    assert!(about.contains("photo.jpg"), "largest-file name missing");
    // The MTProto mention links to grammers.
    assert!(
        about.contains("https://github.com/Lonami/grammers"),
        "MTProto link missing:\n{about}"
    );

    let _ = fs::remove_dir_all(&dir);
}
