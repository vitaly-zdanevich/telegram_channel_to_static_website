//! Post-process a built Zola site (the `public/` directory) so it opens directly
//! via `file://` with no web server:
//!
//! - make root-absolute links (`/style.css`, `/posts/…`) relative to each page,
//! - append `index.html` to directory links (browsers don't auto-resolve it for
//!   `file://`),
//! - drop cachebust/query strings from local links (`style.css?h=…` can't be
//!   found on a filesystem).
//!
//! Assumes the site was built with `base_url = "/"` (the default). External
//! links (`https://`, `mailto:`, `#anchors`) are left untouched.

use anyhow::{Context, Result};
use once_cell::sync::Lazy;
use regex::Regex;
use std::path::Path;

// Any root-absolute link `href="/..."` / `src="/..."`. The leading slash may be
// HTML-entity-escaped (`&#x2F;` / `&#47;`) when a template interpolates a URL
// without Tera's `| safe` filter; match those too so such links relativize.
static LOCAL_LINK: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"((?:href|src)=")((?:/|&#x2[fF];|&#47;)[^"]*)""#).unwrap());
// Zola's pagination `page/1/` redirect stubs carry a <script>; strip it so the
// output is JavaScript-free (the <noscript> meta-refresh + link still work).
static SCRIPT: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?s)<script\b[^>]*>.*?</script>").unwrap());

pub fn relativize(dir: &Path) -> Result<()> {
    anyhow::ensure!(dir.is_dir(), "{} is not a directory", dir.display());
    let mut count = 0usize;
    visit(dir, dir, &mut count)?;
    tracing::info!(
        "offline: rewrote {} HTML files under {} — open index.html directly",
        count,
        dir.display()
    );
    Ok(())
}

fn visit(root: &Path, cur: &Path, count: &mut usize) -> Result<()> {
    for entry in std::fs::read_dir(cur)? {
        let path = entry?.path();
        if path.is_dir() {
            visit(root, &path, count)?;
        } else if path
            .extension()
            .is_some_and(|e| e.eq_ignore_ascii_case("html"))
        {
            let html = std::fs::read_to_string(&path)?;
            let out = rewrite(&html, depth_of(root, &path));
            std::fs::write(&path, out).with_context(|| format!("writing {}", path.display()))?;
            *count += 1;
        }
    }
    Ok(())
}

/// Directory levels between `file` and `root` (root/index.html → 0).
fn depth_of(root: &Path, file: &Path) -> usize {
    file.strip_prefix(root)
        .map(|rel| rel.components().count().saturating_sub(1))
        .unwrap_or(0)
}

fn rewrite(html: &str, depth: usize) -> String {
    let prefix = "../".repeat(depth);
    let html = SCRIPT.replace_all(html, "");
    LOCAL_LINK
        .replace_all(&html, |c: &regex::Captures| {
            format!("{}{}\"", &c[1], fix_local(&c[2], &prefix))
        })
        .into_owned()
}

/// Map a root-absolute link (`/…`) to a relative, file://-openable path:
/// strip the leading `/`, drop any `?query` (meaningless on a filesystem), and
/// append `index.html` to anything that's a directory (trailing slash, or a
/// final segment with no file extension).
fn fix_local(abs: &str, prefix: &str) -> String {
    // Normalize any entity-escaped slashes back to `/` before splitting.
    let decoded = abs
        .replace("&#x2F;", "/")
        .replace("&#x2f;", "/")
        .replace("&#47;", "/");
    let no_slash = decoded.trim_start_matches('/');
    let (path, anchor) = match no_slash.split_once('#') {
        Some((p, a)) => (p, format!("#{a}")),
        None => (no_slash, String::new()),
    };
    let path = path.split('?').next().unwrap_or(path);
    let target = if path.is_empty() {
        "index.html".to_string()
    } else if path.ends_with('/') {
        format!("{path}index.html")
    } else if path.rsplit('/').next().is_some_and(|seg| seg.contains('.')) {
        path.to_string() // a file (has an extension)
    } else {
        format!("{path}/index.html") // a directory without a trailing slash
    };
    format!("{prefix}{target}{anchor}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn depth_two_page() {
        let html = r#"<a href="/posts/1/"><img src="/posts/1/x.jpg"><link href="/style.css?h=abc"><a href="/posts"><a href="/"><a href="https://x.com/">"#;
        let out = rewrite(html, 2);
        assert!(out.contains(r#"href="../../posts/1/index.html""#), "{out}"); // dir w/ slash
        assert!(out.contains(r#"src="../../posts/1/x.jpg""#), "{out}"); // file kept
        assert!(out.contains(r#"href="../../style.css""#), "{out}"); // query dropped
        assert!(out.contains(r#"href="../../posts/index.html""#), "{out}"); // dir w/o slash
        assert!(out.contains(r#"href="../../index.html""#), "{out}"); // root
        assert!(out.contains(r#"href="https://x.com/""#), "{out}"); // external untouched
    }

    #[test]
    fn entity_escaped_slashes() {
        // A pager link Tera escaped to `&#x2F;page&#x2F;2&#x2F;` must still
        // relativize (otherwise file:// resolves it to the filesystem root).
        let out = rewrite(r#"<a href="&#x2F;page&#x2F;2&#x2F;">Older</a>"#, 0);
        assert!(out.contains(r#"href="page/2/index.html""#), "{out}");
    }

    #[test]
    fn root_page_and_anchor() {
        let out = rewrite(r#"<link href="/style.css"><a href="/tags"><a href="/posts/1/#sec">"#, 0);
        assert!(out.contains(r#"href="style.css""#), "{out}");
        assert!(out.contains(r#"href="tags/index.html""#), "{out}");
        assert!(out.contains(r#"href="posts/1/index.html#sec""#), "{out}");
    }
}
