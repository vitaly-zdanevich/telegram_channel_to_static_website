//! Turn a [`Post`] into a Zola page bundle: an `index.md` (TOML front matter +
//! Markdown body) and the list of media files to place alongside it.
//!
//! The output references **only local files and YouTube** — never `t.me`.

use once_cell::sync::Lazy;
use regex::Regex;
use std::collections::HashMap;

use crate::media::ext_from_url;
use crate::model::{Forward, Media, Post};

/// One media file to fetch directly into the page bundle. The bundle (committed
/// to the `blog` branch) is itself the cache: if the file is already there, the
/// download is skipped.
#[derive(Debug, Clone)]
pub struct Download {
    pub url: String,
    /// Name the file takes inside the page bundle (referenced from Markdown).
    pub filename: String,
    /// Re-download even if already present (no stable key, and post was edited).
    pub force: bool,
}

pub struct RenderedPost {
    pub slug: String,
    pub index_md: String,
    pub downloads: Vec<Download>,
}

/// Slug (and content-dir name) for a post: `YYYY-MM-DD-<id>`. Zola strips the
/// date prefix to produce the permalink `/posts/<id>/`.
pub fn slug_for(post: &Post) -> String {
    format!("{}-{}", post.date.format("%Y-%m-%d"), post.primary_id)
}

/// Rewrites links that point at *this* channel's own messages into internal
/// (relative) Zola links, so the backup is self-navigating and survives the
/// channel's removal. Links to other channels are left untouched.
pub struct LinkRewriter {
    index: HashMap<u64, String>,
    md_link: Regex,
    root_md: Regex,
    bare: Regex,
}

impl LinkRewriter {
    pub fn new(channel: &str, posts: &[Post]) -> Self {
        let mut index = HashMap::new();
        for p in posts {
            let slug = slug_for(p);
            for id in &p.ids {
                index.insert(*id, slug.clone());
            }
        }
        Self::with_index(channel, index)
    }

    fn with_index(channel: &str, index: HashMap<u64, String>) -> Self {
        let ch = regex::escape(channel);
        let host = r"(?:https?:)?//(?:t\.me|telegram\.me|telegram\.dog)";
        Self {
            index,
            md_link: Regex::new(&format!(r"\[([^\]]*)\]\({host}/(?:s/)?{ch}/(\d+)[^)\s]*\)"))
                .unwrap(),
            root_md: Regex::new(&format!(r"\[([^\]]*)\]\({host}/{ch}/?\)")).unwrap(),
            bare: Regex::new(&format!(r"<?{host}/(?:s/)?{ch}/(\d+)[^\s>)\]]*>?")).unwrap(),
        }
    }

    /// Rewrite same-channel links in a Markdown string. Zola's `@/path` syntax
    /// resolves to the right URL even under a GitHub Pages subpath.
    pub fn rewrite(&self, md: &str) -> String {
        let s = self
            .md_link
            .replace_all(md, |c: &regex::Captures| {
                match c[2].parse::<u64>().ok().and_then(|id| self.index.get(&id)) {
                    Some(slug) => format!("[{}](@/posts/{}/index.md)", &c[1], slug),
                    None => c[0].to_string(),
                }
            })
            .into_owned();
        let s = self
            .root_md
            .replace_all(&s, |c: &regex::Captures| format!("[{}](@/_index.md)", &c[1]))
            .into_owned();
        self.bare
            .replace_all(&s, |c: &regex::Captures| {
                match c[1].parse::<u64>().ok().and_then(|id| self.index.get(&id)) {
                    Some(slug) => format!("[#{}](@/posts/{}/index.md)", &c[1], slug),
                    None => c[0].to_string(),
                }
            })
            .into_owned()
    }
}

/// The post's display title (used by callers to label neighbour links).
pub fn post_title(post: &Post, title_max: usize) -> String {
    title_and_body(post, title_max).0
}

