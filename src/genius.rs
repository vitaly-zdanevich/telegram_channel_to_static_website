//! Resolve `genius.com` song links to the YouTube video they point to (embedded
//! via the normal no-JS iframe) and the song id (for the optional lyrics widget).
//!
//! Two backends: the **Genius API** when `GENIUS_TOKEN` is set — reliable, works
//! from CI (the web pages are Cloudflare-blocked on datacenter IPs) — otherwise
//! **page scraping** (best-effort; opt out entirely with `--no-genius`).

use futures::stream::{self, StreamExt};
use once_cell::sync::Lazy;
use regex::Regex;

use crate::model::Post;

static YT_ID: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"youtube\.com/(?:watch\?v=|embed/)([A-Za-z0-9_-]{11})|youtu\.be/([A-Za-z0-9_-]{11})")
        .unwrap()
});
static SONG_ID: Lazy<Regex> = Lazy::new(|| Regex::new(r"songs/(\d+)/embed").unwrap());

const GENIUS_API: &str = "https://api.genius.com";

fn genius_link(p: &Post) -> Option<String> {
    p.links.iter().find(|l| l.contains("genius.com")).cloned()
}

/// For each post that links to genius.com and has no YouTube video yet, resolve
/// the genius link and fill in `youtube` (its video) and `genius_song_id`.
pub async fn enrich(client: &reqwest::Client, posts: &mut [Post], concurrency: usize) {
    let targets: Vec<(usize, String)> = posts
        .iter()
        .enumerate()
        .filter(|(_, p)| p.youtube.is_none())
        .filter_map(|(i, p)| genius_link(p).map(|u| (i, u)))
        .collect();
    if targets.is_empty() {
        return;
    }
    // A Client Access Token switches on the API backend (env-only secret).
    let token = std::env::var("GENIUS_TOKEN")
        .ok()
        .map(|t| t.trim().to_string())
        .filter(|t| !t.is_empty());
    tracing::info!(
        "resolving {} genius.com link(s) {}",
        targets.len(),
        if token.is_some() {
            "via the Genius API"
        } else {
            "by scraping"
        }
    );

    let fetched: Vec<(usize, Option<String>, Option<String>)> = stream::iter(targets)
        .map(|(i, url)| {
            let client = client.clone();
            let token = token.clone();
            async move {
                let (yt, song) = match &token {
                    Some(t) => resolve_via_api(&client, t, GENIUS_API, &url).await,
                    None => scrape_page(&client, &url).await,
                };
                (i, yt, song)
            }
        })
        .buffer_unordered(concurrency.max(1))
        .collect()
        .await;

    for (i, yt, song) in fetched {
        if yt.is_some() {
            posts[i].youtube = yt;
        }
        posts[i].genius_song_id = song;
    }
}

// --- API backend (GENIUS_TOKEN) ---

#[derive(serde::Deserialize)]
struct SearchResp {
    response: SearchInner,
}
#[derive(serde::Deserialize)]
struct SearchInner {
    hits: Vec<Hit>,
}
#[derive(serde::Deserialize)]
struct Hit {
    result: HitResult,
}
#[derive(serde::Deserialize)]
struct HitResult {
    id: u64,
    url: String,
}
#[derive(serde::Deserialize)]
struct SongResp {
    response: SongInner,
}
#[derive(serde::Deserialize)]
struct SongInner {
    song: Song,
}
#[derive(serde::Deserialize)]
struct Song {
    media: Option<Vec<MediaItem>>,
}
#[derive(serde::Deserialize)]
struct MediaItem {
    provider: String,
    url: String,
}

/// Resolve a genius link via the API. Failures are logged and yield `(None,
/// None)` — enrichment is best-effort, never fatal.
async fn resolve_via_api(
    client: &reqwest::Client,
    token: &str,
    base: &str,
    link: &str,
) -> (Option<String>, Option<String>) {
    match resolve_via_api_inner(client, token, base, link).await {
        Ok(pair) => pair,
        Err(e) => {
            tracing::info!("genius API lookup skipped for {link}: {e}");
            (None, None)
        }
    }
}

/// Search by the URL's title, take the hit that is *this* song (matched by URL —
/// search alone can return a near-named song), then read its YouTube media.
async fn resolve_via_api_inner(
    client: &reqwest::Client,
    token: &str,
    base: &str,
    link: &str,
) -> anyhow::Result<(Option<String>, Option<String>)> {
    let search: SearchResp = client
        .get(format!("{base}/search"))
        .query(&[("q", slug_to_query(link))])
        .bearer_auth(token)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;
    let Some(id) = search
        .response
        .hits
        .iter()
        .find(|h| same_song(&h.result.url, link))
        .map(|h| h.result.id)
    else {
        return Ok((None, None)); // exact song not in the results — don't guess
    };
    let song: SongResp = client
        .get(format!("{base}/songs/{id}"))
        .bearer_auth(token)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;
    let yt = song
        .response
        .song
        .media
        .unwrap_or_default()
        .into_iter()
        .find(|m| m.provider == "youtube")
        .and_then(|m| youtube_id(&m.url));
    Ok((yt, Some(id.to_string())))
}

