//! Safe, single-post-only HTML for cached Telegram discussion snapshots.

use crate::discussions::Discussion;
use crate::discussions::{escape_html as esc, safe_link};
use crate::i18n::Ui;
use crate::render::RenderedPost;
use scraper::{ElementRef, Html, Node};

/// Attach a discussion snapshot to a generated post's extra front matter.
pub fn attach(rendered: &mut RenderedPost, channel: &str, threads: &[Discussion], ui: &Ui) {
    if threads.is_empty() {
        return;
    }
    let count: usize = threads.iter().map(|thread| thread.comments.len()).sum();
    let mut html = format!("<section class='discussions' aria-labelledby='comments-heading'><h2 id='comments-heading'>{} ({count})</h2>", esc(ui.comments));
    for thread in threads {
        let base = format!("https://t.me/{}/{}", esc(channel), thread.post_id);
        html.push_str("<ol class='discussion-comments'>");
        for comment in &thread.comments {
            let id = format!("comment-{}-{}", thread.peer_id, comment.id);
            let link = format!("{base}?comment={}", comment.id);
            let date = chrono::DateTime::parse_from_rfc3339(&comment.date)
                .map(|date| date.format("%Y-%m-%d %H:%M %:z").to_string())
                .unwrap_or_else(|_| comment.date.clone());
            html.push_str(&format!("<li id='{id}'><div class='comment-meta'><span class='comment-author'>{}</span> · <a href='{link}'><time datetime='{}'>{}</time></a></div>", esc(&comment.author), esc(&comment.date), esc(&date)));
            if let Some(parent) = comment.reply_to.filter(|parent| *parent != thread.root_id) {
                let href = if thread.comments.iter().any(|comment| comment.id == parent) {
                    format!("#comment-{}-{parent}", thread.peer_id)
                } else {
                    format!("{base}?comment={parent}")
                };
                html.push_str(&format!(
                    "<a class='comment-parent' href='{href}'>{} #{parent}</a>",
                    esc(ui.reply_to_comment)
                ));
            }
            html.push_str("<div class='comment-body'>");
            sanitize_children(
                Html::parse_fragment(&comment.body_html).root_element(),
                &mut html,
            );
            html.push_str("</div>");
            if comment.has_media {
                html.push_str(&format!(
                    "<p class='comment-attachment'><a href='{link}'>{} ({})</a></p>",
                    esc(ui.attachment),
                    esc(ui.not_archived)
                ));
            }
            html.push_str("</li>");
        }
        html.push_str(&format!(
            "</ol><p><a href='{base}?comment=0'>{}</a></p>",
            esc(ui.reply_on_telegram)
        ));
    }
    html.push_str("</section>");
    // Put the string immediately under [extra], before any nested related
    // tables. Zola does not interpret front-matter values as shortcodes/Tera.
    // Force one physical line: TOML's pretty serializer may emit multiline
    // strings containing a literal +++ line, which closes Zola front matter.
    let field = format!(
        "\n[extra]\ndiscussions_html = {}\n",
        crate::render::toml_str(&html)
    );
    rendered.index_md = rendered.index_md.replacen("\n[extra]\n", &field, 1);
}

