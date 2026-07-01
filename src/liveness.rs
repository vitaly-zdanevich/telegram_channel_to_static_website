//! Best-effort liveness checks for external video links.
//!
//! When a post's attached media is replaced by an external embed to save hosting
//! space, we first confirm the external copy still exists. Currently YouTube: a
//! *removed* video (oEmbed `404`) makes its local media worth keeping instead of
//! showing a dead embed. Only a definitive 404 counts as dead — a `200`, a
//! network error or a timeout is treated as alive, so we never keep media (or
//! suppress an embed) on a transient hiccup.

use futures::stream::{self, StreamExt};

use crate::model::Post;

/// Mark posts whose YouTube link is confirmed removed (`Post::youtube_dead`).
pub async fn check_youtube(client: &reqwest::Client, posts: &mut [Post], concurrency: usize) {
    let targets: Vec<(usize, String)> = posts
        .iter()
        .enumerate()
        .filter_map(|(i, p)| p.youtube.clone().map(|id| (i, id)))
        .collect();
    if targets.is_empty() {
        return;
    }
    tracing::info!("checking {} YouTube link(s) for liveness", targets.len());

    let results: Vec<(usize, bool)> = stream::iter(targets.into_iter().map(|(i, id)| {
        let client = client.clone();
        async move { (i, is_removed(&client, &id).await) }
    }))
    .buffer_unordered(concurrency.max(1))
    .collect()
    .await;

    let mut dead = 0usize;
    for (i, removed) in results {
        if removed {
            posts[i].youtube_dead = true;
            dead += 1;
        }
    }
    if dead > 0 {
        tracing::info!("{dead} YouTube video(s) removed — keeping their local media");
    }
}

/// True when YouTube's oEmbed endpoint says the video can't be embedded — gone,
/// private or embedding-disabled. oEmbed returns `200` for a live public video
/// and a client error otherwise (a *nonexistent* id is `400`, not `404`). Only a
/// definitive 4xx counts; `429`/`5xx`/network errors are transient → treated as
/// alive, so rate-limiting never makes us keep media for live videos.
async fn is_removed(client: &reqwest::Client, id: &str) -> bool {
    let url = format!("https://www.youtube.com/oembed?url=https://youtu.be/{id}&format=json");
    match client.get(&url).send().await {
        Ok(resp) => matches!(resp.status().as_u16(), 400 | 401 | 403 | 404),
        Err(_) => false,
    }
}

/// Mark posts whose Apple Podcasts *podcast* is confirmed removed (iTunes Lookup
/// `resultCount` 0), so the audio is kept instead of a dead embed. Only the
/// podcast id is checkable — Apple's public API doesn't expose per-episode
/// existence, so an episode pulled from a still-live podcast isn't detected.
pub async fn check_apple(client: &reqwest::Client, posts: &mut [Post], concurrency: usize) {
    let targets: Vec<(usize, String)> = posts
        .iter()
        .enumerate()
        .filter_map(|(i, p)| {
            p.apple_podcast
                .as_deref()
                .and_then(apple_podcast_id)
                .map(|id| (i, id))
        })
        .collect();
    if targets.is_empty() {
        return;
    }
    tracing::info!("checking {} Apple Podcasts link(s) for liveness", targets.len());

    let results: Vec<(usize, bool)> = stream::iter(targets.into_iter().map(|(i, id)| {
        let client = client.clone();
        async move { (i, apple_removed(&client, &id).await) }
    }))
    .buffer_unordered(concurrency.max(1))
    .collect()
    .await;

    let mut dead = 0usize;
    for (i, removed) in results {
        if removed {
            posts[i].apple_dead = true;
            dead += 1;
        }
    }
    if dead > 0 {
        tracing::info!("{dead} Apple Podcasts podcast(s) removed — keeping their local audio");
    }
}

/// The `idNNNN` podcast id from an Apple Podcasts URL.
fn apple_podcast_id(url: &str) -> Option<String> {
    let rest = &url[url.find("/id")? + 3..];
    let digits: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
    (!digits.is_empty()).then_some(digits)
}

