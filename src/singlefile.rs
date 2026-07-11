//! `tg2zola single-file`: fold a built site (the Zola `public/` directory, built
//! with `base_url = "/"`) into one self-contained HTML file — every post
//! concatenated, the stylesheet inlined, and local images/media embedded as
//! `data:` URIs. Practical for small/text-leaning archives; inlining media
//! inflates ~33%, so files above a cap are left as a note rather than embedded.

use anyhow::{Context, Result};
use once_cell::sync::Lazy;
use regex::Regex;
use std::fs;
use std::path::Path;

/// Don't embed a single asset larger than this (a 25 MB data-URI is already a lot
/// for one HTML file); it's replaced with a short "omitted" note instead.
const MAX_INLINE: u64 = 25 * 1024 * 1024;

static ARTICLE: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?s)<article[^>]*>.*?</article>").unwrap());
// Root-absolute local asset ref, `src="/…"` (a base_url="/" build).
static ASSET: Lazy<Regex> = Lazy::new(|| Regex::new(r#"(src|href)="(/[^"]+)""#).unwrap());

/// Build `output` from the site under `public`.
pub fn build(public: &Path, output: &Path) -> Result<()> {
    let title = fs::read_to_string(public.join("index.html"))
        .ok()
        .and_then(|h| {
            Regex::new(r"<title>([^<]*)</title>").unwrap().captures(&h).map(|c| c[1].to_string())
        })
        .unwrap_or_else(|| "Archive".into());

    // Every post page, oldest → newest by numeric id.
    let mut posts: Vec<(u64, std::path::PathBuf)> = Vec::new();
    if let Ok(rd) = fs::read_dir(public.join("posts")) {
        for e in rd.flatten() {
            let idx = e.path().join("index.html");
            if let Some(id) = e.file_name().to_str().and_then(|n| n.parse::<u64>().ok()) {
                if idx.is_file() {
                    posts.push((id, idx));
                }
            }
        }
    }
    posts.sort_by_key(|(id, _)| *id);

    let mut body = String::new();
    for (_, path) in &posts {
        let html = fs::read_to_string(path).unwrap_or_default();
        if let Some(m) = ARTICLE.find(&html) {
            body.push_str(m.as_str());
            body.push('\n');
        }
    }

    let css = fs::read_to_string(public.join("style.css")).unwrap_or_default();
    let mut out = format!(
        "<!DOCTYPE html>\n<html><head><meta charset=\"utf-8\">\
         <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\
         <title>{}</title>\n<style>\n{css}\n</style>\n</head>\n<body><main>\n{body}\n</main></body></html>\n",
        esc(&title)
    );

    // Inline local assets (images, css already inlined) as data: URIs.
    let (mut inlined, mut omitted) = (0usize, 0usize);
    out = ASSET
        .replace_all(&out.clone(), |c: &regex::Captures| {
            let (attr, url) = (&c[1], &c[2]);
            let rel = url.trim_start_matches('/').split(['?', '#']).next().unwrap_or("");
            let file = public.join(rel);
            match fs::metadata(&file) {
                Ok(m) if m.len() <= MAX_INLINE => match fs::read(&file) {
                    Ok(bytes) => {
                        inlined += 1;
                        format!("{attr}=\"data:{};base64,{}\"", mime(rel), b64(&bytes))
                    }
                    Err(_) => c[0].to_string(),
                },
                Ok(_) => {
                    omitted += 1;
                    // Keep the reference but flag it (broken in a single file, but
                    // makes clear something large was left out).
                    format!("{attr}=\"#omitted-large-file\"")
                }
                Err(_) => c[0].to_string(),
            }
        })
        .into_owned();

    fs::write(output, &out).with_context(|| format!("writing {}", output.display()))?;
    tracing::info!(
        "single-file: {} post(s) → {} ({} asset(s) inlined, {} large one(s) omitted)",
        posts.len(),
        output.display(),
        inlined,
        omitted
    );
    Ok(())
}

fn mime(name: &str) -> &'static str {
    let n = name.to_ascii_lowercase();
    for (ext, m) in [
        (".png", "image/png"),
        (".jpg", "image/jpeg"),
        (".jpeg", "image/jpeg"),
        (".gif", "image/gif"),
        (".webp", "image/webp"),
        (".avif", "image/avif"),
        (".svg", "image/svg+xml"),
        (".mp3", "audio/mpeg"),
        (".ogg", "audio/ogg"),
        (".mp4", "video/mp4"),
        (".webm", "video/webm"),
        (".css", "text/css"),
    ] {
        if n.ends_with(ext) {
            return m;
        }
    }
    "application/octet-stream"
}

fn esc(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}

/// Minimal standard base64 (no external dep — `base64` is gated behind mtproto).
pub(crate) fn b64(data: &[u8]) -> String {
    const T: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(data.len().div_ceil(3) * 4);
    for chunk in data.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = *chunk.get(1).unwrap_or(&0) as u32;
        let b2 = *chunk.get(2).unwrap_or(&0) as u32;
        let n = (b0 << 16) | (b1 << 8) | b2;
        out.push(T[(n >> 18 & 63) as usize] as char);
        out.push(T[(n >> 12 & 63) as usize] as char);
        out.push(if chunk.len() > 1 { T[(n >> 6 & 63) as usize] as char } else { '=' });
        out.push(if chunk.len() > 2 { T[(n & 63) as usize] as char } else { '=' });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base64_matches_known_vectors() {
        assert_eq!(b64(b""), "");
        assert_eq!(b64(b"f"), "Zg==");
        assert_eq!(b64(b"fo"), "Zm8=");
        assert_eq!(b64(b"foo"), "Zm9v");
        assert_eq!(b64(b"foobar"), "Zm9vYmFy");
    }

    #[test]
    fn builds_one_file_with_inlined_assets() {
        let dir = std::env::temp_dir().join(format!("tg2zola-sf-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        let pub_ = dir.join("public");
        fs::create_dir_all(pub_.join("posts/1")).unwrap();
        fs::write(pub_.join("index.html"), "<title>My Site</title>").unwrap();
        fs::write(pub_.join("style.css"), "body{color:red}").unwrap();
        fs::write(pub_.join("posts/1/photo.jpg"), b"JPEGDATA").unwrap();
        fs::write(
            pub_.join("posts/1/index.html"),
            "<article class=\"post\">hi <img src=\"/posts/1/photo.jpg\"></article>",
        )
        .unwrap();
        let out = dir.join("archive.html");
        build(&pub_, &out).unwrap();
        let html = fs::read_to_string(&out).unwrap();
        assert!(html.contains("<title>My Site</title>"), "{html}");
        assert!(html.contains("body{color:red}"), "css not inlined: {html}");
        assert!(html.contains("hi <img src=\"data:image/jpeg;base64,"), "img not inlined: {html}");
        assert!(!html.contains("/posts/1/photo.jpg"), "raw asset ref remains: {html}");
        let _ = fs::remove_dir_all(&dir);
    }
}
