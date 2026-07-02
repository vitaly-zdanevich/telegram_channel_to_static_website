//! Group raw messages into blog posts.
//!
//! Telegram albums already arrive as a single message bubble (one `data-post`
//! with several media), so those are handled for free by the parser. The
//! remaining case is a *burst*: messages posted together (forwarding several at
//! once, or an album plus a same-instant caption). We fold a message into the
//! previous post when it's a continuation — consecutive within `window_secs`, or
//! posted at the very same instant (albums share a timestamp but not adjacent
//! IDs) — unless both carry their own hashtags, since distinct tag sets mean
//! distinct posts.

use crate::media;
use crate::model::{Media, Post, RawMessage};

pub fn group(mut msgs: Vec<RawMessage>, window_secs: i64) -> Vec<Post> {
    msgs.sort_by_key(|m| m.id);
    let mut posts: Vec<Post> = Vec::new();

    for m in msgs {
        // A message that is only a sticker is merged into the previous post.
        let sticker_only = m.body_md.trim().is_empty()
            && !m.media.is_empty()
            && m.media.iter().all(|x| matches!(x, Media::Sticker { .. }));
        // Distinct tag sets mean distinct posts, so two messages that each carry
        // their own hashtags are never merged. Otherwise a continuation — a
        // consecutive message within the burst window, or one posted at the very
        // same instant (an album and its caption share a timestamp but not
        // adjacent IDs) — is folded into the previous post.
        // A "the rest didn't fit in the podcast above" follow-up carrying a
        // YouTube link folds into the previous post when that post has attached
        // audio, so the video embed replaces the (skipped) audio there.
        let marker_followup = CONTINUATION_MARKERS.iter().any(|mk| m.body_md.contains(mk))
            && m.links.iter().any(|l| media::youtube_id(l).is_some());
        let merge = posts.last().is_some_and(|last| {
            let last_id = *last.ids.last().unwrap();
            let secs = (m.date - last.date).num_seconds();
            let consecutive = m.id == last_id + 1;
            let together = (consecutive && secs.abs() <= window_secs) || secs == 0;
            let both_tagged = !last.tags.is_empty() && !m.tags.is_empty();
            // A "#podcast" announcement immediately followed by the episode's
            // attached audio folds into one post.
            let podcast_audio = last.tags.iter().any(|t| t == "podcast") && has_audio(&m.media);
            (together && last.author == m.author && !both_tagged)
                || sticker_only
                || (marker_followup && has_audio(&last.media))
                || podcast_audio
        });

        if merge {
            let last = posts.last_mut().unwrap();
            last.ids.push(m.id);
            if !m.body_md.trim().is_empty() {
                if !last.body_md.trim().is_empty() {
                    last.body_md.push_str("\n\n");
                }
                last.body_md.push_str(&m.body_md);
            }
            for t in m.tags {
                if !last.tags.contains(&t) {
                    last.tags.push(t);
                }
            }
            last.media.extend(m.media);
            last.views = last.views.max(m.views);
            last.edited |= m.edited;
            last.links.extend(m.links);
            if last.youtube.is_none() {
                last.youtube = media::youtube_from(&last.links);
            }
            if last.apple_podcast.is_none() {
                last.apple_podcast = media::apple_podcast_from(&last.links);
            }
            if last.yandex_music.is_none() {
                last.yandex_music = media::yandex_music_from(&last.links);
            }
            if last.instagram.is_none() {
                last.instagram = media::instagram_from(&last.links);
            }
        } else {
            posts.push(to_post(m));
        }
    }

    posts
}

fn to_post(m: RawMessage) -> Post {
    let youtube = media::youtube_from(&m.links);
    let apple_podcast = media::apple_podcast_from(&m.links);
    let yandex_music = media::yandex_music_from(&m.links);
    let instagram = media::instagram_from(&m.links);
    Post {
        primary_id: m.id,
        ids: vec![m.id],
        channel: m.channel,
        date: m.date,
        author: m.author,
        forwarded_from: m.forwarded_from,
        body_md: m.body_md,
        tags: m.tags,
        media: m.media,
        views: m.views,
        edited: m.edited,
        links: m.links,
        youtube,
        apple_podcast,
        yandex_music,
        instagram,
        youtube_dead: false,
        apple_dead: false,
        yandex_dead: false,
        instagram_dead: false,
        genius_song_id: None,
    }
}

