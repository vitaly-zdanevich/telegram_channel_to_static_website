//! Resolve `genius.com` song links: fetch the page to pull out the YouTube
//! video it points to (embedded via the normal no-JS iframe) and the song id
//! (for the optional lyrics widget). Genius exposes neither in the Telegram
//! link itself, so a page fetch is required — hence this is best-effort and
//! opt-out (`--no-genius`).

use futures::stream::{self, StreamExt};
use once_cell::sync::Lazy;
use regex::Regex;

use crate::model::Post;

static YT_ID: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"youtube\.com/(?:watch\?v=|embed/)([A-Za-z0-9_-]{11})|youtu\.be/([A-Za-z0-9_-]{11})")
        .unwrap()
});
static SONG_ID: Lazy<Regex> = Lazy::new(|| Regex::new(r"songs/(\d+)/embed").unwrap());

fn genius_link(p: &Post) -> Option<String> {
    p.links
        .iter()
        .find(|l| l.contains("genius.com"))
        .cloned()
}

/// Parse a fetched genius.com song page: the YouTube video id it embeds
/// (watch / embed / youtu.be forms) and the genius song id (for the lyrics widget).
fn parse_genius(html: &str) -> (Option<String>, Option<String>) {
    let yt = YT_ID
        .captures(html)
        .and_then(|c| c.get(1).or_else(|| c.get(2)))
        .map(|m| m.as_str().to_string());
    let song = SONG_ID.captures(html).map(|c| c[1].to_string());
    (yt, song)
}

/// For each post that links to genius.com and has no YouTube video yet, fetch
/// the genius page and fill in `youtube` (its video) and `genius_song_id`.
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
    tracing::info!("resolving {} genius.com link(s)", targets.len());

    let fetched: Vec<(usize, Option<String>, Option<String>)> = stream::iter(targets)
        .map(|(i, url)| {
            let client = client.clone();
            async move {
                let html = match client.get(&url).send().await.and_then(|r| r.error_for_status()) {
                    Ok(r) => r.text().await.unwrap_or_default(),
                    Err(e) => {
                        // Best-effort enrichment; genius.com routinely 403s
                        // datacenter IPs (CI). Not critical — the post keeps its
                        // plain link — so this is INFO, not a warning.
                        tracing::info!("genius enrichment skipped for {url}: {e}");
                        return (i, None, None);
                    }
                };
                let (yt, song) = parse_genius(&html);
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

#[cfg(test)]
mod tests {
    use super::parse_genius;

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
}
