//! Turn a [`Post`] into a Zola page bundle: an `index.md` (TOML front matter +
//! Markdown body) and the list of media files to place alongside it.
//!
//! The output references **only local files and YouTube** — never `t.me`.

use once_cell::sync::Lazy;
use regex::Regex;
use std::collections::{HashMap, HashSet};

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
    /// When set, the file already exists locally (fetched via MTProto) and is
    /// copied from here instead of fetched over HTTP (`url` is then empty).
    pub local: Option<std::path::PathBuf>,
    /// Stage this file for upload to GitHub Releases (kept out of the published
    /// bundle) instead of the page bundle — used for videos when `video_releases`
    /// is on, so they don't count against the Pages quota.
    pub release: bool,
}

pub struct RenderedPost {
    pub slug: String,
    pub title: String,
    pub index_md: String,
    pub downloads: Vec<Download>,
}

/// True if the post's first non-empty line is the lone marker `PAGE` (so it
/// should become a standalone page rather than a feed post).
pub fn is_page(post: &Post) -> bool {
    post.body_md.lines().map(str::trim).find(|l| !l.is_empty()) == Some("PAGE")
}

/// True if the post would render to nothing worth keeping: no text/tags, no
/// YouTube, and no real media — only a non-archived attachment reference, or
/// media the public page dropped (e.g. a lone non-downloadable music file).
pub fn is_empty_post(post: &Post) -> bool {
    post.body_md.trim().is_empty()
        && post.youtube.is_none()
        && !post
            .media
            .iter()
            .any(|m| !matches!(m, Media::DocumentRef { .. }))
}

