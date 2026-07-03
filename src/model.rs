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
    /// A document/attachment whose file the public page doesn't expose; we keep
    /// only its name so the reader knows it existed.
    DocumentRef { filename: String },
    /// Sticker rendered as an image (webp).
    Sticker { url: String, key: Option<String> },
    /// A voice note / audio file fetched via the optional MTProto backend and
    /// already on local disk at `path` (the web preview never exposes these).
    /// Copied into the bundle by the normal media step. Only constructed with
    /// the `mtproto` feature.
    #[cfg_attr(not(feature = "mtproto"), allow(dead_code))]
    LocalAudio {
        path: std::path::PathBuf,
        /// Original filename (`Document::name()`) for the saved file; `None`
        /// (tagged music with no filename) → positional name.
        name: Option<String>,
        /// Label shown above the player — full (untruncated) audio title, with
        /// performer if present.
        title: Option<String>,
    },
    /// An original-quality photo fetched via MTProto, replacing a web [`Media::Photo`].
    /// `key` keeps the web photo's content-addressed bundle filename.
    #[cfg_attr(not(feature = "mtproto"), allow(dead_code))]
    LocalPhoto {
        path: std::path::PathBuf,
        key: Option<String>,
    },
    /// A full video fetched via MTProto, replacing a web [`Media::VideoPoster`] —
    /// the large/long videos `t.me/s/` exposes only as a poster + duration, with
    /// no downloadable file. Copied into the bundle by the normal media step.
    /// Only constructed with the `mtproto` feature (opt-in `MTPROTO_VIDEOS=1`).
    #[cfg_attr(not(feature = "mtproto"), allow(dead_code))]
    LocalVideo { path: std::path::PathBuf },
    /// Any attachment (pdf, zip, rar, …) fetched via MTProto, replacing a web
    /// [`Media::DocumentRef`] the public page couldn't download. Copied into the
    /// bundle and shown as a download link. Only constructed with the `mtproto`
    /// feature (on by default; disable with `MTPROTO_FILES=false`).
    #[cfg_attr(not(feature = "mtproto"), allow(dead_code))]
    LocalDocument {
        path: std::path::PathBuf,
        /// Original attachment filename (`Document::name()`).
        name: String,
    },
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
    /// Apple Podcasts embed URL, if any link points at podcasts.apple.com.
    pub apple_podcast: Option<String>,
    /// Yandex Music iframe embed URL, if any link points at a music.yandex track.
    pub yandex_music: Option<String>,
    /// Canonical Instagram post URL, if any link points at an instagram post/reel.
    pub instagram: Option<String>,
    /// Spotify embed URL, if any link points at open.spotify.com (opt-in embed).
    pub spotify: Option<String>,
    /// Canonical Pinterest pin URL, if any link points at a pinterest pin.
    pub pinterest: Option<String>,
    /// YouTube link not embeddable (oEmbed non-200) — keep the local media / link
    /// out instead of a dead embed. Default false (assume embeddable).
    pub youtube_dead: bool,
    /// A non-embeddable YouTube video that still plays on youtube.com (its
    /// thumbnail exists) — embedding is merely disabled, so we can link out to it
    /// rather than drop it. Only meaningful with `youtube_dead`. Default false.
    pub youtube_watchable: bool,
    /// Apple Podcasts podcast confirmed removed (iTunes lookup `resultCount` 0) —
    /// keep the local audio instead of a dead embed. Default false.
    pub apple_dead: bool,
    /// Yandex Music track removed/unavailable (API `available` not true) — keep
    /// the local audio instead of a dead embed. Default false.
    pub yandex_dead: bool,
    /// Instagram post not confirmed live (no og:title via the crawler UA) — keep
    /// the local video instead of a dead embed. Default false.
    pub instagram_dead: bool,
    /// Spotify track/album removed (oEmbed 404) — show the link, not a dead embed.
    pub spotify_dead: bool,
    /// Pinterest pin removed (oEmbed 400/404) — show the link, not a dead embed.
    pub pinterest_dead: bool,
    /// Genius song id (resolved by fetching a linked genius.com page), for the
    /// lyrics widget when the post carries no lyrics of its own.
    pub genius_song_id: Option<String>,
}
