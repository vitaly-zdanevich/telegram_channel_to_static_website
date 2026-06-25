//! Group raw messages into blog posts.
//!
//! Telegram albums already arrive as a single message bubble (one `data-post`
//! with several media), so those are handled for free by the parser. The
//! remaining case is a *burst*: several separate messages posted at the same
//! instant (typically forwarding many messages at once). We merge consecutive
//! messages from the same author whose timestamps fall within `window_secs` of
//! each other — but only continuations (no hashtags of their own); a message
//! that carries its own tags is kept as a separate post.

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
        // A message carrying its own hashtags is a post in its own right, so it
        // is never folded into the previous one even within the burst window —
        // distinct tags mean distinct posts. Loose continuations (extra album
        // photos, captionless media) have no tags of their own and still merge.
        let continuation = m.tags.is_empty();
        let merge = posts.last().is_some_and(|last| {
            let last_id = *last.ids.last().unwrap();
            let consecutive = m.id == last_id + 1;
            let close = (m.date - last.date).num_seconds().abs() <= window_secs;
            (consecutive && close && last.author == m.author && continuation) || sticker_only
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
        } else {
            posts.push(to_post(m));
        }
    }

    posts
}

fn to_post(m: RawMessage) -> Post {
    let youtube = media::youtube_from(&m.links);
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
    }
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
}