/// Follow-up messages whose text folds into the previous (audio/podcast) post.
/// Hardcoded on purpose — this "the rest didn't fit above" + link pattern recurs
/// across podcast channels, so the link's embed can replace the audio there.
const CONTINUATION_MARKERS: &[&str] = &["Не влазит в сообщение с подкастом выше."];

fn has_audio(items: &[Media]) -> bool {
    items.iter().any(|m| match m {
        Media::Audio { .. } => true,
        Media::Document { filename, .. } | Media::DocumentRef { filename } => {
            media::is_probably_audio_doc(filename)
        }
        _ => false,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn msg(id: u64, tags: &[&str], body: &str) -> RawMessage {
        RawMessage {
            id,
            channel: "c".into(),
            // Same instant for all, so only the tag rule decides merging.
            date: chrono::FixedOffset::east_opt(0)
                .unwrap()
                .timestamp_opt(1_700_000_000, 0)
                .unwrap(),
            author: Some("c".into()),
            forwarded_from: None,
            body_md: body.into(),
            tags: tags.iter().map(|s| s.to_string()).collect(),
            links: vec![],
            media: vec![],
            views: None,
            edited: false,
        }
    }

    #[test]
    fn tagged_messages_stay_separate() {
        // Two same-instant consecutive posts, each with its own tags → 2 posts.
        let posts = group(
            vec![
                msg(1413, &["webdesign", "shopify"], "a"),
                msg(1414, &["anime", "japan"], "b"),
            ],
            1,
        );
        assert_eq!(posts.len(), 2, "tagged posts must not merge");
        assert_eq!(posts[1].tags, vec!["anime", "japan"]);
    }

    #[test]
    fn tagless_continuation_merges() {
        // A captionless follow-up (no tags of its own) folds into the post.
        let posts = group(vec![msg(10, &["trip"], "caption"), msg(11, &[], "")], 1);
        assert_eq!(posts.len(), 1, "tagless continuation should merge");
        assert_eq!(posts[0].ids, vec![10, 11]);
    }

    #[test]
    fn podcast_marker_followup_merges_despite_tags() {
        // The hardcoded "didn't fit above" + YouTube follow-up folds into the
        // previous audio post even though both carry their own tags.
        let mut audio = msg(500, &["podcast"], "Episode 5");
        audio.media = vec![Media::DocumentRef {
            filename: "ep5.mp3".into(),
        }];
        let mut follow = msg(
            501,
            &["video"],
            "Не влазит в сообщение с подкастом выше. https://youtu.be/abc123",
        );
        follow.links = vec!["https://youtu.be/abc123".into()];
        let posts = group(vec![audio, follow], 1);
        assert_eq!(posts.len(), 1, "marker follow-up should merge");
        assert_eq!(posts[0].ids, vec![500, 501]);
        assert_eq!(posts[0].youtube.as_deref(), Some("abc123"));
    }

    #[test]
    fn podcast_tag_then_audio_merges_despite_tags() {
        // A #podcast announcement folds together with the next post's audio,
        // even when that audio post carries its own tags.
        let ann = msg(300, &["podcast"], "New episode announcement");
        let mut audio = msg(301, &["health"], "Episode");
        audio.media = vec![Media::DocumentRef {
            filename: "Georgy Gorgiladze: from illness to a Guinness record".into(),
        }];
        let posts = group(vec![ann, audio], 1);
        assert_eq!(posts.len(), 1, "#podcast then audio should merge");
        assert_eq!(posts[0].ids, vec![300, 301]);
        assert!(posts[0].tags.contains(&"podcast".to_string()));
    }

    #[test]
    fn same_instant_album_and_caption_merge() {
        // An album (no tags) and a same-instant post with a non-adjacent ID (the
        // album consumed the IDs between) still unite; the tags carry over.
        let posts = group(
            vec![msg(1787, &[], "album"), msg(1797, &["crypto", "wallet"], "caption")],
            1,
        );
        assert_eq!(posts.len(), 1, "same-instant posts should unite");
        assert_eq!(posts[0].ids, vec![1787, 1797]);
        assert_eq!(posts[0].tags, vec!["crypto", "wallet"]);
    }
}
