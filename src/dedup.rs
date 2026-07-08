//! Content-address duplicate media. The same file (an image, but also a zip/rar,
//! pdf, video, audio …) reposted across several posts is downloaded into each
//! post's bundle, so the *published* site serves N copies (git already dedups
//! the repo, but Pages counts every physical file). After downloads we hash every
//! bundle file; any content that appears 2+ times is moved once into a shared
//! `static/media/<hash>.<ext>` store and each post's reference is rewritten to
//! that URL. Byte-exact — files are compared, not just hashed, so there are no
//! false merges. Lossless: identical bytes in, identical bytes out.

use std::collections::hash_map::DefaultHasher;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};

#[derive(Default, Debug, PartialEq, Eq)]
pub struct Stats {
    /// Distinct contents that had duplicates.
    pub groups: usize,
    /// Physical bundle files removed.
    pub files_removed: usize,
    /// Bytes no longer served (removed copies minus the one shared copy).
    pub bytes_saved: u64,
}

/// Deduplicate bundle media under `site` into `static/media/`, rewriting each
/// post's reference to the shared URL (built from `base_url`).
pub fn run(site: &Path, base_url: &str) -> std::io::Result<Stats> {
    // Bucket every bundle file by (size, content hash) — cheap and enough to
    // gather candidates; exact bytes decide the real grouping below.
    let mut buckets: HashMap<(u64, u64), Vec<PathBuf>> = HashMap::new();
    for file in bundle_files(site) {
        let Ok(bytes) = fs::read(&file) else { continue };
        let mut h = DefaultHasher::new();
        bytes.hash(&mut h);
        buckets.entry((bytes.len() as u64, h.finish())).or_default().push(file);
    }

    let media_dir = site.join("static/media");
    let mut used: HashSet<String> = HashSet::new();
    let mut stats = Stats::default();
    for (_, candidates) in buckets {
        if candidates.len() < 2 {
            continue;
        }
        // Split the (size, hash) bucket into byte-exact groups (hash collisions
        // are astronomically unlikely, but this keeps it *definitely* correct).
        let mut groups: Vec<(Vec<u8>, Vec<PathBuf>)> = Vec::new();
        for p in candidates {
            let Ok(b) = fs::read(&p) else { continue };
            match groups.iter_mut().find(|(c, _)| *c == b) {
                Some((_, v)) => v.push(p),
                None => groups.push((b, vec![p])),
            }
        }
        for (content, dupes) in groups {
            if dupes.len() < 2 {
                continue;
            }
            merge_group(&content, &dupes, &media_dir, base_url, &mut used, &mut stats)?;
        }
    }
    Ok(stats)
}

/// Move one copy to the shared store and repoint every duplicate's post to it.
fn merge_group(
    content: &[u8],
    dupes: &[PathBuf],
    media_dir: &Path,
    base_url: &str,
    used: &mut HashSet<String>,
    stats: &mut Stats,
) -> std::io::Result<()> {
    let ext = dupes[0].extension().and_then(|e| e.to_str()).filter(|e| !e.is_empty()).unwrap_or("bin");
    let mut h = DefaultHasher::new();
    content.hash(&mut h);
    // Stable, collision-proof name (append a counter only on the rare hash clash).
    let base = format!("{:016x}", h.finish());
    let mut name = format!("{base}.{ext}");
    let mut n = 1;
    while !used.insert(name.clone()) {
        name = format!("{base}-{n}.{ext}");
        n += 1;
    }
    fs::create_dir_all(media_dir)?;
    fs::write(media_dir.join(&name), content)?;

    let sep = if base_url.ends_with('/') { "" } else { "/" };
    let url = format!("{base_url}{sep}media/{name}");
    for f in dupes {
        let Some(fname) = f.file_name().and_then(|s| s.to_str()) else { continue };
        let idx = f.with_file_name("index.md");
        if let Ok(md) = fs::read_to_string(&idx) {
            // The filename appears as a Markdown target `](name)`, a shortcode /
            // <img> attribute `"name"`, or the front-matter `og_image = "name"`.
            let rewritten = md
                .replace(&format!("]({fname})"), &format!("]({url})"))
                .replace(&format!("\"{fname}\""), &format!("\"{url}\""));
            fs::write(&idx, rewritten)?;
        }
        fs::remove_file(f)?;
    }
    stats.groups += 1;
    stats.files_removed += dupes.len();
    stats.bytes_saved += content.len() as u64 * (dupes.len() as u64 - 1);
    Ok(())
}

/// Every media file inside a post/page bundle (i.e. not the `index.md`).
fn bundle_files(site: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    for sub in ["content/posts", "content/pages"] {
        let Ok(bundles) = fs::read_dir(site.join(sub)) else { continue };
        for b in bundles.flatten() {
            if !b.path().is_dir() {
                continue;
            }
            let Ok(files) = fs::read_dir(b.path()) else { continue };
            for f in files.flatten() {
                let p = f.path();
                if p.is_file() && p.file_name().and_then(|s| s.to_str()) != Some("index.md") {
                    out.push(p);
                }
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write(p: &Path, s: &[u8]) {
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        fs::write(p, s).unwrap();
    }

    #[test]
    fn dedups_identical_files_across_bundles_of_any_type() {
        let dir = std::env::temp_dir().join(format!("tg2zola-dedup-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        let a = dir.join("content/posts/2024-01-01-1");
        let b = dir.join("content/posts/2024-01-02-2");
        let c = dir.join("content/posts/2024-01-03-3");
        // Same zip content in posts 1 and 2; a unique file in post 3.
        write(&a.join("archive.zip"), b"PK-same-bytes");
        write(&b.join("archive.zip"), b"PK-same-bytes");
        write(&c.join("other.zip"), b"PK-different");
        write(&a.join("index.md"), b"+++\n+++\n[archive](archive.zip)\n");
        write(&b.join("index.md"), b"+++\n+++\ngrab [it](archive.zip) here\n");
        write(&c.join("index.md"), b"+++\n+++\n[x](other.zip)\n");

        let stats = run(&dir, "https://ex.com/site/").unwrap();
        assert_eq!(stats.groups, 1);
        assert_eq!(stats.files_removed, 2);
        assert_eq!(stats.bytes_saved, "PK-same-bytes".len() as u64); // 2 removed − 1 shared

        // Both duplicates gone; the unique file untouched.
        assert!(!a.join("archive.zip").exists());
        assert!(!b.join("archive.zip").exists());
        assert!(c.join("other.zip").exists());
        // One shared copy exists and both posts now point at it.
        let shared: Vec<_> = fs::read_dir(dir.join("static/media")).unwrap().collect();
        assert_eq!(shared.len(), 1);
        let md_a = fs::read_to_string(a.join("index.md")).unwrap();
        let md_b = fs::read_to_string(b.join("index.md")).unwrap();
        assert!(md_a.contains("https://ex.com/site/media/"), "{md_a}");
        assert!(md_b.contains("https://ex.com/site/media/"), "{md_b}");
        assert!(!md_a.contains("(archive.zip)"), "ref not rewritten: {md_a}");
        let _ = fs::remove_dir_all(&dir);
    }
}
