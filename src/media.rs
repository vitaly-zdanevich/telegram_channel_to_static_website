//! Media helpers: YouTube link detection and cache-aware downloading.

use anyhow::{Context, Result};
use futures::stream::{self, StreamExt};
use once_cell::sync::Lazy;
use regex::Regex;
use std::path::{Path, PathBuf};

/// A single media download: fetch `url` into `dest`. `force` re-downloads even
/// if `dest` already exists (used for edited posts whose media may have changed).
#[derive(Debug, Clone)]
pub struct Job {
    pub url: String,
    pub dest: PathBuf,
    pub force: bool,
    /// When set, copy this already-downloaded local file (MTProto) into `dest`
    /// instead of fetching `url` over HTTP.
    pub local: Option<PathBuf>,
}

static YT: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"(?:youtube\.com/(?:watch\?(?:[^ ]*&)?v=|embed/|shorts/|live/|v/)|youtu\.be/)([A-Za-z0-9_-]{6,})",
    )
    .unwrap()
});

/// Extract a YouTube video id from a single URL, if present.
pub fn youtube_id(url: &str) -> Option<String> {
    YT.captures(url).map(|c| c[1].to_string())
}

/// Return the **last** YouTube id across a set of links (links are in post
/// order, so the latest one wins when a post carries several).
pub fn youtube_from(links: &[String]) -> Option<String> {
    links.iter().rev().find_map(|l| youtube_id(l))
}

/// The Apple Podcasts **embed** URL for the last `podcasts.apple.com` link in a
/// post, if any: `…//podcasts.apple.com/…` → `…//embed.podcasts.apple.com/…`
/// (the path and `?i=<episode>` are kept). Links are in post order, latest wins.
pub fn apple_podcast_from(links: &[String]) -> Option<String> {
    links.iter().rev().find_map(|l| apple_podcast_embed(l))
}

fn apple_podcast_embed(url: &str) -> Option<String> {
    if url.contains("//embed.podcasts.apple.com/") {
        return Some(url.to_string());
    }
    url.contains("//podcasts.apple.com/")
        .then(|| url.replacen("//podcasts.apple.com/", "//embed.podcasts.apple.com/", 1))
}

static INSTAGRAM_POST: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"instagram\.com/(p|reel|tv)/([A-Za-z0-9_-]+)").unwrap());

/// The canonical Instagram post URL for the last instagram post/reel link in a
/// post, if any (used for the embed + liveness). Links are in post order.
pub fn instagram_from(links: &[String]) -> Option<String> {
    links.iter().rev().find_map(|l| {
        INSTAGRAM_POST
            .captures(l)
            .map(|c| format!("https://www.instagram.com/{}/{}/", &c[1], &c[2]))
    })
}

static YANDEX_TRACK: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"music\.yandex\.[a-z]+/album/(\d+)/track/(\d+)").unwrap());

/// The Yandex Music iframe embed URL for the last track link in a post, if any:
/// `music.yandex.*/album/<a>/track/<t>` → `music.yandex.ru/iframe/#track/<t>/<a>`.
pub fn yandex_music_from(links: &[String]) -> Option<String> {
    links.iter().rev().find_map(|l| {
        YANDEX_TRACK
            .captures(l)
            .map(|c| format!("https://music.yandex.ru/iframe/#track/{}/{}", &c[2], &c[1]))
    })
}

static SPOTIFY: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"open\.spotify\.com/(track|album|playlist|episode|show)/([A-Za-z0-9]+)").unwrap()
});

static WIKIDATA: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?i)wikidata\.org/(?:wiki|entity)/(Q\d+)").unwrap());

static BANDCAMP: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"https?://[a-z0-9][a-z0-9-]*\.bandcamp\.com/(?:album|track)/[a-z0-9-]+").unwrap()
});

/// The first Bandcamp album/track page URL linked from a post, if any — the
/// page is fetched later to resolve the embeddable player.
pub fn bandcamp_page(links: &[String]) -> Option<String> {
    links.iter().find_map(|l| BANDCAMP.find(l).map(|m| m.as_str().to_string()))
}

