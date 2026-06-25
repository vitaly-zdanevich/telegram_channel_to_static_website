//! Group raw messages into blog posts.
//!
//! Telegram albums already arrive as a single message bubble (one `data-post`
//! with several media), so those are handled for free by the parser. The
//! remaining case is a *burst*: several separate messages posted at the same
//! instant (typically forwarding many messages at once). We merge consecutive
//! messages from the same author whose timestamps fall within
//! `window_secs` of each other.

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
        let merge = posts.last().is_some_and(|last| {
            let last_id = *last.ids.last().unwrap();
            let consecutive = m.id == last_id + 1;
            let close = (m.date - last.date).num_seconds().abs() <= window_secs;
            (consecutive && close && last.author == m.author) || sticker_only
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