pub fn render_post(
    post: &Post,
    links: &LinkRewriter,
    title_max: usize,
    newer: Option<(u64, &str)>,
    older: Option<(u64, &str)>,
) -> RenderedPost {
    let slug = slug_for(post);
    let (title, mut body_src) = title_and_body(post, title_max);
    // A forwarded post often ends with a link back to the source channel; we
    // already show that as the "forwarded from" attribution, so drop the dupe.
    if let Some(fwd) = &post.forwarded_from {
        body_src = strip_forward_backlink(&body_src, fwd);
    }

    let mut downloads = Vec::new();
    let mut body = String::new();
    let mut idx = 0usize;
    // First image in the post → og:image for social/Mastodon link previews.
    let mut og_image: Option<String> = None;

    if !body_src.trim().is_empty() {
        let text = links.rewrite(&body_src);
        body.push_str(text.trim());
        body.push_str("\n\n");
    }

    // Per spec, a YouTube link only *replaces an attached video*. On posts with
    // no video the YouTube URL stays as an ordinary link in the text.
    let has_video = post
        .media
        .iter()
        .any(|m| matches!(m, Media::Video { .. } | Media::VideoPoster { .. }));
    let drop_videos = has_video && post.youtube.is_some();
    // Embed YouTube for ANY post that links to YouTube (not only video posts).
    // The shortcode renders the iframe plus a plain "Watch on YouTube" link, so
    // it still works where the iframe can't load (e.g. over file://).
    if let Some(yt) = &post.youtube {
        body.push_str(&format!("{{{{ youtube(id=\"{yt}\") }}}}\n\n"));
    }

    for m in &post.media {
        idx += 1;
        match m {
            Media::Photo { url, key } | Media::Sticker { url, key } => {
                let (fname, force) = media_name(key, url, "jpg", idx, post.edited);
                push_dl(&mut downloads, url, &fname, force);
                if og_image.is_none() {
                    og_image = Some(fname.clone());
                }
                body.push_str(&format!("![]({fname})\n\n"));
            }
            Media::Video { url } => {
                if drop_videos {
                    continue;
                }
                let fname = format!("{idx:02}.{}", ext_from_url(url, "mp4"));
                push_dl(&mut downloads, url, &fname, post.edited);
                body.push_str(&format!("{{{{ video(src=\"{fname}\") }}}}\n\n"));
            }
            Media::VideoPoster { poster, duration } => {
                if drop_videos {
                    continue;
                }
                let dur = duration.clone().unwrap_or_default();
                if let Some(p) = poster {
                    let fname = format!("{idx:02}.{}", ext_from_url(p, "jpg"));
                    push_dl(&mut downloads, p, &fname, post.edited);
                    if og_image.is_none() {
                        og_image = Some(fname.clone());
                    }
                    body.push_str(&format!("![video]({fname})\n\n"));
                }
                let label = if dur.is_empty() {
                    "▶ video".to_string()
                } else {
                    format!("▶ video — {dur}")
                };
                body.push_str(&format!("*{label}*\n\n"));
            }
            Media::Audio { url, title } => {
                let fname = format!("{idx:02}.{}", ext_from_url(url, "mp3"));
                push_dl(&mut downloads, url, &fname, post.edited);
                if let Some(t) = title {
                    if !t.is_empty() {
                        body.push_str(&format!("*{}*\n\n", label_escape(t)));
                    }
                }
                body.push_str(&format!("{{{{ audio(src=\"{fname}\") }}}}\n\n"));
            }
            Media::Document { url, filename } => {
                let ext = ext_from_url(url, "bin");
                let fname = sanitize_filename(filename, &ext, idx);
                push_dl(&mut downloads, url, &fname, post.edited);
                body.push_str(&format!("[📎 {}]({fname})\n\n", label_escape(filename)));
            }
        }
    }

    let description = excerpt(&body_src, 200);
    let index_md = format!(
        "{}{}\n",
        front_matter(post, &title, &description, og_image.as_deref(), newer, older),
        body.trim_end()
    );
    RenderedPost {
        slug,
        index_md,
        downloads,
    }
}

/// Pick a bundle filename and whether to force re-download. With a stable key we
/// content-address the file (a replaced media yields a new key → new file, old
/// one pruned); without one we use a positional name and re-download on edits.
fn media_name(
    key: &Option<String>,
    url: &str,
    default_ext: &str,
    idx: usize,
    edited: bool,
) -> (String, bool) {
    let ext = ext_from_url(url, default_ext);
    match key {
        Some(k) => (format!("{}.{ext}", sanitize_key(k)), false),
        None => (format!("{idx:02}.{ext}"), edited),
    }
}

fn sanitize_key(k: &str) -> String {
    k.chars()
        .map(|c| if c.is_ascii_alphanumeric() || c == '_' { c } else { '_' })
        .collect()
}

