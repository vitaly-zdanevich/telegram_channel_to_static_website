//! Resolve every post attachment to a local file for portable archive exports.
//!
//! Published page bundles already contain ordinary downloads, while large media
//! and original MTProto images may live in GitHub Releases. Export preparation
//! prefers this run's staging files and downloads only missing Release assets to
//! the unpublished .mtproto-cache/export cache.

use anyhow::{bail, Result};
use md5::{Digest, Md5};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::media::Job;
use crate::model::{Media, Post};
use crate::render::RenderedPost;

/// One attachment ready for SQLite or ENEX serialization.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExportFile {
    /// Original bundle or GitHub Release asset filename.
    pub filename: String,
    /// Local source containing the bytes to serialize.
    pub path: PathBuf,
}

/// Materialize every attachment used by posts into local files.
///
/// This function is intentionally called only when at least one portable export
/// was requested; normal site builds must not download already-published Release
/// assets merely to render their external URLs.
pub async fn prepare(
    client: &reqwest::Client,
    posts: &[Post],
    rendered: &[RenderedPost],
    site: &Path,
    release_base: Option<&str>,
    concurrency: usize,
) -> Result<Vec<Vec<ExportFile>>> {
    if posts.len() != rendered.len() {
        bail!(
            "cannot prepare export media: {} posts but {} rendered posts",
            posts.len(),
            rendered.len()
        );
    }

    let mut remote_paths: HashMap<String, PathBuf> = HashMap::new();
    let mut jobs = Vec::new();
    let mut prepared = Vec::with_capacity(posts.len());
    for (post, rendered) in posts.iter().zip(rendered) {
        let mut files = Vec::new();
        for download in &rendered.downloads {
            let Some(filename) = safe_filename(&download.filename) else {
                tracing::warn!(
                    "export: ignoring unsafe media filename {:?}",
                    download.filename
                );
                continue;
            };
            let path = if download.release {
                let staged = site.join(".video-releases").join(&filename);
                if staged.is_file() {
                    staged
                } else {
                    let Some(base) = release_base else {
                        tracing::warn!(
                            "export: no GitHub Release base for staged media {filename}"
                        );
                        continue;
                    };
                    let url = format!("{}/{filename}", base.trim_end_matches('/'));
                    cached_release_path(site, &url, &filename, &mut remote_paths, &mut jobs)
                }
            } else {
                site.join("content/posts")
                    .join(&rendered.slug)
                    .join(&filename)
            };
            files.push(ExportFile { filename, path });
        }

        for media in &post.media {
            let Media::ReleasePhoto { url } = media else {
                continue;
            };
            let Some((tag, filename)) = release_url_parts(url) else {
                tracing::warn!("export: ignoring malformed GitHub Release image URL {url}");
                continue;
            };
            let staged = site.join(".image-releases").join(&tag).join(&filename);
            let path = if staged.is_file() {
                staged
            } else {
                cached_release_path(site, url, &filename, &mut remote_paths, &mut jobs)
            };
            files.push(ExportFile { filename, path });
        }
        prepared.push(files);
    }

    crate::media::download_all(client, &jobs, concurrency).await?;
    for files in &prepared {
        for file in files {
            if !file.path.is_file() {
                tracing::warn!(
                    "export: media bytes are unavailable for {} ({})",
                    file.filename,
                    file.path.display()
                );
            }
        }
    }
    Ok(prepared)
}

/// MIME type inferred from a preserved attachment's filename.
pub fn mime_for_filename(name: &str) -> &'static str {
    let name = name.to_ascii_lowercase();
    for (extension, mime) in [
        (".jpg", "image/jpeg"),
        (".jpeg", "image/jpeg"),
        (".png", "image/png"),
        (".webp", "image/webp"),
        (".gif", "image/gif"),
        (".avif", "image/avif"),
        (".bmp", "image/bmp"),
        (".mp4", "video/mp4"),
        (".webm", "video/webm"),
        (".mov", "video/quicktime"),
        (".m4v", "video/x-m4v"),
        (".mkv", "video/x-matroska"),
        (".mp3", "audio/mpeg"),
        (".ogg", "audio/ogg"),
        (".oga", "audio/ogg"),
        (".m4a", "audio/mp4"),
        (".aac", "audio/aac"),
        (".opus", "audio/opus"),
        (".wav", "audio/wav"),
        (".flac", "audio/flac"),
        (".pdf", "application/pdf"),
        (".zip", "application/zip"),
        (".tar.xz", "application/x-xz"),
        (".xz", "application/x-xz"),
    ] {
        if name.ends_with(extension) {
            return mime;
        }
    }
    "application/octet-stream"
}

