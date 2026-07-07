//! Resolve Bandcamp album/track links to their embeddable player. Bandcamp has
//! no oEmbed, but every album/track page carries an `EmbeddedPlayer/…album=<id>`
//! (or `track=<id>`) reference, so we fetch the page once and build the iframe
//! player URL from that id. Default on; disable with `--no-bandcamp`.

use crate::model::Post;
use futures::StreamExt;
use once_cell::sync::Lazy;
use regex::Regex;

static EMBED_ID: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"EmbeddedPlayer/(?:v=\d+/)?(album|track)=(\d+)").unwrap());

/// Fetch each post's Bandcamp page (once per distinct URL) and set `post.bandcamp`
/// to the player URL. Best-effort — a failed fetch just leaves the plain link.
pub async fn enrich(client: &reqwest::Client, posts: &mut [Post], concurrency: usize) {
    let mut pages: Vec<String> = Vec::new();
    for p in posts.iter() {
        if let Some(u) = crate::media::bandcamp_page(&p.links) {
            if !pages.contains(&u) {
                pages.push(u);
            }
        }
    }
    if pages.is_empty() {
        return;
    }
    let resolved: std::collections::HashMap<String, String> =
        futures::stream::iter(pages.into_iter().map(|u| async {
            let embed = fetch_embed(client, &u).await;
            (u, embed)
        }))
        .buffer_unordered(concurrency.max(1))
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .filter_map(|(u, e)| e.map(|e| (u, e)))
        .collect();

    for p in posts.iter_mut() {
        if let Some(u) = crate::media::bandcamp_page(&p.links) {
            if let Some(embed) = resolved.get(&u) {
                p.bandcamp = Some(embed.clone());
            }
        }
    }
}

async fn fetch_embed(client: &reqwest::Client, url: &str) -> Option<String> {
    let html = client
        .get(url)
        .send()
        .await
        .and_then(|r| r.error_for_status())
        .ok()?
        .text()
        .await
        .ok()?;
    embed_url(&html)
}

/// Build the player URL from the `(album|track)=<id>` in a page's embed
/// reference. `transparent=true` lets it blend with either site theme.
fn embed_url(html: &str) -> Option<String> {
    let c = EMBED_ID.captures(html)?;
    let (kind, id) = (&c[1], &c[2]);
    Some(format!(
        "https://bandcamp.com/EmbeddedPlayer/{kind}={id}/size=large/tracklist=false/artwork=small/transparent=true/"
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_album_player_from_page() {
        let html = r#"<meta content="https://bandcamp.com/EmbeddedPlayer/v=2/album=3840020883/size=large/tracklist=false/artwork=small/">"#;
        assert_eq!(
            embed_url(html).as_deref(),
            Some("https://bandcamp.com/EmbeddedPlayer/album=3840020883/size=large/tracklist=false/artwork=small/transparent=true/")
        );
    }

    #[test]
    fn extracts_track_player() {
        let html = "x EmbeddedPlayer/track=123456 y";
        assert_eq!(
            embed_url(html).as_deref(),
            Some("https://bandcamp.com/EmbeddedPlayer/track=123456/size=large/tracklist=false/artwork=small/transparent=true/")
        );
    }

    #[test]
    fn none_without_embed() {
        assert_eq!(embed_url("<html>no player here</html>"), None);
    }
}
