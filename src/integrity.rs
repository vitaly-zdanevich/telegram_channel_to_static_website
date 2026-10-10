//! Post-generation integrity check: every *local* media file a post references
//! must actually exist on disk. Catches a failed download, a missing MTProto
//! file, or a broken dedup rewrite before they ship as a broken `<img>`/link.
//! Read-only — local checks report to CI; missing Release media blocks publishing.

use once_cell::sync::Lazy;
use regex::Regex;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

/// Require every planned video/archive Release asset to exist before publishing
/// its URL. A nonempty staging file will be uploaded by CI; otherwise only an
/// exact tag/name match in CI's freshly fetched GitHub inventory is accepted.
/// This keeps an exhausted download retry from publishing a guaranteed 404.
pub fn require_release_media<'a>(
    site: &Path,
    posts: impl IntoIterator<Item = &'a crate::render::RenderedPost>,
) -> anyhow::Result<()> {
    let inventory = match fs::read_to_string(site.join(".image-releases.remote.tsv")) {
        Ok(raw) => raw,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(error) => return Err(error.into()),
    };
    let published: HashSet<&str> = inventory
        .lines()
        .filter_map(|line| {
            let mut fields = line.split('\t');
            let tag = fields.next()?;
            let asset = fields.next()?;
            let size = fields.next()?.parse::<u64>().ok()?;
            (tag == "media" && !asset.is_empty() && size > 0 && fields.next().is_none())
                .then_some(asset)
        })
        .collect();
    let staging = site.join(".video-releases");
    let mut missing = Vec::new();
    for post in posts {
        for download in post.downloads.iter().filter(|download| download.release) {
            let staged = staging.join(&download.filename);
            // Any staged file takes precedence because CI uploads it with
            // --clobber: an empty file must not overwrite a valid remote asset.
            let available = match staged.metadata() {
                Ok(meta) => meta.is_file() && meta.len() > 0,
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                    published.contains(download.filename.as_str())
                }
                Err(error) => return Err(error.into()),
            };
            if available {
                continue;
            }
            missing.push(format!("{}: {}", post.slug, download.filename));
        }
    }
    anyhow::ensure!(
        missing.is_empty(),
        "Release media is missing or has invalid staged bytes; refusing to publish broken files: {}",
        missing.join(", ")
    );
    Ok(())
}

