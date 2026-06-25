//! Scaffold the Zola site (config + templates) and reconcile generated posts on
//! every run. Config/templates are regenerated deterministically; when a theme
//! is configured the built-in templates are removed so the theme drives the
//! look. Media lives in the post bundles — the bundle (committed to the `blog`
//! branch) is the cache.

use anyhow::{Context, Result};
use std::collections::HashSet;
use std::fs;
use std::path::Path;

use crate::config::Settings;
use crate::model::ChannelInfo;
use crate::render::RenderedPost;

/// (Re)write config + content indexes, and either install our built-in
/// templates (default) or clear them so an external theme takes over.
pub fn scaffold(s: &Settings, info: Option<&ChannelInfo>, tags: &[(String, usize)]) -> Result<()> {
    let site = &s.site;
    fs::create_dir_all(site.join("templates/shortcodes"))?;
    fs::create_dir_all(site.join("content/posts"))?;
    fs::create_dir_all(site.join("static"))?;
    fs::create_dir_all(site.join("themes"))?;

    let pages = parse_pages(s.pages.as_deref());

    // Remove stray top-level content files from older layouts (About and custom
    // pages now live under content/pages/), which would otherwise collide.
    if let Ok(entries) = fs::read_dir(site.join("content")) {
        for e in entries.flatten() {
            let p = e.path();
            let is_md = p.extension().is_some_and(|x| x == "md");
            let keep = p.file_name().and_then(|n| n.to_str()) == Some("_index.md");
            if is_md && !keep {
                let _ = fs::remove_file(p);
            }
        }
    }

    // Regenerated every run (deterministic) so theme switches take effect.
    write_file(&site.join("config.toml"), &config_toml(s, &pages, tags))?;
    write_file(&site.join("content/_index.md"), &root_index_md(s))?;
    write_file(&site.join("content/posts/_index.md"), &posts_index_md(s))?;
    // Static pages live in a non-rendered subsection so they don't appear in the
    // homepage post feed; `path` keeps them at /about/, /<slug>/.
    fs::create_dir_all(site.join("content/pages"))?;
    write_file(&site.join("content/pages/_index.md"), "+++\nrender = false\n+++\n")?;
    write_file(&site.join("content/pages/about.md"), &about_md(s, info))?;
    for p in &pages {
        write_file(&site.join(format!("content/pages/{}.md", p.slug)), &page_md(s, p))?;
    }
    // Always provide our YouTube shortcode (project shortcodes override the
    // theme's), so generated `{{ youtube(...) }}` always resolves.
    write_file(&site.join("templates/shortcodes/youtube.html"), YOUTUBE_SHORTCODE)?;
    write_file(&site.join("templates/shortcodes/video.html"), VIDEO_SHORTCODE)?;
    write_file(&site.join("templates/shortcodes/audio.html"), AUDIO_SHORTCODE)?;
    write_file(&site.join("templates/shortcodes/tag.html"), TAG_SHORTCODE)?;

    let builtins = [
        ("templates/base.html", BASE_HTML),
        ("templates/index.html", INDEX_HTML),
        ("templates/section.html", SECTION_HTML),
        ("templates/page.html", PAGE_HTML),
        ("templates/tags/single.html", TAGS_SINGLE),
        ("templates/tags/list.html", TAGS_LIST),
    ];
    if s.theme.is_none() {
        fs::create_dir_all(site.join("templates/tags"))?;
        for (path, content) in builtins {
            write_file(&site.join(path), content)?;
        }
        write_file(&site.join("static/style.css"), &style_css(s))?;
    } else {
        // Theme mode: remove our built-ins so they don't shadow the theme.
        for (path, _) in builtins {
            let _ = fs::remove_file(site.join(path));
        }
        let _ = fs::remove_dir_all(site.join("templates/tags"));
        let _ = fs::remove_file(site.join("static/style.css"));
    }
    Ok(())
}

