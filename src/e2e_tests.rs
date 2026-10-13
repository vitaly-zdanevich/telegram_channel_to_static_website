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

/// Recursively collect every `.html` file under `dir`.
fn collect_html(dir: &std::path::Path, out: &mut Vec<PathBuf>) {
    if let Ok(rd) = fs::read_dir(dir) {
        for e in rd.flatten() {
            let p = e.path();
            if p.is_dir() {
                collect_html(&p, out);
            } else if p.extension().and_then(|x| x.to_str()) == Some("html") {
                out.push(p);
            }
        }
    }
}

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

/// Untrusted discussion content must remain inert through TOML, Tera, and the
/// real HTML-minifying build, while preserving Telegram formatting and replies.
fn mock_discussion() -> crate::discussions::Discussion {
    use crate::discussions::{Comment, Discussion};

    Discussion {
        peer_id: 77,
        root_id: 10,
        post_id: 1,
        comments: vec![
            Comment {
                id: 11,
                author: "Discussion fixture <img src=x onerror=alert(1)>".into(),
                date: "2026-10-12T12:34:00+00:00".into(),
                body_html: concat!(
                    "<b onclick='alert(1)'>Formatted discussion</b>",
                    "<pre>    preserved_indent()</pre>",
                    "<script>alert('discussion injection')</script>",
                    "<iframe srcdoc='<script>alert(1)</script>'></iframe>",
                    "<a href='java&#x73;cript:alert(1)'>unsafe link</a>",
                    "<a href='https://example.test/comment?a=1&amp;b=2'>safe link</a>",
                    "<span class='tg-spoiler'>spoiler text</span>",
                    "\n{{ video(src=\"evil\") }} {% if true %}literal{% endif %}",
                    "\n+++\n[extra]\ndiscussion_injection = true\n+++\n"
                )
                .into(),
                reply_to: Some(10),
                has_media: false,
            },
            Comment {
                id: 12,
                author: "Аўтар".into(),
                date: "2026-10-12T12:35:00+00:00".into(),
                body_html: "Reply with an attachment".into(),
                reply_to: Some(11),
                has_media: true,
            },
            Comment {
                id: 13,
                author: "Bob".into(),
                date: "2026-10-12T12:36:00+00:00".into(),
                body_html: "Reply to an unavailable comment".into(),
                reply_to: Some(9),
                has_media: false,
            },
        ],
    }
}

/// Discussion snapshots render only on their own post, never in aggregate
/// pages, feeds, or the public snapshot file, including project-base builds.
fn assert_discussions(public: &std::path::Path) {
    let html = fs::read_to_string(public.join("posts/1/index.html")).unwrap();
    let document = scraper::Html::parse_document(&html);
    let select = |selector: &str| scraper::Selector::parse(selector).unwrap();
    let section = document
        .select(&select("section.discussions"))
        .next()
        .expect("discussion section missing on its post");
    assert_eq!(
        section
            .select(&select("h2"))
            .next()
            .unwrap()
            .text()
            .collect::<String>(),
        "Comments (3)"
    );
    let comments: Vec<_> = section
        .select(&select(".discussion-comments > li"))
        .collect();
    assert_eq!(comments.len(), 3);
    assert_eq!(comments[0].value().attr("id"), Some("comment-77-11"));
    assert_eq!(
        comments[0]
            .select(&select(".comment-author"))
            .next()
            .unwrap()
            .text()
            .collect::<String>(),
        "Discussion fixture <img src=x onerror=alert(1)>"
    );
    let date = comments[0].select(&select("time")).next().unwrap();
    assert_eq!(
        date.value().attr("datetime"),
        Some("2026-10-12T12:34:00+00:00")
    );
    assert_eq!(date.text().collect::<String>(), "2026-10-12 12:34 +00:00");
    assert!(comments[0]
        .select(&select(".comment-parent"))
        .next()
        .is_none());
    assert_eq!(
        comments[1]
            .select(&select(".comment-parent"))
            .next()
            .unwrap()
            .value()
            .attr("href"),
        Some("#comment-77-11")
    );
    assert_eq!(
        comments[2]
            .select(&select(".comment-parent"))
            .next()
            .unwrap()
            .value()
            .attr("href"),
        Some("https://t.me/testchan/1?comment=9")
    );
    let attachment = comments[1]
        .select(&select(".comment-attachment a"))
        .next()
        .unwrap();
    assert_eq!(
        attachment.value().attr("href"),
        Some("https://t.me/testchan/1?comment=12")
    );
    assert_eq!(
        attachment.text().collect::<String>(),
        "Attachment (not archived)"
    );
    let reply = section
        .select(&select("a"))
        .find(|link| link.text().collect::<String>() == "Reply on Telegram")
        .unwrap();
    assert_eq!(
        reply.value().attr("href"),
        Some("https://t.me/testchan/1?comment=0")
    );
    assert_eq!(
        section
            .select(&select("b"))
            .next()
            .unwrap()
            .text()
            .collect::<String>(),
        "Formatted discussion"
    );
    assert_eq!(
        section
            .select(&select("pre"))
            .next()
            .unwrap()
            .text()
            .collect::<String>(),
        "    preserved_indent()"
    );
    assert!(section
        .select(&select("details.comment-spoiler"))
        .next()
        .is_some());
    assert!(section
        .select(&select(
            "script, style, iframe, img, video, svg, [onclick], [onerror]"
        ))
        .next()
        .is_none());
    let body = comments[0]
        .select(&select(".comment-body"))
        .next()
        .unwrap()
        .text()
        .collect::<String>();
    assert!(
        body.contains("{{ video(src=\"evil\") }} {% if true %}literal{% endif %}"),
        "{body}"
    );
    assert!(body.contains("+++"), "{body}");
    assert!(
        body.contains("[extra] discussion_injection = true")
            || body.contains("[extra]\ndiscussion_injection = true"),
        "{body}"
    );
    assert!(!body.contains("discussion injection"));
    let safe = section
        .select(&select("a"))
        .find(|link| link.text().collect::<String>() == "safe link")
        .unwrap();
    assert_eq!(
        safe.value().attr("href"),
        Some("https://example.test/comment?a=1&b=2")
    );
    assert_eq!(safe.value().attr("rel"), Some("nofollow ugc"));
    assert!(!section
        .select(&select("a"))
        .any(|link| link.text().collect::<String>() == "unsafe link"));

    let mut pages = Vec::new();
    collect_html(public, &mut pages);
    for page in pages {
        if page == public.join("posts/1/index.html") {
            continue;
        }
        let html = fs::read_to_string(&page).unwrap();
        let document = scraper::Html::parse_document(&html);
        assert!(
            document
                .select(&select("section.discussions"))
                .next()
                .is_none(),
            "discussion leaked into {}",
            page.display()
        );
        assert!(
            !html.contains("Discussion fixture"),
            "comment author leaked into {}",
            page.display()
        );
    }
    let feed = fs::read_to_string(public.join("rss.xml")).unwrap();
    assert!(!feed.contains("Discussion fixture"));
    assert!(!feed.contains("Formatted discussion"));
    assert!(!public.join(".discussions.json").exists());
}

