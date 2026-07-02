//! Parse one `t.me/s/<channel>` HTML page into [`RawMessage`]s and find the
//! pagination cursor (`data-before`) for the next, older page.

use anyhow::Result;
use chrono::DateTime;
use once_cell::sync::Lazy;
use regex::Regex;
use scraper::{ElementRef, Html, Selector};

use crate::html2md;
use crate::model::{ChannelInfo, Forward, Media, RawMessage};

macro_rules! sel {
    ($name:ident, $q:literal) => {
        static $name: Lazy<Selector> = Lazy::new(|| Selector::parse($q).unwrap());
    };
}

sel!(S_WRAP, ".tgme_widget_message_wrap");
sel!(S_MSG, ".js-widget_message");
sel!(S_DATE_TIME, ".tgme_widget_message_date time");
sel!(S_OWNER, ".tgme_widget_message_owner_name");
sel!(S_FWD, ".tgme_widget_message_forwarded_from_name");
sel!(S_VIEWS, ".tgme_widget_message_views");
sel!(S_META, ".tgme_widget_message_meta");
sel!(S_TEXT, ".tgme_widget_message_text");
sel!(S_PHOTO, ".tgme_widget_message_photo_wrap");
sel!(S_VIDEO_TAG, "video");
sel!(S_VIDEO_PLAYER, ".tgme_widget_message_video_player");
sel!(S_VIDEO_THUMB, ".tgme_widget_message_video_thumb");
sel!(S_VIDEO_DUR, ".message_video_duration");
sel!(S_AUDIO_TAG, "audio");
sel!(S_VOICE, ".tgme_widget_message_voice");
sel!(S_DOC, ".tgme_widget_message_document_wrap");
sel!(S_DOC_TITLE, ".tgme_widget_message_document_title");
sel!(S_STICKER, ".tgme_widget_message_sticker");
sel!(S_MORE, "a.tme_messages_more");
sel!(S_CH_TITLE, ".tgme_channel_info_header_title");
sel!(S_CH_DESC, ".tgme_channel_info_description");
sel!(S_CH_PHOTO, ".tgme_page_photo_image img");
sel!(S_CH_COUNTER, ".tgme_channel_info_counter");
sel!(S_COUNTER_VAL, ".counter_value");
sel!(S_COUNTER_TYPE, ".counter_type");

static BG_URL: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"background-image\s*:\s*url\(['\x22]?(?P<u>[^'\x22)]+)['\x22]?\)").unwrap());

/// Parse a full page; returns the messages and the `before` cursor for the
/// next (older) page, if any.
pub fn parse_page(html: &str, channel: &str) -> Result<(Vec<RawMessage>, Option<u64>)> {
    let doc = Html::parse_document(html);
    let mut out = Vec::new();
    for wrap in doc.select(&S_WRAP) {
        if let Some(m) = parse_message(wrap, channel) {
            out.push(m);
        }
    }
    let next_before = doc
        .select(&S_MORE)
        .filter_map(|a| a.value().attr("data-before"))
        .filter_map(|s| s.parse::<u64>().ok())
        .min();
    Ok((out, next_before))
}

/// Parse the channel header (title, description, avatar, counters) — present
/// only on the first page.
pub fn parse_channel_info(html: &str) -> Option<ChannelInfo> {
    let doc = Html::parse_document(html);
    let title = doc
        .select(&S_CH_TITLE)
        .next()
        .map(|e| collapse_ws(&e.text().collect::<String>()))
        .filter(|s| !s.is_empty());
    let description_md = doc
        .select(&S_CH_DESC)
        .next()
        .map(|e| html2md::convert(e).md)
        .filter(|s| !s.is_empty());
    let avatar_url = doc
        .select(&S_CH_PHOTO)
        .filter_map(|img| img.value().attr("src"))
        .find(|s| s.starts_with("http"))
        .map(|s| s.to_string());
    let counters = doc
        .select(&S_CH_COUNTER)
        .filter_map(|c| {
            let v = collapse_ws(&c.select(&S_COUNTER_VAL).next()?.text().collect::<String>());
            let t = collapse_ws(&c.select(&S_COUNTER_TYPE).next()?.text().collect::<String>());
            (!v.is_empty()).then_some((v, t))
        })
        .collect::<Vec<_>>();

    if title.is_none() && description_md.is_none() && avatar_url.is_none() && counters.is_empty() {
        None
    } else {
        Some(ChannelInfo {
            title,
            description_md,
            avatar_url,
            counters,
        })
    }
}

