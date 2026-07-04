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

/// How a post's YouTube link renders after the liveness check.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum YtStatus {
    /// oEmbed 200 (or a transient failure) — embed the iframe.
    Embeddable,
    /// Not embeddable, but the video still plays on youtube.com (its thumbnail
    /// exists) — embedding is merely disabled, so link out instead of dropping it.
    EmbedDisabled,
    /// Gone: not embeddable and no thumbnail — drop the embed.
    Removed,
}

/// Classify each post's YouTube link. A removed/embedding-disabled video sets
/// `Post::youtube_dead` (no dead embed); an embedding-disabled-but-playable one
/// also sets `Post::youtube_watchable` so the renderer can link out to it.
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

    let results: Vec<(usize, YtStatus)> = stream::iter(targets.into_iter().map(|(i, id)| {
        let client = client.clone();
        async move { (i, youtube_status(&client, &id).await) }
    }))
    .buffer_unordered(concurrency.max(1))
    .collect()
    .await;

    let (mut removed, mut disabled) = (0usize, 0usize);
    for (i, status) in results {
        match status {
            YtStatus::Embeddable => {}
            YtStatus::EmbedDisabled => {
                posts[i].youtube_dead = true;
                posts[i].youtube_watchable = true;
                disabled += 1;
            }
            YtStatus::Removed => {
                posts[i].youtube_dead = true;
                removed += 1;
            }
        }
    }
    if removed > 0 || disabled > 0 {
        tracing::info!(
            "YouTube: {removed} removed, {disabled} embedding-disabled (still playable) \
             — keeping local media / linking out"
        );
    }
}

/// Classify a YouTube video. oEmbed `200` (or a transient `429`/`5xx`/network
/// failure) is treated as embeddable so rate-limiting never drops a live embed.
/// A definitive 4xx means "not embeddable"; a thumbnail-existence check then
/// tells "embedding disabled but still plays" (real thumbnail) apart from "gone"
/// (a nonexistent id 404s its thumbnail).
async fn youtube_status(client: &reqwest::Client, id: &str) -> YtStatus {
    let url = format!("https://www.youtube.com/oembed?url=https://youtu.be/{id}&format=json");
    let status = match client.get(&url).send().await {
        Ok(resp) => resp.status().as_u16(),
        Err(_) => return YtStatus::Embeddable, // network error → assume alive
    };
    if !youtube_status_removed(status) {
        return YtStatus::Embeddable; // 200, or transient (429 / 5xx / …)
    }
    if youtube_thumbnail_exists(client, id).await {
        YtStatus::EmbedDisabled
    } else {
        YtStatus::Removed
    }
}

/// Whether YouTube still serves a thumbnail for the id — a real video (even one
/// with embedding disabled) returns `200`; a removed/nonexistent id `404`s. A
/// transient error is treated as "exists" (prefer linking out over dropping).
async fn youtube_thumbnail_exists(client: &reqwest::Client, id: &str) -> bool {
    let url = format!("https://img.youtube.com/vi/{id}/hqdefault.jpg");
    match client.get(&url).send().await {
        Ok(resp) => resp.status().is_success(),
        Err(_) => true,
    }
}