fn push_dl(downloads: &mut Vec<Download>, url: &str, fname: &str, force: bool) {
    downloads.push(Download {
        url: url.to_string(),
        filename: fname.to_string(),
        force,
    });
}

fn front_matter(
    post: &Post,
    title: &str,
    description: &str,
    og_image: Option<&str>,
    newer: Option<(u64, &str)>,
    older: Option<(u64, &str)>,
) -> String {
    let mut fm = String::from("+++\n");
    fm.push_str(&format!("title = {}\n", toml_str(title)));
    // RFC3339 is a valid TOML offset date-time literal (unquoted).
    fm.push_str(&format!("date = {}\n", post.date.to_rfc3339()));
    // Plain-text excerpt → <meta description> + og:/twitter: descriptions.
    if !description.is_empty() {
        fm.push_str(&format!("description = {}\n", toml_str(description)));
    }

    if !post.tags.is_empty() {
        fm.push_str("\n[taxonomies]\n");
        let tags: Vec<String> = post.tags.iter().map(|t| toml_str(t)).collect();
        fm.push_str(&format!("tags = [{}]\n", tags.join(", ")));
    }

    fm.push_str("\n[extra]\n");
    fm.push_str(&format!(
        "tg_url = \"https://t.me/{}/{}\"\n",
        post.channel, post.primary_id
    ));
    if let Some(img) = og_image {
        fm.push_str(&format!("og_image = {}\n", toml_str(img)));
    }
    if let Some((id, t)) = newer {
        fm.push_str(&format!("next_id = {id}\n"));
        fm.push_str(&format!("next_title = {}\n", toml_str(t)));
    }
    if let Some((id, t)) = older {
        fm.push_str(&format!("prev_id = {id}\n"));
        fm.push_str(&format!("prev_title = {}\n", toml_str(t)));
    }
    if let Some(v) = post.views {
        fm.push_str(&format!("views = {v}\n"));
    }
    if let Some(f) = &post.forwarded_from {
        if !f.name.is_empty() {
            fm.push_str(&format!("forwarded_from = {}\n", toml_str(&f.name)));
            if let Some(u) = &f.url {
                if u.starts_with("http") {
                    fm.push_str(&format!("forwarded_from_url = {}\n", toml_str(u)));
                }
            }
        }
    }
    let ids: Vec<String> = post.ids.iter().map(|i| i.to_string()).collect();
    fm.push_str(&format!("ids = [{}]\n", ids.join(", ")));
    fm.push_str("+++\n\n");
    fm
}

static MD_LINK: Lazy<Regex> = Lazy::new(|| Regex::new(r"\[[^\]]*\]\([^)]*\)").unwrap());
static AUTOLINK: Lazy<Regex> = Lazy::new(|| Regex::new(r"<https?://[^>]+>").unwrap());
static BARE_URL: Lazy<Regex> = Lazy::new(|| Regex::new(r"https?://\S+").unwrap());
static TAG_SC: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"\{\{\s*tag\(t="([^"]*)"\)\s*\}\}"#).unwrap());
static HTML_TAG: Lazy<Regex> = Lazy::new(|| Regex::new(r"<[^>]+>").unwrap());
static SHORTCODE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\{\{[^}]*\}\}").unwrap());
static IMG_MD: Lazy<Regex> = Lazy::new(|| Regex::new(r"!\[[^\]]*\]\([^)]*\)").unwrap());
static MD_LINK_LABEL: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\[([^\]]*)\]\([^)]*\)").unwrap());

/// A plain-text excerpt for the meta/OG/Twitter description: keep hashtag words,
/// drop shortcodes, images, links and Markdown markup, collapse whitespace.
fn excerpt(md: &str, max: usize) -> String {
    let mut s = TAG_SC.replace_all(md, "$1").to_string();
    s = SHORTCODE.replace_all(&s, " ").to_string();
    s = IMG_MD.replace_all(&s, " ").to_string();
    s = MD_LINK_LABEL.replace_all(&s, "$1").to_string();
    s = AUTOLINK.replace_all(&s, " ").to_string();
    s = BARE_URL.replace_all(&s, " ").to_string();
    s = HTML_TAG.replace_all(&s, " ").to_string();
    s = s.replace(['*', '_', '`', '~', '#', '>', '\\'], "");
    let joined = s.split_whitespace().collect::<Vec<_>>().join(" ");
    truncate_chars(joined.trim(), max)
}