/// Every distinct Wikidata item id (`Q…`) linked from a post, in first-seen
/// order — one statements table is rendered per id.
pub fn wikidata_qids(links: &[String]) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for l in links {
        for c in WIKIDATA.captures_iter(l) {
            let qid = c[1].to_ascii_uppercase();
            if !out.contains(&qid) {
                out.push(qid);
            }
        }
    }
    out
}

/// The Spotify **embed** URL for the last spotify link in a post, if any:
/// `open.spotify.com/<type>/<id>` → `open.spotify.com/embed/<type>/<id>`.
pub fn spotify_from(links: &[String]) -> Option<String> {
    links.iter().rev().find_map(|l| {
        SPOTIFY
            .captures(l)
            .map(|c| format!("https://open.spotify.com/embed/{}/{}", &c[1], &c[2]))
    })
}

static PINTEREST: Lazy<Regex> = Lazy::new(|| Regex::new(r"pinterest\.[a-z.]+/pin/(\d+)").unwrap());

/// The canonical Pinterest pin URL for the last pinterest pin link in a post.
pub fn pinterest_from(links: &[String]) -> Option<String> {
    links.iter().rev().find_map(|l| {
        PINTEREST
            .captures(l)
            .map(|c| format!("https://www.pinterest.com/pin/{}/", &c[1]))
    })
}

/// True if a filename looks like audio (by extension).
pub fn is_audio_name(name: &str) -> bool {
    let n = name.to_ascii_lowercase();
    [".mp3", ".ogg", ".oga", ".opus", ".m4a", ".wav", ".flac", ".aac"]
        .iter()
        .any(|e| n.ends_with(e))
}

/// A "(not archived)" document to treat as the audio track (and drop once MTProto
/// has fetched it): an audio extension, **or** a title with no real file
/// extension — Telegram shows a podcast document's episode *title*, not a
/// filename (e.g. "Георгий … здоровьем"). A name ending in a real extension like
/// `.pdf` is a distinct file and is kept.
pub fn is_probably_audio_doc(name: &str) -> bool {
    is_audio_name(name) || !has_file_extension(name)
}

/// A "(not archived)" document that's an image — a pasted screenshot Telegram
/// stored as a *file* rather than a photo, so the web preview can't download it.
/// MTProto (MTPROTO_IMAGES) fetches these and shows them as photos. Matched by a
/// common image extension. Only used by the MTProto backend.
#[cfg(feature = "mtproto")]
pub fn is_probably_image_doc(name: &str) -> bool {
    let n = name.trim().to_ascii_lowercase();
    [".png", ".jpg", ".jpeg", ".gif", ".webp", ".avif", ".bmp"]
        .iter()
        .any(|e| n.ends_with(e))
}

fn has_file_extension(name: &str) -> bool {
    match name.rsplit_once('.') {
        Some((_, ext)) => {
            (1..=4).contains(&ext.chars().count()) && ext.chars().all(|c| c.is_ascii_alphanumeric())
        }
        None => false,
    }
}

/// Best-effort file extension from a URL (query string stripped).
pub fn ext_from_url(url: &str, default: &str) -> String {
    let path = url.split(['?', '#']).next().unwrap_or(url);
    let ext = path
        .rsplit('/')
        .next()
        .and_then(|f| f.rsplit_once('.'))
        .map(|(_, e)| e)
        .unwrap_or("")
        .to_ascii_lowercase();
    if ext.is_empty() || ext.len() > 5 || !ext.chars().all(|c| c.is_ascii_alphanumeric()) {
        default.to_string()
    } else {
        ext
    }
}

