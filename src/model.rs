//! Core data types shared across the pipeline.

use chrono::{DateTime, FixedOffset};

/// Channel header info scraped once from the first page (for the About page and
/// the site header).
#[derive(Debug, Clone, Default)]
pub struct ChannelInfo {
    pub title: Option<String>,
    pub description_md: Option<String>,
    pub avatar_url: Option<String>,
    /// (value, type) pairs, e.g. ("1.34K", "photos").
    pub counters: Vec<(String, String)>,
}

/// A "forwarded from" attribution. We keep only the display name as plain
/// text — never a `t.me` link — so the generated site has no Telegram
/// dependency.
#[derive(Debug, Clone)]
pub struct Forward {
    pub name: String,
    /// Link to the original source (kept as a normal `<a>` in output).
    pub url: Option<String>,
}

/// A single piece of media attached to a message.
#[derive(Debug, Clone)]
pub enum Media {
    /// Photo with a directly downloadable CDN URL. `key` is the stable Telegram
    /// file id (from the markup); the URL itself rotates between scrapes, so the
    /// key is what we use for content-addressed caching.
    Photo { url: String, key: Option<String> },
    /// Video that the public page exposes as a real `<video src=...>` — downloadable.
    Video { url: String },
    /// Video with no downloadable file in the public page. We keep the poster
    /// thumbnail (itself downloadable) and the duration label.
    VideoPoster {
        poster: Option<String>,
        duration: Option<String>,
    },
    /// Voice note or music file (.ogg/.oga/.mp3).
    Audio { url: String, title: Option<String> },
    /// Arbitrary document/attachment with a downloadable URL.
    Document { url: String, filename: String },
    /// Sticker rendered as an image (webp).
    Sticker { url: String, key: Option<String> },
}

/// One raw Telegram message as scraped from `t.me/s/<channel>`.
#[derive(Debug, Clone)]
pub struct RawMessage {
    pub id: u64,
    pub channel: String,
    pub date: DateTime<FixedOffset>,
    pub author: Option<String>,
    pub forwarded_from: Option<Forward>,
    /// Message text already converted to Markdown.
    pub body_md: String,
    /// Hashtags lifted out of the text (without the leading `#`).
    pub tags: Vec<String>,
    /// External links found in the text (used for YouTube detection).
    pub links: Vec<String>,
    pub media: Vec<Media>,
    pub views: Option<u64>,
    /// Whether the message was edited. Drives media re-download so replaced
    /// images/videos are captured instead of served stale from cache.
    pub edited: bool,
}

/// One blog post = one or more messages that were posted together
/// (albums, or a burst of messages forwarded at the same instant).
#[derive(Debug, Clone)]
pub struct Post {
    pub primary_id: u64,
    pub ids: Vec<u64>,
    #[allow(dead_code)]
    pub channel: String,
    pub date: DateTime<FixedOffset>,
    pub author: Option<String>,
    pub forwarded_from: Option<Forward>,
    pub body_md: String,
    pub tags: Vec<String>,
    pub media: Vec<Media>,
    pub views: Option<u64>,
    /// True if any constituent message was edited (forces media re-download).
    pub edited: bool,
    /// Aggregated external links (not serialized; used for YouTube detection).
    pub links: Vec<String>,
    /// YouTube video id, if any link in the post points at YouTube.
    pub youtube: Option<String>,
    /// Genius song id (resolved by fetching a linked genius.com page), for the
    /// lyrics widget when the post carries no lyrics of its own.
    pub genius_song_id: Option<String>,
}
