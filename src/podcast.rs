//! Optional podcast feed (`--podcast` / `PODCAST`, off by default): the channel's
//! audio posts as an iTunes-compatible RSS `podcast.xml`, so the archive can be
//! subscribed to in podcast apps. Apple also wants a square cover ≥ 1400×1400px
//! (see the cover sourcing in `main`) and a real category; those aside, this is a
//! valid podcast feed.

use chrono::{DateTime, FixedOffset};

/// One audio episode.
pub struct Episode {
    pub title: String,
    pub description: String,
    /// Absolute enclosure URL of the audio file.
    pub url: String,
    /// Audio MIME type, e.g. `audio/mpeg`.
    pub mime: String,
    /// File size in bytes (0 if unknown).
    pub length: u64,
    pub date: DateTime<FixedOffset>,
    /// Stable per-episode id (the post permalink).
    pub guid: String,
}

/// Channel-level podcast metadata.
pub struct Channel {
    pub title: String,
    pub description: String,
    /// The site's home URL.
    pub link: String,
    /// Square cover image URL (empty → omitted).
    pub cover: String,
    pub author: String,
    /// RFC 5646 language tag, e.g. `en`.
    pub language: String,
    /// iTunes category (Apple requires one).
    pub category: String,
}

fn esc(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// RFC 2822 date, e.g. `Tue, 01 Mar 2025 12:00:00 +0000` (RSS `pubDate`).
fn rfc2822(d: &DateTime<FixedOffset>) -> String {
    d.format("%a, %d %b %Y %H:%M:%S %z").to_string()
}

/// Render the whole `podcast.xml`.
pub fn feed(ch: &Channel, episodes: &[Episode]) -> String {
    let mut items = String::new();
    for e in episodes {
        let length = if e.length > 0 { e.length.to_string() } else { "0".to_string() };
        items.push_str(&format!(
            "  <item>\n\
             \x20   <title>{title}</title>\n\
             \x20   <description>{desc}</description>\n\
             \x20   <pubDate>{date}</pubDate>\n\
             \x20   <guid isPermaLink=\"true\">{guid}</guid>\n\
             \x20   <enclosure url=\"{url}\" type=\"{mime}\" length=\"{length}\"/>\n\
             \x20 </item>\n",
            title = esc(&e.title),
            desc = esc(&e.description),
            date = rfc2822(&e.date),
            guid = esc(&e.guid),
            url = esc(&e.url),
            mime = esc(&e.mime),
        ));
    }
    let cover = if ch.cover.is_empty() {
        String::new()
    } else {
        format!("  <itunes:image href=\"{}\"/>\n", esc(&ch.cover))
    };
    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
         <rss version=\"2.0\" xmlns:itunes=\"http://www.itunes.com/dtds/podcast-1.0.dtd\">\n\
         <channel>\n\
         \x20 <title>{title}</title>\n\
         \x20 <link>{link}</link>\n\
         \x20 <language>{lang}</language>\n\
         \x20 <description>{desc}</description>\n\
         \x20 <itunes:author>{author}</itunes:author>\n\
         \x20 <itunes:explicit>false</itunes:explicit>\n\
         \x20 <itunes:category text=\"{cat}\"/>\n\
         {cover}{items}</channel>\n\
         </rss>\n",
        title = esc(&ch.title),
        link = esc(&ch.link),
        lang = esc(&ch.language),
        desc = esc(&ch.description),
        author = esc(&ch.author),
        cat = esc(&ch.category),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn dt() -> DateTime<FixedOffset> {
        FixedOffset::east_opt(0).unwrap().timestamp_opt(1_700_000_000, 0).unwrap()
    }

    #[test]
    fn feed_has_channel_and_episode_with_enclosure() {
        let ch = Channel {
            title: "My <Channel> & Co".into(),
            description: "desc".into(),
            link: "https://x.github.io/y/".into(),
            cover: "https://cdn/cover.jpg".into(),
            author: "Me".into(),
            language: "en".into(),
            category: "Personal Journals".into(),
        };
        let eps = vec![Episode {
            title: "Ep 1".into(),
            description: "notes".into(),
            url: "https://x.github.io/y/posts/5/01.mp3".into(),
            mime: "audio/mpeg".into(),
            length: 12345,
            date: dt(),
            guid: "https://x.github.io/y/posts/5/".into(),
        }];
        let xml = feed(&ch, &eps);
        assert!(xml.contains("<rss version=\"2.0\""));
        assert!(xml.contains("xmlns:itunes="));
        assert!(xml.contains("<itunes:image href=\"https://cdn/cover.jpg\"/>"));
        assert!(xml.contains("<itunes:category text=\"Personal Journals\"/>"));
        // XML-escaped channel title.
        assert!(xml.contains("My &lt;Channel&gt; &amp; Co"), "{xml}");
        // Episode with a proper enclosure.
        assert!(xml.contains(
            "<enclosure url=\"https://x.github.io/y/posts/5/01.mp3\" type=\"audio/mpeg\" length=\"12345\"/>"
        ), "{xml}");
        assert!(xml.contains("<pubDate>"));
    }

    #[test]
    fn no_cover_omits_itunes_image() {
        let ch = Channel {
            title: "t".into(),
            description: "d".into(),
            link: "l".into(),
            cover: String::new(),
            author: "a".into(),
            language: "en".into(),
            category: "Personal Journals".into(),
        };
        assert!(!feed(&ch, &[]).contains("itunes:image"));
    }
}