/// Both tag views link to each other using their own canonical paths. Zola
/// transliterates taxonomy slugs, unlike the generator's full-view slugs.
fn assert_tag_view_links(public: &std::path::Path, base: &str) {
    let heading_link = scraper::Selector::parse("main h1 a.count").unwrap();
    let origin = reqwest::Url::parse("https://example.test").unwrap();
    for (titles_slug, full_slug) in [
        ("greeting", "greeting"),
        ("aleksandrovshchina", "александровщина"),
    ] {
        let titles = format!("tags/{titles_slug}");
        let full = format!("tags/{full_slug}/full");
        for (source, target, label) in [(&titles, &full, "full ↗"), (&full, &titles, "titles ↗")]
        {
            let html = fs::read_to_string(public.join(source).join("index.html")).unwrap();
            let document = scraper::Html::parse_document(&html);
            let link = document
                .select(&heading_link)
                .next()
                .unwrap_or_else(|| panic!("{source} has no tag-view link"));
            let href = link
                .value()
                .attr("href")
                .expect("tag-view link has no href");
            assert_eq!(link.text().collect::<String>().trim(), label);
            assert_eq!(
                origin.join(href.trim_end_matches('/')).unwrap(),
                origin.join(&format!("{base}/{target}")).unwrap(),
                "wrong target from {source}"
            );
            assert!(public.join(target).join("index.html").exists());
        }
    }
}

/// Each titles-only row offers only that post's other tags, while preserving
/// its post/date destinations under both root and project-subpath deployments.
fn assert_tag_post_tags(public: &std::path::Path, base: &str) {
    let row_selector = scraper::Selector::parse(".tag-post-list > li").unwrap();
    let summary_selector = scraper::Selector::parse(".tag-post-summary").unwrap();
    let tags_selector = scraper::Selector::parse(".tag-post-tags").unwrap();
    let link_selector = scraper::Selector::parse("a").unwrap();
    let day_selector = scraper::Selector::parse("a.day").unwrap();
    let origin = reqwest::Url::parse("https://example.test").unwrap();
    let resolve = |href: &str| origin.join(href.trim_end_matches('/')).unwrap();
    let target = |path: &str| resolve(&format!("{base}/{path}"));
    let tags = [
        ("greeting", "greeting"),
        ("photography", "photography"),
        ("александровщина", "aleksandrovshchina"),
        ("video", "video"),
    ];
    let posts: [(u64, &[&str]); 4] = [
        (1, &["greeting", "photography", "александровщина"]),
        (2, &["greeting", "video"]),
        (3, &["video"]),
        (4, &["video"]),
    ];
    for (current_tag, slug) in tags {
        let html = fs::read_to_string(public.join(format!("tags/{slug}/index.html"))).unwrap();
        let document = scraper::Html::parse_document(&html);
        let rows: Vec<_> = document.select(&row_selector).collect();
        let expected_posts: Vec<_> = posts
            .iter()
            .filter(|(_, post_tags)| post_tags.contains(&current_tag))
            .collect();
        assert_eq!(
            rows.len(),
            expected_posts.len(),
            "wrong rows for #{current_tag}"
        );
        for (id, post_tags) in expected_posts {
            let post_url = target(&format!("posts/{id}"));
            let row = rows
                .iter()
                .find(|row| {
                    row.select(&summary_selector).any(|summary| {
                        summary.select(&link_selector).any(|link| {
                            link.value()
                                .attr("href")
                                .is_some_and(|href| resolve(href) == post_url)
                        })
                    })
                })
                .unwrap_or_else(|| panic!("#{current_tag}: post {id} lost its destination"));
            let summary = row.select(&summary_selector).next().unwrap();
            let post_link = summary
                .select(&link_selector)
                .find(|link| resolve(link.value().attr("href").unwrap()) == post_url)
                .unwrap();
            let label = post_link.text().collect::<String>();
            if *id == 1 {
                assert_eq!(label, "Hello world with a photo.");
            } else {
                assert_eq!(label, id.to_string());
            }
            let day = summary
                .select(&day_selector)
                .next()
                .expect("post date link missing");
            assert_eq!(
                resolve(day.value().attr("href").unwrap()),
                target(&format!("day/2023-11-{:02}", 14 + id)),
                "#{current_tag}: post {id} date destination changed"
            );
            let expected: Vec<_> = post_tags
                .iter()
                .filter(|&&tag| tag != current_tag)
                .collect();
            let containers: Vec<_> = row.select(&tags_selector).collect();
            if expected.is_empty() {
                assert!(
                    containers.is_empty(),
                    "post {id} has an empty other-tags container"
                );
                continue;
            }
            assert_eq!(
                containers.len(),
                1,
                "post {id} needs one other-tags container"
            );
            let links: Vec<_> = containers[0].select(&link_selector).collect();
            assert_eq!(
                links.len(),
                expected.len(),
                "post {id} has the wrong other tags"
            );
            for tag in expected {
                let link = links
                    .iter()
                    .find(|link| link.text().collect::<String>() == format!("#{tag}"))
                    .unwrap_or_else(|| panic!("post {id} is missing other tag #{tag}"));
                let (_, tag_slug) = tags.iter().find(|(name, _)| name == tag).unwrap();
                assert_eq!(
                    resolve(link.value().attr("href").unwrap()),
                    target(&format!("tags/{tag_slug}")),
                    "post {id}: #{tag} must link to its titles-only taxonomy page"
                );
                assert!(public.join(format!("tags/{tag_slug}/index.html")).exists());
            }
        }
    }
}

