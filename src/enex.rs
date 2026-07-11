//! `--enex <file>`: export the archive as an Evernote **ENEX** file — one `<note>`
//! per post, with its text as ENML and every media file attached as a base64
//! `<resource>` (linked by its MD5 `en-media` hash, as Evernote requires).

use anyhow::{Context, Result};
use chrono::Utc;
use md5::{Digest, Md5};
use std::fmt::Write as _;
use std::path::Path;

use crate::model::Post;
use crate::render::RenderedPost;
use crate::singlefile::b64;

/// Write an ENEX at `out` from the posts and their downloaded media (read from
/// each post's bundle under `site`).
pub fn export(posts: &[Post], rendered: &[RenderedPost], site: &Path, out: &Path) -> Result<()> {
    let now = Utc::now().format("%Y%m%dT%H%M%SZ");
    let mut xml = format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
         <!DOCTYPE en-export SYSTEM \"http://xml.evernote.com/pub/evernote-export4.dtd\">\n\
         <en-export export-date=\"{now}\" application=\"tg2zola\" version=\"1\">\n"
    );
    let (mut n_notes, mut n_res) = (0usize, 0usize);
    for (post, r) in posts.iter().zip(rendered) {
        let title = crate::render::post_title(post, 200, true);
        let title = if title.is_empty() { format!("#{}", post.primary_id) } else { title };
        let created = post.date.with_timezone(&Utc).format("%Y%m%dT%H%M%SZ");

        // ENML body: the post text (newlines → <br/>), then an <en-media> per file.
        let mut body = esc(&crate::render::post_text_plain(post)).replace('\n', "<br/>");
        let mut resources = String::new();
        for d in &r.downloads {
            let path = site.join("content/posts").join(&r.slug).join(&d.filename);
            let Ok(bytes) = std::fs::read(&path) else { continue };
            let hash = hex(&Md5::digest(&bytes));
            let mime = mime(&d.filename);
            let _ = write!(body, "<br/><en-media type=\"{mime}\" hash=\"{hash}\"/>");
            let _ = writeln!(
                resources,
                "  <resource><data encoding=\"base64\">{}</data><mime>{mime}</mime>\
                 <resource-attributes><file-name>{}</file-name></resource-attributes></resource>",
                b64(&bytes),
                esc(&d.filename)
            );
            n_res += 1;
        }
        let _ = write!(
            xml,
            "<note>\n  <title>{}</title>\n  \
             <content><![CDATA[<?xml version=\"1.0\" encoding=\"UTF-8\"?>\
             <!DOCTYPE en-note SYSTEM \"http://xml.evernote.com/pub/enml2.dtd\">\
             <en-note>{body}</en-note>]]></content>\n  \
             <created>{created}</created>\n",
            esc(&title)
        );
        for t in &post.tags {
            let _ = writeln!(xml, "  <tag>{}</tag>", esc(t));
        }
        xml.push_str(&resources);
        xml.push_str("</note>\n");
        n_notes += 1;
    }
    xml.push_str("</en-export>\n");
    std::fs::write(out, &xml).with_context(|| format!("writing {}", out.display()))?;
    tracing::info!("enex: wrote {} — {n_notes} note(s), {n_res} resource(s)", out.display());
    Ok(())
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn esc(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
}

fn mime(name: &str) -> &'static str {
    let n = name.to_ascii_lowercase();
    for (ext, m) in [
        (".jpg", "image/jpeg"),
        (".jpeg", "image/jpeg"),
        (".png", "image/png"),
        (".webp", "image/webp"),
        (".gif", "image/gif"),
        (".mp4", "video/mp4"),
        (".webm", "video/webm"),
        (".mp3", "audio/mpeg"),
        (".ogg", "audio/ogg"),
        (".m4a", "audio/mp4"),
        (".pdf", "application/pdf"),
        (".zip", "application/zip"),
    ] {
        if n.ends_with(ext) {
            return m;
        }
    }
    "application/octet-stream"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn md5_hex_matches_known_vector() {
        // Guards the en-media hash: MD5("abc") is a well-known value.
        assert_eq!(hex(&Md5::digest(b"abc")), "900150983cd24fb0d6963f7d28e17f72");
    }

    #[test]
    fn escapes_xml() {
        assert_eq!(esc("a<b>&\"c"), "a&lt;b&gt;&amp;&quot;c");
    }
}