/// Reconcile `content/posts`: (re)write every `index.md`, keep already-present
/// media (the cache), and prune posts that no longer exist plus stale media
/// left from a previous version of a post. Media is downloaded afterwards.
pub fn write_site(s: &Settings, rendered: &[RenderedPost]) -> Result<()> {
    let posts_dir = s.site.join("content/posts");
    fs::create_dir_all(&posts_dir)?;
    write_if_absent(&posts_dir.join("_index.md"), &posts_index_md(s))?;

    // Prune post directories that no longer exist on the channel.
    let current: HashSet<&str> = rendered.iter().map(|r| r.slug.as_str()).collect();
    for entry in fs::read_dir(&posts_dir)? {
        let entry = entry?;
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if name == "_index.md" {
            continue;
        }
        if entry.path().is_dir() && !current.contains(name.as_ref()) {
            fs::remove_dir_all(entry.path())?;
        }
    }

    for r in rendered {
        let dir = posts_dir.join(&r.slug);
        fs::create_dir_all(&dir)?;
        fs::write(dir.join("index.md"), &r.index_md)
            .with_context(|| format!("writing {}", dir.join("index.md").display()))?;

        // Keep index.md + the media this post currently references; remove the
        // rest (e.g. media left over from a previous edit of the post).
        let mut keep: HashSet<String> = r.downloads.iter().map(|d| d.filename.clone()).collect();
        keep.insert("index.md".to_string());
        for e in fs::read_dir(&dir)? {
            let e = e?;
            let n = e.file_name().to_string_lossy().into_owned();
            if !keep.contains(&n) {
                let _ = fs::remove_file(e.path());
            }
        }
    }
    tracing::info!("wrote {} post bundles", rendered.len());
    Ok(())
}

fn write_file(path: &Path, contents: &str) -> Result<()> {
    if let Some(p) = path.parent() {
        fs::create_dir_all(p)?;
    }
    fs::write(path, contents).with_context(|| format!("writing {}", path.display()))
}

fn write_if_absent(path: &Path, contents: &str) -> Result<()> {
    if path.exists() {
        return Ok(());
    }
    write_file(path, contents)
}

fn config_toml(s: &Settings, pages: &[Page], tags: &[(String, usize)]) -> String {
    let theme_line = match &s.theme {
        Some(t) => format!("theme = \"{}\"\n", toml_escape(t)),
        None => String::new(),
    };
    // Zola's built-in RSS 2.0 feed; cap it so a huge archive stays a sane size.
    let feeds = if s.rss {
        "generate_feeds = true\nfeed_filenames = [\"rss.xml\"]\nfeed_limit = 50"
    } else {
        "generate_feeds = false"
    };
    let tags_toml = if tags.is_empty() {
        String::new()
    } else {
        let items: Vec<String> = tags
            .iter()
            .map(|(n, c)| format!("{{ name = \"{}\", count = {} }}", toml_escape(n), c))
            .collect();
        format!("tags = [{}]", items.join(", "))
    };
    let avatar = if s.site.join("static/channel-avatar.jpg").exists() {
        "avatar = \"channel-avatar.jpg\""
    } else {
        ""
    };
    let nav = if pages.is_empty() {
        String::new()
    } else {
        let items: Vec<String> = pages
            .iter()
            .map(|p| {
                format!(
                    "{{ title = \"{}\", path = \"/{}/\" }}",
                    toml_escape(&p.title),
                    p.slug
                )
            })
            .collect();
        format!("nav = [{}]", items.join(", "))
    };
    CONFIG_TOML
        .replace("__BASE_URL__", &toml_escape(&s.base_url))
        .replace("__TITLE__", &toml_escape(&s.title))
        .replace("__DESC__", &toml_escape(&s.description))
        .replace("__THEME__", &theme_line)
        .replace("__FEEDS__", feeds)
        .replace("__RSS__", if s.rss { "true" } else { "false" })
        .replace("__CHANNEL__", &toml_escape(&s.channel))
        .replace("__TAGS_FOOTER__", if s.tags_footer { "true" } else { "false" })
        .replace("__NEXT_PREV__", if s.next_prev { "true" } else { "false" })
        .replace(
            "__TELEGRAM_LINK__",
            if s.telegram_link { "true" } else { "false" },
        )
        .replace("__AVATAR__", avatar)
        .replace("__NAV__", &nav)
        .replace("__TAGS__", &tags_toml)
}

/// Homepage = the paginated full-posts feed (posts bubble up from the
/// `transparent` posts section). Zola emits a `page/1/` redirect stub with a
/// <script>; the offline pass strips it so the output stays JS-free.
fn root_index_md(s: &Settings) -> String {
    let mut o = format!("+++\nsort_by = \"date\"\npaginate_by = {}\n", s.posts_per_page);
    if s.theme.is_none() {
        o.push_str("template = \"index.html\"\n");
    }
    o.push_str("+++\n");
    o
}