/// Drop the first non-empty line (the `PAGE` marker) from a body.
fn strip_page_marker(body: &str) -> String {
    let mut removed = false;
    body.lines()
        .filter(|l| {
            if !removed && !l.trim().is_empty() {
                removed = true;
                false
            } else {
                true
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
        .trim_start()
        .to_string()
}

/// Slug (and content-dir name) for a post: `YYYY-MM-DD-<id>`. Zola strips the
/// date prefix to produce the permalink `/posts/<id>/`.
pub fn slug_for(post: &Post) -> String {
    format!("{}-{}", post.date.format("%Y-%m-%d"), post.primary_id)
}

/// For each post, the top `n` related posts ranked by how many tags they share
/// (the *whole* tag set, not a single tag). An inverted index keeps it near
/// linear. Ties break toward a tighter match (candidate with fewer tags) then
/// recency. A post with no tags — or no tag-sharing neighbour — gets none.
pub fn compute_related(posts: &[Post], n: usize) -> Vec<Vec<(String, String)>> {
    use std::collections::HashMap;
    let mut by_tag: HashMap<&str, Vec<usize>> = HashMap::new();
    for (i, p) in posts.iter().enumerate() {
        for t in &p.tags {
            by_tag.entry(t.as_str()).or_default().push(i);
        }
    }
    posts
        .iter()
        .enumerate()
        .map(|(i, p)| {
            if p.tags.is_empty() {
                return Vec::new();
            }
            let mut shared: HashMap<usize, u32> = HashMap::new();
            for t in &p.tags {
                for &j in &by_tag[t.as_str()] {
                    if j != i {
                        *shared.entry(j).or_default() += 1;
                    }
                }
            }
            let mut ranked: Vec<(usize, u32)> = shared.into_iter().collect();
            ranked.sort_by(|&(ja, sa), &(jb, sb)| {
                sb.cmp(&sa)
                    .then(posts[ja].tags.len().cmp(&posts[jb].tags.len()))
                    .then(posts[jb].primary_id.cmp(&posts[ja].primary_id))
            });
            ranked
                .into_iter()
                .take(n)
                .map(|(j, _)| (slug_for(&posts[j]), related_label(&posts[j])))
                .collect()
        })
        .collect()
}

/// A short, link-safe label for a related post: its first line of text with
/// Markdown-active characters dropped, or its `#id` when it's media-only.
fn related_label(post: &Post) -> String {
    let clean: String = post_preview(post)
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .chars()
        .filter(|c| !matches!(c, '[' | ']' | '(' | ')' | '*' | '_' | '`' | '<' | '>' | '|'))
        .collect();
    if clean.is_empty() {
        return format!("#{}", post.primary_id);
    }
    if clean.chars().count() > 64 {
        format!("{}…", clean.chars().take(64).collect::<String>().trim_end())
    } else {
        clean
    }
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
pub fn post_title(post: &Post, title_max: usize, derive: bool) -> String {
    if !derive {
        return String::new();
    }
    if let Some((name, _)) = wiki_title_for(post, title_max) {
        return name;
    }
    title_and_body(post, title_max, true, false).0
}

/// The wiki page title to use for this post, but only when it has no prose
/// title of its own — i.e. the title would otherwise come from the wiki link
/// line, or there's no prose line at all. A post that merely *links* to a wiki
/// page (while having its own text) keeps its own title.
fn wiki_title_for(post: &Post, max: usize) -> Option<(String, String)> {
    let wiki = wikimedia_commons_title(&post.links)?;
    let from_wiki_line = match title_from_body(&post.body_md, max) {
        None => true,
        Some((_, idx, _, _)) => post
            .body_md
            .lines()
            .nth(idx)
            .is_some_and(contains_wiki_domain),
    };
    from_wiki_line.then_some(wiki)
}

/// Wikimedia-family domains whose `/wiki/Name` or `?title=Name` pages we turn
/// into a readable title (Wikipedia, Commons, Wiktionary, Wikidata, …).
const WIKI_DOMAINS: [&str; 6] = [
    "wikipedia.org",
    "wikimedia.org",
    "wiktionary.org",
    "wikidata.org",
    "wikisource.org",
    "mediawiki.org",
];

fn contains_wiki_domain(s: &str) -> bool {
    let s = s.to_lowercase();
    WIKI_DOMAINS.iter().any(|d| s.contains(d))
}

/// If the post links to a Wikimedia/Wikipedia page, derive a readable title from
/// that page name. Returns `(name, url)`.
fn wikimedia_commons_title(links: &[String]) -> Option<(String, String)> {
    links.iter().find_map(|u| {
        if !contains_wiki_domain(u) {
            return None;
        }
        commons_page_name(u).map(|name| (name, u.clone()))
    })
}

/// Decode a MediaWiki page name from either URL form — `?title=Name` or the
/// pretty `/wiki/Name` path — into a readable title: percent-decoded, namespace
/// prefix (`Category:`/`File:` …) dropped, `_` → space.
fn commons_page_name(url: &str) -> Option<String> {
    let parsed = url::Url::parse(url).ok()?;
    let raw = parsed
        .query_pairs()
        .find(|(k, _)| k == "title")
        .map(|(_, v)| v.into_owned())
        .or_else(|| {
            let seg = parsed.path().strip_prefix("/wiki/").filter(|s| !s.is_empty())?;
            Some(
                percent_encoding::percent_decode_str(seg)
                    .decode_utf8_lossy()
                    .into_owned(),
            )
        })?;
    let name = strip_wiki_namespace(&raw.replace('_', " ")).trim().to_string();
    (!name.is_empty()).then_some(name)
}

fn strip_wiki_namespace(s: &str) -> &str {
    if let Some((ns, rest)) = s.split_once(':') {
        if matches!(
            ns.to_lowercase().as_str(),
            "category" | "file" | "template" | "creator" | "institution" | "gallery"
        ) {
            return rest;
        }
    }
    s
}

/// Body for a Wikimedia Commons post: the page link (readable name), then the
/// rest of the message with its original (URL-mangled) line dropped. Parens in
/// the URL are encoded so the Markdown link isn't cut short.
fn wiki_body(body: &str, text: &str, url: &str) -> String {
    let href = url.replace('(', "%28").replace(')', "%29");
    let rest: Vec<&str> = body
        .lines()
        .filter(|l| !contains_wiki_domain(l))
        .collect();
    format!("[{text}]({href})\n\n{}", rest.join("\n").trim())
        .trim_end()
        .to_string()
}

/// Per-run rendering options shared by every post — grouped so `render_post`'s
/// signature stays small.
#[derive(Clone, Copy)]
pub struct RenderOpts<'a> {
    pub ui: &'a crate::i18n::Ui,
    pub title_max: usize,
    pub derive_titles: bool,
    pub strip_title: bool,
    pub keep_media: bool,
    /// Replace a Spotify link with the Spotify player (opt-in).
    pub spotify: bool,
    /// Embed a live Instagram post replacing an attached video (opt-in).
    pub instagram: bool,
    /// Replace a Pinterest pin link with the embedded pin (default on).
    pub pinterest: bool,
    /// When set, videos are offloaded to GitHub Releases: the base download URL
    /// (`…/releases/download/<tag>`) that each video's filename is appended to,
    /// and the file is staged for upload instead of bundled.
    pub video_releases: Option<&'a str>,
    /// Show a photo-album post (2+ images, nothing else) as a swipeable carousel.
    pub carousel: bool,
}

pub fn render_post(
    post: &Post,
    links: &LinkRewriter,
    page: bool,
    newer: Option<(u64, &str, &str)>,
    older: Option<(u64, &str, &str)>,
    opts: &RenderOpts,
) -> RenderedPost {
    let RenderOpts {
        ui,
        title_max,
        derive_titles,
        strip_title,
        keep_media,
        spotify,
        instagram,
        pinterest,
        video_releases,
        carousel,
    } = *opts;
    // A PAGE-marked post becomes a standalone page; work on a copy with the
    // marker line removed and use a plain first-sentence title.
    let page_post = page.then(|| {
        let mut p = post.clone();
        p.body_md = strip_page_marker(&post.body_md);
        p
    });
    let post = page_post.as_ref().unwrap_or(post);

    // A Wikimedia/Wikipedia page link gives a readable title + clean body link;
    // otherwise derive the title from the body.
    let (title, body_src) = if page {
        title_and_body(post, title_max, true, true)
    } else if !derive_titles {
        // Default: no derived title — the post is identified by its #id. Keep the
        // body intact, only dropping a forwarded post's trailing source backlink
        // (already shown by the "forwarded from" header).
        let mut b = post.body_md.clone();
        if let Some(fwd) = &post.forwarded_from {
            b = strip_forward_backlink(&b, fwd);
        }
        (String::new(), b)
    } else {
        match wiki_title_for(post, title_max) {
            Some((name, url)) => (name.clone(), wiki_body(&post.body_md, &name, &url)),
            None => {
                let (t, mut b) = title_and_body(post, title_max, true, strip_title);
                // A forwarded post often ends with a link back to the source
                // channel; the "forwarded from" header already shows it, drop it.
                if let Some(fwd) = &post.forwarded_from {
                    b = strip_forward_backlink(&b, fwd);
                }
                (t, b)
            }
        }
    };

    // Opt-in Spotify / default Pinterest link → embed (replacing the link).
    let spotify_embed = (spotify && !post.spotify_dead)
        .then(|| post.spotify.clone())
        .flatten();
    let pinterest_embed = (pinterest && !post.pinterest_dead)
        .then(|| post.pinterest.clone())
        .flatten();

    // Drop the standalone link(s) an embed replaces, so nothing shows twice.
    let mut body_src = body_src;
    if post.apple_podcast.is_some() {
        body_src = strip_apple_links(&body_src);
    }
    if spotify_embed.is_some() {
        body_src = strip_platform_links(&body_src, &SPOTIFY_LINK);
    }
    if pinterest_embed.is_some() {
        body_src = strip_platform_links(&body_src, &PINTEREST_LINK);
    }
    // Bandcamp player replacing its album/track link (default on; resolved only
    // when enabled, so presence of `post.bandcamp` is the gate).
    if post.bandcamp.is_some() {
        body_src = strip_platform_links(&body_src, &BANDCAMP_LINK);
    }
    // VK playlist widget replacing its link (opt-in; populated only when enabled).
    if post.vk_playlist.is_some() {
        body_src = strip_platform_links(&body_src, &VK_LINK);
    }

    let mut downloads = Vec::new();
    let mut body = String::new();
    let mut idx = 0usize;
    // Bundle filenames already taken (so original audio names stay unique).
    let mut used_names: HashSet<String> = HashSet::new();
    // First image in the post → og:image for social/Mastodon link previews.
    let mut og_image: Option<String> = None;

    if !body_src.trim().is_empty() {
        let text = links.rewrite(&body_src);
        body.push_str(text.trim());
        body.push_str("\n\n");
    }

    // Per spec, a YouTube link only *replaces an attached video*. On posts with
    // no video the YouTube URL stays as an ordinary link in the text.
    let has_video = post.media.iter().any(|m| {
        matches!(
            m,
            Media::Video { .. } | Media::VideoPoster { .. } | Media::LocalVideo { .. }
        )
    });
    // A YouTube link normally *replaces* the attached video. With `keep_media`
    // we download and show the video anyway (the YouTube embed stays too).
    // A *live* YouTube link replaces the attached video; a removed one (liveness
    // check) keeps the local video instead. keep_media keeps it regardless.
    let youtube_live = post.youtube.is_some() && !post.youtube_dead;
    let apple_live = post.apple_podcast.is_some() && !post.apple_dead;
    // A live Instagram post replaces an attached *video* (opt-in embed).
    let instagram_live = instagram && post.instagram.is_some() && !post.instagram_dead;
    // Yandex Music replaces an *attached audio* file only (per the rule), and
    // only when the track is still live.
    let has_attached_audio = post.media.iter().any(|m| match m {
        Media::Audio { .. } | Media::LocalAudio { .. } => true,
        Media::Document { filename, .. } | Media::DocumentRef { filename } => {
            crate::media::is_probably_audio_doc(filename)
        }
        _ => false,
    });
    let yandex_replace = post.yandex_music.is_some() && !post.yandex_dead && has_attached_audio;
    let drop_videos = has_video && (youtube_live || instagram_live) && !keep_media;
    // A live YouTube / Apple Podcasts / Yandex Music link replaces attached
    // *audio* unless keep_media is set; a removed one keeps the local audio.
    let drop_audio = (youtube_live || apple_live || yandex_replace) && !keep_media;
    // Embed a live YouTube link for ANY post (not only video posts); a removed
    // video shows no (broken) embed, and its local media is kept above.
    if let Some(yt) = &post.youtube {
        if !post.youtube_dead {
            body.push_str(&format!("{{{{ youtube(id=\"{yt}\") }}}}\n\n"));
        } else if post.youtube_watchable && !has_video {
            // Plays on YouTube but embedding is disabled, and there's no local
            // video to fall back on — link out with a thumbnail facade.
            body.push_str(&format!("{{{{ youtube_link(id=\"{yt}\") }}}}\n\n"));
        }
    }
    // Apple Podcasts episode embed (an <iframe>; over file:// it degrades to the
    // "Listen on Apple Podcasts" link the shortcode also renders).
    if let Some(url) = &post.apple_podcast {
        if !post.apple_dead {
            body.push_str(&format!("{{{{ apple_podcast(url=\"{url}\") }}}}\n\n"));
        }
    }
    // Yandex Music track embed replacing an attached audio file (a live track).
    if yandex_replace {
        if let Some(url) = &post.yandex_music {
            body.push_str(&format!("{{{{ yandex_music(url=\"{url}\") }}}}\n\n"));
        }
    }
    // Instagram post embed replacing an attached video (a live post).
    if has_video && instagram_live {
        if let Some(url) = &post.instagram {
            body.push_str(&format!("{{{{ instagram(url=\"{url}\") }}}}\n\n"));
        }
    }
    // Spotify player replacing its link (opt-in).
    if let Some(url) = &spotify_embed {
        body.push_str(&format!("{{{{ spotify(url=\"{url}\") }}}}\n\n"));
    }
    // Pinterest embedded pin replacing its link (default on).
    if let Some(url) = &pinterest_embed {
        body.push_str(&format!("{{{{ pinterest(url=\"{url}\") }}}}\n\n"));
    }
    // Bandcamp player replacing its link (default on).
    if let Some(url) = &post.bandcamp {
        body.push_str(&format!("{{{{ bandcamp(url=\"{url}\") }}}}\n\n"));
    }
    // VK playlist widget replacing its link (opt-in). The widget is login/region
    // gated, so the shortcode also renders a fallback "Open on VK" link.
    if let Some(url) = &post.vk_playlist {
        if let Some((owner, id, key)) = crate::media::vk_playlist_parts(url) {
            let elid = format!("vk_pl_{}_{}", owner.replace('-', "n"), id);
            body.push_str(&format!(
                "{{{{ vk_playlist(elid=\"{elid}\", owner=\"{owner}\", id=\"{id}\", key=\"{key}\", url=\"{url}\") }}}}\n\n"
            ));
        }
    }

    // Photo-album carousel (opt-in): when a post's *media* is 2+ images and no
    // video/audio/documents, show them as one swipeable widget instead of a
    // vertical stack. The post's text and tags render normally, above.
    let carousel_album = carousel
        && post.media.len() >= 2
        && post.media.iter().all(|m| {
            matches!(m, Media::Photo { .. } | Media::Sticker { .. } | Media::LocalPhoto { .. })
        });
    if carousel_album {
        let mut imgs = String::new();
        for (i, m) in post.media.iter().enumerate() {
            let n = i + 1;
            let (fname, dl) = match m {
                Media::Photo { url, key } | Media::Sticker { url, key } => {
                    let (fname, force) = media_name(key, url, "jpg", n, post.edited);
                    let dl = Download {
                        url: url.clone(),
                        filename: fname.clone(),
                        force,
                        local: None,
                        release: false,
                    };
                    (fname, dl)
                }
                Media::LocalPhoto { path, key } => {
                    let ext = path.extension().and_then(|e| e.to_str()).filter(|e| !e.is_empty()).unwrap_or("jpg");
                    let fname = match key {
                        Some(k) => format!("{}.{ext}", sanitize_key(k)),
                        None => format!("{n:02}.{ext}"),
                    };
                    let dl = Download {
                        url: String::new(),
                        filename: fname.clone(),
                        force: true,
                        local: Some(path.clone()),
                        release: false,
                    };
                    (fname, dl)
                }
                _ => unreachable!("carousel_album is all-images"),
            };
            downloads.push(dl);
            if og_image.is_none() {
                og_image = Some(fname.clone());
            }
            imgs.push_str(&format!("<img src=\"{fname}\" loading=\"lazy\">"));
        }
        body.push_str(&format!(
            "<div class=\"carousel\"><div class=\"carousel-track\">{imgs}</div>\
             <button class=\"carousel-prev\" type=\"button\" aria-label=\"Previous\">‹</button>\
             <button class=\"carousel-next\" type=\"button\" aria-label=\"Next\">›</button></div>\n\n"
        ));
    }

    for m in &post.media {
        if carousel_album {
            break; // the album was rendered as the carousel above
        }
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
                let ext = ext_from_url(url, "mp4");
                if let Some(base) = video_releases {
                    // Offload to GitHub Releases (kept off the Pages quota): stage
                    // the file under a globally-unique name, link to the release URL.
                    let fname = format!("{}-{idx:02}.{ext}", post.primary_id);
                    downloads.push(Download {
                        url: url.to_string(),
                        filename: fname.clone(),
                        force: post.edited,
                        local: None,
                        release: true,
                    });
                    body.push_str(&format!("{{{{ video_ext(url=\"{base}/{fname}\") }}}}\n\n"));
                } else {
                    let fname = format!("{idx:02}.{ext}");
                    push_dl(&mut downloads, url, &fname, post.edited);
                    body.push_str(&format!("{{{{ video(src=\"{fname}\") }}}}\n\n"));
                }
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
                    format!("▶ {}", ui.video)
                } else {
                    format!("▶ {} — {dur}", ui.video)
                };
                body.push_str(&format!("*{label}*\n\n"));
            }
            Media::Audio { url, title } => {
                if drop_audio {
                    continue;
                }
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
            Media::DocumentRef { filename } => {
                // A YouTube/Apple Podcasts embed stands in for a podcast track —
                // drop the redundant "(not archived)" note for that audio.
                if drop_audio && crate::media::is_probably_audio_doc(filename) {
                    continue;
                }
                // The file isn't on the public page; note the attachment + name.
                body.push_str(&format!(
                    "📎 {} *({})*\n\n",
                    label_escape(filename),
                    ui.not_archived
                ));
            }
            Media::LocalAudio { path, name, title } => {
                if drop_audio {
                    continue;
                }
                // Audio from MTProto: copy the cached file in under its original
                // filename (or positional), then render the player with a label.
                let ext = path
                    .extension()
                    .and_then(|e| e.to_str())
                    .filter(|e| !e.is_empty())
                    .unwrap_or("ogg");
                let fname = local_audio_name(name.as_deref(), ext, idx, &mut used_names);
                downloads.push(Download {
                    url: String::new(),
                    filename: fname.clone(),
                    force: false,
                    local: Some(path.clone()),
                    release: false,
                });
                if let Some(t) = title {
                    if !t.is_empty() {
                        body.push_str(&format!("*{}*\n\n", label_escape(t)));
                    }
                }
                body.push_str(&format!("{{{{ audio(src=\"{fname}\") }}}}\n\n"));
            }
            Media::LocalPhoto { path, key } => {
                // Original-quality photo from MTProto, replacing the web Photo.
                // Keep the content-addressed name (`key`) so the bundle file the
                // web body referenced is overwritten in place.
                let ext = path
                    .extension()
                    .and_then(|e| e.to_str())
                    .filter(|e| !e.is_empty())
                    .unwrap_or("jpg");
                let fname = match key {
                    Some(k) => format!("{}.{ext}", sanitize_key(k)),
                    None => format!("{idx:02}.{ext}"),
                };
                downloads.push(Download {
                    url: String::new(),
                    filename: fname.clone(),
                    // Overwrite the smaller web photo cached from an earlier run.
                    force: true,
                    local: Some(path.clone()),
                    release: false,
                });
                if og_image.is_none() {
                    og_image = Some(fname.clone());
                }
                body.push_str(&format!("![]({fname})\n\n"));
            }
            Media::LocalVideo { path } => {
                // Full video from MTProto, replacing the web's poster-only
                // placeholder. A live YouTube/Instagram embed still replaces it,
                // the same drop_videos rule as Media::Video (enrich already skips
                // the download in that case unless keep_media, so this is belt +
                // braces).
                if drop_videos {
                    continue;
                }
                let ext = path
                    .extension()
                    .and_then(|e| e.to_str())
                    .filter(|e| !e.is_empty())
                    .unwrap_or("mp4");
                if let Some(base) = video_releases {
                    // Offload to GitHub Releases, same as Media::Video.
                    let fname = format!("{}-{idx:02}.{ext}", post.primary_id);
                    downloads.push(Download {
                        url: String::new(),
                        filename: fname.clone(),
                        force: false,
                        local: Some(path.clone()),
                        release: true,
                    });
                    body.push_str(&format!("{{{{ video_ext(url=\"{base}/{fname}\") }}}}\n\n"));
                } else {
                    let fname = format!("{idx:02}.{ext}");
                    downloads.push(Download {
                        url: String::new(),
                        filename: fname.clone(),
                        force: false,
                        local: Some(path.clone()),
                        release: false,
                    });
                    body.push_str(&format!("{{{{ video(src=\"{fname}\") }}}}\n\n"));
                }
            }
            Media::LocalDocument { path, name } => {
                // Any attachment fetched via MTProto (pdf, zip, …): copy the local
                // file into the bundle under a safe name and link to it.
                let ext = path
                    .extension()
                    .and_then(|e| e.to_str())
                    .filter(|e| !e.is_empty())
                    .unwrap_or("bin");
                let fname = sanitize_filename(name, ext, idx);
                downloads.push(Download {
                    url: String::new(),
                    filename: fname.clone(),
                    force: false,
                    local: Some(path.clone()),
                    release: false,
                });
                body.push_str(&format!("[📎 {}]({fname})\n\n", label_escape(name)));
            }
        }
    }

    // Reaction counts (MTProto only) as a subtle footer line.
    if !post.reactions.is_empty() {
        let line = post
            .reactions
            .iter()
            .map(|(emoji, count)| format!("{emoji} {count}"))
            .collect::<Vec<_>>()
            .join(" · ");
        body.push_str(&format!("{line}\n\n"));
    }

    // Auto #video tag in the body for posts with a playable video, unless the
    // author already wrote it (matches the taxonomy tag added in main).
    let has_video = post
        .media
        .iter()
        .any(|m| matches!(m, Media::Video { .. } | Media::LocalVideo { .. }));
    if has_video && !body.contains("tag(t=\"video\")") {
        body.push_str("{{ tag(t=\"video\") }}\n\n");
    }

    // Genius lyrics widget — only when the post carries no lyrics of its own (no
    // blockquote). It's JavaScript (Genius has no static embed), so the offline
    // pass strips it, leaving just the fallback link.
    if let Some(song) = &post.genius_song_id {
        let has_quote = post.body_md.lines().any(|l| l.trim_start().starts_with('>'));
        if !has_quote {
            let url = post
                .links
                .iter()
                .find(|l| l.contains("genius.com"))
                .map(String::as_str)
                .unwrap_or("https://genius.com");
            body.push_str(&format!(
                "<div class=\"rg_embed_link\" data-song-id=\"{song}\">\
<a href=\"{url}\">Lyrics on Genius</a></div>\
<script crossorigin src=\"//genius.com/songs/{song}/embed.js\"></script>\n\n"
            ));
        }
    }

    // Wikidata statement tables for any wikidata.org link in the post (raw HTML,
    // already spoiler-wrapped upstream when enabled). Static — survives offline.
    for table in &post.wikidata_html {
        body.push_str(table);
        body.push_str("\n\n");
    }

    // Related posts (opt-in), by shared-tag overlap. Internal @/ links resolve
    // under any base_url and survive the channel's removal.
    if !post.related.is_empty() {
        body.push_str(&format!("<nav class=\"related\">\n\n**{}:**\n\n", ui.related));
        for (slug, label) in &post.related {
            body.push_str(&format!("- [{label}](@/posts/{slug}/index.md)\n"));
        }
        body.push_str("\n</nav>\n\n");
    }

    let (slug, front) = if page {
        let slug = crate::site::slugify(&title);
        let front = format!(
            "+++\ntitle = {}\npath = \"/{slug}/\"\ntemplate = \"page.html\"\n+++\n\n",
            toml_str(&title)
        );
        (slug, front)
    } else {
        let description = excerpt(&body_src, 200);
        let front = front_matter(post, &title, &description, og_image.as_deref(), newer, older);
        (slug_for(post), front)
    };
    let index_md = format!("{}{}\n", front, body.trim_end());
    RenderedPost {
        slug,
        title,
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
        local: None,
        release: false,
    });
}