/// oEmbed status → a *definitive* non-embeddable response? A 4xx (gone / private
/// / embedding-off; a nonexistent id is 400, not 404) counts; 200 / 429 / 5xx /
/// network do not (transient → assume embeddable).
fn youtube_status_removed(status: u16) -> bool {
    matches!(status, 400 | 401 | 403 | 404)
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
        async move { (i, is_apple_removed(&client, &id).await) }
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

/// True only when iTunes Lookup reports the podcast id no longer exists
/// (`resultCount` 0). Non-200 responses and errors are treated as alive.
async fn is_apple_removed(client: &reqwest::Client, id: &str) -> bool {
    let url = format!("https://itunes.apple.com/lookup?id={id}");
    let Ok(resp) = client.get(&url).send().await else {
        return false;
    };
    if resp.status() != reqwest::StatusCode::OK {
        return false;
    }
    resp.text().await.map(|b| apple_body_removed(&b)).unwrap_or(false)
}

/// iTunes Lookup body → removed? `resultCount` 0 means the podcast id is gone.
fn apple_body_removed(body: &str) -> bool {
    let compact: String = body.chars().filter(|c| !c.is_whitespace()).collect();
    compact.contains("\"resultCount\":0,") || compact.contains("\"resultCount\":0}")
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
        async move { (i, is_yandex_removed(&client, &id).await) }
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
async fn is_yandex_removed(client: &reqwest::Client, id: &str) -> bool {
    let url = format!("https://api.music.yandex.net/tracks/{id}");
    let Ok(resp) = client.get(&url).send().await else {
        return false;
    };
    let Ok(body) = resp.text().await else {
        return false;
    };
    yandex_body_removed(&body)
}

/// Yandex tracks-API body → removed/unavailable? A present `result` without
/// `"available":true` is a removed or region-locked track.
fn yandex_body_removed(body: &str) -> bool {
    let compact: String = body.chars().filter(|c| !c.is_whitespace()).collect();
    compact.contains("\"result\":") && !compact.contains("\"available\":true")
}

/// Instagram serves og meta only to a link-preview crawler UA; a live post has a
/// non-empty `og:title`, a removed one doesn't.
const INSTAGRAM_UA: &str =
    "facebookexternalhit/1.1 (+http://www.facebook.com/externalhit_uatext.php)";
/// Instagram aggressively rate-limits datacenter IPs, so checks run one at a time
/// with this gap between them (not concurrent).
const INSTAGRAM_DELAY_SECS: u64 = 5;

/// Mark posts whose Instagram post isn't confirmed live, so an attached video is
/// kept instead of a dead embed. Only posts that actually have a video are
/// checked (that's the only case the embed replaces). Sequential with a delay,
/// and every failure is logged so throttling is visible later.
pub async fn check_instagram(client: &reqwest::Client, posts: &mut [Post], base_url: &str) {
    let targets: Vec<(usize, String)> = posts
        .iter()
        .enumerate()
        .filter(|(_, p)| has_video(&p.media))
        .filter_map(|(i, p)| p.instagram.clone().map(|u| (i, u)))
        .collect();
    if targets.is_empty() {
        return;
    }
    tracing::info!(
        "checking {} Instagram link(s) for liveness ({}s apart)",
        targets.len(),
        INSTAGRAM_DELAY_SECS
    );
    let (mut live, mut kept) = (0usize, 0usize);
    for (n, (i, url)) in targets.iter().enumerate() {
        if n > 0 {
            tokio::time::sleep(std::time::Duration::from_secs(INSTAGRAM_DELAY_SECS)).await;
        }
        let post_url = post_link(base_url, posts[*i].primary_id);
        if is_instagram_removed(client, url, &post_url).await {
            posts[*i].instagram_dead = true;
            kept += 1;
        } else {
            live += 1;
        }
    }
    tracing::info!("Instagram liveness: {live} live (embedded), {kept} kept (gone/unverifiable)");
}

/// True unless the post is confirmed live (HTTP 200 with a non-empty `og:title`,
/// which Instagram serves to this crawler UA even from datacenter IPs). A removed
/// post, a throttle/login redirect and a network error all fail that test and
/// keep the video — we never drop a video on an unverified post. Every
/// non-confirmed case is logged with status and body size: a login/challenge
/// redirect is a small body, a real "post unavailable" page is the full ~600 KB
/// shell, so throttling stays distinguishable in the logs if it shows up.
async fn is_instagram_removed(client: &reqwest::Client, url: &str, post_url: &str) -> bool {
    let resp = match client
        .get(url)
        .header(reqwest::header::USER_AGENT, INSTAGRAM_UA)
        .send()
        .await
    {
        Ok(r) => r,
        Err(e) => {
            tracing::warn!("Instagram liveness request failed ({url}) for {post_url}: {e:#}");
            return true;
        }
    };
    let status = resp.status();
    let body = resp.text().await.unwrap_or_default();
    if status.is_success() && has_nonempty_og(&body, "og:title") {
        return false; // confirmed live → replace the video with the embed
    }
    if status.is_success() {
        // A live IG post always exposes og:title; its absence means the post is
        // gone — a normal, expected outcome (not a warning). Keep the local video.
        tracing::info!(
            "Instagram liveness: 200 without og:title (post gone) — keeping the video for {post_url} ({url}, {} bytes)",
            body.len()
        );
    } else {
        tracing::warn!("Instagram liveness: HTTP {status} for {url} (post {post_url}) — keeping the video");
    }
    true
}

/// True if `property="<prop>" content="` is present with a non-empty value.
fn has_nonempty_og(body: &str, prop: &str) -> bool {
    let needle = format!("property=\"{prop}\" content=\"");
    body.find(&needle)
        .is_some_and(|i| !body[i + needle.len()..].starts_with('"'))
}

/// The deployed blog URL of a post, for logs (base_url is "/" for local/offline
/// builds, the real host in CI).
fn post_link(base_url: &str, id: u64) -> String {
    format!("{}/posts/{id}/", base_url.trim_end_matches('/'))
}

fn has_video(media: &[crate::model::Media]) -> bool {
    use crate::model::Media;
    media
        .iter()
        .any(|m| matches!(m, Media::Video { .. } | Media::VideoPoster { .. }))
}

/// Mark posts whose Spotify track/album is removed (oEmbed 404), so the link is
/// shown instead of a dead player. Run only when the Spotify embed is enabled.
pub async fn check_spotify(client: &reqwest::Client, posts: &mut [Post], concurrency: usize) {
    let targets: Vec<(usize, String)> = posts
        .iter()
        .enumerate()
        .filter_map(|(i, p)| p.spotify.clone().map(|u| (i, u)))
        .collect();
    if targets.is_empty() {
        return;
    }
    tracing::info!("checking {} Spotify link(s) for liveness", targets.len());
    let results: Vec<(usize, bool)> = stream::iter(targets.into_iter().map(|(i, url)| {
        let client = client.clone();
        async move { (i, is_spotify_removed(&client, &url).await) }
    }))
    .buffer_unordered(concurrency.max(1))
    .collect()
    .await;
    let mut dead = 0usize;
    for (i, removed) in results {
        if removed {
            posts[i].spotify_dead = true;
            dead += 1;
        }
    }
    if dead > 0 {
        tracing::info!("{dead} Spotify item(s) removed — showing the link, not a dead embed");
    }
}

/// Spotify oEmbed says the track/album is gone (`404`). The embed URL is turned
/// back into the public URL the oEmbed endpoint expects.
async fn is_spotify_removed(client: &reqwest::Client, embed_url: &str) -> bool {
    let public = embed_url.replacen("/embed/", "/", 1);
    match client
        .get("https://open.spotify.com/oembed")
        .query(&[("url", public.as_str())])
        .send()
        .await
    {
        Ok(resp) => oembed_removed(resp.status().as_u16()),
        Err(_) => false,
    }
}

/// Mark posts whose Pinterest pin is removed (oEmbed 400/404), so the link is
/// shown instead of a broken embed. Run only when the Pinterest embed is enabled.
pub async fn check_pinterest(client: &reqwest::Client, posts: &mut [Post], concurrency: usize) {
    let targets: Vec<(usize, String)> = posts
        .iter()
        .enumerate()
        .filter_map(|(i, p)| p.pinterest.clone().map(|u| (i, u)))
        .collect();
    if targets.is_empty() {
        return;
    }
    tracing::info!("checking {} Pinterest link(s) for liveness", targets.len());
    let results: Vec<(usize, bool)> = stream::iter(targets.into_iter().map(|(i, url)| {
        let client = client.clone();
        async move { (i, is_pinterest_removed(&client, &url).await) }
    }))
    .buffer_unordered(concurrency.max(1))
    .collect()
    .await;
    let mut dead = 0usize;
    for (i, removed) in results {
        if removed {
            posts[i].pinterest_dead = true;
            dead += 1;
        }
    }
    if dead > 0 {
        tracing::info!("{dead} Pinterest pin(s) removed — showing the link, not a broken embed");
    }
}

/// Pinterest oEmbed says the pin is gone (`400`/`404`).
async fn is_pinterest_removed(client: &reqwest::Client, url: &str) -> bool {
    match client
        .get("https://www.pinterest.com/oembed.json")
        .query(&[("url", url)])
        .send()
        .await
    {
        Ok(resp) => oembed_removed(resp.status().as_u16()),
        Err(_) => false,
    }
}

/// A definitive oEmbed client error (`400`/`404`) means the item is gone; other
/// statuses and network errors are treated as alive.
fn oembed_removed(status: u16) -> bool {
    matches!(status, 400 | 404)
}


#[cfg(test)]
mod tests {
    use super::{
        apple_body_removed, apple_podcast_id, has_nonempty_og, oembed_removed, yandex_body_removed,
        yandex_track_id, youtube_status_removed,
    };

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

    #[test]
    fn og_title_presence() {
        // A live Instagram post has a non-empty og:title; a removed one doesn't.
        assert!(has_nonempty_og(
            r#"<meta property="og:title" content="Venjent on Instagram">"#,
            "og:title"
        ));
        assert!(!has_nonempty_og(
            r#"<meta property="og:title" content="">"#,
            "og:title"
        ));
        assert!(!has_nonempty_og(
            r#"<meta property="og:description" content="x">"#,
            "og:title"
        ));
    }

    #[test]
    fn youtube_status_classification() {
        for s in [400, 401, 403, 404] {
            assert!(youtube_status_removed(s), "{s} should be removed");
        }
        for s in [200, 429, 500, 301] {
            assert!(!youtube_status_removed(s), "{s} should not be removed");
        }
    }

    #[test]
    fn apple_lookup_body() {
        assert!(apple_body_removed(r#"{"resultCount":0, "results":[]}"#));
        assert!(apple_body_removed(r#"{"results":[],"resultCount":0}"#));
        assert!(!apple_body_removed(r#"{"resultCount":1,"results":[{"x":1}]}"#));
        assert!(!apple_body_removed("garbage"));
    }

    #[test]
    fn yandex_track_body() {
        assert!(yandex_body_removed(r#"{"result":{"available":false}}"#));
        assert!(yandex_body_removed(r#"{"result":{}}"#)); // present but not available
        assert!(!yandex_body_removed(r#"{"result":{"available":true,"id":"1"}}"#));
        assert!(!yandex_body_removed(r#"{"error":"not-found"}"#)); // no result → alive
    }

    #[test]
    fn oembed_status_classification() {
        // Spotify/Pinterest oEmbed: a definitive client error means the item is gone.
        assert!(oembed_removed(400)); // Pinterest's "gone"
        assert!(oembed_removed(404)); // Spotify's "gone"
        assert!(!oembed_removed(200));
        assert!(!oembed_removed(429)); // rate-limited → assume alive
        assert!(!oembed_removed(500));
    }
}