/// About page. The channel link is built from the configured channel (never
/// hardcoded), so the project works for any channel; the repo link is
/// configurable. In theme mode the template line is omitted.
fn about_md(s: &Settings, info: Option<&ChannelInfo>) -> String {
    let template = if s.theme.is_none() {
        "template = \"page.html\"\n"
    } else {
        ""
    };
    let body = match &s.about {
        // Custom HTML (from --about / ABOUT) becomes the page body verbatim.
        Some(html) => html.trim().to_string(),
        None => {
            let mut b = format!(
                "A self-contained static backup of the public Telegram channel \
**[@{ch}](https://t.me/{ch})**.\n\n",
                ch = s.channel,
            );
            if let Some(info) = info {
                if let Some(desc) = &info.description_md {
                    b.push_str(desc.trim());
                    b.push_str("\n\n");
                }
                if !info.counters.is_empty() {
                    let stats: Vec<String> = info
                        .counters
                        .iter()
                        .map(|(v, t)| format!("**{v}** {t}"))
                        .collect();
                    b.push_str(&stats.join(" · "));
                    b.push_str("\n\n");
                }
            }
            b.push_str(&format!(
                "Source repository: [{repo}]({repo})\n\n\
Generated by [tg2zola](https://github.com/vitaly-zdanevich/telegram_channel_to_static_website), \
an open-source tool that turns a public Telegram channel into a static website by reading the \
public web preview — **no Telegram bot, token, or API is needed**. All media is downloaded \
locally and the site has no Telegram dependency at runtime, so it keeps working even if the \
channel is removed.",
                repo = s.repo_url,
            ));
            b
        }
    };
    format!("+++\ntitle = \"About\"\npath = \"about\"\n{template}+++\n\n{body}\n")
}

/// One extra page parsed from the `PAGES` input.
struct Page {
    title: String,
    slug: String,
    body: String,
}

/// Split the `pages` input into pages at each top-level `# Title` heading.
fn parse_pages(src: Option<&str>) -> Vec<Page> {
    let src = match src {
        Some(s) if !s.trim().is_empty() => s,
        _ => return Vec::new(),
    };
    let mut pages = Vec::new();
    let mut cur: Option<(String, String)> = None;
    let flush = |cur: &mut Option<(String, String)>, pages: &mut Vec<Page>| {
        if let Some((title, body)) = cur.take() {
            let slug = slugify(&title);
            if !slug.is_empty() {
                pages.push(Page {
                    title,
                    slug,
                    body: body.trim().to_string(),
                });
            }
        }
    };
    for line in src.lines() {
        if let Some(title) = line.strip_prefix("# ") {
            flush(&mut cur, &mut pages);
            cur = Some((title.trim().to_string(), String::new()));
        } else if let Some((_, body)) = cur.as_mut() {
            body.push_str(line);
            body.push('\n');
        }
    }
    flush(&mut cur, &mut pages);
    pages
}

fn slugify(s: &str) -> String {
    let mut out = String::new();
    let mut dash = false;
    for c in s.chars() {
        if c.is_alphanumeric() {
            out.extend(c.to_lowercase());
            dash = false;
        } else if !out.is_empty() && !dash {
            out.push('-');
            dash = true;
        }
    }
    out.trim_matches('-').to_string()
}

fn page_md(s: &Settings, p: &Page) -> String {
    let template = if s.theme.is_none() {
        "template = \"page.html\"\n"
    } else {
        ""
    };
    format!(
        "+++\ntitle = \"{}\"\npath = \"{}\"\n{}+++\n\n{}\n",
        toml_escape(&p.title),
        p.slug,
        template,
        p.body
    )
}

/// The posts live here but are `transparent`, so they appear in the homepage
/// feed while keeping their `/posts/<id>/` URLs.
fn posts_index_md(s: &Settings) -> String {
    let mut o = String::from("+++\ntransparent = true\nrender = false\nsort_by = \"date\"\n");
    if s.theme.is_none() {
        o.push_str("page_template = \"page.html\"\n");
    }
    o.push_str("+++\n");
    o
}

fn style_css(s: &Settings) -> String {
    // Strip characters that could break out of the CSS value.
    let clean = |c: &str| -> String {
        c.chars()
            .filter(|ch| !matches!(ch, ';' | '{' | '}' | '"' | '\n' | '\r'))
            .collect()
    };
    let mut css = STYLE_CSS
        .replace("__BG_LIGHT__", &clean(&s.background_light))
        .replace("__BG_DARK__", &clean(&s.background_dark));
    if let Some(extra) = &s.css {
        css.push_str("\n/* custom CSS (from --css / CSS) */\n");
        css.push_str(extra.trim());
        css.push('\n');
    }
    css
}