/// The post title plus the body to render. The title is the first body line
/// carrying real prose (keeping hashtag words, dropping `#`); falls back to the
/// tags, then the date. When that line is *pure prose* (no hashtags) it is cut
/// from the body so the headline isn't shown twice — but a line that also
/// carries hashtags is kept, so its clickable tags survive.
fn title_and_body(post: &Post, max: usize) -> (String, String) {
    if let Some((title, idx, had_tags, partial)) = title_from_body(&post.body_md, max) {
        // Keep the line in the body when it carries hashtags (their clickable
        // tags must survive) or when the title is only part of the line (a
        // first-sentence/truncated title), so no text is lost; otherwise cut it
        // to avoid showing the same line twice.
        let body = if had_tags || partial {
            post.body_md.clone()
        } else {
            remove_line(&post.body_md, idx)
        };
        (title, body)
    } else if !post.tags.is_empty() {
        (post.tags.join(" "), post.body_md.clone())
    } else {
        (
            post.date.format("%Y-%m-%d %H:%M").to_string(),
            post.body_md.clone(),
        )
    }
}

fn remove_line(body: &str, idx: usize) -> String {
    body.lines()
        .enumerate()
        .filter(|(i, _)| *i != idx)
        .map(|(_, l)| l)
        .collect::<Vec<_>>()
        .join("\n")
}

/// Returns `(title, line_index, line_has_hashtags, was_truncated)`.
fn title_from_body(body: &str, max: usize) -> Option<(String, usize, bool, bool)> {
    for (idx, raw) in body.lines().enumerate() {
        let lt = raw.trim_start();
        // Skip code fences and blockquotes (lyrics etc. shouldn't be the title).
        if lt.starts_with("```") || lt.starts_with('>') {
            continue;
        }
        // Is there prose on this line beyond hashtags and links?
        let mut probe = TAG_SC.replace_all(raw, "").to_string();
        probe = MD_LINK.replace_all(&probe, "").to_string();
        probe = AUTOLINK.replace_all(&probe, "").to_string();
        probe = BARE_URL.replace_all(&probe, "").to_string();
        probe = probe.replace(['*', '_', '`', '~', '\\', '#'], "");
        if !probe.trim().chars().any(|c| c.is_alphanumeric()) {
            continue; // hashtag/link-only line
        }

        // Build the title: keep the hashtag words, drop links, drop the `#`.
        let mut t = TAG_SC.replace_all(raw, "$1").to_string();
        t = HTML_TAG.replace_all(&t, "").to_string();
        t = MD_LINK.replace_all(&t, "").to_string();
        t = AUTOLINK.replace_all(&t, "").to_string();
        t = BARE_URL.replace_all(&t, "").to_string();
        t = t.replace(['*', '`', '~', '\\', '#'], "");
        let joined = t.split_whitespace().collect::<Vec<_>>().join(" ");
        // Drop a trailing ":" (e.g. "… #alias:" -> "… alias").
        let t = joined.trim_end_matches(':').trim();
        if t.is_empty()
            || matches!(t.to_lowercase().as_str(), "from" | "source" | "src" | "via" | "link")
        {
            continue;
        }
        let (title, partial) = first_sentence_capped(t, max);
        return Some((title, idx, TAG_SC.is_match(raw), partial));
    }
    None
}

/// The first sentence of `line`, capped at `max` chars. Returns the title and
/// whether it is *partial* — a subset of the line, because the line continues
/// with more sentences or the first sentence had to be truncated. A partial
/// title means the line must stay in the body so its full text isn't lost.
fn first_sentence_capped(line: &str, max: usize) -> (String, bool) {
    let chars: Vec<char> = line.chars().collect();
    // End of the first sentence: . ! ? followed by whitespace (or end of line).
    // Requiring whitespace avoids splitting "three.js" or "Opus 4.5".
    let mut end = None;
    for i in 0..chars.len() {
        if matches!(chars[i], '.' | '!' | '?')
            && chars.get(i + 1).map_or(true, |c| c.is_whitespace())
        {
            end = Some(i);
            break;
        }
    }
    let (sentence, partial) = match end {
        // A sentence break with more text after it → keep just the sentence.
        Some(i) if i + 1 < chars.len() => (chars[..=i].iter().collect::<String>(), true),
        // A single sentence (terminator at line end, or none at all).
        _ => (line.to_string(), false),
    };
    if sentence.chars().count() > max {
        (truncate_chars(&sentence, max), true)
    } else {
        (sentence.trim().to_string(), partial)
    }
}

