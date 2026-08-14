//! `--enex <file>`: export the archive as an Evernote **ENEX** file — one `<note>`
//! per post, with its text as ENML and every media file attached as a base64
//! `<resource>` (linked by its MD5 `en-media` hash, as Evernote requires).

use anyhow::{Context, Result};
use chrono::Utc;
use md5::{Digest, Md5};
use std::fmt::Write as _;
use std::path::Path;

use crate::export_media::ExportFile;
use crate::model::Post;
use crate::singlefile::b64;

/// Write an ENEX at `out` from posts and locally prepared media.
pub fn export(posts: &[Post], media: &[Vec<ExportFile>], out: &Path) -> Result<()> {
    if posts.len() != media.len() {
        anyhow::bail!(
            "cannot export ENEX: {} posts but {} media groups",
            posts.len(),
            media.len()
        );
    }
    let now = Utc::now().format("%Y%m%dT%H%M%SZ");
    let mut xml = format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
         <!DOCTYPE en-export SYSTEM \"http://xml.evernote.com/pub/evernote-export4.dtd\">\n\
         <en-export export-date=\"{now}\" application=\"tg2zola\" version=\"1\">\n"
    );
    let (mut n_notes, mut n_res) = (0usize, 0usize);
    for (post, files) in posts.iter().zip(media) {
        let title = crate::render::post_title(post, 200, true);
        let title = if title.is_empty() {
            format!("#{}", post.primary_id)
        } else {
            title
        };
        let created = post.date.with_timezone(&Utc).format("%Y%m%dT%H%M%SZ");

        // ENML body: the post text (newlines → <br/>), then an <en-media> per file.
        let mut body = esc(&crate::render::post_text_plain(post)).replace('\n', "<br/>");
        let mut resources = String::new();
        n_res += append_media(&mut body, &mut resources, files)?;
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
    tracing::info!(
        "enex: wrote {} — {n_notes} note(s), {n_res} resource(s)",
        out.display()
    );
    Ok(())
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn esc(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// Append locally available attachments to one ENML body and resource list.
fn append_media(body: &mut String, resources: &mut String, files: &[ExportFile]) -> Result<usize> {
    let mut count = 0;
    for file in files {
        let Ok(bytes) = std::fs::read(&file.path) else {
            continue;
        };
        let hash = hex(&Md5::digest(&bytes));
        let mime = crate::export_media::mime_for_filename(&file.filename);
        let _ = write!(body, "<br/><en-media type=\"{mime}\" hash=\"{hash}\"/>");
        let _ = writeln!(
            resources,
            "  <resource><data encoding=\"base64\">{}</data><mime>{mime}</mime>\
             <resource-attributes><file-name>{}</file-name></resource-attributes></resource>",
            b64(&bytes),
            esc(&file.filename)
        );
        count += 1;
    }
    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn md5_hex_matches_known_vector() {
        // Guards the en-media hash: MD5("abc") is a well-known value.
        assert_eq!(
            hex(&Md5::digest(b"abc")),
            "900150983cd24fb0d6963f7d28e17f72"
        );
    }

    #[test]
    fn escapes_xml() {
        assert_eq!(esc("a<b>&\"c"), "a&lt;b&gt;&amp;&quot;c");
    }

    #[test]
    fn export_resource_preserves_release_name_mime_and_bytes() {
        let dir = std::env::temp_dir().join(format!("tg2zola-enex-media-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("cached");
        std::fs::write(&path, b"release-image").unwrap();
        let files = [ExportFile {
            filename: "Original.AVIF".into(),
            path,
        }];
        let mut body = String::new();
        let mut resources = String::new();

        assert_eq!(append_media(&mut body, &mut resources, &files).unwrap(), 1);
        assert!(body.contains(r#"type="image/avif""#));
        assert!(resources.contains("<mime>image/avif</mime>"));
        assert!(resources.contains("<file-name>Original.AVIF</file-name>"));
        assert!(resources.contains(&b64(b"release-image")));
        std::fs::remove_dir_all(dir).unwrap();
    }
}