// A Markdown link/image target `](target)` and a raw `src="target"` attribute —
// the two ways a bundle file is referenced (images, downloads, audio/video/
// carousel). `url="…"` (release videos) and `@/…` (internal links) are ignored.
static REF_LINK: Lazy<Regex> = Lazy::new(|| Regex::new(r"\]\(([^)\s]+)").unwrap());
static REF_SRC: Lazy<Regex> = Lazy::new(|| Regex::new(r#"src="([^"]+)""#).unwrap());

#[derive(Default, Debug)]
pub struct Report {
    /// Local media references checked.
    pub checked: usize,
    /// `(bundle, reference)` for each referenced file that's missing.
    pub missing: Vec<(String, String)>,
}

/// Check every post/page bundle's `index.md` for local media references that
/// don't resolve to a file.
pub fn check(site: &Path) -> Report {
    let static_dir = site.join("static");
    let mut r = Report::default();
    for sub in ["content/posts", "content/pages"] {
        let Ok(bundles) = fs::read_dir(site.join(sub)) else {
            continue;
        };
        for entry in bundles.flatten() {
            let bundle = entry.path();
            if !bundle.is_dir() {
                continue;
            }
            let Ok(md) = fs::read_to_string(bundle.join("index.md")) else {
                continue;
            };
            let name = bundle
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_string();
            let mut seen = HashSet::new();
            let refs = REF_LINK
                .captures_iter(&md)
                .chain(REF_SRC.captures_iter(&md));
            for c in refs {
                let rf = c[1].trim();
                if !seen.insert(rf.to_string()) {
                    continue;
                }
                let candidates = candidate_paths(&bundle, &static_dir, rf);
                if candidates.is_empty() {
                    continue; // external / internal-link / not a local file
                }
                r.checked += 1;
                if !candidates.iter().any(|p| p.exists()) {
                    r.missing.push((name.clone(), rf.to_string()));
                }
            }
        }
    }
    r
}

/// Where a reference could resolve on disk. Empty = not a local media file
/// (external URL, `@/` internal link, anchor, root-absolute nav link, …).
fn candidate_paths(bundle: &Path, static_dir: &Path, rf: &str) -> Vec<PathBuf> {
    if let Some(i) = rf.find("/media/") {
        // Deduped shared media (absolute or root-relative /media/<hash>.<ext>).
        return vec![static_dir.join("media").join(&rf[i + "/media/".len()..])];
    }
    if rf.starts_with("http")
        || rf.starts_with("@/")
        || rf.starts_with('#')
        || rf.starts_with("mailto:")
        || rf.starts_with('/')
    {
        return Vec::new();
    }
    // A bundle-relative filename: in the post's own bundle, or a shared static
    // asset (e.g. the avatar / about.me photo referenced via a shortcode).
    vec![bundle.join(rf), static_dir.join(rf)]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn release_media_requires_staged_or_confirmed_remote_bytes() {
        let dir =
            std::env::temp_dir().join(format!("tg2zola-release-check-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        let post = crate::render::RenderedPost {
            slug: "2026-10-08-2190".into(),
            title: String::new(),
            index_md: String::new(),
            og_image: None,
            downloads: vec![crate::render::Download {
                url: "https://cdn.example/video.mp4".into(),
                filename: "2190-01.mp4".into(),
                force: false,
                local: None,
                release: true,
            }],
        };
        let check = || require_release_media(&dir, [&post]);
        assert!(check().unwrap_err().to_string().contains("2190-01.mp4"));

        let staged = dir.join(".video-releases/2190-01.mp4");
        write(&staged, "");
        assert!(check().is_err(), "empty staging file must not count");
        write(&staged, "video bytes");
        check().unwrap();
        fs::remove_file(&staged).unwrap();

        let inventory = dir.join(".image-releases.remote.tsv");
        for invalid in [
            "images-2000\t2190-01.mp4\t11\n",
            "media\t2189-01.mp4\t11\n",
            "media\t2190-01.mp4\t0\n",
            "media\t2190-01.mp4\tinvalid\n",
            "media\t2190-01.mp4\t11\textra\n",
        ] {
            write(&inventory, invalid);
            assert!(check().is_err(), "invalid inventory accepted: {invalid}");
        }
        write(&inventory, "media\t2190-01.mp4\t6464966\n");
        check().unwrap();
        write(&staged, "");
        assert!(
            check().is_err(),
            "empty staging would clobber the remote asset"
        );
        fs::remove_dir_all(&dir).unwrap();
    }

    fn write(p: &Path, s: &str) {
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        fs::write(p, s).unwrap();
    }

    #[test]
    fn flags_missing_but_not_present_or_external() {
        let dir = std::env::temp_dir().join(format!("tg2zola-int-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        let bundle = dir.join("content/posts/2024-01-01-1");
        write(&bundle.join("photo.jpg"), "img");
        fs::create_dir_all(dir.join("static/media")).unwrap();
        fs::write(dir.join("static/media/abc.jpg"), "shared").unwrap();
        write(
            &bundle.join("index.md"),
            "+++\n+++\n![](photo.jpg)\n\n![](gone.jpg)\n\n{{ audio(src=\"lost.mp3\") }}\n\n\
             ![](https://ex.com/x.jpg)\n\n[a](@/posts/2/index.md)\n\n![](/media/abc.jpg)\n",
        );
        let r = check(&dir);
        // photo.jpg + /media/abc.jpg present; gone.jpg + lost.mp3 missing;
        // external + @/ ignored → 4 checked, 2 missing.
        assert_eq!(r.checked, 4, "{r:?}");
        let missing: Vec<&str> = r.missing.iter().map(|(_, f)| f.as_str()).collect();
        assert!(missing.contains(&"gone.jpg"), "{r:?}");
        assert!(missing.contains(&"lost.mp3"), "{r:?}");
        assert_eq!(r.missing.len(), 2, "{r:?}");
        let _ = fs::remove_dir_all(&dir);
    }
}