fn cached_release_path(
    site: &Path,
    url: &str,
    filename: &str,
    remote_paths: &mut HashMap<String, PathBuf>,
    jobs: &mut Vec<Job>,
) -> PathBuf {
    if let Some(path) = remote_paths.get(url) {
        return path.clone();
    }
    let digest = Md5::digest(url.as_bytes());
    let path = site
        .join(".mtproto-cache/export")
        .join(format!("{digest:x}-{filename}"));
    remote_paths.insert(url.to_string(), path.clone());
    jobs.push(Job {
        url: url.to_string(),
        dest: path.clone(),
        force: false,
        local: None,
    });
    path
}

fn release_url_parts(url: &str) -> Option<(String, String)> {
    let parsed = url::Url::parse(url).ok()?;
    let segments = parsed.path_segments()?.collect::<Vec<_>>();
    let release = segments
        .windows(2)
        .position(|pair| pair == ["releases", "download"])?;
    if segments.len() != release + 4 {
        return None;
    }
    let tag = decode_safe_segment(segments[release + 2])?;
    let filename = decode_safe_segment(segments[release + 3])?;
    Some((tag, filename))
}

fn decode_safe_segment(segment: &str) -> Option<String> {
    let decoded = percent_encoding::percent_decode_str(segment)
        .decode_utf8()
        .ok()?
        .into_owned();
    safe_filename(&decoded)
}