fn front_matter(
    post: &Post,
    title: &str,
    description: &str,
    og_image: Option<&str>,
    newer: Option<(u64, &str, &str)>,
    older: Option<(u64, &str, &str)>,
) -> String {
    let mut fm = String::from("+++\n");
    fm.push_str(&format!("title = {}\n", toml_str(title)));
    // RFC3339 is a valid TOML offset date-time literal (unquoted).
    fm.push_str(&format!("date = {}\n", post.date.to_rfc3339()));
    // Plain-text excerpt → <meta description> + og:/twitter: descriptions.
    if !description.is_empty() {
        fm.push_str(&format!("description = {}\n", toml_str(description)));
    }

    // Every post joins a `days` taxonomy term (its date) so the /day/<date>/
    // pages can render that day in full; tags are an additional taxonomy.
    fm.push_str("\n[taxonomies]\n");
    fm.push_str(&format!(
        "days = [{}]\n",
        toml_str(&post.date.format("%Y-%m-%d").to_string())
    ));
    if !post.tags.is_empty() {
        let tags: Vec<String> = post.tags.iter().map(|t| toml_str(t)).collect();
        fm.push_str(&format!("tags = [{}]\n", tags.join(", ")));
    }

    fm.push_str("\n[extra]\n");
    fm.push_str(&format!("id = {}\n", post.primary_id));
    fm.push_str(&format!(
        "day = {}\n",
        toml_str(&post.date.format("%Y-%m-%d").to_string())
    ));
    fm.push_str(&format!(
        "tg_url = \"https://t.me/{}/{}\"\n",
        post.channel, post.primary_id
    ));
    if let Some(img) = og_image {
        fm.push_str(&format!("og_image = {}\n", toml_str(img)));
    }
    if let Some((id, t, body)) = newer {
        fm.push_str(&format!("next_id = {id}\n"));
        fm.push_str(&format!("next_title = {}\n", toml_str(t)));
        fm.push_str(&format!("next_body = {}\n", toml_str(body)));
    }
    if let Some((id, t, body)) = older {
        fm.push_str(&format!("prev_id = {id}\n"));
        fm.push_str(&format!("prev_title = {}\n", toml_str(t)));
        fm.push_str(&format!("prev_body = {}\n", toml_str(body)));
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
static BR_TAG: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?i)<br\s*/?>").unwrap());
static SHORTCODE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\{\{[^}]*\}\}").unwrap());
static IMG_MD: Lazy<Regex> = Lazy::new(|| Regex::new(r"!\[[^\]]*\]\([^)]*\)").unwrap());
static MD_LINK_LABEL: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\[([^\]]*)\]\([^)]*\)").unwrap());

