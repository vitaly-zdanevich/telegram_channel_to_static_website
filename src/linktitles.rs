//! Hover tooltips for links. For a Wikipedia / MediaWiki article, a Fandom or
//! miraheze wiki page, or a YouTube video, fetch a short description at build
//! time and attach it as the link's `title=` (via a CommonMark link title), so
//! hovering the link shows an intro without leaving the page. Static — the
//! rendered `title` attribute needs no JavaScript and survives the offline pass.

use crate::model::Post;
use futures::StreamExt;
use percent_encoding::{percent_decode_str, utf8_percent_encode, NON_ALPHANUMERIC};
use serde_json::Value as J;
use std::collections::HashMap;

/// Resolve tooltips for every eligible link across `posts` and splice them into
/// each post's body. One fetch per distinct URL, bounded by `concurrency`.
pub async fn enrich(client: &reqwest::Client, posts: &mut [Post], concurrency: usize) {
    let mut urls: Vec<String> = Vec::new();
    for p in posts.iter() {
        for l in &p.links {
            if (is_youtube(l) || mediawiki(l).is_some() || habr_user(l).is_some())
                && !urls.contains(l)
            {
                urls.push(l.clone());
            }
        }
    }
    if urls.is_empty() {
        return;
    }
    let titles: HashMap<String, String> = futures::stream::iter(urls.into_iter().map(|u| async {
        let t = fetch_title(client, &u).await;
        (u, t)
    }))
    .buffer_unordered(concurrency.max(1))
    .collect::<Vec<_>>()
    .await
    .into_iter()
    .filter_map(|(u, t)| t.map(|t| (u, t)))
    .collect();

    for p in posts.iter_mut() {
        for l in &p.links {
            if let Some(t) = titles.get(l) {
                add_title(&mut p.body_md, l, t);
            }
        }
    }
}

fn is_youtube(url: &str) -> bool {
    (url.contains("youtube.com/watch") || url.contains("youtu.be/")) && url.starts_with("http")
}

/// `(origin, page title)` for a MediaWiki `/wiki/<Title>` URL, else `None`.
fn mediawiki(url: &str) -> Option<(String, String)> {
    let u = url::Url::parse(url).ok()?;
    if !matches!(u.scheme(), "http" | "https") {
        return None;
    }
    let title = u.path().strip_prefix("/wiki/").filter(|t| !t.is_empty())?;
    let origin = format!("{}://{}", u.scheme(), u.host_str()?);
    let title = percent_decode_str(title).decode_utf8_lossy().replace('_', " ");
    Some((origin, title))
}

async fn fetch_title(client: &reqwest::Client, url: &str) -> Option<String> {
    if is_youtube(url) {
        youtube_title(client, url).await
    } else if let Some(alias) = habr_user(url) {
        habr_card(client, &alias).await
    } else if let Some(file) = commons_file(url) {
        // A Commons file page → author + date instead of a (useless) extract.
        commons_credit(client, &file).await
    } else if let Some((origin, title)) = mediawiki(url) {
        mediawiki_extract(client, &origin, &title).await
    } else {
        None
    }
}

/// The user alias for a `habr.com/…/users/<alias>/…` profile link, else `None`.
fn habr_user(url: &str) -> Option<String> {
    let u = url::Url::parse(url).ok()?;
    let host = u.host_str()?;
    if host != "habr.com" && !host.ends_with(".habr.com") {
        return None;
    }
    let mut segs = u.path_segments()?.peekable();
    while let Some(s) = segs.next() {
        if s == "users" {
            return segs.next().filter(|a| !a.is_empty()).map(str::to_string);
        }
    }
    None
}

/// A one-line stats summary for a Habr user, from Habr's public card JSON
/// (`/kek/v2/users/<alias>/card/`). Modern Habr exposes `rating` and `score`
/// rather than a separate "karma" number.
async fn habr_card(client: &reqwest::Client, alias: &str) -> Option<String> {
    let api = format!("https://habr.com/kek/v2/users/{alias}/card/");
    let j = get_json(client, &api).await?;
    let mut parts: Vec<String> = Vec::new();
    let count = |ptr: &str, label: &str| {
        j.pointer(ptr).and_then(J::as_i64).map(|v| format!("{v} {label}"))
    };
    parts.extend(count("/counterStats/publicationStats/articleCount", "articles"));
    parts.extend(count("/counterStats/publicationStats/postCount", "posts"));
    parts.extend(count("/counterStats/publicationStats/newsCount", "news"));
    parts.extend(count("/counterStats/commentCount", "comments"));
    if let Some(reg) = j.get("registerDateTime").and_then(J::as_str) {
        parts.push(format!("registered {}", reg.split('T').next().unwrap_or(reg)));
    }
    if let Some(r) = j.get("rating").and_then(J::as_f64) {
        parts.push(format!("rating {r}"));
    }
    parts.extend(count("/scoreStats/score", "score"));
    (!parts.is_empty()).then(|| parts.join(" · "))
}

/// The `File:…` title for a Wikimedia Commons file page, else `None`.
fn commons_file(url: &str) -> Option<String> {
    let (origin, title) = mediawiki(url)?;
    (origin == "https://commons.wikimedia.org" && title.starts_with("File:")).then_some(title)
}