/// Download media into the post bundles. A file already present is skipped
/// (the committed bundle is the cache) unless `force` is set — Telegram CDN
/// URLs are tokenized and expire, so we persist the bytes, not the URL.
/// Failures are logged and tolerated — a backup run should never abort because
/// one file 404'd.
pub async fn download_all(client: &reqwest::Client, jobs: &[Job], concurrency: usize) -> Result<()> {
    let mut seen = std::collections::HashSet::new();
    let mut todo: Vec<Job> = Vec::new();
    for j in jobs {
        if !seen.insert(j.dest.clone()) {
            continue; // same destination requested twice this run
        }
        if j.dest.exists() && !j.force {
            continue; // already cached and not edited
        }
        todo.push(j.clone());
    }

    let total = todo.len();
    if total == 0 {
        tracing::info!("all media already present");
        return Ok(());
    }
    tracing::info!("downloading {} media files", total);

    let results = stream::iter(todo.into_iter().map(|j| {
        let client = client.clone();
        async move {
            match &j.local {
                Some(src) => copy_one(src, &j.dest)
                    .await
                    .with_context(|| format!("copying {}", src.display())),
                None => download_one(&client, &j.url, &j.dest)
                    .await
                    .with_context(|| format!("downloading {}", j.url)),
            }
        }
    }))
    .buffer_unordered(concurrency.max(1))
    .collect::<Vec<_>>()
    .await;

    let errs = results.iter().filter(|r| r.is_err()).count();
    for r in results {
        if let Err(e) = r {
            tracing::warn!("media download failed: {:#}", e);
        }
    }
    if errs > 0 {
        tracing::warn!("{}/{} media downloads failed (continuing)", errs, total);
    }
    Ok(())
}

/// Copy a local file (an MTProto-fetched original) into a bundle, atomically.
async fn copy_one(src: &Path, dest: &Path) -> Result<()> {
    if let Some(parent) = dest.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    let tmp = dest.with_extension("part");
    tokio::fs::copy(src, &tmp).await?;
    tokio::fs::rename(&tmp, dest).await?;
    Ok(())
}