/// A search query from a genius URL: its slug, minus the `-lyrics` suffix.
fn slug_to_query(link: &str) -> String {
    let slug = link.trim_end_matches('/').rsplit('/').next().unwrap_or("");
    slug.strip_suffix("-lyrics").unwrap_or(slug).replace('-', " ")
}

/// Whether two genius URLs are the same song (ignoring a trailing `/` and scheme).
fn same_song(a: &str, b: &str) -> bool {
    fn norm(s: &str) -> String {
        s.trim_end_matches('/').replacen("http://", "https://", 1)
    }
    norm(a) == norm(b)
}

/// The YouTube video id in a URL (same regex the scraper uses).
fn youtube_id(url: &str) -> Option<String> {
    YT_ID
        .captures(url)
        .and_then(|c| c.get(1).or_else(|| c.get(2)))
        .map(|m| m.as_str().to_string())
}

// --- Scraping backend (fallback, no token) ---

/// Fetch the genius page and parse it. Cloudflare blocks datacenter/CI IPs, so
/// this is best-effort — a failure just leaves the post's plain link.
async fn scrape_page(client: &reqwest::Client, url: &str) -> (Option<String>, Option<String>) {
    match client.get(url).send().await.and_then(|r| r.error_for_status()) {
        Ok(r) => parse_genius(&r.text().await.unwrap_or_default()),
        Err(e) => {
            tracing::info!("genius enrichment skipped for {url}: {e}");
            (None, None)
        }
    }
}

/// Parse a fetched genius.com song page: the YouTube video id it embeds and the
/// genius song id.
fn parse_genius(html: &str) -> (Option<String>, Option<String>) {
    let yt = YT_ID
        .captures(html)
        .and_then(|c| c.get(1).or_else(|| c.get(2)))
        .map(|m| m.as_str().to_string());
    let song = SONG_ID.captures(html).map(|c| c[1].to_string());
    (yt, song)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_youtube_and_song_id() {
        let html = r#"<iframe src="https://www.youtube.com/embed/dQw4w9WgXcQ"></iframe>
            <div data-src="/songs/12345/embed"></div>"#;
        assert_eq!(
            parse_genius(html),
            (Some("dQw4w9WgXcQ".into()), Some("12345".into()))
        );
    }

    #[test]
    fn youtu_be_form_and_missing() {
        let (yt, song) = parse_genius("watch https://youtu.be/uWSLHcAyy90 — no song embed");
        assert_eq!(yt.as_deref(), Some("uWSLHcAyy90"));
        assert_eq!(song, None);
        assert_eq!(parse_genius("no links here"), (None, None));
    }

    #[test]
    fn slug_query_and_same_song() {
        assert_eq!(slug_to_query("https://genius.com/Tool-parabol-lyrics"), "Tool parabol");
        assert_eq!(
            slug_to_query("https://genius.com/Some-artist-a-song-lyrics/"),
            "Some artist a song"
        );
        assert!(same_song("https://genius.com/x-lyrics", "https://genius.com/x-lyrics/"));
        assert!(same_song("http://genius.com/x", "https://genius.com/x"));
        assert!(!same_song("https://genius.com/a", "https://genius.com/b"));
    }

    #[tokio::test]
    async fn api_matches_url_and_picks_youtube() {
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let server = MockServer::start().await;
        // Search returns a near-named decoy first, then the exact song.
        Mock::given(method("GET"))
            .and(path("/search"))
            .respond_with(ResponseTemplate::new(200).set_body_string(
                r#"{"response":{"hits":[
                    {"result":{"id":1,"url":"https://genius.com/Tool-parabola-lyrics"}},
                    {"result":{"id":134666,"url":"https://genius.com/Tool-parabol-lyrics"}}
                ]}}"#,
            ))
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/songs/134666"))
            .respond_with(ResponseTemplate::new(200).set_body_string(
                r#"{"response":{"song":{"media":[
                    {"provider":"spotify","url":"https://open.spotify.com/track/x"},
                    {"provider":"youtube","url":"https://www.youtube.com/watch?v=-_nQhGR0K8M"}
                ]}}}"#,
            ))
            .mount(&server)
            .await;

        let (yt, song) = resolve_via_api(
            &reqwest::Client::new(),
            "tok",
            &server.uri(),
            "https://genius.com/Tool-parabol-lyrics",
        )
        .await;
        assert_eq!(yt.as_deref(), Some("-_nQhGR0K8M")); // youtube, not the spotify entry
        assert_eq!(song.as_deref(), Some("134666")); // the URL-matched song, not the top hit
    }
}