/// Only full-post listings expose keyboard destinations. The built-in helper
/// loads correctly at both root and project base URLs.
fn assert_post_navigation(public: &std::path::Path, base: &str) {
    let posts = scraper::Selector::parse("article.post[data-post-id]").unwrap();
    let scripts = scraper::Selector::parse("script[src]").unwrap();
    let origin = reqwest::Url::parse("https://example.test").unwrap();
    let script_url = origin.join(&format!("{base}/post-navigation.js")).unwrap();
    let cases: [(&str, &[&str]); 10] = [
        ("index.html", &["4", "3"]),
        ("page/2/index.html", &["2", "1"]),
        ("tags/greeting/full/index.html", &["2", "1"]),
        ("day/2023-11-15/index.html", &["1"]),
        ("day/2023-11-16/index.html", &["2"]),
        ("day/2023-11-18/index.html", &["4"]),
        ("posts/1/index.html", &[]),
        ("tags/greeting/index.html", &[]),
        ("calendar/index.html", &[]),
        ("about/index.html", &[]),
    ];
    for (path, ids) in cases {
        let html = fs::read_to_string(public.join(path))
            .unwrap_or_else(|error| panic!("reading {path}: {error}"));
        let document = scraper::Html::parse_document(&html);
        let articles: Vec<_> = document.select(&posts).collect();
        assert_eq!(articles.len(), ids.len(), "wrong keyboard targets: {path}");
        for (article, id) in articles.iter().zip(ids) {
            assert_eq!(article.value().attr("data-post-id"), Some(*id));
            assert_eq!(
                article.value().attr("id"),
                Some(format!("post-{id}").as_str())
            );
            assert_eq!(article.value().attr("tabindex"), Some("-1"));
        }
        let script = document
            .select(&scripts)
            .find(|script| origin.join(script.value().attr("src").unwrap()).unwrap() == script_url)
            .unwrap_or_else(|| panic!("missing keyboard script: {path}"));
        assert!(script.value().attr("defer").is_some(), "{path}");
        // Crossing a listing boundary follows its existing Older/Newer links,
        // including day views containing only one post. Individual post pages
        // keep their separate navigation and must not become listing targets.
        let next = match path {
            "index.html" => Some("page/2"),
            "day/2023-11-16/index.html" => Some("day/2023-11-15"),
            "day/2023-11-18/index.html" => Some("day/2023-11-17"),
            _ => None,
        };
        let previous = match path {
            "page/2/index.html" => Some(""),
            "day/2023-11-15/index.html" => Some("day/2023-11-16"),
            "day/2023-11-16/index.html" => Some("day/2023-11-17"),
            _ => None,
        };
        for (attribute, target) in [("data-post-next", next), ("data-post-prev", previous)] {
            let selector = scraper::Selector::parse(&format!("a[{attribute}]")).unwrap();
            let links: Vec<_> = document.select(&selector).collect();
            assert_eq!(
                links.len(),
                usize::from(target.is_some()),
                "wrong {attribute} link count: {path}"
            );
            if let Some(target) = target {
                let href = links[0].value().attr("href").unwrap();
                assert_eq!(
                    origin.join(href.trim_end_matches('/')).unwrap(),
                    origin
                        .join(format!("{base}/{target}").trim_end_matches('/'))
                        .unwrap(),
                    "wrong {attribute} destination: {path}"
                );
            }
        }
    }
    assert!(public.join("post-navigation.js").exists());
}