async fn download_one(client: &reqwest::Client, url: &str, dest: &Path) -> Result<()> {
    if let Some(parent) = dest.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    let resp = client.get(url).send().await?.error_for_status()?;
    let bytes = resp.bytes().await?;
    // Write to a temp file then rename, so an interrupted run never leaves a
    // truncated file that a later run would treat as "already cached".
    let tmp = dest.with_extension("part");
    tokio::fs::write(&tmp, &bytes).await?;
    tokio::fs::rename(&tmp, dest).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn youtube_variants() {
        assert_eq!(
            youtube_id("https://www.youtube.com/watch?v=uWSLHcAyy90"),
            Some("uWSLHcAyy90".to_string())
        );
        assert_eq!(
            youtube_id("https://youtu.be/6lPaEat4GgQ"),
            Some("6lPaEat4GgQ".to_string())
        );
        assert_eq!(
            youtube_id("https://www.youtube.com/shorts/abc123XYZ"),
            Some("abc123XYZ".to_string())
        );
        assert_eq!(youtube_id("https://example.com/watch?v=nope"), None);
    }

    #[test]
    fn probably_audio_doc_covers_titles() {
        // A podcast document shows a title, not a filename → treated as audio.
        assert!(is_probably_audio_doc("Георгий Мевлудович Горгиладзе: занялся здоровьем"));
        assert!(is_probably_audio_doc("Episode 5 — the best one"));
        assert!(is_probably_audio_doc("track.mp3"));
        // A distinct real file (non-audio extension) is kept.
        assert!(!is_probably_audio_doc("report.pdf"));
        assert!(!is_probably_audio_doc("archive.zip"));
    }

    #[cfg(feature = "mtproto")]
    #[test]
    fn probably_image_doc_matches_image_extensions() {
        // Pasted screenshots Telegram stores as files.
        assert!(is_probably_image_doc("image_2026-07-01_07-36-10.png"));
        assert!(is_probably_image_doc("Photo.JPG"));
        assert!(is_probably_image_doc("meme.webp"));
        // Non-images are left alone.
        assert!(!is_probably_image_doc("report.pdf"));
        assert!(!is_probably_image_doc("track.mp3"));
        assert!(!is_probably_image_doc("Episode 5 — the best one"));
    }

    #[test]
    fn wikidata_qids_dedup_and_normalize() {
        let links = vec![
            "see https://www.wikidata.org/wiki/Q42 and".into(),
            "http://wikidata.org/entity/q42".into(), // dup, lowercased
            "https://www.wikidata.org/wiki/Q5".into(),
            "https://example.com/Q999".into(), // not wikidata
        ];
        assert_eq!(wikidata_qids(&links), vec!["Q42".to_string(), "Q5".to_string()]);
        assert!(wikidata_qids(&["https://example.com".into()]).is_empty());
    }

    #[test]
    fn instagram_post_url() {
        assert_eq!(
            instagram_from(&["x https://www.instagram.com/p/DKOrF01i-_4/?utm=1".into()]).as_deref(),
            Some("https://www.instagram.com/p/DKOrF01i-_4/")
        );
        assert_eq!(
            instagram_from(&["https://instagram.com/reel/ABC123/".into()]).as_deref(),
            Some("https://www.instagram.com/reel/ABC123/")
        );
        assert_eq!(instagram_from(&["https://example.com/x".into()]), None);
    }

    #[test]
    fn spotify_embed_url() {
        assert_eq!(
            spotify_from(&["listen https://open.spotify.com/track/1ZKipeRdw2frIZBd6Y0wNZ?si=x".into()])
                .as_deref(),
            Some("https://open.spotify.com/embed/track/1ZKipeRdw2frIZBd6Y0wNZ")
        );
        assert_eq!(
            spotify_from(&["https://open.spotify.com/album/ABC123".into()]).as_deref(),
            Some("https://open.spotify.com/embed/album/ABC123")
        );
        assert_eq!(spotify_from(&["https://example.com/x".into()]), None);
    }

    #[test]
    fn pinterest_pin_url() {
        assert_eq!(
            pinterest_from(&["see https://www.pinterest.com/pin/1234567890/".into()]).as_deref(),
            Some("https://www.pinterest.com/pin/1234567890/")
        );
        assert_eq!(
            pinterest_from(&["https://pinterest.co.uk/pin/42/sent".into()]).as_deref(),
            Some("https://www.pinterest.com/pin/42/")
        );
        assert_eq!(pinterest_from(&["https://example.com/x".into()]), None);
    }

    #[test]
    fn yandex_music_embed_url() {
        assert_eq!(
            yandex_music_from(&["https://music.yandex.ru/album/22206733/track/103670414".into()])
                .as_deref(),
            Some("https://music.yandex.ru/iframe/#track/103670414/22206733")
        );
        assert_eq!(yandex_music_from(&["https://music.yandex.ru/album/1".into()]), None);
        assert_eq!(yandex_music_from(&["https://example.com/x".into()]), None);
    }

    #[test]
    fn apple_podcast_embed_url() {
        assert_eq!(
            apple_podcast_from(&["https://podcasts.apple.com/us/podcast/x/id123?i=456".into()]),
            Some("https://embed.podcasts.apple.com/us/podcast/x/id123?i=456".into())
        );
        // Latest link wins; non-apple links ignored.
        assert_eq!(
            apple_podcast_from(&[
                "https://podcasts.apple.com/a/id1".into(),
                "https://example.com/x".into(),
                "https://podcasts.apple.com/b/id2".into(),
            ])
            .as_deref(),
            Some("https://embed.podcasts.apple.com/b/id2")
        );
        assert_eq!(apple_podcast_from(&["https://example.com/x".into()]), None);
    }

    #[test]
    fn extensions() {
        assert_eq!(ext_from_url("https://cdn/file/x.jpg", "bin"), "jpg");
        assert_eq!(ext_from_url("https://cdn/file/x.mp4?token=abc", "bin"), "mp4");
        assert_eq!(ext_from_url("https://cdn/file/noext", "jpg"), "jpg");
    }

    #[tokio::test]
    async fn download_writes_then_caches() {
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/a.jpg"))
            .respond_with(ResponseTemplate::new(200).set_body_string("IMAGEDATA"))
            .expect(1) // fetched exactly once — the second run must hit the cache
            .mount(&server)
            .await;

        let dir = std::env::temp_dir().join(format!("tg2-dl-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let job = Job {
            url: format!("{}/a.jpg", server.uri()),
            dest: dir.join("a.jpg"),
            force: false,
            local: None,
        };
        let client = reqwest::Client::new();
        download_all(&client, std::slice::from_ref(&job), 2).await.unwrap();
        assert_eq!(std::fs::read(&job.dest).unwrap(), b"IMAGEDATA");
        // dest now exists and force is off → the second call skips the download.
        download_all(&client, std::slice::from_ref(&job), 2).await.unwrap();
        // The mock's expect(1) is verified when `server` drops.
        let _ = std::fs::remove_dir_all(&dir);
    }
}