/// Strip a Markdown body to plain text: keep hashtag words + link text, drop
/// shortcodes, images, URLs and markup. Line breaks are left intact; the caller
/// normalizes whitespace.
fn strip_md(md: &str, hashtags: bool) -> String {
    // `#music` keeps hashtags recognizable (tooltip); the meta description drops
    // the `#` for cleaner prose.
    let mut s = TAG_SC
        .replace_all(md, if hashtags { "#$1" } else { "$1" })
        .to_string();
    s = SHORTCODE.replace_all(&s, " ").to_string();
    s = IMG_MD.replace_all(&s, " ").to_string();
    s = MD_LINK_LABEL.replace_all(&s, "$1").to_string();
    s = AUTOLINK.replace_all(&s, " ").to_string();
    s = BARE_URL.replace_all(&s, " ").to_string();
    // `<br>` (blockquote internal breaks) → newline, before other tags are dropped.
    s = BR_TAG.replace_all(&s, "\n").to_string();
    s = HTML_TAG.replace_all(&s, " ").to_string();
    if hashtags {
        s.replace(['*', '_', '`', '~', '>', '\\'], "")
    } else {
        s.replace(['*', '_', '`', '~', '#', '>', '\\'], "")
    }
}

/// A plain-text excerpt (single line) for the meta/OG/Twitter description.
fn excerpt(md: &str, max: usize) -> String {
    let joined = strip_md(md, false)
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    truncate_chars(joined.trim(), max)
}