#[cfg(test)]
mod tests {
    use super::{apple_podcast_id, yandex_track_id};

    #[test]
    fn extract_yandex_track_id() {
        assert_eq!(
            yandex_track_id("https://music.yandex.ru/iframe/#track/103670414/22206733").as_deref(),
            Some("103670414")
        );
        assert_eq!(yandex_track_id("https://music.yandex.ru/iframe/#album/1"), None);
    }

    #[test]
    fn extract_apple_podcast_id() {
        assert_eq!(
            apple_podcast_id("https://embed.podcasts.apple.com/us/podcast/x/id1234567?i=456")
                .as_deref(),
            Some("1234567")
        );
        assert_eq!(
            apple_podcast_id("https://podcasts.apple.com/podcast/id999").as_deref(),
            Some("999")
        );
        assert_eq!(apple_podcast_id("https://example.com/foo"), None);
    }
}

/// True only when iTunes Lookup reports the podcast id no longer exists
/// (`resultCount` 0). Non-200 responses and errors are treated as alive.
async fn apple_removed(client: &reqwest::Client, id: &str) -> bool {
    let url = format!("https://itunes.apple.com/lookup?id={id}");
    let Ok(resp) = client.get(&url).send().await else {
        return false;
    };
    if resp.status() != reqwest::StatusCode::OK {
        return false;
    }
    match resp.text().await {
        Ok(body) => {
            let compact: String = body.chars().filter(|c| !c.is_whitespace()).collect();
            compact.contains("\"resultCount\":0,") || compact.contains("\"resultCount\":0}")
        }
        Err(_) => false,
    }
}

/// Mark posts whose Yandex Music track is removed/unavailable, so the audio is
/// kept instead of a dead embed. Uses the unofficial
/// `api.music.yandex.net/tracks/<id>` API: a live+available track carries
/// `"available":true`; a removed one returns an error object.
pub async fn check_yandex(client: &reqwest::Client, posts: &mut [Post], concurrency: usize) {
    let targets: Vec<(usize, String)> = posts
        .iter()
        .enumerate()
        .filter_map(|(i, p)| {
            p.yandex_music
                .as_deref()
                .and_then(yandex_track_id)
                .map(|id| (i, id))
        })
        .collect();
    if targets.is_empty() {
        return;
    }
    tracing::info!("checking {} Yandex Music link(s) for liveness", targets.len());

    let results: Vec<(usize, bool)> = stream::iter(targets.into_iter().map(|(i, id)| {
        let client = client.clone();
        async move { (i, yandex_removed(&client, &id).await) }
    }))
    .buffer_unordered(concurrency.max(1))
    .collect()
    .await;

    let mut dead = 0usize;
    for (i, removed) in results {
        if removed {
            posts[i].yandex_dead = true;
            dead += 1;
        }
    }
    if dead > 0 {
        tracing::info!("{dead} Yandex Music track(s) gone/unavailable — keeping their local audio");
    }
}

/// The track id from a Yandex Music iframe embed URL (`…/iframe/#track/<t>/<a>`).
fn yandex_track_id(embed_url: &str) -> Option<String> {
    let rest = &embed_url[embed_url.find("#track/")? + 7..];
    let id: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
    (!id.is_empty()).then_some(id)
}

/// True only when the API responded but the track isn't available (`"available":
/// true` absent while a `"result"` is present) — a removed or region-locked
/// track. Network errors / unparseable responses are treated as alive.
async fn yandex_removed(client: &reqwest::Client, id: &str) -> bool {
    let url = format!("https://api.music.yandex.net/tracks/{id}");
    let Ok(resp) = client.get(&url).send().await else {
        return false;
    };
    let Ok(body) = resp.text().await else {
        return false;
    };
    let compact: String = body.chars().filter(|c| !c.is_whitespace()).collect();
    compact.contains("\"result\":") && !compact.contains("\"available\":true")
}