/// The keyboard helper belongs to the built-in templates; changing themes
/// removes the old copy rather than silently injecting it into another theme.
#[test]
fn post_navigation_asset_follows_builtin_theme() {
    let dir = std::env::temp_dir().join(format!("tg2zola-hotkeys-{}", std::process::id()));
    let mut s = settings(dir.join("site"));
    site::scaffold(&s, None, &[], &[], &[], &[]).unwrap();
    let script = s.site.join("static/post-navigation.js");
    assert!(script.exists(), "built-in keyboard helper missing");
    s.theme = Some("external-theme".into());
    site::scaffold(&s, None, &[], &[], &[], &[]).unwrap();
    assert!(
        !script.exists(),
        "keyboard helper leaked into external theme"
    );
    fs::remove_dir_all(dir).unwrap();
}

/// The scheduled backup must not force MTProto's potentially multi-gigabyte
/// download cache into ordinary Git history.
#[test]
fn daily_blog_backup_excludes_mtproto_cache() {
    let workflow = include_str!("../.github/workflows/daily.yml");
    let (_, backup_step) = workflow
        .split_once("- name: Commit site to blog branch")
        .expect("daily workflow must contain the blog backup step");

    assert!(
        backup_step.contains("--exclude '/.mtproto-cache'"),
        "the blog snapshot must exclude .mtproto-cache"
    );
    assert!(
        backup_step.contains("rm -rf -- \"${work:?}/site/.mtproto-cache\""),
        "the blog snapshot must remove cache files already tracked on the blog branch"
    );
}

