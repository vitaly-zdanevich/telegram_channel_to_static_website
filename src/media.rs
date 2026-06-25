//! Media helpers: YouTube link detection and cache-aware downloading.

use anyhow::{Context, Result};
use futures::stream::{self, StreamExt};
use once_cell::sync::Lazy;
use regex::Regex;
use std::path::{Path, PathBuf};

/// A single media download: fetch `url` into `dest`. `force` re-downloads even
/// if `dest` already exists (used for edited posts whose media may have changed).
#[derive(Debug, Clone)]
pub struct Job {
    pub url: String,
    pub dest: PathBuf,
    pub force: bool,
}

static YT: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"(?:youtube\.com/(?:watch\?(?:[^ ]*&)?v=|embed/|shorts/|live/|v/)|youtu\.be/)([A-Za-z0-9_-]{6,})",
    )
    .unwrap()
});

/// Extract a YouTube video id from a single URL, if present.
pub fn youtube_id(url: &str) -> Option<String> {
    YT.captures(url).map(|c| c[1].to_string())
}

/// Return the first YouTube id found across a set of links.
pub fn youtube_from(links: &[String]) -> Option<String> {
    links.iter().find_map(|l| youtube_id(l))
}

/// Best-effort file extension from a URL (query string stripped).
pub fn ext_from_url(url: &str, default: &str) -> String {
    let path = url.split(['?', '#']).next().unwrap_or(url);
    let ext = path
        .rsplit('/')
        .next()
        .and_then(|f| f.rsplit_once('.'))
        .map(|(_, e)| e)
        .unwrap_or("")
        .to_ascii_lowercase();
    if ext.is_empty() || ext.len() > 5 || !ext.chars().all(|c| c.is_ascii_alphanumeric()) {
        default.to_string()
    } else {
        ext
    }
}

/// Download media into the post bundles. A file already present is skipped
/// (the committed bundle is the cache) unless `force` is set — Telegram CDN
/// URLs are tokenized and expire, so we persist the bytes, not the URL.
/// Failures are logged and tolerated — a backup run should never abort because
/// one file 404'd.
pub async fn download_all(client: &reqwest::Client, jobs: &[Job], concurrency: usize) -> Result<()> {
    let mut seen = std::collections::HashSet::new();
    let mut todo: Vec<Job> = Vec::new();
    for j in jobs {
        if !seen.insert(j.dest.clone()) {
            continue; // same destination requested twice this run
        }
        if j.dest.exists() && !j.force {
            continue; // already cached and not edited
        }
        todo.push(j.clone());
    }

    let total = todo.len();
    if total == 0 {
        tracing::info!("all media already present");
        return Ok(());
    }
    tracing::info!("downloading {} media files", total);

    let results = stream::iter(todo.into_iter().map(|j| {
        let client = client.clone();
        async move {
            download_one(&client, &j.url, &j.dest)
                .await
                .with_context(|| format!("downloading {}", j.url))
        }
    }))
    .buffer_unordered(concurrency.max(1))
    .collect::<Vec<_>>()
    .await;

    let errs = results.iter().filter(|r| r.is_err()).count();
    for r in results {
        if let Err(e) = r {
            tracing::warn!("media download failed: {:#}", e);
        }
    }
    if errs > 0 {
        tracing::warn!("{}/{} media downloads failed (continuing)", errs, total);
    }
    Ok(())
}

async fn download_one(client: &reqwest::Client, url: &str, dest: &Path) -> Result<()> {
    if let Some(parent) = dest.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    let resp = client.get(url).send().await?.error_for_status()?;
    let bytes = resp.bytes().await?;
    // Write to a temp file then rename, so an interrupted run never leaves a
    // truncated file that a later run would treat as "already cached".
    let tmp = dest.with_extension("part");
    tokio::fs::write(&tmp, &bytes).await?;
    tokio::fs::rename(&tmp, dest).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn youtube_variants() {
        assert_eq!(
            youtube_id("https://www.youtube.com/watch?v=uWSLHcAyy90"),
            Some("uWSLHcAyy90".to_string())
        );
        assert_eq!(
            youtube_id("https://youtu.be/6lPaEat4GgQ"),
            Some("6lPaEat4GgQ".to_string())
        );
        assert_eq!(
            youtube_id("https://www.youtube.com/shorts/abc123XYZ"),
            Some("abc123XYZ".to_string())
        );
        assert_eq!(youtube_id("https://example.com/watch?v=nope"), None);
    }

    #[test]
    fn extensions() {
        assert_eq!(ext_from_url("https://cdn/file/x.jpg", "bin"), "jpg");
        assert_eq!(ext_from_url("https://cdn/file/x.mp4?token=abc", "bin"), "mp4");
        assert_eq!(ext_from_url("https://cdn/file/noext", "jpg"), "jpg");
    }
}