fn safe_filename(filename: &str) -> Option<String> {
    (!filename.is_empty()
        && filename != "."
        && filename != ".."
        && !filename.contains(['/', '\\', '\0']))
    .then(|| filename.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Media;
    use crate::render::Download;
    use chrono::TimeZone;
    use std::sync::atomic::{AtomicU64, Ordering};
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    static NEXT_DIR: AtomicU64 = AtomicU64::new(0);

    fn temp_dir(label: &str) -> PathBuf {
        let path = std::env::temp_dir().join(format!(
            "tg2zola-export-{label}-{}-{}",
            std::process::id(),
            NEXT_DIR.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&path).unwrap();
        path
    }

    fn post(media: Vec<Media>) -> Post {
        Post {
            primary_id: 7,
            ids: vec![7],
            channel: "channel".into(),
            date: chrono::FixedOffset::east_opt(0)
                .unwrap()
                .timestamp_opt(1_700_000_000, 0)
                .unwrap(),
            author: None,
            forwarded_from: None,
            reply: None,
            poll: None,
            body_md: "body".into(),
            tags: vec![],
            media,
            views: None,
            edited: false,
            reactions: vec![],
            links: vec![],
            youtube: None,
            apple_podcast: None,
            yandex_music: None,
            instagram: None,
            spotify: None,
            pinterest: None,
            youtube_dead: false,
            youtube_watchable: false,
            apple_dead: false,
            yandex_dead: false,
            instagram_dead: false,
            spotify_dead: false,
            pinterest_dead: false,
            pinterest_live: false,
            genius_song_id: None,
            bandcamp: None,
            vk_playlist: None,
            related: vec![],
            wikidata_html: vec![],
        }
    }

    fn rendered(downloads: Vec<Download>) -> RenderedPost {
        RenderedPost {
            slug: "2026-01-01-7".into(),
            title: String::new(),
            index_md: String::new(),
            og_image: None,
            downloads,
        }
    }

    fn download(filename: &str, release: bool) -> Download {
        Download {
            url: "https://telegram.invalid/source".into(),
            filename: filename.into(),
            force: false,
            local: None,
            release,
        }
    }

    #[tokio::test]
    async fn prefers_staging_and_fetches_each_missing_release_url_once() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/releases/download/media/missing.webm"))
            .respond_with(ResponseTemplate::new(200).set_body_bytes(b"remote-video"))
            .expect(1)
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/releases/download/images-0000/remote.png"))
            .respond_with(ResponseTemplate::new(200).set_body_bytes(b"remote-image"))
            .expect(1)
            .mount(&server)
            .await;

        let site = temp_dir("staging");
        let bundle = site.join("content/posts/2026-01-01-7/local.jpg");
        std::fs::create_dir_all(bundle.parent().unwrap()).unwrap();
        std::fs::write(&bundle, b"bundle-image").unwrap();
        let video_stage = site.join(".video-releases/staged.mp4");
        std::fs::create_dir_all(video_stage.parent().unwrap()).unwrap();
        std::fs::write(&video_stage, b"staged-video").unwrap();
        let image_stage = site.join(".image-releases/images-0000/staged.jpg");
        std::fs::create_dir_all(image_stage.parent().unwrap()).unwrap();
        std::fs::write(&image_stage, b"staged-image").unwrap();

        let remote_image = format!("{}/releases/download/images-0000/remote.png", server.uri());
        let posts = [post(vec![
            Media::ReleasePhoto {
                url: format!("{}/releases/download/images-0000/staged.jpg", server.uri()),
            },
            Media::ReleasePhoto {
                url: remote_image.clone(),
            },
            Media::ReleasePhoto { url: remote_image },
        ])];
        let rendered = [rendered(vec![
            download("local.jpg", false),
            download("staged.mp4", true),
            download("missing.webm", true),
        ])];
        let release_base = format!("{}/releases/download/media", server.uri());

        let files = prepare(
            &reqwest::Client::new(),
            &posts,
            &rendered,
            &site,
            Some(&release_base),
            4,
        )
        .await
        .unwrap();

        assert_eq!(
            files[0]
                .iter()
                .map(|file| file.filename.as_str())
                .collect::<Vec<_>>(),
            [
                "local.jpg",
                "staged.mp4",
                "missing.webm",
                "staged.jpg",
                "remote.png",
                "remote.png",
            ]
        );
        let bytes = files[0]
            .iter()
            .map(|file| std::fs::read(&file.path).unwrap())
            .collect::<Vec<_>>();
        assert_eq!(bytes[0], b"bundle-image");
        assert_eq!(bytes[1], b"staged-video");
        assert_eq!(bytes[2], b"remote-video");
        assert_eq!(bytes[3], b"staged-image");
        assert_eq!(bytes[4], b"remote-image");
        assert_eq!(files[0][4].path, files[0][5].path);
        assert!(files[0][4]
            .path
            .starts_with(site.join(".mtproto-cache/export")));
        std::fs::remove_dir_all(site).unwrap();
    }

    #[tokio::test]
    async fn cache_keys_include_the_full_release_url() {
        let server = MockServer::start().await;
        for tag in ["images-0000", "images-0500"] {
            Mock::given(method("GET"))
                .and(path(format!("/releases/download/{tag}/same.jpg")))
                .respond_with(ResponseTemplate::new(200).set_body_string(tag))
                .expect(1)
                .mount(&server)
                .await;
        }
        let site = temp_dir("collision");
        let posts = [post(vec![
            Media::ReleasePhoto {
                url: format!("{}/releases/download/images-0000/same.jpg", server.uri()),
            },
            Media::ReleasePhoto {
                url: format!("{}/releases/download/images-0500/same.jpg", server.uri()),
            },
        ])];
        let rendered = [rendered(vec![])];

        let files = prepare(&reqwest::Client::new(), &posts, &rendered, &site, None, 2)
            .await
            .unwrap();

        assert_ne!(files[0][0].path, files[0][1].path);
        assert_eq!(
            std::fs::read_to_string(&files[0][0].path).unwrap(),
            "images-0000"
        );
        assert_eq!(
            std::fs::read_to_string(&files[0][1].path).unwrap(),
            "images-0500"
        );
        std::fs::remove_dir_all(site).unwrap();
    }

    #[test]
    fn shared_mime_table_covers_release_media() {
        assert_eq!(mime_for_filename("PHOTO.AVIF"), "image/avif");
        assert_eq!(mime_for_filename("clip.mov"), "video/quicktime");
        assert_eq!(mime_for_filename("voice.opus"), "audio/opus");
        assert_eq!(mime_for_filename("archive.tar.xz"), "application/x-xz");
    }
}