/// A post's body as normalized plain text: markdown stripped (hashtag words
/// kept), each line's inner whitespace collapsed, blank lines dropped. Uncapped
/// — used for the daily-run largest-files log, which prints the whole post.
pub fn post_text_plain(post: &Post) -> String {
    strip_md(&post.body_md, true)
        .lines()
        .map(|l| l.split_whitespace().collect::<Vec<_>>().join(" "))
        .filter(|l| !l.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

/// Plain-text preview of a post's body for the Newer/Older hover `title`: line
/// breaks kept (intra-line whitespace collapsed, blank lines dropped), capped so
/// a long post doesn't produce a giant tooltip.
pub fn post_preview(post: &Post) -> String {
    // Most posts are short (shown in full); only a very long one is trimmed.
    truncate_chars(post_text_plain(post).trim(), 1000)
}

/// The post title plus the body to render. The title is the first body line
/// carrying real prose (keeping hashtag words, dropping `#`); falls back to the
/// tags, then the date. When that line is *pure prose* (no hashtags) it is cut
/// from the body so the headline isn't shown twice — but a line that also
/// carries hashtags is kept, so its clickable tags survive.
fn title_and_body(post: &Post, max: usize, derive: bool, strip: bool) -> (String, String) {
    if !derive {
        return (String::new(), post.body_md.clone());
    }
    if let Some((title, idx, had_tags, partial)) = title_from_body(&post.body_md, max) {
        // With --strip-title, cut the headline line from the body so it isn't
        // shown twice — but only when it's pure prose (a line that also carries
        // hashtags, or only part of which became the title, is kept so no text or
        // clickable tag is lost).
        let body = if strip && !had_tags && !partial {
            remove_line(&post.body_md, idx)
        } else {
            post.body_md.clone()
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
        t = IMG_MD.replace_all(&t, "").to_string();
        // Keep a link's text in the title; drop only its URL.
        t = MD_LINK_LABEL.replace_all(&t, "$1").to_string();
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
        // Keep the line in the body when it carries a link: the title drops
        // links, so removing the line would lose it.
        let has_link =
            MD_LINK.is_match(raw) || AUTOLINK.is_match(raw) || BARE_URL.is_match(raw);
        return Some((title, idx, TAG_SC.is_match(raw), partial || has_link));
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
            && chars.get(i + 1).is_none_or(|c| c.is_whitespace())
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

static APPLE_LINK: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"\[[^\]]*\]\(\s*<?https?://(?:embed\.)?podcasts\.apple\.com/[^)\s]*>?\s*\)|<?https?://(?:embed\.)?podcasts\.apple\.com/[^\s>)\]]*>?",
    )
    .unwrap()
});

/// Remove Apple Podcasts links from the body (the embed replaces them): a line
/// that is *only* such a link is dropped; a line with other prose keeps the prose
/// with the link removed. Non-apple lines (incl. blank paragraph breaks) are left
/// untouched.
fn strip_apple_links(body: &str) -> String {
    let mut out: Vec<String> = Vec::new();
    for l in body.lines() {
        if l.contains("podcasts.apple.com") {
            let stripped = APPLE_LINK.replace_all(l, "").trim().to_string();
            if !stripped.is_empty() {
                out.push(stripped);
            }
        } else {
            out.push(l.to_string());
        }
    }
    out.join("\n")
}

static SPOTIFY_LINK: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"\[[^\]]*\]\(\s*<?https?://open\.spotify\.com/[^)\s]*>?\s*\)|<?https?://open\.spotify\.com/[^\s>)\]]*>?",
    )
    .unwrap()
});

static PINTEREST_LINK: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"\[[^\]]*\]\(\s*<?https?://(?:[a-z]+\.)?pinterest\.[a-z.]+/pin/[^)\s]*>?\s*\)|<?https?://(?:[a-z]+\.)?pinterest\.[a-z.]+/pin/[^\s>)\]]*>?",
    )
    .unwrap()
});

static BANDCAMP_LINK: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"\[[^\]]*\]\(\s*<?https?://[a-z0-9-]+\.bandcamp\.com/(?:album|track)/[^)\s]*>?\s*\)|<?https?://[a-z0-9-]+\.bandcamp\.com/(?:album|track)/[^\s>)\]]*>?",
    )
    .unwrap()
});

static VK_LINK: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"\[[^\]]*\]\(\s*<?https?://(?:www\.)?vk\.com/(?:music/(?:playlist|album)|audio_playlist)[^)\s]*>?\s*\)|<?https?://(?:www\.)?vk\.com/(?:music/(?:playlist|album)|audio_playlist)[^\s>)\]]*>?",
    )
    .unwrap()
});