/// Original Telegram images are staged outside Pages, uploaded in bounded
/// release buckets, and only then recorded as reusable.
#[test]
fn daily_uploads_and_persists_original_image_releases() {
    let workflow = include_str!("../.github/workflows/daily.yml");
    let inventory = workflow
        .find("- name: Inventory GitHub Release media")
        .expect("daily workflow must inventory remote release assets");
    let build = workflow
        .find("- name: Build tg2zola")
        .expect("daily workflow must build tg2zola");
    let commit = workflow
        .find("- name: Commit site to blog branch")
        .expect("daily workflow must persist the confirmed manifest");
    let cleanup = workflow
        .find("- name: Clean superseded image Release assets")
        .expect("daily workflow must prune immutable asset versions");
    let (_, upload_step) = workflow
        .split_once("- name: Upload large assets to GitHub Releases")
        .expect("daily workflow must contain the release upload step");
    let (upload_step, backup_step) = upload_step
        .split_once("- name: Commit site to blog branch")
        .expect("release uploads must happen before the blog backup");

    assert!(
        inventory < build,
        "release inventory must exist before the build"
    );
    assert!(workflow.contains("site/.image-releases.remote.tsv"));
    assert!(workflow.contains("RELEASES_SIZE_BYTES=$total"));
    assert!(workflow.contains(r#".tag_name == "media""#));
    assert!(workflow.contains(r#"startswith("images-")"#));
    assert!(workflow.contains("[ -f site/.image-releases.json ]"));
    assert!(upload_step.contains("site/.image-releases.pending.json"));
    assert!(upload_step.contains("site/.image-releases"));
    assert!(upload_step.contains(r#"gh release upload "$tag" "${uploads[@]}""#));
    assert!(!upload_step.contains(r#"gh release upload "$tag" "${uploads[@]}" --clobber"#));
    assert!(upload_step.contains("remote_size") && upload_step.contains("local_size"));
    let repair_upload = upload_step
        .find(r#"gh release upload "$tag" "$repair_path""#)
        .expect("a corrupt canonical asset must be replaced only after staging repair bytes");
    let corrupt_delete = upload_step
        .find("gh api --method DELETE")
        .expect("the corrupt canonical asset must be removed after repair upload");
    let repair_rename = upload_step
        .find("gh api --method PATCH")
        .expect("the validated repair asset must assume the canonical name");
    assert!(repair_upload < corrupt_delete && corrupt_delete < repair_rename);
    assert!(upload_step.contains("repair_size") && upload_step.contains("local_size"));
    assert!(upload_step.contains(r#".images[] | [.tag, .asset, (.bytes | tostring)]"#));
    assert!(upload_step.contains("manifest_tag") && upload_step.contains("expected_size"));
    assert!(
        !upload_step.contains("validated == 0"),
        "an empty or source-only pending manifest must be promotable"
    );
    // Videos and archives retain their pre-existing fixed-release semantics.
    assert!(upload_step.contains("gh release upload media site/.video-releases/* --clobber"));
    assert!(backup_step.contains("--exclude '/.image-releases'"));
    assert!(backup_step.contains("--exclude '/.image-releases.remote.tsv'"));
    assert!(backup_step.contains("site/.image-releases.remote.tsv"));
    assert!(
        commit < cleanup,
        "cleanup must follow deployment and manifest backup"
    );
    assert!(workflow.contains(r#"^telegram-image-([0-9]+)-"#));
    assert!(workflow.contains("Deleting superseded Release asset"));
}

/// Feature-gated regressions must run in CI, not merely compile under Clippy.
#[test]
fn build_runs_mtproto_tests() {
    let workflow = include_str!("../.github/workflows/build.yml");
    let clippy = workflow
        .split_once("  clippy:")
        .and_then(|(_, tail)| tail.split_once("\n  biome:").map(|(job, _)| job))
        .expect("build workflow must contain the Clippy job");
    let zola = clippy
        .find("- name: Install zola")
        .expect("the MTProto test job must install Zola for end-to-end tests");
    let tests = clippy
        .find("cargo test --locked --features mtproto")
        .expect("feature-gated regressions must run in CI");
    assert!(zola < tests, "Zola must be installed before MTProto tests");
    assert!(clippy.contains("${ZOLA_VERSION}"));
}

/// Oversized Pages output must fail before replacing the last good deployment.
#[test]
fn daily_checks_pages_size_before_deploying() {
    let workflow = include_str!("../.github/workflows/daily.yml");
    let pwa = workflow
        .find("- name: Write PWA precache list")
        .expect("daily workflow must write the PWA precache list");
    let size = workflow
        .find("- name: Check Pages artifact size")
        .expect("daily workflow must check the built site size");
    let upload = workflow
        .find("- name: Upload Pages artifact")
        .expect("daily workflow must upload the Pages artifact");

    assert!(pwa < size, "size guard must include generated PWA files");
    assert!(size < upload, "size guard must run before the Pages upload");
    assert!(workflow.contains("bytes >= 1073741824"));
}

/// The scheduled build must expose the opt-out that retains a Pinterest post's
/// local image beside its confirmed-live widget.
#[test]
fn daily_wires_pinterest_keep_image_toggle() {
    let workflow = include_str!("../.github/workflows/daily.yml");
    assert!(
        workflow.contains("PINTEREST_KEEP_IMAGE: ${{ vars.PINTEREST_KEEP_IMAGE }}"),
        "daily workflow must read the Pinterest image repository variable"
    );
    assert!(
        workflow.contains(
            r#"[ "${PINTEREST_KEEP_IMAGE:-}" = "true" ] && args+=(--pinterest-keep-image)"#
        ),
        "daily workflow must pass the Pinterest image override to tg2zola"
    );
}

/// The scheduled build must expose the default-on post-header line opt-out.
#[test]
fn daily_wires_post_header_line_toggle() {
    let workflow = include_str!("../.github/workflows/daily.yml");
    assert!(
        workflow.contains("POST_HEADER_LINE: ${{ vars.POST_HEADER_LINE }}"),
        "daily workflow must read the post-header line repository variable"
    );
    assert!(
        workflow
            .contains(r#"[ "${POST_HEADER_LINE:-}" = "false" ] && args+=(--no-post-header-line)"#),
        "daily workflow must pass the post-header line opt-out to tg2zola"
    );
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
        video_podcast: false,
        fediverse_creator: None,
        search: Search::None,
        footer: None,
        pages_host: None,
        date_format: "%Y %B %d".into(),
        language: "en".into(),
        derive_titles: false,
        strip_title: false,
        link_underline: false,
        post_header_line: true,
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
        dead_links: false,
        sqlite: None,
        enex: None,
        keep_media: false,
        genius: false,
        spotify: false,
        instagram: false,
        pinterest: false,
        pinterest_keep_image: false,
        pinterest_save: false,
        pagespeed: false,
        pwa: false,
        offline: false,
        video_releases: false,
        liveness: false,
        reactions: false,
        discussions: true,
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

fn post(
    id: u64,
    body: &str,
    tags: &[&str],
    media: Vec<Media>,
    youtube: Option<&str>,
    instagram: Option<&str>,
) -> Post {
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
        pinterest_live: false,
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
    let mut s = settings(dir.join("site"));
    s.posts_per_page = 2;
    s.related = true;

    let mut posts = vec![
        post(
            1,
            "Hello world with a photo.",
            // A non-ASCII tag guards the tag-count tooltip lookup: a `[t]` map
            // subscript on a Cyrillic key panics Tera mid-char (broke the daily
            // build); the `get(key=t)` filter must handle it.
            &["greeting", "photography", "александровщина"],
            vec![Media::Photo {
                url: "https://example.com/a.jpg".into(),
                key: Some("k1".into()),
            }],
            None,
            None,
        ),
        // Attached video + a live YouTube link → the embed replaces the video.
        post(
            2,
            "Watch this clip.",
            &["greeting"],
            vec![Media::Video {
                url: "https://example.com/v.mp4".into(),
            }],
            Some("dQw4w9WgXcQ"),
            None,
        ),
        // Attached video + an Instagram link → the video is kept (IG embed is
        // opt-in and off by default).
        post(
            3,
            "A reel.",
            &[],
            vec![Media::Video {
                url: "https://example.com/r.mp4".into(),
            }],
            None,
            Some("https://www.instagram.com/reel/ABC123/"),
        ),
        // A downloaded MTProto video (opt-in MTPROTO_VIDEOS): no link, so it plays.
        post(
            4,
            "MTProto video.",
            &[],
            vec![Media::LocalVideo {
                path: PathBuf::from("/nonexistent/4.mp4"),
            }],
            None,
            None,
        ),
    ];
    // Post 2199 has code-shaped text inside a quote, not a <pre>. Its leading
    // spaces must survive conversion and the real minify_html-enabled build.
    let quoted_html = scraper::Html::parse_fragment(
        "<div><blockquote>    func main() {<br/>        var (<br/>            wg <b>sync.WaitGroup</b><br/>        )<br/><br/>    }</blockquote></div>",
    );
    let quote = crate::html2md::convert(
        quoted_html
            .select(&scraper::Selector::parse("div").unwrap())
            .next()
            .unwrap(),
    );
    posts[0].body_md.push_str("\n\n");
    posts[0].body_md.push_str(&quote.md);
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
    let related = render::compute_related(&posts, 5);
    for (post, related_posts) in posts.iter_mut().zip(related) {
        post.related = related_posts;
    }

    // Drive the real pipeline (mirrors main::run from grouping onward).
    let rewriter = render::LinkRewriter::new(&s.channel, &posts);
    let ui = crate::i18n::ui(&s.language);

    let mut tc: HashMap<String, usize> = HashMap::new();
    let mut by_day: BTreeMap<String, (usize, BTreeSet<String>)> = BTreeMap::new();
    for p in &posts {
        let e = by_day
            .entry(p.date.format("%Y-%m-%d").to_string())
            .or_default();
        e.0 += 1;
        for t in &p.tags {
            *tc.entry(t.clone()).or_default() += 1;
            e.1.insert(t.clone());
        }
    }
    let tag_counts: Vec<(String, usize)> = tc.into_iter().collect();
    let days: Vec<DayMeta> = by_day
        .into_iter()
        .map(|(day, (count, tags))| DayMeta {
            day,
            count,
            tags: tags.into_iter().collect(),
        })
        .collect();

    let page_tag_titles = site::pagination_titles(
        posts
            .iter()
            .map(|p| (p.date.date_naive(), p.tags.as_slice())),
        s.posts_per_page,
        &s.language,
    );
    site::scaffold(&s, None, &tag_counts, &[], &days, &page_tag_titles).expect("scaffold");
    let mut rendered: Vec<RenderedPost> = posts
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
                    // Exercise both titled and date/ID-only taxonomy rows.
                    derive_titles: p.primary_id == 1,
                    strip_title: false,
                    keep_media: s.keep_media,
                    spotify: false,
                    instagram: false,
                    pinterest: false,
                    pinterest_keep_image: false,
                    video_releases: None,
                    carousel: false,
                },
            )
        })
        .collect();
    // Exercise the persisted snapshot boundary as well as single-post HTML;
    // the backup stays outside Zola's published static directory.
    let mut discussions = crate::discussions::Archive::empty(&s.channel);
    discussions.posts.insert(1, vec![mock_discussion()]);
    discussions
        .save(&s.site)
        .expect("save discussion snapshots");
    let discussions =
        crate::discussions::Archive::load(&s.site, &s.channel).expect("load discussion snapshots");
    crate::discussion_render::attach(
        &mut rendered[0],
        &s.channel,
        discussions.posts.get(&1).unwrap(),
        &ui,
    );
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
    let read = |rel: &str| {
        fs::read_to_string(public.join(rel)).unwrap_or_else(|e| panic!("read {rel}: {e}"))
    };
    assert_tag_view_links(&public, "");
    assert_tag_post_tags(&public, "");
    assert_post_navigation(&public, "");
    assert_discussions(&public);

    // Core pages exist.
    assert!(public.join("index.html").exists(), "home page missing");
    assert!(
        public.join("about/index.html").exists(),
        "about page missing"
    );
    assert!(public.join("rss.xml").exists(), "rss feed missing");
    assert!(
        public.join("tags/greeting/index.html").exists(),
        "tag page missing"
    );

    // Full-post metadata gets a viewport-edge rule by default, while a static
    // page without a post date (About) must not be mistaken for a post header.
    let home = read("index.html");
    let post_page = read("posts/1/index.html");
    let related_post_page = read("posts/2/index.html");
    let tag_full_page = read("tags/greeting/full/index.html");
    let day_full_page = read("day/2023-11-15/index.html");
    let about_page = read("about/index.html");
    let css = read("style.css");
    let document = scraper::Html::parse_document(&post_page);
    // Indentation and formatting survive in a regular quote after minification.
    let quote = document
        .select(&scraper::Selector::parse(".content blockquote").unwrap())
        .next()
        .expect("blockquote missing");
    let quote_text = quote.text().collect::<String>().replace('\u{a0}', " ");
    assert!(
        quote_text.starts_with("    func main() {"),
        "{quote_text:?}"
    );
    assert!(quote_text.contains("        var ("), "{quote_text:?}");
    assert!(
        quote_text.contains("            wg sync.WaitGroup"),
        "{quote_text:?}"
    );
    assert_eq!(
        quote
            .select(&scraper::Selector::parse("br").unwrap())
            .count(),
        5
    );
    assert!(quote
        .select(&scraper::Selector::parse("pre").unwrap())
        .next()
        .is_none());
    assert_eq!(
        quote
            .select(&scraper::Selector::parse("strong").unwrap())
            .next()
            .unwrap()
            .text()
            .collect::<String>(),
        "sync.WaitGroup"
    );
    // The icon-only Telegram link retains its destination and localized name
    // for hover and assistive technology; the SVG itself is decorative.
    let telegram = document
        .select(&scraper::Selector::parse(".tg-link a").unwrap())
        .next()
        .expect("Telegram link missing");
    assert_eq!(
        telegram.value().attr("href"),
        Some("https://t.me/testchan/1")
    );
    assert_eq!(telegram.value().attr("title"), Some("View on Telegram"));
    assert_eq!(
        telegram.value().attr("aria-label"),
        Some("View on Telegram")
    );
    assert!(telegram.text().collect::<String>().trim().is_empty());
    let icon = telegram
        .select(&scraper::Selector::parse("svg").unwrap())
        .next()
        .expect("Telegram SVG missing");
    assert_eq!(icon.value().attr("aria-hidden"), Some("true"));
    assert_eq!(icon.value().attr("focusable"), Some("false"));
    // The Telegram mark is a borderless plane, with no circular backing. It
    // inherits the link color so the same SVG stays visible in either theme.
    assert!(icon
        .select(&scraper::Selector::parse("circle, ellipse, rect").unwrap())
        .next()
        .is_none());
    assert_eq!(icon.value().attr("fill"), Some("currentColor"));
    assert!(!about_page.contains("tg-link"));
    assert!(home.contains(r#"class="meta post-meta""#), "{home}");
    assert!(
        post_page.contains(r#"class="meta post-meta""#),
        "{post_page}"
    );
    assert!(
        tag_full_page.contains(r#"class="meta post-meta""#),
        "{tag_full_page}"
    );
    assert!(
        day_full_page.contains(r#"class="meta post-meta""#),
        "{day_full_page}"
    );
    assert!(!about_page.contains("post-meta"), "{about_page}");
    assert!(
        css.contains(".post-meta::before")
            && css.contains("left: calc(50% - 50vw)")
            && css.contains("right: 100%")
            && css.contains("overflow-x: clip")
            && !css.contains("width: 100vw"),
        "post-header viewport rule missing: {css}"
    );

    // Related-link tooltips expose the target post's canonical publication day.
    let related_date =
        regex::Regex::new(r#"<a href="?/posts/4/"? title="?2023-11-18"?>MTProto video\.</a>"#)
            .unwrap();
    assert!(
        related_date.is_match(&related_post_page),
        "related publication-date tooltip missing: {related_post_page}"
    );

    // Each tag link carries a post-count hover tooltip (localized "N posts").
    let tags_page = read("tags/index.html");
    assert!(
        tags_page.contains("title=\"1 posts\""),
        "tag count tooltip missing: {tags_page}"
    );

    // Homepage pagination previews the destination page's date range and tags,
    // ranked by post count. Character references keep the native tooltip
    // multiline, with a blank line separating dates from tags.
    assert!(
        home.contains(
            "title=\"15 November 2023 - 16 November 2023\n\n#greeting — 2\n#photography — 1\n#video — 1\n#александровщина — 1\""
        ),
        "Older page tooltip dates or tags are missing or unsorted: {home}"
    );
    let older_page = read("page/2/index.html");
    assert!(
        older_page.contains("title=\"17 November 2023 - 18 November 2023\n\n#video — 2\""),
        "Newer page tooltip dates or tags are missing: {older_page}"
    );

    // Calendar year labels show the total number of posts in that year in both
    // the jump navigation and the year heading.
    let calendar = read("calendar/index.html");
    let year_count = regex::Regex::new(r#"class="?cal-ycount"?>4</span>"#).unwrap();
    assert_eq!(
        year_count.find_iter(&calendar).count(),
        2,
        "calendar year post count missing: {calendar}"
    );

    // Every absolute internal link (href/src="/…") in the built site must resolve
    // to a real file — guards dangling nav links (e.g. a calendar 404).
    let mut htmls = Vec::new();
    collect_html(&public, &mut htmls);
    let link_re = regex::Regex::new(r#"(?:href|src)="(/[^"]*)""#).unwrap();
    let mut checked = 0usize;
    for f in &htmls {
        let html = fs::read_to_string(f).unwrap();
        for cap in link_re.captures_iter(&html) {
            let path = cap[1].split(['?', '#']).next().unwrap_or("");
            if path.is_empty() || path == "/" || path.starts_with("//") {
                continue;
            }
            let rel = path.trim_start_matches('/');
            let ok = public.join(rel).exists() || public.join(rel).join("index.html").exists();
            assert!(ok, "dead internal link {path} (in {})", f.display());
            checked += 1;
        }
    }
    assert!(
        checked > 5,
        "internal-link checker found too few links ({checked})"
    );

    // Custom 404 exists and links the themed stylesheet (which carries the
    // prefers-color-scheme dark/light rules), so it follows the OS theme.
    assert!(public.join("404.html").exists(), "404 page missing");
    let notfound = read("404.html");
    assert!(
        notfound.contains("style.css"),
        "404 doesn't link the theme stylesheet"
    );
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
    assert!(
        !p2.contains("<video"),
        "attached video not dropped for the embed:\n{p2}"
    );

    // Post 3: Instagram embedding is off by default (opt-in), so the attached
    // video is kept and no Instagram embed is emitted.
    let p3 = read("posts/3/index.html");
    assert!(
        p3.contains("<video"),
        "attached video should be kept when IG is off:\n{p3}"
    );
    assert!(
        !p3.contains("instagram-media"),
        "no IG embed expected (off by default):\n{p3}"
    );
    assert!(
        !p3.contains("embed.js"),
        "no IG embed.js expected (off by default):\n{p3}"
    );

    // Post 4: a downloaded MTProto video plays as a <video>, and is #video-tagged.
    let p4 = read("posts/4/index.html");
    assert!(p4.contains("<video"), "LocalVideo <video> missing:\n{p4}");
    assert!(
        public.join("tags/video/index.html").exists(),
        "video tag page missing"
    );

    // The offline pass keeps the local keyboard helper and rewrites links, so
    // the copy opens from file:// without losing post navigation.
    crate::offline::relativize(&public).expect("offline relativize");
    let p3o = read("posts/3/index.html");
    assert!(
        p3o.contains("<video"),
        "kept video should remain after offline:\n{p3o}"
    );
    let offline_document = scraper::Html::parse_document(&p3o);
    let offline_scripts: Vec<_> = offline_document
        .select(&scraper::Selector::parse("script").unwrap())
        .collect();
    assert_eq!(offline_scripts.len(), 1, "{p3o}");
    assert_eq!(
        offline_scripts[0].value().attr("src"),
        Some("../../post-navigation.js")
    );
    // Boundary navigation must follow the rewritten local HTML files rather
    // than the deployed site's directory URLs when opened over file://.
    for (path, attribute, target) in [
        ("index.html", "data-post-next", "page/2/index.html"),
        ("page/2/index.html", "data-post-prev", "../../index.html"),
        (
            "day/2023-11-16/index.html",
            "data-post-next",
            "../../day/2023-11-15/index.html",
        ),
        (
            "day/2023-11-16/index.html",
            "data-post-prev",
            "../../day/2023-11-17/index.html",
        ),
    ] {
        let document = scraper::Html::parse_document(&read(path));
        let selector = scraper::Selector::parse(&format!("a[{attribute}]")).unwrap();
        let link = document
            .select(&selector)
            .next()
            .unwrap_or_else(|| panic!("offline {attribute} link missing: {path}"));
        assert_eq!(link.value().attr("href"), Some(target), "{path}");
        assert!(
            public.join(path).parent().unwrap().join(target).exists(),
            "offline navigation target missing: {path} → {target}"
        );
    }

    // Reuse the same fixture to check a GitHub Pages-style deployment prefix.
    let out = Command::new("zola")
        .arg("--root")
        .arg(&s.site)
        .args(["build", "--base-url", "https://example.test/channel/"])
        .output()
        .expect("build with a project base URL");
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_tag_view_links(&public, "https://example.test/channel");
    assert_tag_post_tags(&public, "https://example.test/channel");
    assert_post_navigation(&public, "https://example.test/channel");
    assert_discussions(&public);

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

    let posts = vec![post(
        1,
        "a searchable body of text",
        &[],
        vec![],
        None,
        None,
    )];
    let rewriter = render::LinkRewriter::new(&s.channel, &posts);
    let ui = crate::i18n::ui(&s.language);
    site::scaffold(&s, None, &[], &[], &[], &[]).expect("scaffold");
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
                    pinterest_keep_image: false,
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
    assert!(
        out.status.success(),
        "zola build failed:\n{}",
        String::from_utf8_lossy(&out.stderr)
    );

    let public = s.site.join("public");
    // Zola built the Elasticlunr index (http://elasticlunr.com/), and we bundled
    // the library + wiring.
    assert!(
        public.join("search_index.en.js").exists(),
        "search index not generated"
    );
    assert!(
        public.join("elasticlunr.min.js").exists(),
        "elasticlunr library not bundled"
    );
    assert!(
        public.join("search.js").exists(),
        "search wiring not bundled"
    );
    let home = fs::read_to_string(public.join("index.html")).expect("home html");
    assert!(
        home.contains("search-results"),
        "search UI missing:\n{home}"
    );

    // The offline pass keeps the local search scripts (they run from local files),
    // so client-side search works over file:// too.
    crate::offline::relativize(&public).expect("offline relativize");
    let home_off = fs::read_to_string(public.join("index.html")).expect("home html");
    assert!(
        home_off.contains("elasticlunr.min.js"),
        "search library dropped offline:\n{home_off}"
    );
    assert!(
        home_off.contains("search.js"),
        "search wiring dropped offline"
    );

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
    let posts = vec![post(
        1,
        "The body of the biggest post.",
        &[],
        vec![],
        None,
        None,
    )];
    let slug = render::slug_for(&posts[0]);

    let rewriter = render::LinkRewriter::new(&s.channel, &posts);
    let ui = crate::i18n::ui(&s.language);
    site::scaffold(&s, None, &[], &[], &[], &[]).expect("scaffold");
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
                    pinterest_keep_image: false,
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
    let biggest = vec![(
        s.site.join(format!("content/posts/{slug}/photo.jpg")),
        123_456u64,
    )];
    let mut previews = HashMap::new();
    previews.insert(
        slug.clone(),
        "Tooltip body of the biggest post.".to_string(),
    );
    site::set_about_size(
        &s.site,
        &breakdown,
        Some(1_000_000_000),
        std::time::Duration::from_secs(3),
        &crate::i18n::about(&s.language),
        &site::LargestFiles {
            files: &biggest,
            previews: &previews,
        },
        true,
        "2026-07-04 12:00 UTC",
        Some("https://github.com/x/y/actions/runs/42"),
        7_654_321,
        "https://github.com/x/y",
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
        Some("https://pagespeed.web.dev/analysis?url=x"),
    );
    site::set_about_me(&s.site, None, None, false, false);

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

    let about = fs::read_to_string(s.site.join("public/about/index.html")).expect("about html");
    // Placeholders were substituted.
    assert!(
        !about.contains("__TOTAL_SIZE__"),
        "size placeholder left unfilled:\n{about}"
    );
    assert!(
        !about.contains("__LARGEST_FILES__"),
        "largest-files placeholder left unfilled"
    );
    assert!(
        !about.contains("__PAGESPEED__"),
        "pagespeed placeholder left unfilled"
    );
    // The Lighthouse scores render on the About page.
    assert!(
        about.contains("Performance") && about.contains("98"),
        "lighthouse scores missing:\n{about}"
    );
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
