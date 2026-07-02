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
        keep_media: false,
        genius: false,
        liveness: false,
        tags_to_pages: None,
        pages: None,
        posts_per_page: 20,
        title_max_len: 200,
        background_dark: "#000000".into(),
        background_light: "#ffffff".into(),
        css: None,
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
        body_md: body.into(),
        tags: tags.iter().map(|t| t.to_string()).collect(),
        media,
        views: Some(42),
        edited: false,
        links: vec![],
        youtube: youtube.map(String::from),
        apple_podcast: None,
        yandex_music: None,
        instagram: instagram.map(String::from),
        youtube_dead: false,
        apple_dead: false,
        yandex_dead: false,
        instagram_dead: false,
        genius_song_id: None,
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
            &["greeting"],
            vec![Media::Photo {
                url: "https://example.com/a.jpg".into(),
                key: Some("k1".into()),
            }],
            None,
            None,
        ),
        // Attached video + a live YouTube link → the embed replaces the video.
        post(2, "Watch this clip.", &[], vec![Media::Video { url: "https://example.com/v.mp4".into() }], Some("dQw4w9WgXcQ"), None),
        // Attached video + a live Instagram link → the Instagram embed.
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

    // Post 3: the Instagram embed (blockquote + embed.js) replaced the video.
    let p3 = read("posts/3/index.html");
    assert!(p3.contains("instagram-media"), "instagram embed missing:\n{p3}");
    assert!(p3.contains("embed.js"), "embed.js should be present before the offline pass");
    assert!(!p3.contains("<video"), "attached video not dropped for the IG embed:\n{p3}");

    // Post 4: a downloaded MTProto video plays as a <video>, and is #video-tagged.
    let p4 = read("posts/4/index.html");
    assert!(p4.contains("<video"), "LocalVideo <video> missing:\n{p4}");
    assert!(public.join("tags/video/index.html").exists(), "video tag page missing");

    // The offline pass strips scripts (the IG embed degrades to its link) and
    // rewrites to relative links, so the copy opens from file://.
    crate::offline::relativize(&public).expect("offline relativize");
    let p3o = read("posts/3/index.html");
    assert!(p3o.contains("instagram-media"), "IG blockquote should remain after offline");
    assert!(!p3o.contains("embed.js"), "offline pass should strip embed.js:\n{p3o}");
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

    let _ = fs::remove_dir_all(&dir);
}
