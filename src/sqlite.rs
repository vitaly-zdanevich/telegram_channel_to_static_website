//! `--sqlite <db>`: export the whole archive into one SQLite database — posts,
//! tags, links, reactions and every media file as a raw BLOB. A single portable
//! file that's great for preservation and `SELECT`-based analytics, and (unlike
//! the single-file HTML) fine for media-heavy channels since blobs aren't
//! base64-inflated.

use anyhow::{Context, Result};
use sqlite::{Connection, Value};
use std::path::Path;

use crate::model::Post;
use crate::render::RenderedPost;

const SCHEMA: &str = "\
CREATE TABLE posts (id INTEGER PRIMARY KEY, date TEXT, author TEXT, body_md TEXT, views INTEGER, edited INTEGER);\
CREATE TABLE tags (post_id INTEGER, tag TEXT);\
CREATE TABLE links (post_id INTEGER, url TEXT);\
CREATE TABLE reactions (post_id INTEGER, emoji TEXT, count INTEGER);\
CREATE TABLE media (post_id INTEGER, filename TEXT, mime TEXT, bytes BLOB);";

/// Write a fresh SQLite archive at `db` from the posts and their downloaded media
/// (read from each post's bundle under `site`).
pub fn export(posts: &[Post], rendered: &[RenderedPost], site: &Path, db: &Path) -> Result<()> {
    let _ = std::fs::remove_file(db); // always a fresh, deterministic file
    let conn = Connection::open(db).with_context(|| format!("opening {}", db.display()))?;
    conn.execute(SCHEMA).context("creating schema")?;
    conn.execute("BEGIN")?;

    let (mut n_posts, mut n_media) = (0usize, 0usize);
    let insert = |sql: &str, vals: &[(usize, Value)]| -> Result<()> {
        let mut st = conn.prepare(sql)?;
        for (i, v) in vals {
            st.bind((*i, v))?;
        }
        st.next()?;
        Ok(())
    };
    for (post, r) in posts.iter().zip(rendered) {
        let id = post.primary_id as i64;
        insert(
            "INSERT INTO posts (id, date, author, body_md, views, edited) VALUES (?,?,?,?,?,?)",
            &[
                (1, Value::Integer(id)),
                (2, Value::String(post.date.to_rfc3339())),
                (3, post.author.clone().map_or(Value::Null, Value::String)),
                (4, Value::String(post.body_md.clone())),
                (5, post.views.map_or(Value::Null, |v| Value::Integer(v as i64))),
                (6, Value::Integer(post.edited as i64)),
            ],
        )?;
        for t in &post.tags {
            insert(
                "INSERT INTO tags (post_id, tag) VALUES (?,?)",
                &[(1, Value::Integer(id)), (2, Value::String(t.clone()))],
            )?;
        }
        for l in &post.links {
            insert(
                "INSERT INTO links (post_id, url) VALUES (?,?)",
                &[(1, Value::Integer(id)), (2, Value::String(l.clone()))],
            )?;
        }
        for (emoji, count) in &post.reactions {
            insert(
                "INSERT INTO reactions (post_id, emoji, count) VALUES (?,?,?)",
                &[
                    (1, Value::Integer(id)),
                    (2, Value::String(emoji.clone())),
                    (3, Value::Integer(*count as i64)),
                ],
            )?;
        }
        for d in &r.downloads {
            let path = site.join("content/posts").join(&r.slug).join(&d.filename);
            if let Ok(bytes) = std::fs::read(&path) {
                insert(
                    "INSERT INTO media (post_id, filename, mime, bytes) VALUES (?,?,?,?)",
                    &[
                        (1, Value::Integer(id)),
                        (2, Value::String(d.filename.clone())),
                        (3, Value::String(mime(&d.filename).to_string())),
                        (4, Value::Binary(bytes)),
                    ],
                )?;
                n_media += 1;
            }
        }
        n_posts += 1;
    }
    conn.execute("COMMIT")?;
    tracing::info!("sqlite: wrote {} — {n_posts} post(s), {n_media} media blob(s)", db.display());
    Ok(())
}

fn mime(name: &str) -> &'static str {
    let n = name.to_ascii_lowercase();
    for (ext, m) in [
        (".jpg", "image/jpeg"),
        (".jpeg", "image/jpeg"),
        (".png", "image/png"),
        (".webp", "image/webp"),
        (".gif", "image/gif"),
        (".avif", "image/avif"),
        (".mp4", "video/mp4"),
        (".webm", "video/webm"),
        (".mov", "video/quicktime"),
        (".mp3", "audio/mpeg"),
        (".ogg", "audio/ogg"),
        (".oga", "audio/ogg"),
        (".m4a", "audio/mp4"),
        (".opus", "audio/opus"),
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
    fn schema_and_blob_roundtrip() {
        // Valid schema + a blob round-trips (guards SQL typos and the bundled
        // sqlite integration without constructing a full Post).
        let conn = Connection::open(":memory:").unwrap();
        conn.execute(SCHEMA).unwrap();
        let mut st = conn
            .prepare("INSERT INTO media (post_id, filename, mime, bytes) VALUES (?,?,?,?)")
            .unwrap();
        st.bind((1, 1i64)).unwrap();
        st.bind((2, "a.jpg")).unwrap();
        st.bind((3, mime("a.JPG"))).unwrap();
        st.bind((4, &[1u8, 2, 3][..])).unwrap();
        st.next().unwrap();

        let mut q =
            conn.prepare("SELECT mime, length(bytes) FROM media WHERE post_id = 1").unwrap();
        q.next().unwrap();
        assert_eq!(q.read::<String, _>(0).unwrap(), "image/jpeg");
        assert_eq!(q.read::<i64, _>(1).unwrap(), 3);
    }
}