/// "By <author> · <date>" from a Commons file's `extmetadata`, either part
/// optional. Values are HTML fragments, so tags are stripped.
async fn commons_credit(client: &reqwest::Client, file: &str) -> Option<String> {
    let enc = utf8_percent_encode(file, NON_ALPHANUMERIC).to_string();
    let api = format!(
        "https://commons.wikimedia.org/w/api.php?action=query&prop=imageinfo\
         &iiprop=extmetadata&format=json&titles={enc}"
    );
    let j = get_json(client, &api).await?;
    let pages = j.pointer("/query/pages")?.as_object()?;
    let meta = pages
        .values()
        .find_map(|p| p.pointer("/imageinfo/0/extmetadata"))?;
    let field = |k: &str| {
        meta.get(k)
            .and_then(|v| v.get("value"))
            .and_then(J::as_str)
            .and_then(|s| clean(&strip_tags(s)))
    };
    let author = field("Artist");
    let date = field("DateTimeOriginal").or_else(|| field("DateTime"));
    match (author, date) {
        (Some(a), Some(d)) => Some(format!("By {a} · {d}")),
        (Some(a), None) => Some(format!("By {a}")),
        (None, Some(d)) => Some(d),
        (None, None) => None,
    }
}

/// Drop HTML tags and collapse the result (Commons `extmetadata` values are HTML).
fn strip_tags(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut in_tag = false;
    for c in s.chars() {
        match c {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => out.push(c),
            _ => {}
        }
    }
    out
}

/// YouTube video title via oEmbed — no API key required.
async fn youtube_title(client: &reqwest::Client, url: &str) -> Option<String> {
    let enc = utf8_percent_encode(url, NON_ALPHANUMERIC).to_string();
    let api = format!("https://www.youtube.com/oembed?url={enc}&format=json");
    let j = get_json(client, &api).await?;
    clean(j.get("title")?.as_str()?)
}

/// The lead paragraph of a MediaWiki page via the action API. Tries the usual
/// `/w/api.php` script path first, then the bare `/api.php` some wikis use.
async fn mediawiki_extract(client: &reqwest::Client, origin: &str, title: &str) -> Option<String> {
    let enc = utf8_percent_encode(title, NON_ALPHANUMERIC).to_string();
    for path in ["/w/api.php", "/api.php"] {
        let api = format!(
            "{origin}{path}?action=query&prop=extracts&exintro=1&explaintext=1\
             &exsentences=2&redirects=1&format=json&titles={enc}"
        );
        let Some(j) = get_json(client, &api).await else { continue };
        if let Some(pages) = j.pointer("/query/pages").and_then(J::as_object) {
            for (_, page) in pages {
                if let Some(text) = page.get("extract").and_then(J::as_str) {
                    if let Some(c) = clean(text) {
                        return Some(c);
                    }
                }
            }
        }
    }
    None
}

async fn get_json(client: &reqwest::Client, url: &str) -> Option<J> {
    client
        .get(url)
        .send()
        .await
        .and_then(|r| r.error_for_status())
        .ok()?
        .json::<J>()
        .await
        .ok()
}

/// Collapse whitespace and cap length so the tooltip stays a short intro.
fn clean(s: &str) -> Option<String> {
    let mut out = s.split_whitespace().collect::<Vec<_>>().join(" ");
    if out.is_empty() {
        return None;
    }
    if out.chars().count() > 300 {
        out = out.chars().take(297).collect::<String>().trim_end().to_string();
        out.push('…');
    }
    Some(out)
}

/// Splice a CommonMark link title onto every occurrence of `url` in `body`,
/// whether it's an autolink (`<url>`) or an inline link (`[text](url)`). The
/// destination is angle-bracketed so a URL containing `()` (e.g. a wiki page
/// like `One_Bad_Day_(Allies)`) still parses.
fn add_title(body: &mut String, url: &str, title: &str) {
    let t = escape_title(title);
    // Autolink first: the inline rewrite below inserts a `<url>` of its own, so
    // doing it first would let this pass wrap it a second time.
    *body = body.replace(&format!("<{url}>"), &format!("[{url}](<{url}> \"{t}\")"));
    *body = body.replace(&format!("]({url})"), &format!("](<{url}> \"{t}\")"));
}

/// Escape a string for a double-quoted CommonMark link title.
fn escape_title(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_mediawiki_pages() {
        assert_eq!(
            mediawiki("https://en.wikipedia.org/wiki/Rust_(programming_language)"),
            Some(("https://en.wikipedia.org".into(), "Rust (programming language)".into()))
        );
        assert_eq!(
            mediawiki("https://homm.miraheze.org/wiki/One_Bad_Day_(Allies)"),
            Some(("https://homm.miraheze.org".into(), "One Bad Day (Allies)".into()))
        );
        assert_eq!(mediawiki("https://example.com/blog/post"), None);
    }

    #[test]
    fn detects_habr_user() {
        assert_eq!(
            habr_user("https://habr.com/en/users/zdanevich-vitaly/").as_deref(),
            Some("zdanevich-vitaly")
        );
        assert_eq!(habr_user("https://habr.com/ru/users/foo/posts/").as_deref(), Some("foo"));
        assert_eq!(habr_user("https://habr.com/en/articles/123/"), None);
        assert_eq!(habr_user("https://example.com/users/x/"), None);
    }

    #[test]
    fn detects_youtube() {
        assert!(is_youtube("https://www.youtube.com/watch?v=abc"));
        assert!(is_youtube("https://youtu.be/abc"));
        assert!(!is_youtube("https://vimeo.com/1"));
    }

    #[test]
    fn splices_titles_into_both_link_forms() {
        let url = "https://homm.miraheze.org/wiki/One_Bad_Day_(Allies)";
        let mut auto = format!("see <{url}> ok");
        add_title(&mut auto, url, "A scenario.");
        assert_eq!(auto, format!("see [{url}](<{url}> \"A scenario.\") ok"));

        let mut inline = format!("see [here]({url}) ok");
        add_title(&mut inline, url, r#"He said "hi""#);
        assert_eq!(inline, format!("see [here](<{url}> \"He said \\\"hi\\\"\") ok"));
    }
}
