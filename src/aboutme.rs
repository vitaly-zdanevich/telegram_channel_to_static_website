//! Optional About-page enrichment (`--no-about-me` to disable; on by default):
//! if the channel description links to an [about.me](https://about.me) profile,
//! pull its bio (the meta description) and social links onto the About page.
//! about.me renders server-side, so a plain HTTP fetch is enough.

use once_cell::sync::Lazy;
use regex::Regex;
use scraper::{Html, Selector};

static ABOUT_ME_URL: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"https?://about\.me/[A-Za-z0-9_.\-]+").unwrap());

/// A scraped about.me profile.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct AboutMe {
    /// The profile URL (used for the "message me" contact button).
    pub url: String,
    /// Full profile photo URL (`og:image`), if any.
    pub image: Option<String>,
    /// Short bio (the page's meta description).
    pub bio: String,
    /// `(label, url)` for each social link, in page order.
    pub links: Vec<(String, String)>,
}

impl AboutMe {
    pub fn is_empty(&self) -> bool {
        self.bio.is_empty() && self.links.is_empty() && self.image.is_none()
    }
}

/// The first about.me profile URL in a channel description, if any.
pub fn url_in(description: &str) -> Option<String> {
    ABOUT_ME_URL.find(description).map(|m| m.as_str().to_string())
}

/// Fetch + parse an about.me profile. `None` on any network/HTTP error — the
/// enrichment is best-effort and never fails the build.
pub async fn fetch(client: &reqwest::Client, url: &str) -> Option<AboutMe> {
    let html = client
        .get(url)
        .send()
        .await
        .ok()?
        .error_for_status()
        .ok()?
        .text()
        .await
        .ok()?;
    let mut am = parse(&html);
    if am.is_empty() {
        return None;
    }
    am.url = url.to_string();
    Some(am)
}

fn parse(html: &str) -> AboutMe {
    let doc = Html::parse_document(html);
    let bio = Selector::parse(r#"meta[name="description"]"#)
        .ok()
        .and_then(|s| doc.select(&s).next())
        .and_then(|e| e.value().attr("content"))
        .map(|c| c.split_whitespace().collect::<Vec<_>>().join(" "))
        .unwrap_or_default();
    let image = Selector::parse(r#"meta[property="og:image"]"#)
        .ok()
        .and_then(|s| doc.select(&s).next())
        .and_then(|e| e.value().attr("content"))
        .map(str::trim)
        .filter(|c| c.starts_with("http"))
        .map(String::from);

    let mut links = Vec::new();
    if let Ok(sel) = Selector::parse("a.social-link") {
        for a in doc.select(&sel) {
            let Some(href) = a.value().attr("href") else {
                continue;
            };
            if href.trim().is_empty() {
                continue;
            }
            let label = a
                .value()
                .attr("title")
                .map(clean_label)
                .filter(|l| !l.is_empty())
                .unwrap_or_else(|| host_of(href));
            links.push((label, href.to_string()));
        }
    }
    AboutMe {
        url: String::new(),
        image,
        bio,
        links,
    }
}

/// "Visit me on GitHub" → "GitHub"; leaves other titles as-is.
fn clean_label(title: &str) -> String {
    let t = title.trim();
    for p in [
        "Visit me on ",
        "Follow me on ",
        "Contact me on ",
        "Find me on ",
        "Message me on ",
        "Email me at ",
    ] {
        if let Some(rest) = t.strip_prefix(p) {
            return rest.trim().to_string();
        }
    }
    t.to_string()
}

/// Host of a URL, sans `www.`, as a fallback label (`https://github.com/x` → `github.com`).
fn host_of(url: &str) -> String {
    url.split("://")
        .nth(1)
        .unwrap_or(url)
        .split('/')
        .next()
        .unwrap_or(url)
        .trim_start_matches("www.")
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_about_me_url() {
        assert_eq!(
            url_in("bio\nhttps://about.me/zdanevich more"),
            Some("https://about.me/zdanevich".into())
        );
        assert_eq!(url_in("no link here"), None);
    }

    #[test]
    fn parses_bio_and_social_links() {
        let html = r#"<html><head>
            <meta name="description" content="I am a  software engineer.
            Hire me.">
            </head><body>
            <ul>
            <li><a class="social-link" title="Visit me on GitHub" href="https://github.com/x"><svg></svg></a></li>
            <li><a class="social-link" title="Follow me on Twitter" href="https://twitter.com/y"></a></li>
            <li><a class="social-link" href="https://gitlab.com/z"></a></li>
            <li><a class="nav-link" href="https://about.me/help">Help</a></li>
            </ul></body></html>"#;
        let am = parse(html);
        // Meta description → bio, whitespace collapsed.
        assert_eq!(am.bio, "I am a software engineer. Hire me.");
        // Only a.social-link anchors, labels cleaned; missing title → host.
        assert_eq!(
            am.links,
            vec![
                ("GitHub".into(), "https://github.com/x".into()),
                ("Twitter".into(), "https://twitter.com/y".into()),
                ("gitlab.com".into(), "https://gitlab.com/z".into()),
            ]
        );
    }
}
