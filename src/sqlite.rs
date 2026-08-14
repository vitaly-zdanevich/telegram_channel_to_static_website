//! `--sqlite <db>`: export the whole archive into one SQLite database — posts,
//! tags, links, reactions and every media file as a raw BLOB. A single portable
//! file that's great for preservation and `SELECT`-based analytics, and (unlike
//! the single-file HTML) fine for media-heavy channels since blobs aren't
//! base64-inflated.

use anyhow::{Context, Result};
use sqlite::{Connection, Value};
use std::path::Path;

use crate::export_media::ExportFile;
use crate::model::Post;

const SCHEMA: &str = "\
CREATE TABLE posts (id INTEGER PRIMARY KEY, date TEXT, author TEXT, body_md TEXT, views INTEGER, edited INTEGER);\
CREATE TABLE tags (post_id INTEGER, tag TEXT);\
CREATE TABLE links (post_id INTEGER, url TEXT);\
CREATE TABLE reactions (post_id INTEGER, emoji TEXT, count INTEGER);\
CREATE TABLE media (post_id INTEGER, filename TEXT, mime TEXT, bytes BLOB);";

/// Write a fresh SQLite archive at `db` from posts and locally prepared media.
pub fn export(posts: &[Post], media: &[Vec<ExportFile>], db: &Path) -> Result<()> {
    if posts.len() != media.len() {
        anyhow::bail!(
            "cannot export SQLite: {} posts but {} media groups",
            posts.len(),
            media.len()
        );
    }
    let _ = std::fs::remove_file(db); // always a fresh, deterministic file
    let conn = Connection::open(db).with_context(|| format!("opening {}", db.display()))?;
    conn.execute(SCHEMA).context("creating schema")?;
    conn.execute("BEGIN")?;

    let (mut n_posts, mut n_media) = (0usize, 0usize);
    for (post, files) in posts.iter().zip(media) {
        let id = post.primary_id as i64;
        insert(
            &conn,
            "INSERT INTO posts (id, date, author, body_md, views, edited) VALUES (?,?,?,?,?,?)",
            &[
                (1, Value::Integer(id)),
                (2, Value::String(post.date.to_rfc3339())),
                (3, post.author.clone().map_or(Value::Null, Value::String)),
                (4, Value::String(post.body_md.clone())),
                (
                    5,
                    post.views.map_or(Value::Null, |v| Value::Integer(v as i64)),
                ),
                (6, Value::Integer(post.edited as i64)),
            ],
        )?;
        for t in &post.tags {
            insert(
                &conn,
                "INSERT INTO tags (post_id, tag) VALUES (?,?)",
                &[(1, Value::Integer(id)), (2, Value::String(t.clone()))],
            )?;
        }
        for l in &post.links {
            insert(
                &conn,
                "INSERT INTO links (post_id, url) VALUES (?,?)",
                &[(1, Value::Integer(id)), (2, Value::String(l.clone()))],
            )?;
        }
        for (emoji, count) in &post.reactions {
            insert(
                &conn,
                "INSERT INTO reactions (post_id, emoji, count) VALUES (?,?,?)",
                &[
                    (1, Value::Integer(id)),
                    (2, Value::String(emoji.clone())),
                    (3, Value::Integer(*count as i64)),
                ],
            )?;
        }
        n_media += insert_media(&conn, id, files)?;
        n_posts += 1;
    }
    conn.execute("COMMIT")?;
    tracing::info!(
        "sqlite: wrote {} — {n_posts} post(s), {n_media} media blob(s)",
        db.display()
    );
    Ok(())
}

fn insert(conn: &Connection, sql: &str, values: &[(usize, Value)]) -> Result<()> {
    let mut statement = conn.prepare(sql)?;
    for (index, value) in values {
        statement.bind((*index, value))?;
    }
    statement.next()?;
    Ok(())
}

/// Insert every locally available attachment and return the inserted row count.
fn insert_media(conn: &Connection, post_id: i64, files: &[ExportFile]) -> Result<usize> {
    let mut count = 0;
    for file in files {
        if let Ok(bytes) = std::fs::read(&file.path) {
            insert(
                conn,
                "INSERT INTO media (post_id, filename, mime, bytes) VALUES (?,?,?,?)",
                &[
                    (1, Value::Integer(post_id)),
                    (2, Value::String(file.filename.clone())),
                    (
                        3,
                        Value::String(
                            crate::export_media::mime_for_filename(&file.filename).to_string(),
                        ),
                    ),
                    (4, Value::Binary(bytes)),
                ],
            )?;
            count += 1;
        }
    }
    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn export_media_blob_roundtrip_preserves_name_mime_and_bytes() {
        let dir = std::env::temp_dir().join(format!("tg2zola-sqlite-media-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("cached");
        std::fs::write(&path, b"release-image").unwrap();

        let conn = Connection::open(":memory:").unwrap();
        conn.execute(SCHEMA).unwrap();
        let files = [ExportFile {
            filename: "Original.AVIF".into(),
            path,
        }];
        assert_eq!(insert_media(&conn, 7, &files).unwrap(), 1);

        let mut q = conn
            .prepare("SELECT filename, mime, bytes FROM media WHERE post_id = 7")
            .unwrap();
        q.next().unwrap();
        assert_eq!(q.read::<String, _>(0).unwrap(), "Original.AVIF");
        assert_eq!(q.read::<String, _>(1).unwrap(), "image/avif");
        assert_eq!(q.read::<Vec<u8>, _>(2).unwrap(), b"release-image");
        std::fs::remove_dir_all(dir).unwrap();
    }
}
