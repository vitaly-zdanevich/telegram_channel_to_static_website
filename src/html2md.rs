//! Convert the inner HTML of a `.tgme_widget_message_text` node into Markdown,
//! preserving bold / italic / strikethrough / code / links, and lifting out
//! hashtags as taxonomy terms. Hashtag anchors point at `?q=%23tag` (a Telegram
//! search) — we render them as plain text, never as links, so the output has no
//! Telegram dependency.

use once_cell::sync::Lazy;
use regex::Regex;
use scraper::node::Node;
use scraper::ElementRef;

pub struct Converted {
    pub md: String,
    pub tags: Vec<String>,
    pub links: Vec<String>,
}

struct Ctx {
    tags: Vec<String>,
    links: Vec<String>,
}

pub fn convert(el: ElementRef) -> Converted {
    let mut ctx = Ctx {
        tags: Vec::new(),
        links: Vec::new(),
    };
    let mut s = String::new();
    walk_children(el, &mut s, &mut ctx);
    Converted {
        md: postprocess(&s),
        tags: dedup(ctx.tags),
        links: ctx.links,
    }
}

fn walk_children(parent: ElementRef, out: &mut String, ctx: &mut Ctx) {
    for child in parent.children() {
        match child.value() {
            Node::Text(t) => out.push_str(&escape_inline(t)),
            Node::Element(_) => {
                if let Some(e) = ElementRef::wrap(child) {
                    handle_element(e, out, ctx);
                }
            }
            _ => {}
        }
    }
}

fn inner(el: ElementRef, ctx: &mut Ctx) -> String {
    let mut s = String::new();
    walk_children(el, &mut s, ctx);
    s
}

/// Collect verbatim (entity-decoded) text, turning `<br>` into `repl`. Used for
/// code/pre, where the content is literal and Telegram writes line breaks as
/// `<br/>` rather than newlines.
fn raw_text(el: ElementRef, repl: &str) -> String {
    let mut s = String::new();
    raw_walk(el, &mut s, repl);
    s
}

fn raw_walk(parent: ElementRef, out: &mut String, repl: &str) {
    for child in parent.children() {
        match child.value() {
            Node::Text(t) => out.push_str(t),
            Node::Element(_) => {
                if let Some(e) = ElementRef::wrap(child) {
                    if e.value().name() == "br" {
                        out.push_str(repl);
                    } else {
                        raw_walk(e, out, repl);
                    }
                }
            }
            _ => {}
        }
    }
}

fn handle_element(el: ElementRef, out: &mut String, ctx: &mut Ctx) {
    let name = el.value().name();
    let class = el.value().attr("class").unwrap_or("");
    match name {
        "br" => out.push('\n'),
        "b" | "strong" => wrap(out, "**", &inner(el, ctx), "**"),
        "i" | "em" => {
            // Telegram custom emoji are `<i class="emoji">...</i>` — emit the
            // fallback glyph, not italics.
            if class.contains("emoji") {
                out.push_str(&inner(el, ctx));
            } else {
                wrap(out, "*", &inner(el, ctx), "*");
            }
        }
        "u" | "ins" => {
            out.push_str("<u>");
            out.push_str(&inner(el, ctx));
            out.push_str("</u>");
        }
        "s" | "strike" | "del" => wrap(out, "~~", &inner(el, ctx), "~~"),
        "code" => {
            let raw = raw_text(el, " ");
            out.push('`');
            out.push_str(&raw.trim().replace('`', "\\`"));
            out.push('`');
        }
        "pre" => {
            // Telegram encodes code line breaks as <br/>; preserve them.
            let raw = raw_text(el, "\n");
            out.push_str("\n\n```\n");
            out.push_str(raw.trim_matches('\n'));
            out.push_str("\n```\n\n");
        }
        "blockquote" => {
            // Markdown blockquote; preserve internal line breaks (e.g. lyrics)
            // with <br> since raw newlines collapse.
            let body = inner(el, ctx).trim().replace('\n', "<br>");
            out.push_str("\n> ");
            out.push_str(body.trim());
            out.push('\n');
        }
        "tg-spoiler" => spoiler(out, &inner(el, ctx)),
        "span" if class.contains("spoiler") => spoiler(out, &inner(el, ctx)),
        "a" => handle_anchor(el, out, ctx),
        // Unknown wrapper: just emit its children.
        _ => out.push_str(&inner(el, ctx)),
    }
}

