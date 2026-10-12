//! Last complete snapshots of public channel-post discussions. This small JSON
//! file lives outside the published tree but inside the `blog` branch backup;
//! older successfully committed text remains in Git when a snapshot changes.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::Path;

/// Escape arbitrary Telegram text for both HTML text and quoted attributes.
pub fn escape_html(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('\'', "&#39;")
        .replace('"', "&quot;")
}

/// Only passive public links are allowed inside archived comment content.
pub fn safe_link(value: &str) -> bool {
    url::Url::parse(value).is_ok_and(|url| matches!(url.scheme(), "http" | "https" | "mailto"))
}

/// Text and reply metadata only; attachments stay linked to Telegram for now.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Comment {
    pub id: i32,
    pub author: String,
    /// RFC 3339 publication timestamp in UTC.
    pub date: String,
    /// Untrusted Telegram entity HTML, sanitized again at the rendering boundary.
    pub body_html: String,
    pub reply_to: Option<i32>,
    pub has_media: bool,
}

/// A resolved discussion root. Message IDs are unique only within `peer_id`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Discussion {
    pub peer_id: i64,
    pub root_id: i32,
    /// Original public channel message used for Telegram comment deep links.
    pub post_id: u64,
    pub comments: Vec<Comment>,
}

/// Channel-scoped snapshots keyed by the generated post's primary message ID.
/// No fetch timestamps; a cursor lets bounded syncs revisit every old thread.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Archive {
    pub version: u8,
    pub channel: String,
    pub posts: BTreeMap<u64, Vec<Discussion>>,
    /// Resume a bounded sync here, wrapping around after the oldest backlog.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next_post_id: Option<u64>,
}

impl Archive {
    /// Start a channel-scoped archive, including when discussions are disabled.
    pub fn empty(channel: &str) -> Self {
        Self {
            version: 1,
            channel: channel.to_owned(),
            posts: BTreeMap::new(),
            next_post_id: None,
        }
    }

    /// Missing snapshots are expected on the first build; other errors must not
    /// be mistaken for an empty successful fetch and overwrite the backup.
    pub fn load(site: &Path, channel: &str) -> Result<Self> {
        let path = site.join(".discussions.json");
        let raw = match std::fs::read(&path) {
            Ok(raw) => raw,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return Ok(Self::empty(channel))
            }
            Err(error) => return Err(error).with_context(|| format!("reading {}", path.display())),
        };
        let archive: Self =
            serde_json::from_slice(&raw).with_context(|| format!("parsing {}", path.display()))?;
        anyhow::ensure!(
            archive.version == 1,
            "unsupported discussion snapshot version {}",
            archive.version
        );
        anyhow::ensure!(
            archive.channel == channel,
            "discussion snapshot belongs to another channel"
        );
        Ok(archive)
    }

    /// Replace the snapshot atomically so a failed write leaves the old copy.
    #[cfg_attr(not(feature = "mtproto"), allow(dead_code))]
    pub fn save(&self, site: &Path) -> Result<()> {
        std::fs::create_dir_all(site)?;
        let path = site.join(".discussions.json");
        let temporary = site.join(".discussions.json.tmp");
        let mut raw = serde_json::to_vec_pretty(self)?;
        raw.push(b'\n');
        std::fs::write(&temporary, raw)
            .with_context(|| format!("writing {}", temporary.display()))?;
        std::fs::rename(&temporary, &path)
            .with_context(|| format!("replacing {}", path.display()))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Exercise persisted snapshots, including edits/deletions and load errors.
    #[test]
    fn snapshots_round_trip_and_reject_other_channels_or_invalid_data() {
        let site = std::env::temp_dir().join(format!("tg2zola-discussions-{}", std::process::id()));
        std::fs::create_dir_all(&site).unwrap();
        let mut archive = Archive::empty("example");
        archive.posts.insert(
            42,
            vec![Discussion {
                peer_id: 123,
                root_id: 7,
                post_id: 42,
                comments: vec![Comment {
                    id: 8,
                    author: "Alice".into(),
                    date: "2026-10-12T12:00:00Z".into(),
                    body_html: "First version".into(),
                    reply_to: None,
                    has_media: false,
                }],
            }],
        );
        archive.save(&site).unwrap();
        assert_eq!(Archive::load(&site, "example").unwrap(), archive);
        assert!(Archive::load(&site, "another_channel").is_err());

        archive.posts.get_mut(&42).unwrap()[0].comments[0].body_html = "Edited".into();
        archive.save(&site).unwrap();
        assert_eq!(Archive::load(&site, "example").unwrap(), archive);
        archive.posts.get_mut(&42).unwrap()[0].comments.clear();
        archive.save(&site).unwrap();
        assert_eq!(Archive::load(&site, "example").unwrap(), archive);

        std::fs::write(site.join(".discussions.json"), "{broken").unwrap();
        assert!(Archive::load(&site, "example").is_err());
        std::fs::remove_file(site.join(".discussions.json")).unwrap();
        assert_eq!(
            Archive::load(&site, "example").unwrap(),
            Archive::empty("example")
        );
        std::fs::remove_dir_all(site).unwrap();
    }
}