fn parse_message(wrap: ElementRef, channel: &str) -> Option<RawMessage> {
    let msg = wrap.select(&S_MSG).next()?;
    let data_post = msg.value().attr("data-post")?;
    let id = data_post.rsplit('/').next()?.parse::<u64>().ok()?;

    let date = wrap
        .select(&S_DATE_TIME)
        .filter_map(|t| t.value().attr("datetime"))
        .find_map(|d| DateTime::parse_from_rfc3339(d).ok())?;

    let author = wrap
        .select(&S_OWNER)
        .next()
        .map(|e| collapse_ws(&e.text().collect::<String>()))
        .filter(|s| !s.is_empty());

    let forwarded_from = wrap.select(&S_FWD).next().map(|e| Forward {
        name: collapse_ws(&e.text().collect::<String>()),
        url: e.value().attr("href").map(|h| h.to_string()),
    });

    let views = wrap
        .select(&S_VIEWS)
        .next()
        .map(|e| e.text().collect::<String>())
        .and_then(|s| parse_views(&s));

    let edited = wrap
        .select(&S_META)
        .next()
        .map(|e| e.text().collect::<String>().contains("edited"))
        .unwrap_or(false);

    let (body_md, tags, links) = match wrap.select(&S_TEXT).next() {
        Some(txt) => {
            let c = html2md::convert(txt);
            (c.md, c.tags, c.links)
        }
        None => (String::new(), Vec::new(), Vec::new()),
    };

    let media = parse_media(wrap);

    Some(RawMessage {
        id,
        channel: channel.to_string(),
        date,
        author,
        forwarded_from,
        body_md,
        tags,
        links,
        media,
        views,
        edited,
    })
}

fn parse_media(wrap: ElementRef) -> Vec<Media> {
    let mut media = Vec::new();

    for p in wrap.select(&S_PHOTO) {
        if let Some(url) = bg_url(p.value().attr("style")) {
            let key = media_key_from_class(p.value().attr("class"));
            media.push(Media::Photo { url, key });
        }
    }

    // Directly downloadable videos (deduped — the same video can appear in two
    // <video> elements).
    let mut seen_video = std::collections::HashSet::new();
    for v in wrap.select(&S_VIDEO_TAG) {
        if let Some(src) = v.value().attr("src") {
            if src.starts_with("http") && seen_video.insert(src.to_string()) {
                media.push(Media::Video {
                    url: src.to_string(),
                });
            }
        }
    }
    // Video players with no `<video>` file -> keep poster + duration.
    for vp in wrap.select(&S_VIDEO_PLAYER) {
        if vp.select(&S_VIDEO_TAG).next().is_some() {
            continue;
        }
        let poster = vp
            .select(&S_VIDEO_THUMB)
            .next()
            .and_then(|t| bg_url(t.value().attr("style")))
            .or_else(|| bg_url(vp.value().attr("style")));
        let duration = vp
            .select(&S_VIDEO_DUR)
            .next()
            .map(|d| collapse_ws(&d.text().collect::<String>()))
            .filter(|s| !s.is_empty());
        media.push(Media::VideoPoster { poster, duration });
    }

    // Audio: voice notes / music files expose an <audio src=...>.
    for a in wrap.select(&S_AUDIO_TAG) {
        if let Some(src) = a.value().attr("src") {
            if src.starts_with("http") {
                media.push(Media::Audio {
                    url: src.to_string(),
                    title: None,
                });
            }
        }
    }
    // Voice players sometimes carry the URL on a data attribute.
    for v in wrap.select(&S_VOICE) {
        for attr in ["src", "data-src", "data-audio"] {
            if let Some(src) = v.value().attr(attr) {
                if src.starts_with("http") && is_audio(src) {
                    media.push(Media::Audio {
                        url: src.to_string(),
                        title: None,
                    });
                }
            }
        }
    }

    for d in wrap.select(&S_DOC) {
        let filename = d
            .select(&S_DOC_TITLE)
            .next()
            .map(|t| collapse_ws(&t.text().collect::<String>()))
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "file".to_string());
        match d.value().attr("href").filter(|h| is_downloadable(h)) {
            // Downloadable file.
            Some(href) => media.push(Media::Document {
                url: href.to_string(),
                filename,
            }),
            // No direct URL in the public page — keep the name only.
            None => media.push(Media::DocumentRef { filename }),
        }
    }

    for s in wrap.select(&S_STICKER) {
        if let Some(url) = bg_url(s.value().attr("style")).or_else(|| {
            s.value()
                .attr("data-webp")
                .filter(|u| u.starts_with("http"))
                .map(|u| u.to_string())
        }) {
            let key = media_key_from_class(s.value().attr("class"));
            media.push(Media::Sticker { url, key });
        }
    }

    media
}