fn handle_anchor(el: ElementRef, out: &mut String, ctx: &mut Ctx) {
    let href = el.value().attr("href").unwrap_or("");
    let raw_text = el.text().collect::<String>();
    let rt = raw_text.trim();

    // Hashtag search link -> taxonomy term + a clickable `tag` shortcode (which
    // resolves to the right base_url-aware /tags/<slug>/ URL).
    if href.starts_with("?q=") || href.contains("q=%23") || (href.contains("/s/") && href.contains("%23"))
    {
        let tag = rt.trim_start_matches('#').trim();
        if !tag.is_empty() && !tag.contains('"') {
            ctx.tags.push(tag.to_string());
            out.push_str(&format!("{{{{ tag(t=\"{tag}\") }}}}"));
        } else {
            out.push_str(&escape_inline(rt));
        }
        return;
    }

    if href.is_empty() {
        out.push_str(&escape_inline(rt));
        return;
    }

    ctx.links.push(href.to_string());
    if rt.is_empty() || rt == href {
        // Bare URL -> autolink (no escaping needed inside <>).
        out.push('<');
        out.push_str(href);
        out.push('>');
    } else {
        out.push('[');
        out.push_str(&escape_inline(rt));
        out.push_str("](");
        out.push_str(href);
        out.push(')');
    }
}

fn spoiler(out: &mut String, inner: &str) {
    out.push_str("<span class=\"spoiler\">");
    out.push_str(inner);
    out.push_str("</span>");
}

fn wrap(out: &mut String, open: &str, inner: &str, close: &str) {
    let t = inner.trim();
    if t.is_empty() {
        return;
    }
    out.push_str(open);
    out.push_str(t);
    out.push_str(close);
}

/// Escape characters that have inline-Markdown meaning. Newlines and `#`/`>`
/// at line starts are handled later in [`postprocess`].
fn escape_inline(s: &str) -> String {
    let mut o = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '\\' | '`' | '*' | '_' | '[' | ']' => {
                o.push('\\');
                o.push(c);
            }
            '<' => o.push_str("&lt;"),
            '>' => o.push_str("&gt;"),
            _ => o.push(c),
        }
    }
    o
}

static MANY_NL: Lazy<Regex> = Lazy::new(|| Regex::new(r"\n{3,}").unwrap());

fn postprocess(s: &str) -> String {
    let mut lines: Vec<String> = s.split('\n').map(|l| l.trim_end().to_string()).collect();
    let mut in_code = false;
    for l in &mut lines {
        // Never escape inside fenced code blocks.
        if l.trim_start().starts_with("```") {
            in_code = !in_code;
            continue;
        }
        if in_code {
            continue;
        }
        let trimmed = l.trim_start();
        let indent = l.len() - trimmed.len();
        // A line starting with `#` would become a heading; `>` a blockquote;
        // `- `/`* `/`+ ` a list. Escape the leading marker.
        let needs_escape = trimmed.starts_with('#')
            || trimmed.starts_with("- ")
            || trimmed.starts_with("* ")
            || trimmed.starts_with("+ ");
        if needs_escape {
            let (head, tail) = l.split_at(indent);
            *l = format!("{}\\{}", head, tail);
        }
    }
    let joined = lines.join("\n");
    MANY_NL.replace_all(&joined, "\n\n").trim().to_string()
}

fn dedup(v: Vec<String>) -> Vec<String> {
    let mut seen = std::collections::HashSet::new();
    v.into_iter().filter(|x| seen.insert(x.clone())).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use scraper::{Html, Selector};

    fn conv(html: &str) -> String {
        let doc = Html::parse_fragment(html);
        let sel = Selector::parse("div").unwrap();
        convert(doc.select(&sel).next().unwrap()).md
    }

    #[test]
    fn pre_keeps_br_line_breaks_unescaped() {
        let md = conv("<div><pre># one<br/># two<br/>echo hi</pre></div>");
        assert!(md.contains("# one\n# two\necho hi"), "{md:?}");
        assert!(!md.contains("\\#"), "code must not be escaped: {md:?}");
    }

    #[test]
    fn formatting_and_hashtags() {
        let md = conv(r#"<div><b>Bold</b> <i>it</i> <a href="?q=%23tag">#tag</a></div>"#);
        assert!(md.contains("**Bold**"), "{md:?}");
        assert!(md.contains("*it*"), "{md:?}");
        // Hashtags become a clickable `tag` shortcode.
        assert!(md.contains(r#"{{ tag(t="tag") }}"#), "{md:?}");
    }

    #[test]
    fn hashtag_becomes_tag_shortcode() {
        let md = conv(r#"<div><a href="?q=%23ad">#ad</a></div>"#);
        assert!(md.contains(r#"{{ tag(t="ad") }}"#), "{md:?}");
    }
}
