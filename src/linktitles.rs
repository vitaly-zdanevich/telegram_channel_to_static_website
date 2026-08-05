//! Hover tooltips for links. For a Wikipedia / MediaWiki article, a Fandom or
//! miraheze wiki page, a YouTube video, or a GitHub repository, fetch a short
//! description or useful metadata at build time and attach it as the link's
//! `title=` (via a CommonMark link title), so hovering the link shows an intro
//! without leaving the page. Static — the rendered `title` attribute needs no
//! JavaScript and survives the offline pass.

use crate::model::Post;
use futures::StreamExt;
use percent_encoding::{percent_decode_str, utf8_percent_encode, NON_ALPHANUMERIC};
use serde_json::Value as J;
use std::collections::HashMap;

/// Resolve tooltips for every eligible link across `posts` and splice them into
/// each post's body. Ordinary links are fetched once per distinct URL, bounded
/// by `concurrency`. GitHub links are deduplicated by repository and fetched
/// serially to avoid the API's secondary rate limit.
pub async fn enrich(client: &reqwest::Client, posts: &mut [Post], concurrency: usize) {
    let mut urls: Vec<String> = Vec::new();
    let mut github_repos: Vec<(GitHubRepo, Vec<String>)> = Vec::new();
    for p in posts.iter() {
        for l in &p.links {
            if let Some(repo) = github_repo(l) {
                if let Some((_, links)) = github_repos.iter_mut().find(|(r, _)| *r == repo) {
                    if !links.contains(l) {
                        links.push(l.clone());
                    }
                } else {
                    github_repos.push((repo, vec![l.clone()]));
                }
            } else if (is_youtube(l) || mediawiki(l).is_some() || habr_user(l).is_some())
                && !urls.contains(l)
            {
                urls.push(l.clone());
            }
        }
    }
    if urls.is_empty() && github_repos.is_empty() {
        return;
    }
    let mut titles: HashMap<String, String> =
        futures::stream::iter(urls.into_iter().map(|u| async {
            let t = fetch_title(client, &u).await;
            (u, t)
        }))
        .buffer_unordered(concurrency.max(1))
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .filter_map(|(u, t)| t.map(|t| (u, t)))
        .collect();

    // A repository needs up to three REST calls (metadata, languages, commits).
    // Keep those calls serial, reuse one result for every URL into the same
    // repository, and stop when GitHub says the current rate-limit bucket is
    // exhausted. A token raises the limit substantially but is not required.
    let token = github_token();
    for (repo, links) in github_repos {
        let result =
            github_repo_title_from_api(client, &repo, "https://api.github.com", token.as_deref())
                .await;
        if let Some(title) = result.title {
            for link in links {
                titles.insert(link, title.clone());
            }
        }
        if result.stop_requests || result.remaining == Some(0) {
            break;
        }
    }

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

#[derive(Clone, Debug, Eq, PartialEq)]
struct GitHubRepo {
    owner: String,
    name: String,
}

/// The owner and repository for a `github.com/<owner>/<repo>/…` URL.
///
/// GitHub's non-repository top-level routes are excluded so links such as
/// `/topics/rust` do not spend an API request on a guaranteed 404.
fn github_repo(url: &str) -> Option<GitHubRepo> {
    let u = url::Url::parse(url).ok()?;
    if !matches!(u.scheme(), "http" | "https") {
        return None;
    }
    let host = u.host_str()?;
    if !host.eq_ignore_ascii_case("github.com") && !host.eq_ignore_ascii_case("www.github.com") {
        return None;
    }
    let mut segments = u.path_segments()?;
    let owner = percent_decode_str(segments.next()?)
        .decode_utf8()
        .ok()?
        .trim()
        .to_ascii_lowercase();
    let name = percent_decode_str(segments.next()?)
        .decode_utf8()
        .ok()?
        .trim()
        .to_ascii_lowercase();
    let name = name.strip_suffix(".git").unwrap_or(&name).to_owned();
    const NON_REPOSITORY_ROUTES: &[&str] = &[
        "about",
        "collections",
        "customer-stories",
        "enterprise",
        "events",
        "features",
        "login",
        "marketplace",
        "new",
        "notifications",
        "orgs",
        "pricing",
        "readme",
        "search",
        "settings",
        "signup",
        "sponsors",
        "topics",
        "users",
    ];
    if owner.is_empty() || name.is_empty() || NON_REPOSITORY_ROUTES.contains(&owner.as_str()) {
        return None;
    }
    Some(GitHubRepo { owner, name })
}

/// `(origin, page title)` for a MediaWiki `/wiki/<Title>` URL, else `None`.
fn mediawiki(url: &str) -> Option<(String, String)> {
    let u = url::Url::parse(url).ok()?;
    if !matches!(u.scheme(), "http" | "https") {
        return None;
    }
    let title = u.path().strip_prefix("/wiki/").filter(|t| !t.is_empty())?;
    let origin = format!("{}://{}", u.scheme(), u.host_str()?);
    let title = percent_decode_str(title)
        .decode_utf8_lossy()
        .replace('_', " ");
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

#[derive(Default)]
struct GitHubTitleResult {
    title: Option<String>,
    remaining: Option<u64>,
    /// Authentication and rate-limit failures affect every following request,
    /// so do not keep sending calls that GitHub has already told us to stop.
    stop_requests: bool,
}

/// Repository tooltip from GitHub's REST API.
///
/// The repository endpoint supplies the description, exact star/fork/open-item
/// counts, primary language, license, archive state, and last-push time. Two
/// small follow-up calls add the language mix and default-branch commit count.
/// If the response headers say only one request remains, the primary language
/// is kept and the last request is spent on the commit count.
async fn github_repo_title_from_api(
    client: &reqwest::Client,
    repo: &GitHubRepo,
    api_origin: &str,
    token: Option<&str>,
) -> GitHubTitleResult {
    let base = github_api_url(api_origin, repo);
    let Ok(response) = github_request(client, &base, token).send().await else {
        return GitHubTitleResult::default();
    };
    let mut remaining = github_remaining(&response);
    let mut stop_requests = github_should_stop(response.status());
    if !response.status().is_success() {
        return GitHubTitleResult {
            remaining,
            stop_requests,
            ..GitHubTitleResult::default()
        };
    }
    let Ok(repository) = response.json::<J>().await else {
        return GitHubTitleResult {
            remaining,
            stop_requests,
            ..GitHubTitleResult::default()
        };
    };
    if remaining == Some(0) {
        stop_requests = true;
    }

    let primary_language = repository
        .get("language")
        .and_then(J::as_str)
        .and_then(clean)
        .map(|language| format!("Language: {language}"));
    let mut languages = None;

    // Preserve the final request for commits when the response tells us that
    // only one call remains. The repository JSON still supplies one language.
    if !stop_requests && !matches!(remaining, Some(0 | 1)) {
        let url = format!("{base}/languages");
        if let Ok(response) = github_request(client, &url, token).send().await {
            update_github_remaining(&mut remaining, &response);
            stop_requests |= github_should_stop(response.status());
            if response.status().is_success() {
                if let Ok(value) = response.json::<J>().await {
                    languages = github_languages(&value);
                }
            }
            if remaining == Some(0) {
                stop_requests = true;
            }
        }
    }

    let mut commits = None;
    if !stop_requests && remaining != Some(0) {
        let url = format!("{base}/commits?per_page=1");
        if let Ok(response) = github_request(client, &url, token).send().await {
            update_github_remaining(&mut remaining, &response);
            stop_requests |= github_should_stop(response.status());
            if response.status() == reqwest::StatusCode::CONFLICT {
                // GitHub returns 409 for an initialized repository with no
                // commits; for a count that is unambiguously zero.
                commits = Some(0);
            } else if response.status().is_success() {
                let link = response
                    .headers()
                    .get(reqwest::header::LINK)
                    .and_then(|value| value.to_str().ok())
                    .map(str::to_owned);
                let body = response.json::<J>().await.ok();
                commits = github_commit_count(link.as_deref(), body.as_ref());
            }
            if remaining == Some(0) {
                stop_requests = true;
            }
        }
    }

    GitHubTitleResult {
        title: github_title(
            &repository,
            languages.as_deref().or(primary_language.as_deref()),
            commits,
        ),
        remaining,
        stop_requests,
    }
}

fn github_token() -> Option<String> {
    ["GH_TOKEN", "GITHUB_TOKEN"].into_iter().find_map(|name| {
        std::env::var(name).ok().and_then(|value| {
            let value = value.trim();
            (!value.is_empty()).then(|| value.to_owned())
        })
    })
}

fn github_api_url(api_origin: &str, repo: &GitHubRepo) -> String {
    let owner = utf8_percent_encode(&repo.owner, NON_ALPHANUMERIC);
    let name = utf8_percent_encode(&repo.name, NON_ALPHANUMERIC);
    format!("{}/repos/{owner}/{name}", api_origin.trim_end_matches('/'))
}

fn github_request(
    client: &reqwest::Client,
    url: &str,
    token: Option<&str>,
) -> reqwest::RequestBuilder {
    let request = client
        .get(url)
        .header(reqwest::header::ACCEPT, "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2022-11-28");
    match token {
        Some(token) => request.bearer_auth(token),
        None => request,
    }
}

fn github_remaining(response: &reqwest::Response) -> Option<u64> {
    response
        .headers()
        .get("x-ratelimit-remaining")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.parse().ok())
}

fn update_github_remaining(remaining: &mut Option<u64>, response: &reqwest::Response) {
    if let Some(value) = github_remaining(response) {
        *remaining = Some(value);
    }
}

fn github_should_stop(status: reqwest::StatusCode) -> bool {
    matches!(
        status,
        reqwest::StatusCode::UNAUTHORIZED
            | reqwest::StatusCode::FORBIDDEN
            | reqwest::StatusCode::TOO_MANY_REQUESTS
    )
}

/// A compact top-three language mix, calculated from GitHub's byte counts.
fn github_languages(value: &J) -> Option<String> {
    let mut languages: Vec<(&str, u64)> = value
        .as_object()?
        .iter()
        .filter_map(|(language, bytes)| bytes.as_u64().map(|bytes| (language.as_str(), bytes)))
        .filter(|(_, bytes)| *bytes > 0)
        .collect();
    languages.sort_by(|(left_name, left_bytes), (right_name, right_bytes)| {
        right_bytes
            .cmp(left_bytes)
            .then_with(|| left_name.cmp(right_name))
    });
    let total: u128 = languages.iter().map(|(_, bytes)| u128::from(*bytes)).sum();
    if total == 0 {
        return None;
    }
    let values = languages
        .into_iter()
        .take(3)
        .map(|(language, bytes)| {
            let percent = (u128::from(bytes) * 100 + total / 2) / total;
            format!("{language} {percent}%")
        })
        .collect::<Vec<_>>();
    Some(format!("Languages: {}", values.join(", ")))
}

/// With `per_page=1`, the `last` pagination page is the exact number of
/// default-branch commits. A response without pagination contains zero or one.
fn github_commit_count(link: Option<&str>, body: Option<&J>) -> Option<u64> {
    if let Some(link) = link {
        for part in link.split(',') {
            if !part.contains("rel=\"last\"") {
                continue;
            }
            let start = part.find('<')? + 1;
            let end = part[start..].find('>')? + start;
            let url = url::Url::parse(&part[start..end]).ok()?;
            return url
                .query_pairs()
                .find_map(|(key, value)| (key == "page").then(|| value.parse().ok()).flatten());
        }
        // A next link without a last link cannot yield an exact count.
        if link.contains("rel=\"next\"") {
            return None;
        }
    }
    body?.as_array().map(|commits| commits.len() as u64)
}

fn github_title(repository: &J, languages: Option<&str>, commits: Option<u64>) -> Option<String> {
    let mut parts = Vec::new();
    if let Some(description) = repository
        .get("description")
        .and_then(J::as_str)
        .and_then(clean)
    {
        parts.push(truncate(&description, 140));
    }
    if let Some(stars) = repository.get("stargazers_count").and_then(J::as_u64) {
        parts.push(quantity(stars, "star", "stars"));
    }
    if let Some(languages) = languages {
        parts.push(languages.to_owned());
    }
    if let Some(commits) = commits {
        parts.push(quantity(commits, "commit", "commits"));
    }
    if let Some(forks) = repository.get("forks_count").and_then(J::as_u64) {
        if forks > 0 {
            parts.push(quantity(forks, "fork", "forks"));
        }
    }
    if let Some(open) = repository.get("open_issues_count").and_then(J::as_u64) {
        if open > 0 {
            parts.push(format!("{open} open issues/PRs"));
        }
    }
    if let Some(license) = repository
        .pointer("/license/spdx_id")
        .and_then(J::as_str)
        .filter(|license| *license != "NOASSERTION")
    {
        parts.push(license.to_owned());
    }
    if repository.get("archived").and_then(J::as_bool) == Some(true) {
        parts.push("archived".to_owned());
    }
    if let Some(pushed) = repository.get("pushed_at").and_then(J::as_str) {
        parts.push(format!(
            "last push {}",
            pushed.split('T').next().unwrap_or(pushed)
        ));
    }
    clean(&parts.join(" · "))
}

fn quantity(count: u64, singular: &str, plural: &str) -> String {
    format!("{count} {}", if count == 1 { singular } else { plural })
}

fn truncate(value: &str, max_chars: usize) -> String {
    if value.chars().count() <= max_chars {
        return value.to_owned();
    }
    let mut value = value
        .chars()
        .take(max_chars.saturating_sub(1))
        .collect::<String>();
    value = value.trim_end().to_owned();
    value.push('…');
    value
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
        j.pointer(ptr)
            .and_then(J::as_i64)
            .map(|v| format!("{v} {label}"))
    };
    parts.extend(count(
        "/counterStats/publicationStats/articleCount",
        "articles",
    ));
    parts.extend(count("/counterStats/publicationStats/postCount", "posts"));
    parts.extend(count("/counterStats/publicationStats/newsCount", "news"));
    parts.extend(count("/counterStats/commentCount", "comments"));
    if let Some(reg) = j.get("registerDateTime").and_then(J::as_str) {
        parts.push(format!(
            "registered {}",
            reg.split('T').next().unwrap_or(reg)
        ));
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
        let Some(j) = get_json(client, &api).await else {
            continue;
        };
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
        out = out
            .chars()
            .take(297)
            .collect::<String>()
            .trim_end()
            .to_string();
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
    use serde_json::json;
    use wiremock::matchers::{header, method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[test]
    fn detects_mediawiki_pages() {
        assert_eq!(
            mediawiki("https://en.wikipedia.org/wiki/Rust_(programming_language)"),
            Some((
                "https://en.wikipedia.org".into(),
                "Rust (programming language)".into()
            ))
        );
        assert_eq!(
            mediawiki("https://homm.miraheze.org/wiki/One_Bad_Day_(Allies)"),
            Some((
                "https://homm.miraheze.org".into(),
                "One Bad Day (Allies)".into()
            ))
        );
        assert_eq!(mediawiki("https://example.com/blog/post"), None);
    }

    #[test]
    fn detects_github_repositories_and_deep_links() {
        assert_eq!(
            github_repo("https://github.com/Vitaly-Zdanevich/Reeknote"),
            Some(GitHubRepo {
                owner: "vitaly-zdanevich".into(),
                name: "reeknote".into(),
            })
        );
        assert_eq!(
            github_repo("https://www.github.com/acme/tool.GIT/blob/main/README.md"),
            Some(GitHubRepo {
                owner: "acme".into(),
                name: "tool".into(),
            })
        );
        assert_eq!(github_repo("https://github.com/topics/rust"), None);
        assert_eq!(github_repo("https://github.com/only-an-owner"), None);
        assert_eq!(
            github_repo("https://github.com.example.com/acme/tool"),
            None
        );
    }

    #[test]
    fn formats_language_mix_and_commit_pagination() {
        assert_eq!(
            github_languages(&json!({"Rust": 850, "Shell": 100, "Lua": 50, "Other": 1})).as_deref(),
            Some("Languages: Rust 85%, Shell 10%, Lua 5%")
        );
        let link = concat!(
            "<https://api.github.com/repos/acme/tool/commits?per_page=1&page=2>; rel=\"next\", ",
            "<https://api.github.com/repos/acme/tool/commits?per_page=1&page=42>; rel=\"last\""
        );
        assert_eq!(
            github_commit_count(Some(link), Some(&json!([{}]))),
            Some(42)
        );
        assert_eq!(github_commit_count(None, Some(&json!([]))), Some(0));
        assert_eq!(github_commit_count(None, Some(&json!([{}]))), Some(1));
    }

    #[tokio::test]
    async fn fetches_github_repository_metadata() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/repos/acme/tool"))
            .and(header("accept", "application/vnd.github+json"))
            .and(header("x-github-api-version", "2022-11-28"))
            .and(header("authorization", "Bearer test-token"))
            .respond_with(
                ResponseTemplate::new(200)
                    .insert_header("x-ratelimit-remaining", "59")
                    .set_body_json(json!({
                        "description": "A useful \"thing\"",
                        "stargazers_count": 1234,
                        "language": "Rust",
                        "forks_count": 2,
                        "open_issues_count": 3,
                        "archived": false,
                        "license": {"spdx_id": "MIT"},
                        "pushed_at": "2026-07-20T12:34:56Z"
                    })),
            )
            .expect(1)
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/repos/acme/tool/languages"))
            .respond_with(
                ResponseTemplate::new(200)
                    .insert_header("x-ratelimit-remaining", "58")
                    .set_body_json(json!({"Rust": 900, "Shell": 100})),
            )
            .expect(1)
            .mount(&server)
            .await;
        let commit_link = format!(
            "<{}/repos/acme/tool/commits?per_page=1&page=42>; rel=\"last\"",
            server.uri()
        );
        Mock::given(method("GET"))
            .and(path("/repos/acme/tool/commits"))
            .and(query_param("per_page", "1"))
            .respond_with(
                ResponseTemplate::new(200)
                    .insert_header("x-ratelimit-remaining", "57")
                    .insert_header("link", commit_link)
                    .set_body_json(json!([{"sha": "abc"}])),
            )
            .expect(1)
            .mount(&server)
            .await;

        let result = github_repo_title_from_api(
            &reqwest::Client::new(),
            &GitHubRepo {
                owner: "acme".into(),
                name: "tool".into(),
            },
            &server.uri(),
            Some("test-token"),
        )
        .await;
        assert_eq!(
            result.title.as_deref(),
            Some(
                "A useful \"thing\" · 1234 stars · Languages: Rust 90%, Shell 10% · \
                 42 commits · 2 forks · 3 open issues/PRs · MIT · last push 2026-07-20"
            )
        );
        assert_eq!(result.remaining, Some(57));
        assert!(!result.stop_requests);
    }

    #[tokio::test]
    async fn preserves_last_rate_limit_request_for_commit_count() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/repos/acme/tool"))
            .respond_with(
                ResponseTemplate::new(200)
                    .insert_header("x-ratelimit-remaining", "1")
                    .set_body_json(json!({
                        "description": "Small repo",
                        "stargazers_count": 1,
                        "language": "Go"
                    })),
            )
            .expect(1)
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/repos/acme/tool/languages"))
            .respond_with(ResponseTemplate::new(500))
            .expect(0)
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/repos/acme/tool/commits"))
            .and(query_param("per_page", "1"))
            .respond_with(
                ResponseTemplate::new(200)
                    .insert_header("x-ratelimit-remaining", "0")
                    .set_body_json(json!([{"sha": "abc"}])),
            )
            .expect(1)
            .mount(&server)
            .await;

        let result = github_repo_title_from_api(
            &reqwest::Client::new(),
            &GitHubRepo {
                owner: "acme".into(),
                name: "tool".into(),
            },
            &server.uri(),
            None,
        )
        .await;
        assert_eq!(
            result.title.as_deref(),
            Some("Small repo · 1 star · Language: Go · 1 commit")
        );
        assert_eq!(result.remaining, Some(0));
        assert!(result.stop_requests);
    }

    #[test]
    fn detects_habr_user() {
        assert_eq!(
            habr_user("https://habr.com/en/users/zdanevich-vitaly/").as_deref(),
            Some("zdanevich-vitaly")
        );
        assert_eq!(
            habr_user("https://habr.com/ru/users/foo/posts/").as_deref(),
            Some("foo")
        );
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
        assert_eq!(
            inline,
            format!("see [here](<{url}> \"He said \\\"hi\\\"\") ok")
        );
    }
}