/// Drop each `link`-matching URL from the body (keeping surrounding text), so an
/// embed can replace the plain link. Lines that become empty are dropped.
fn strip_platform_links(body: &str, link: &Regex) -> String {
    body.lines()
        .filter_map(|l| {
            if link.is_match(l) {
                let s = link.replace_all(l, "").trim().to_string();
                (!s.is_empty()).then_some(s)
            } else {
                Some(l.to_string())
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
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

/// Bundle filename for an MTProto audio track: the sanitized original filename
/// when present (kept unique within the bundle), else a positional name.
fn local_audio_name(orig: Option<&str>, ext: &str, idx: usize, used: &mut HashSet<String>) -> String {
    let base = match orig {
        Some(n) if !n.trim().is_empty() => sanitize_filename(n, ext, idx),
        _ => format!("{idx:02}.{ext}"),
    };
    if used.insert(base.clone()) {
        return base;
    }
    // Collision within the bundle → disambiguate with the positional index.
    let (stem, e) = base.rsplit_once('.').unwrap_or((base.as_str(), ext));
    let uniq = format!("{stem}-{idx:02}.{e}");
    used.insert(uniq.clone());
    uniq
}

fn sanitize_filename(name: &str, ext: &str, idx: usize) -> String {
    // Keep Unicode letters/digits and a little safe punctuation; collapse any run
    // of other characters (spaces, path separators, punctuation) into a single
    // '_' — so a non-ASCII name stays readable instead of a row of underscores.
    let mut base = String::with_capacity(name.len());
    let mut prev_sep = false;
    for c in name.chars() {
        if c.is_alphanumeric() || matches!(c, '-' | '_' | '.') {
            base.push(c);
            prev_sep = false;
        } else if !prev_sep {
            base.push('_');
            prev_sep = true;
        }
    }
    let base = base
        .trim_matches(|c: char| c == '_' || c == '.')
        .to_string();
    // Nothing usable left → positional name.
    if !base.chars().any(char::is_alphanumeric) {
        return format!("{idx:02}.{ext}");
    }
    // Keep an existing real extension, otherwise append the detected one.
    match base.rsplit_once('.') {
        Some((_, e))
            if (1..=5).contains(&e.chars().count()) && e.chars().all(|c| c.is_ascii_alphanumeric()) =>
        {
            base
        }
        _ => format!("{base}.{ext}"),
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

    #[test]
    fn commons_title_decoded() {
        let url = "https://commons.wikimedia.org/w/index.php?title=Category:Test_Page_%28x%29--01&action=edit&redlink=1";
        assert_eq!(commons_page_name(url).as_deref(), Some("Test Page (x)--01"));
    }

    fn post_with_body(body: &str) -> Post {
        use chrono::TimeZone;
        Post {
            primary_id: 1,
            ids: vec![1],
            channel: "c".into(),
            date: chrono::FixedOffset::east_opt(0)
                .unwrap()
                .timestamp_opt(1_700_000_000, 0)
                .unwrap(),
            author: None,
            forwarded_from: None,
            body_md: body.into(),
            tags: vec![],
            media: vec![],
            views: None,
            edited: false,
            reactions: vec![],
            bandcamp: None,
            vk_playlist: None,
            related: vec![],
            wikidata_html: vec![],
            links: vec![],
            youtube: None,
            apple_podcast: None,
            yandex_music: None,
            instagram: None,
            spotify: None,
            pinterest: None,
            youtube_dead: false,
            youtube_watchable: false,
            apple_dead: false,
            yandex_dead: false,
            instagram_dead: false,
            spotify_dead: false,
            pinterest_dead: false,
            genius_song_id: None,
        }
    }

    #[test]
    fn page_marker_makes_a_page() {
        assert!(is_page(&post_with_body("PAGE\nMy Cool Page\nMore text.")));
        assert!(!is_page(&post_with_body("Not a page")));
        let rw = LinkRewriter::with_index("c", HashMap::new());
        let r = render_post(
            &post_with_body("PAGE\nMy Cool Page\nMore text."),
            &rw,
            true,
            None,
            None,
            &RenderOpts {
                ui: &crate::i18n::ui("en"),
                title_max: 200,
                derive_titles: false,
                strip_title: false,
                keep_media: false,
                spotify: false,
                instagram: false,
                pinterest: false,
                video_releases: None,
                carousel: false,
            },
        );
        assert_eq!(r.title, "My Cool Page");
        assert!(r.index_md.contains("path = \"/my-cool-page/\""), "{}", r.index_md);
        assert!(r.index_md.contains("template = \"page.html\""), "{}", r.index_md);
        assert!(r.index_md.contains("More text."), "{}", r.index_md);
        assert!(!r.index_md.contains("PAGE"), "marker dropped: {}", r.index_md);
    }

    #[test]
    fn preview_blockquote_keeps_line_breaks() {
        // Blockquote internal breaks are <br>; the tooltip must keep them.
        let p = post_with_body("Intro\n\n> line one<br>line two<br>line three");
        let prev = post_preview(&p);
        assert!(prev.contains("line one\nline two\nline three"), "{prev:?}");
    }

    #[test]
    fn sanitize_keeps_unicode_names() {
        assert_eq!(
            sanitize_filename("1. Название эпизода.flac", "flac", 3),
            "1._Название_эпизода.flac"
        );
        assert_eq!(sanitize_filename("005-vitaly-geo.m4a", "m4a", 3), "005-vitaly-geo.m4a");
        assert_eq!(sanitize_filename("Эпизод", "ogg", 1), "Эпизод.ogg");
        // All-punctuation → positional, never a row of underscores.
        assert_eq!(sanitize_filename("!!! ??? …", "mp3", 7), "07.mp3");
    }

    #[test]
    fn commons_file_path_decoded() {
        // Pretty `/wiki/File:` URL form (percent-encoded, namespace dropped).
        let url = "https://commons.wikimedia.org/wiki/File:Slonim_%D1%81%D0%BD%D1%8F%D1%82%D0%BE.jpg";
        assert_eq!(commons_page_name(url).as_deref(), Some("Slonim снято.jpg"));
    }

    #[test]
    fn instagram_embed_replaces_live_video_only() {
        let rw = LinkRewriter::with_index("c", HashMap::new());
        let render = |dead: bool| {
            let mut p = post_with_body("watch this");
            p.media = vec![Media::Video { url: "https://cdn/x.mp4".into() }];
            p.instagram = Some("https://www.instagram.com/reel/ABC/".into());
            p.instagram_dead = dead;
            render_post(
                &p,
                &rw,
                false,
                None,
                None,
                &RenderOpts {
                    ui: &crate::i18n::ui("en"),
                    title_max: 200,
                    derive_titles: false,
                    strip_title: false,
                    keep_media: false,
                    spotify: false,
                    instagram: true,
                    pinterest: false,
                    video_releases: None,
                    carousel: false,
                },
            )
            .index_md
        };
        // Confirmed live → the Instagram embed replaces the attached video.
        let live = render(false);
        assert!(
            live.contains("{{ instagram(url=\"https://www.instagram.com/reel/ABC/\") }}"),
            "no embed: {live}"
        );
        assert!(!live.contains("{{ video("), "video not dropped: {live}");
        // Removed / unverified → keep the video, show no dead embed.
        let dead = render(true);
        assert!(dead.contains("{{ video("), "video dropped on dead IG: {dead}");
        assert!(!dead.contains("{{ instagram("), "embed on dead IG: {dead}");
    }

    #[test]
    fn youtube_link_facade_only_when_embed_disabled_and_no_local_video() {
        let rw = LinkRewriter::with_index("c", HashMap::new());
        let render = |dead: bool, watchable: bool, video: bool| {
            let mut p = post_with_body("song");
            p.youtube = Some("abc123XYZ".into());
            p.youtube_dead = dead;
            p.youtube_watchable = watchable;
            if video {
                p.media = vec![Media::Video { url: "https://cdn/x.mp4".into() }];
            }
            render_post(
                &p,
                &rw,
                false,
                None,
                None,
                &RenderOpts {
                    ui: &crate::i18n::ui("en"),
                    title_max: 200,
                    derive_titles: false,
                    strip_title: false,
                    keep_media: false,
                    spotify: false,
                    instagram: false,
                    pinterest: false,
                    video_releases: None,
                    carousel: false,
                },
            )
            .index_md
        };
        // Embeddable → the normal iframe embed, no facade.
        let live = render(false, false, false);
        assert!(live.contains("{{ youtube(id=\"abc123XYZ\") }}"), "no embed: {live}");
        assert!(!live.contains("youtube_link"), "unexpected facade: {live}");
        // Embedding disabled but still plays, no local video → link-out facade.
        let facade = render(true, true, false);
        assert!(facade.contains("{{ youtube_link(id=\"abc123XYZ\") }}"), "no facade: {facade}");
        assert!(!facade.contains("{{ youtube(id="), "dead embed shown: {facade}");
        // Embedding disabled but a local video exists → keep the video, no facade.
        let with_video = render(true, true, true);
        assert!(with_video.contains("{{ video("), "video dropped: {with_video}");
        assert!(!with_video.contains("youtube_link"), "facade despite local video: {with_video}");
        // Removed (not watchable) → neither a dead embed nor a facade.
        let removed = render(true, false, false);
        assert!(!removed.contains("youtube_link"), "facade on removed: {removed}");
        assert!(!removed.contains("{{ youtube(id="), "embed on removed: {removed}");
    }

    #[test]
    fn local_document_renders_download_link() {
        use std::path::{Path, PathBuf};
        let rw = LinkRewriter::with_index("c", HashMap::new());
        let mut p = post_with_body("here is a file");
        p.media = vec![Media::LocalDocument {
            path: PathBuf::from("/cache/12.zip"),
            name: "archive.zip".into(),
        }];
        let out = render_post(
            &p,
            &rw,
            false,
            None,
            None,
            &RenderOpts {
                ui: &crate::i18n::ui("en"),
                title_max: 200,
                derive_titles: false,
                strip_title: false,
                keep_media: false,
                spotify: false,
                instagram: false,
                pinterest: false,
                video_releases: None,
                carousel: false,
            },
        );
        // A 📎 download link plus one local-copy job for the bundled file.
        assert!(out.index_md.contains("📎 archive.zip"), "no download link: {}", out.index_md);
        assert_eq!(out.downloads.len(), 1, "expected one download job");
        assert_eq!(out.downloads[0].local.as_deref(), Some(Path::new("/cache/12.zip")));
    }

    #[test]
    fn video_offloaded_to_releases_when_enabled() {
        let rw = LinkRewriter::with_index("c", HashMap::new());
        let ui = crate::i18n::ui("en");
        let mut p = post_with_body("clip");
        p.media = vec![Media::Video { url: "https://cdn/x.mp4".into() }];
        let base = "https://github.com/o/r/releases/download/media";
        let common = RenderOpts {
            ui: &ui,
            title_max: 200,
            derive_titles: false,
            strip_title: false,
            keep_media: false,
            spotify: false,
            instagram: false,
            pinterest: false,
            video_releases: None,
            carousel: false,
        };
        // Off → inline <video>, bundled locally.
        let off = render_post(&p, &rw, false, None, None, &common);
        assert!(off.index_md.contains("{{ video(src="), "{}", off.index_md);
        assert!(off.downloads.iter().all(|d| !d.release), "nothing should be staged");
        // On → external video_ext with the release URL, staged for upload under a
        // globally-unique (post-id-prefixed) name.
        let on = render_post(
            &p,
            &rw,
            false,
            None,
            None,
            &RenderOpts { video_releases: Some(base), ..common },
        );
        assert!(on.index_md.contains(&format!("{{{{ video_ext(url=\"{base}/")), "{}", on.index_md);
        assert_eq!(on.downloads.len(), 1);
        assert!(on.downloads[0].release, "video should be staged for release");
        assert!(on.downloads[0].filename.starts_with(&format!("{}-", p.primary_id)), "{}", on.downloads[0].filename);
    }

    #[test]
    fn reactions_render_as_a_footer_line() {
        let rw = LinkRewriter::with_index("c", HashMap::new());
        let ui = crate::i18n::ui("en");
        let opts = RenderOpts {
            ui: &ui,
            title_max: 200,
            derive_titles: false,
            strip_title: false,
            keep_media: false,
            spotify: false,
            instagram: false,
            pinterest: false,
            video_releases: None,
            carousel: false,
        };
        let mut p = post_with_body("hi");
        p.reactions = vec![("👍".into(), 42), ("❤️".into(), 10)];
        let out = render_post(&p, &rw, false, None, None, &opts);
        assert!(out.index_md.contains("👍 42 · ❤️ 10"), "{}", out.index_md);
        // No reactions → no footer.
        let plain = render_post(&post_with_body("hi"), &rw, false, None, None, &opts);
        assert!(!plain.index_md.contains('👍'), "{}", plain.index_md);
    }

    #[test]
    fn carousel_wraps_multi_image_albums_when_enabled() {
        let rw = LinkRewriter::with_index("c", HashMap::new());
        let ui = crate::i18n::ui("en");
        let base = RenderOpts {
            ui: &ui,
            title_max: 200,
            derive_titles: false,
            strip_title: false,
            keep_media: false,
            spotify: false,
            instagram: false,
            pinterest: false,
            video_releases: None,
            carousel: false,
        };
        let mut album = post_with_body("album");
        album.media = vec![
            Media::Photo { url: "https://cdn/a.jpg".into(), key: Some("k1".into()) },
            Media::Photo { url: "https://cdn/b.jpg".into(), key: Some("k2".into()) },
        ];
        // Off → a normal image stack, no carousel.
        let off = render_post(&album, &rw, false, None, None, &base);
        assert!(!off.index_md.contains("carousel"), "{}", off.index_md);
        assert!(off.index_md.contains("!["), "expected markdown images: {}", off.index_md);
        // On → one carousel holding both images, and both are still downloaded.
        let on = render_post(&album, &rw, false, None, None, &RenderOpts { carousel: true, ..base });
        assert!(on.index_md.contains(r#"<div class="carousel">"#), "{}", on.index_md);
        assert_eq!(on.index_md.matches("<img").count(), 2, "{}", on.index_md);
        assert_eq!(on.downloads.len(), 2);
        // A single image is never a carousel, even enabled.
        let mut solo = post_with_body("solo");
        solo.media = vec![Media::Photo { url: "https://cdn/a.jpg".into(), key: None }];
        let out = render_post(&solo, &rw, false, None, None, &RenderOpts { carousel: true, ..base });
        assert!(!out.index_md.contains("carousel"), "{}", out.index_md);
    }

    #[test]
    fn mtproto_video_renders_and_respects_embed() {
        use std::path::PathBuf;
        let rw = LinkRewriter::with_index("c", HashMap::new());
        let render = |youtube: Option<&str>, keep: bool| {
            let mut p = post_with_body("clip");
            p.media = vec![Media::LocalVideo {
                path: PathBuf::from("/cache/1.mp4"),
            }];
            p.youtube = youtube.map(|s| s.to_string());
            render_post(
                &p,
                &rw,
                false,
                None,
                None,
                &RenderOpts {
                    ui: &crate::i18n::ui("en"),
                    title_max: 200,
                    derive_titles: false,
                    strip_title: false,
                    keep_media: keep,
                    spotify: false,
                    instagram: false,
                    pinterest: false,
                    video_releases: None,
                    carousel: false,
                },
            )
            .index_md
        };
        // No link → the fetched video is shown.
        assert!(render(None, false).contains("{{ video("), "video missing");
        // Live YouTube, keep_media off (CI) → the embed replaces the video.
        let r = render(Some("abc123"), false);
        assert!(!r.contains("{{ video("), "video not dropped for a live embed: {r}");
        assert!(r.contains("{{ youtube("), "no youtube embed: {r}");
        // Live YouTube but keep_media on (local) → keep the video too.
        assert!(
            render(Some("abc123"), true).contains("{{ video("),
            "video dropped despite keep_media"
        );
    }

    #[test]
    fn spotify_optin_and_pinterest_default() {
        let rw = LinkRewriter::with_index("c", HashMap::new());
        let mut p = post_with_body("music + pin");
        p.spotify = Some("https://open.spotify.com/embed/track/abc123".into());
        p.pinterest = Some("https://www.pinterest.com/pin/42/".into());
        let ui = crate::i18n::ui("en");
        let opts = |spotify, pinterest| RenderOpts {
            instagram: false,
            video_releases: None,
            carousel: false,
            ui: &ui,
            title_max: 200,
            derive_titles: false,
            strip_title: false,
            keep_media: false,
            spotify,
            pinterest,
        };
        // Pinterest default-on emits; Spotify (off) does not.
        let a = render_post(&p, &rw, false, None, None, &opts(false, true)).index_md;
        assert!(
            a.contains("{{ pinterest(url=\"https://www.pinterest.com/pin/42/\") }}"),
            "no pinterest: {a}"
        );
        assert!(!a.contains("{{ spotify("), "spotify emitted while off: {a}");
        // Spotify opt-in emits the embed URL; Pinterest (off) does not.
        let b = render_post(&p, &rw, false, None, None, &opts(true, false)).index_md;
        assert!(
            b.contains("{{ spotify(url=\"https://open.spotify.com/embed/track/abc123\") }}"),
            "no spotify: {b}"
        );
        assert!(!b.contains("{{ pinterest("), "pinterest emitted while off: {b}");
    }

    #[test]
    fn related_ranks_by_shared_tag_overlap() {
        let mk = |id: u64, tags: &[&str]| {
            let mut p = post_with_body(&format!("body of post {id}"));
            p.primary_id = id;
            p.tags = tags.iter().map(|s| s.to_string()).collect();
            p
        };
        // a shares 2 tags with b, 1 with c, 0 with d (untagged).
        let posts = vec![
            mk(1, &["x", "y", "z"]),
            mk(2, &["x", "y"]),
            mk(3, &["x"]),
            mk(4, &[]),
        ];
        let rel = compute_related(&posts, 5);
        // a's neighbours: b (2 shared) before c (1 shared); d excluded (0).
        assert_eq!(rel[0].len(), 2);
        assert_eq!(rel[0][0].0, slug_for(&posts[1]));
        assert_eq!(rel[0][1].0, slug_for(&posts[2]));
        // an untagged post has no related.
        assert!(rel[3].is_empty());

        // The section renders as a nav with internal @/ links.
        let mut a = posts[0].clone();
        a.related = rel[0].clone();
        let rw = LinkRewriter::with_index("c", HashMap::new());
        let ui = crate::i18n::ui("en");
        let opts = RenderOpts {
            instagram: false,
            video_releases: None,
            carousel: false,
            ui: &ui,
            title_max: 200,
            derive_titles: false,
            strip_title: false,
            keep_media: false,
            spotify: false,
            pinterest: false,
        };
        let out = render_post(&a, &rw, false, None, None, &opts).index_md;
        assert!(out.contains("<nav class=\"related\">"), "no related nav: {out}");
        assert!(out.contains(&format!("(@/posts/{}/index.md)", slug_for(&posts[1]))), "no link: {out}");
    }

    #[test]
    fn bandcamp_player_replaces_link() {
        let rw = LinkRewriter::with_index("c", HashMap::new());
        let mut p =
            post_with_body("great album <https://argonov.bandcamp.com/album/rethinking-progress>");
        let embed = "https://bandcamp.com/EmbeddedPlayer/album=3840020883/size=large/tracklist=false/artwork=small/transparent=true/";
        p.bandcamp = Some(embed.into());
        let ui = crate::i18n::ui("en");
        let opts = RenderOpts {
            instagram: false,
            video_releases: None,
            carousel: false,
            ui: &ui,
            title_max: 200,
            derive_titles: false,
            strip_title: false,
            keep_media: false,
            spotify: false,
            pinterest: false,
        };
        let out = render_post(&p, &rw, false, None, None, &opts).index_md;
        assert!(out.contains(&format!("{{{{ bandcamp(url=\"{embed}\") }}}}")), "no player: {out}");
        assert!(!out.contains("/album/rethinking-progress"), "link not stripped: {out}");
    }

    #[test]
    fn vk_playlist_widget_replaces_link() {
        let rw = LinkRewriter::with_index("c", HashMap::new());
        let mut p = post_with_body("mix <https://vk.com/music/album/-2000123_456_ab12cd>");
        p.vk_playlist = Some("https://vk.com/music/album/-2000123_456_ab12cd".into());
        let ui = crate::i18n::ui("en");
        let opts = RenderOpts {
            instagram: false,
            video_releases: None,
            carousel: false,
            ui: &ui,
            title_max: 200,
            derive_titles: false,
            strip_title: false,
            keep_media: false,
            spotify: false,
            pinterest: false,
        };
        let out = render_post(&p, &rw, false, None, None, &opts).index_md;
        assert!(
            out.contains("vk_playlist(elid=\"vk_pl_n2000123_456\", owner=\"-2000123\", id=\"456\", key=\"ab12cd\""),
            "no vk widget: {out}"
        );
        // The plain autolink is gone (the URL still appears in the shortcode's
        // fallback `url=` arg, which is expected).
        assert!(!out.contains("<https://vk.com"), "vk autolink not stripped: {out}");
    }
}