/// Serialize a tiny HTML allowlist from an HTML5 parse tree. Neither cached
/// markup nor Telegram entity attributes are trusted to be safe HTML.
fn sanitize_children(element: ElementRef<'_>, out: &mut String) {
    for node in element.children() {
        match node.value() {
            Node::Text(text) => out.push_str(&esc(text)),
            Node::Element(_) => {
                let element = ElementRef::wrap(node).unwrap();
                let name = element.value().name();
                if matches!(
                    name,
                    "script"
                        | "style"
                        | "svg"
                        | "math"
                        | "iframe"
                        | "object"
                        | "embed"
                        | "template"
                ) {
                    continue;
                }
                let tag = match name {
                    "b" | "strong" | "i" | "em" | "u" | "s" | "del" | "code" | "pre"
                    | "blockquote" | "p" | "details" | "summary" => Some(name),
                    "br" => {
                        out.push_str("<br>");
                        continue;
                    }
                    "span" if element.value().attr("class") == Some("tg-spoiler") => {
                        out.push_str("<details class='comment-spoiler'><summary>…</summary>");
                        sanitize_children(element, out);
                        out.push_str("</details>");
                        continue;
                    }
                    "a" => {
                        if let Some(href) =
                            element.value().attr("href").filter(|href| safe_link(href))
                        {
                            out.push_str(&format!("<a href='{}' rel='nofollow ugc'>", esc(href)));
                            sanitize_children(element, out);
                            out.push_str("</a>");
                            continue;
                        }
                        None
                    }
                    _ => None,
                };
                if let Some(tag) = tag {
                    out.push_str(&format!("<{tag}>"));
                }
                sanitize_children(element, out);
                if let Some(tag) = tag {
                    out.push_str(&format!("</{tag}>"));
                }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::discussions::Comment;
    use scraper::{Html, Selector};

    /// Include a nested extra table to guard against putting HTML under related.
    fn rendered() -> RenderedPost {
        RenderedPost {
            slug: "2026-12-31-1".into(),
            title: "Post".into(),
            index_md: "+++\ntitle = 'Post'\n[extra]\nid = 1\n[[extra.related]]\nlabel = 'Other'\n+++\n\nOriginal body\n".into(),
            og_image: None,
            downloads: vec![],
        }
    }

    /// Comments exercise arbitrary names, formatting, replies, and attachments.
    fn thread() -> Discussion {
        Discussion {
            peer_id: 77,
            root_id: 10,
            post_id: 1,
            comments: vec![
                Comment {
                    id: 11,
                    author: "Аўтар <img src=x onerror=alert(1)>".into(),
                    date: "2026-12-31T12:34:00+00:00".into(),
                    body_html:
                        "<b>First</b>\n<pre>    code()</pre><span class='tg-spoiler'>secret</span>"
                            .into(),
                    reply_to: None,
                    has_media: false,
                },
                Comment {
                    id: 12,
                    author: "Bob".into(),
                    date: "2026-12-31T12:35:00+00:00".into(),
                    body_html: "Reply".into(),
                    reply_to: Some(11),
                    has_media: true,
                },
            ],
        }
    }

    /// Parse the resulting TOML just as Zola does, separately from its body.
    fn extra_html(post: &RenderedPost) -> String {
        let (front, body) = post
            .index_md
            .strip_prefix("+++\n")
            .unwrap()
            .split_once("\n+++\n")
            .unwrap();
        assert_eq!(body, "\nOriginal body\n");
        let front: toml::Value = toml::from_str(front).unwrap();
        assert_eq!(
            front["extra"]["related"][0]["label"].as_str(),
            Some("Other")
        );
        front["extra"]["discussions_html"]
            .as_str()
            .unwrap()
            .to_owned()
    }

    #[test]
    fn attaches_comments_as_extra_without_changing_the_body() {
        let mut post = rendered();
        attach(&mut post, "testchan", &[thread()], &crate::i18n::ui("en"));
        let html = extra_html(&post);
        let document = Html::parse_fragment(&html);
        let comments = Selector::parse(".discussion-comments > li").unwrap();
        assert_eq!(document.select(&comments).count(), 2);
        assert!(html.contains("href='#comment-77-11'"));
        assert!(html.contains("https://t.me/testchan/1?comment=12"));
        assert!(html.contains("2026-12-31T12:34:00+00:00"));
        assert!(html.contains("2026-12-31 12:34 +00:00"));
        assert!(html.contains("Reply on Telegram"));
        assert!(html.contains("Attachment (not archived)"));
        assert!(document
            .select(&Selector::parse("img").unwrap())
            .next()
            .is_none());
        assert!(document
            .root_element()
            .text()
            .collect::<String>()
            .contains("Аўтар <img src=x onerror=alert(1)>"));
    }

    #[test]
    fn no_snapshot_leaves_post_unchanged() {
        let mut post = rendered();
        let original = post.index_md.clone();
        attach(&mut post, "testchan", &[], &crate::i18n::ui("en"));
        assert_eq!(post.index_md, original);
    }

    /// DEL is forbidden literally in TOML, even though Telegram text can carry
    /// arbitrary control characters. Keep it inside one escaped basic string.
    #[test]
    fn control_characters_cannot_break_comment_front_matter() {
        let mut thread = thread();
        thread.comments[0].body_html = "control:\u{7f}".into();
        let mut post = rendered();
        attach(&mut post, "testchan", &[thread], &crate::i18n::ui("en"));
        assert!(extra_html(&post).contains("control:\u{7f}"));
    }

    #[test]
    fn untrusted_markup_and_template_delimiters_are_inert() {
        let mut thread = thread();
        thread.comments[0].body_html = concat!(
            "<script>alert(1)</script><style>body{display:none}</style>",
            "<svg><script>alert(2)</script></svg><iframe srcdoc='<script>alert(3)</script>'></iframe>",
            "<a href='java&#x73;cript:alert(1)' onclick='alert(2)'>bad</a>",
            "<a href='data:text/html,boom'>data</a><img src=x onerror=alert(1)>",
            "<a href='https://example.test/?a=1&amp;b=2' onmouseover='bad()'>safe</a>",
            "<b onclick='bad()'>bold</b><i>italic</i><u>underline</u><s>strike</s>",
            "<code>&lt;script&gt;</code><blockquote>quote</blockquote>",
            "{{ video(src=\"evil\") }} {% if true %}literal{% endif %}\n+++\n[extra]\n"
        ).into();
        let mut post = rendered();
        attach(&mut post, "testchan", &[thread], &crate::i18n::ui("en"));
        let html = extra_html(&post);
        let document = Html::parse_fragment(&html);
        assert!(document
            .select(
                &Selector::parse("script, style, svg, iframe, img, [onclick], [onmouseover]")
                    .unwrap()
            )
            .next()
            .is_none());
        for link in document.select(&Selector::parse("a[href]").unwrap()) {
            assert!(!["javascript:", "data:"].iter().any(|scheme| link
                .value()
                .attr("href")
                .unwrap()
                .starts_with(scheme)));
        }
        assert!(html.contains("{{ video(src=&quot;evil&quot;) }}"));
        assert!(html.contains("{% if true %}literal{% endif %}"));
        assert!(html.contains("<b>bold</b>"));
        assert!(html.contains("<blockquote>quote</blockquote>"));
        assert!(html.contains("href='https://example.test/?a=1&amp;b=2'"));
    }
}