static STANDALONE_LINK: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^\[([^\]]*)\]\(([^)]+)\)$").unwrap());
static TME_CHANNEL: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?:t\.me|telegram\.me|telegram\.dog)/(?:s/)?([A-Za-z0-9_]+)").unwrap());

/// The Telegram channel username referenced by a URL, lowercased.
fn tme_channel(url: &str) -> Option<String> {
    TME_CHANNEL.captures(url).map(|c| c[1].to_lowercase())
}

/// Drop a trailing standalone link line that just points back to the forwarded
/// source (same channel, or the source's display name) — the "forwarded from"
/// header already links it, so the line is redundant.
fn strip_forward_backlink(body: &str, fwd: &Forward) -> String {
    let lines: Vec<&str> = body.lines().collect();
    let mut last = lines.len();
    while last > 0 && lines[last - 1].trim().is_empty() {
        last -= 1;
    }
    if last == 0 {
        return body.to_string();
    }
    if let Some(caps) = STANDALONE_LINK.captures(lines[last - 1].trim()) {
        let text = caps.get(1).map_or("", |m| m.as_str()).trim();
        let url = caps.get(2).map_or("", |m| m.as_str());
        let by_name = !fwd.name.trim().is_empty() && text == fwd.name.trim();
        let by_channel = match (tme_channel(url), fwd.url.as_deref().and_then(tme_channel)) {
            (Some(a), Some(b)) => a == b,
            _ => false,
        };
        if by_name || by_channel {
            return lines[..last - 1].join("\n").trim_end().to_string();
        }
    }
    body.to_string()
}

fn truncate_chars(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        return s.to_string();
    }
    let mut t: String = s.chars().take(max).collect();
    if let Some(pos) = t.rfind(' ') {
        if pos > max / 2 {
            t.truncate(pos);
        }
    }
    format!("{}…", t.trim_end())
}

/// Escape a string for a TOML basic string.
fn toml_str(s: &str) -> String {
    let mut o = String::with_capacity(s.len() + 2);
    o.push('"');
    for c in s.chars() {
        match c {
            '"' => o.push_str("\\\""),
            '\\' => o.push_str("\\\\"),
            '\n' => o.push_str("\\n"),
            '\t' => o.push_str("\\t"),
            '\r' => {}
            c if (c as u32) < 0x20 => o.push_str(&format!("\\u{:04X}", c as u32)),
            c => o.push(c),
        }
    }
    o.push('"');
    o
}

/// Escape for a Markdown link/emphasis label.
fn label_escape(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('[', "\\[")
        .replace(']', "\\]")
        .replace('`', "\\`")
        .replace('*', "\\*")
}

fn sanitize_filename(name: &str, ext: &str, idx: usize) -> String {
    let base: String = name
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.' {
                c
            } else {
                '_'
            }
        })
        .collect();
    let base = base.trim_matches('_');
    if base.is_empty() {
        return format!("doc-{idx:02}.{ext}");
    }
    if base.contains('.') {
        base.to_string()
    } else {
        format!("{base}.{ext}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rewrites_same_channel_links_only() {
        let mut idx = HashMap::new();
        idx.insert(1234u64, "2026-01-01-1234".to_string());
        let rw = LinkRewriter::with_index("mychan", idx);
        let out = rw.rewrite(
            "see [prev](https://t.me/mychan/1234), [x](https://t.me/otherchan/5), \
             bare https://t.me/mychan/1234 end",
        );
        // same-channel markdown link -> internal
        assert!(
            out.contains("[prev](@/posts/2026-01-01-1234/index.md)"),
            "{out}"
        );
        // other channel -> untouched
        assert!(out.contains("[x](https://t.me/otherchan/5)"), "{out}");
        // same-channel bare URL -> internal
        assert!(
            out.contains("[#1234](@/posts/2026-01-01-1234/index.md)"),
            "{out}"
        );
    }

    #[test]
    fn unknown_same_channel_id_left_alone() {
        let rw = LinkRewriter::with_index("mychan", HashMap::new());
        let out = rw.rewrite("[a](https://t.me/mychan/999)");
        assert_eq!(out, "[a](https://t.me/mychan/999)");
    }
}