/// Telegram embeds stable file ids as bare numeric class tokens on the media
/// wrapper, e.g. `tgme_widget_message_photo_wrap blured 5308054982420535730
/// 1235877858_460003762`. These persist across scrapes (unlike the tokenized
/// URL), so we use them as a content-addressed cache key.
fn media_key_from_class(class: Option<&str>) -> Option<String> {
    let class = class?;
    let ids: Vec<&str> = class
        .split_whitespace()
        .filter(|t| t.chars().any(|c| c.is_ascii_digit()))
        .filter(|t| t.chars().all(|c| c.is_ascii_digit() || c == '_'))
        .collect();
    if ids.is_empty() {
        None
    } else {
        Some(ids.join("_"))
    }
}

fn bg_url(style: Option<&str>) -> Option<String> {
    let style = style?;
    BG_URL
        .captures(style)
        .and_then(|c| c.name("u"))
        .map(|m| m.as_str().to_string())
        .filter(|u| u.starts_with("http"))
}

fn is_audio(url: &str) -> bool {
    let u = url.split('?').next().unwrap_or(url).to_ascii_lowercase();
    [".ogg", ".oga", ".mp3", ".m4a", ".wav", ".opus"]
        .iter()
        .any(|e| u.ends_with(e))
}

fn is_downloadable(href: &str) -> bool {
    href.starts_with("http") && !href.contains("//t.me/") && !href.contains("//telegram.")
}

/// Parse a views string like `"42"`, `"1.2K"`, `"3.4M"` into a number.
fn parse_views(s: &str) -> Option<u64> {
    let s = s.trim();
    if s.is_empty() {
        return None;
    }
    let (num, mult) = match s.chars().last() {
        Some('K') | Some('k') => (&s[..s.len() - 1], 1_000.0),
        Some('M') | Some('m') => (&s[..s.len() - 1], 1_000_000.0),
        _ => (s, 1.0),
    };
    num.replace([',', ' '], "")
        .parse::<f64>()
        .ok()
        .map(|n| (n * mult).round() as u64)
}

fn collapse_ws(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn views() {
        assert_eq!(parse_views("42"), Some(42));
        assert_eq!(parse_views("1.2K"), Some(1200));
        assert_eq!(parse_views("3M"), Some(3_000_000));
        assert_eq!(parse_views(""), None);
    }

    #[test]
    fn parse_page_message_and_cursor() {
        let html = r#"
            <div class="tgme_widget_message_wrap">
              <div class="tgme_widget_message js-widget_message" data-post="testchan/42">
                <div class="tgme_widget_message_text">Hello <b>world</b></div>
                <div class="tgme_widget_message_date"><time datetime="2025-01-15T10:30:00+00:00"></time></div>
                <span class="tgme_widget_message_views">1.2K</span>
              </div>
            </div>
            <a class="tme_messages_more" data-before="40"></a>
        "#;
        let (msgs, before) = parse_page(html, "testchan").unwrap();
        assert_eq!(before, Some(40)); // pagination cursor for the next older page
        assert_eq!(msgs.len(), 1);
        let m = &msgs[0];
        assert_eq!(m.id, 42);
        assert_eq!(m.channel, "testchan");
        assert_eq!(m.views, Some(1200));
        assert_eq!(m.date.format("%Y-%m-%d").to_string(), "2025-01-15");
        assert!(m.body_md.contains("world"), "body: {}", m.body_md);
    }

    #[test]
    fn parse_channel_header() {
        let html = r#"
            <div class="tgme_channel_info_header_title">My Channel</div>
            <div class="tgme_channel_info_description">A test channel</div>
            <div class="tgme_channel_info_counter"><span class="counter_value">1.5K</span> <span class="counter_type">subscribers</span></div>
        "#;
        let info = parse_channel_info(html).expect("channel info");
        assert_eq!(info.title.as_deref(), Some("My Channel"));
        assert_eq!(info.description_md.as_deref(), Some("A test channel"));
        assert_eq!(
            info.counters,
            vec![("1.5K".to_string(), "subscribers".to_string())]
        );
    }
}