fn toml_escape(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

const CONFIG_TOML: &str = r#"# Generated by tg2zola — regenerated every run. Configure via the tool, not here.
base_url = "__BASE_URL__"
title = "__TITLE__"
description = "__DESC__"
default_language = "en"
__THEME____FEEDS__
compile_sass = false
build_search_index = false

taxonomies = [
  { name = "tags" },
]

[markdown]
render_emoji = false

[extra]
generator = "tg2zola"
channel = "__CHANNEL__"
tags_footer = __TAGS_FOOTER__
next_prev = __NEXT_PREV__
telegram_link = __TELEGRAM_LINK__
rss = __RSS__
__AVATAR__
__NAV__
__TAGS__
"#;

const BASE_HTML: &str = r#"<!DOCTYPE html>
<html lang="{{ config.default_language }}">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>{% block title %}{{ config.title }}{% endblock title %}</title>
  {% if config.extra.avatar %}<link rel="icon" type="image/jpeg" href="{{ get_url(path=config.extra.avatar) }}">{% endif %}
  {% if config.extra.rss %}<link rel="alternate" type="application/rss+xml" title="{{ config.title }}" href="{{ get_url(path='rss.xml', trailing_slash=false) | safe }}">{% endif %}
  <link rel="stylesheet" href="{{ get_url(path='style.css', cachebust=true) }}">
</head>
<body>
  <header class="site-header">
    {% if config.extra.avatar %}<a href="{{ config.base_url | safe }}"><img class="site-avatar" src="{{ get_url(path=config.extra.avatar) }}" alt=""></a>{% endif %}
    <a class="site-title" href="{{ config.base_url | safe }}">{{ config.title }}</a>
    <nav>
      <a href="{{ get_url(path='/tags/') }}">Tags</a>
      <a href="{{ get_url(path='/about/') }}">About</a>
      {% for p in config.extra.nav | default(value=[]) %}<a href="{{ get_url(path=p.path) }}">{{ p.title }}</a>{% endfor %}
    </nav>
  </header>
  <main>{% block content %}{% endblock content %}</main>
  <footer class="site-footer">
    Telegram sync by
    <a href="https://github.com/vitaly-zdanevich/telegram_channel_to_static_website">tg2zola</a>.
  </footer>
</body>
</html>
"#;

const INDEX_HTML: &str = r#"{% extends "base.html" %}
{% block content %}
  {% if config.description and paginator.current_index == 1 %}<p class="lead">{{ config.description }}</p>{% endif %}
  {% for page in paginator.pages %}
    <article class="post">
      <h2 class="post-title"><a href="{{ page.permalink | safe }}">{{ page.title }}</a></h2>
      <p class="meta">
        <time datetime="{{ page.date }}" title="{{ page.date | date(format='%Y-%m-%d %H:%M') }}">{{ page.date | date(format="%Y-%m-%d") }}</time>
        {% if page.extra.views %}· 👁 {{ page.extra.views }}{% endif %}
        {% if page.extra.forwarded_from %}· forwarded from {% if page.extra.forwarded_from_url %}<a href="{{ page.extra.forwarded_from_url }}">{{ page.extra.forwarded_from }}</a>{% else %}{{ page.extra.forwarded_from }}{% endif %}{% endif %}
      </p>
      <div class="content">{{ page.content | safe }}</div>
      {% if config.extra.tags_footer and page.taxonomies.tags %}
        <p class="tags">{% for t in page.taxonomies.tags %}<a href="{{ get_taxonomy_url(kind='tags', name=t) }}">#{{ t }}</a> {% endfor %}</p>
      {% endif %}
    </article>
  {% endfor %}
  <nav class="pager">
    {% if paginator.previous %}<a href="{{ paginator.previous | safe }}">← Newer</a>{% else %}<span></span>{% endif %}
    <span>{{ paginator.current_index }} / {{ paginator.number_pagers }}</span>
    {% if paginator.next %}<a href="{{ paginator.next | safe }}">Older →</a>{% else %}<span></span>{% endif %}
  </nav>
{% endblock content %}
"#;

const SECTION_HTML: &str = r#"{% extends "base.html" %}
{% block title %}Archive · {{ config.title }}{% endblock title %}
{% block content %}
  <h1>Archive</h1>
  <ul class="post-list">
  {% for page in paginator.pages %}
    <li>
      <a href="{{ page.permalink | safe }}">{{ page.title }}</a>
      <time datetime="{{ page.date }}" title="{{ page.date | date(format='%Y-%m-%d %H:%M') }}">{{ page.date | date(format="%Y-%m-%d") }}</time>
      {% if page.extra.views %}<span class="views">👁 {{ page.extra.views }}</span>{% endif %}
    </li>
  {% endfor %}
  </ul>
  <nav class="pager">
    {% if paginator.previous %}<a href="{{ paginator.previous | safe }}">← Newer</a>{% else %}<span></span>{% endif %}
    <span>{{ paginator.current_index }} / {{ paginator.number_pagers }}</span>
    {% if paginator.next %}<a href="{{ paginator.next | safe }}">Older →</a>{% else %}<span></span>{% endif %}
  </nav>
{% endblock content %}
"#;

const PAGE_HTML: &str = r#"{% extends "base.html" %}
{% block title %}{{ page.title }} · {{ config.title }}{% endblock title %}
{% block content %}
  <article class="post">
    <h1>{{ page.title }}</h1>
    <p class="meta">
      {% if page.date %}<time datetime="{{ page.date }}">{{ page.date | date(format="%Y-%m-%d %H:%M") }}</time>{% endif %}
      {% if page.extra.views %}· 👁 {{ page.extra.views }} views{% endif %}
      {% if page.extra.forwarded_from %}· forwarded from {% if page.extra.forwarded_from_url %}<a href="{{ page.extra.forwarded_from_url }}">{{ page.extra.forwarded_from }}</a>{% else %}{{ page.extra.forwarded_from }}{% endif %}{% endif %}
    </p>
    <div class="content">{{ page.content | safe }}</div>
    {% if config.extra.tags_footer and page.taxonomies.tags %}
      <p class="tags">
      {% for t in page.taxonomies.tags %}
        <a href="{{ get_taxonomy_url(kind='tags', name=t) }}">#{{ t }}</a>
      {% endfor %}
      </p>
    {% endif %}
    {% if config.extra.telegram_link and page.extra.tg_url %}<p class="tg-link"><a href="{{ page.extra.tg_url }}">View on Telegram ↗</a></p>{% endif %}
  </article>
  {% if config.extra.next_prev %}
  <nav class="post-nav">
    <span>{% if page.extra.next_id %}<a href="{{ get_url(path='/posts/' ~ page.extra.next_id ~ '/') | safe }}" title="{{ page.extra.next_title }}">← Newer</a>{% endif %}</span>
    <span>{% if page.extra.prev_id %}<a href="{{ get_url(path='/posts/' ~ page.extra.prev_id ~ '/') | safe }}" title="{{ page.extra.prev_title }}">Older →</a>{% endif %}</span>
  </nav>
  {% endif %}
{% endblock content %}
"#;

const TAGS_SINGLE: &str = r#"{% extends "base.html" %}
{% block title %}#{{ term.name }} · {{ config.title }}{% endblock title %}
{% block content %}
  <h1>#{{ term.name }}</h1>
  <ul class="post-list">
  {% for page in term.pages %}
    <li>
      <a href="{{ page.permalink | safe }}">{{ page.title }}</a>
      <time datetime="{{ page.date }}" title="{{ page.date | date(format='%Y-%m-%d %H:%M') }}">{{ page.date | date(format="%Y-%m-%d") }}</time>
    </li>
  {% endfor %}
  </ul>
{% endblock content %}
"#;

const TAGS_LIST: &str = r#"{% extends "base.html" %}
{% block title %}Tags · {{ config.title }}{% endblock title %}
{% block content %}
  <h1>Tags</h1>
  <ul class="tag-cloud">
  {% for t in config.extra.tags | default(value=[]) %}
    <li><a href="{{ get_taxonomy_url(kind='tags', name=t.name) | safe }}">#{{ t.name }}</a> <span>({{ t.count }})</span></li>
  {% endfor %}
  </ul>
{% endblock content %}
"#;

// Self-hosted YouTube shortcode (Zola no longer ships one). Uses the
// privacy-friendly nocookie host. Invoked from Markdown as {{ youtube(id="...") }}.
const YOUTUBE_SHORTCODE: &str = r#"<div class="yt-embed">
  <iframe src="https://www.youtube-nocookie.com/embed/{{ id }}"
    title="YouTube video" loading="lazy"
    allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture; web-share"
    referrerpolicy="strict-origin-when-cross-origin" allowfullscreen></iframe>
</div>
"#;

// Resolve colocated media against the post's permalink so it works both on the
// post page and when the post is shown in full on the homepage feed (a relative
// src would otherwise break off the post's own page).
const VIDEO_SHORTCODE: &str =
    "<video controls preload=\"metadata\" src=\"{{ page.permalink | safe }}{{ src }}\"></video>\n";
const AUDIO_SHORTCODE: &str =
    "<audio controls src=\"{{ page.permalink | safe }}{{ src }}\"></audio>\n";

// Inline clickable hashtag → its taxonomy page (base_url-aware).
const TAG_SHORTCODE: &str =
    "<a class=\"tag\" href=\"{{ get_taxonomy_url(kind='tags', name=t) | safe }}\">#{{ t }}</a>";

// Built-in look: light by default, true-black #000 in dark mode (OLED-friendly),
// following the OS via prefers-color-scheme.
const STYLE_CSS: &str = r#":root {
  color-scheme: light dark;
  --bg: __BG_LIGHT__; --fg: #1a1a1a; --muted: #6b6b6b;
  --link: #0a58ca; --border: #e6e6e6; --code-bg: #f4f4f4;
}
@media (prefers-color-scheme: dark) {
  :root {
    --bg: __BG_DARK__; --fg: #e6e6e6; --muted: #8a8a8a;
    --link: #6cb6ff; --border: #1c1c1c; --code-bg: #0d0d0d;
  }
}
* { box-sizing: border-box; }
body {
  width: 95vw; margin: 0 auto; padding: 1rem;
  font-family: system-ui, -apple-system, sans-serif; line-height: 1.6;
  background: var(--bg); color: var(--fg);
}
a, a:visited { color: var(--link); }
::selection {
    background-color: #292;
    color: #000;
}
img, video { max-width: 100%; height: auto; border-radius: 6px; }
audio { width: 100%; }
.yt-embed { position: relative; aspect-ratio: 16 / 9; margin: 1rem 0; }
.yt-embed iframe { position: absolute; inset: 0; width: 100%; height: 100%; border: 0; border-radius: 6px; }
.yt-link { font-size: .9em; margin-top: .25rem; }
.tag { white-space: nowrap; }
pre { background: var(--code-bg); padding: .75rem; border-radius: 4px; overflow-x: auto; }
code { background: var(--code-bg); padding: .1rem .3rem; border-radius: 4px; }
pre code { padding: 0; background: none; }
.site-header { display: flex; gap: .6rem; align-items: center; border-bottom: 1px solid var(--border); padding-bottom: .5rem; margin-bottom: 1rem; flex-wrap: wrap; }
.site-avatar { width: 32px; height: 32px; border-radius: 50%; object-fit: cover; display: block; }
.site-title { font-weight: 700; text-decoration: none; color: var(--fg); }
.site-header nav a { margin-left: .75rem; }
.post-list { list-style: none; padding: 0; }
.post-list li { padding: .2rem 0; }
.post-list time { color: var(--muted); font-size: .85em; margin-left: .5rem; }
.post { margin: 0 0 2.5rem; }
.post-title { margin: 0 0 .25rem; font-size: 1.25rem; }
.more { margin: 2rem 0; }
.views, .meta { color: var(--muted); font-size: .85em; }
.meta { font-size: .9em; }
.tags a { margin-right: .5rem; white-space: nowrap; }
.pager { display: flex; justify-content: space-between; margin: 2rem 0; }
.post-nav { display: flex; justify-content: space-between; gap: 1rem; margin: 1.5rem 0; border-top: 1px solid var(--border); padding-top: 1rem; }
.tg-link { margin-top: 1rem; font-size: .9em; }
.spoiler { background: var(--fg); border-radius: 3px; transition: background .1s; }
.spoiler:hover { background: transparent; }
.site-footer { margin-top: 3rem; color: var(--muted); font-size: .8rem; border-top: 1px solid var(--border); padding-top: .5rem; }
blockquote { border-left: 3px solid var(--border); margin: .5rem 0; padding-left: .75rem; color: var(--muted); }
"#;
