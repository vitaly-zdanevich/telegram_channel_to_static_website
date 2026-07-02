//! Walk the public `t.me/s/<channel>` preview backwards through history,
//! following the `?before=<id>` cursor until the channel's first message.

use anyhow::{Context, Result};
use std::collections::{BTreeMap, HashSet};
use std::time::Duration;

use crate::model::RawMessage;
use crate::parse;

pub struct Scraper {
    client: reqwest::Client,
    channel: String,
    delay: Duration,
    /// URL prefix before `/<channel>`: the real `https://t.me/s`, or a mock
    /// server's base URL in tests.
    base: String,
}

impl Scraper {
    pub fn new(client: reqwest::Client, channel: String, delay_ms: u64) -> Self {
        Self {
            client,
            channel,
            delay: Duration::from_millis(delay_ms),
            base: "https://t.me/s".to_string(),
        }
    }

    /// Fetch every message (or up to `max_pages` pages), newest first across
    /// pages, returned sorted ascending by id and de-duplicated.
    pub async fn fetch_all(
        &self,
        max_pages: Option<usize>,
    ) -> Result<(Vec<RawMessage>, Option<crate::model::ChannelInfo>)> {
        let mut all: BTreeMap<u64, RawMessage> = BTreeMap::new();
        let mut visited: HashSet<u64> = HashSet::new();
        let mut before: Option<u64> = None;
        let mut pages = 0usize;
        let mut info = None;

        loop {
            let html = self.fetch_page(before).await?;
            if before.is_none() {
                info = parse::parse_channel_info(&html);
            }
            let (msgs, next_before) = parse::parse_page(&html, &self.channel)?;
            if msgs.is_empty() {
                break;
            }
            for m in msgs {
                all.insert(m.id, m);
            }
            pages += 1;
            tracing::info!(
                "page {} (before={:?}): {} messages so far",
                pages,
                before,
                all.len()
            );

            if max_pages.is_some_and(|mp| pages >= mp) {
                break;
            }
            match next_before {
                Some(b) if visited.insert(b) => before = Some(b),
                _ => break, // no cursor, or we'd loop forever
            }
            if !self.delay.is_zero() {
                tokio::time::sleep(self.delay).await;
            }
        }

        Ok((all.into_values().collect(), info))
    }

    async fn fetch_page(&self, before: Option<u64>) -> Result<String> {
        let url = match before {
            Some(b) => format!("{}/{}?before={}", self.base, self.channel, b),
            None => format!("{}/{}", self.base, self.channel),
        };
        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .with_context(|| format!("requesting {url}"))?
            .error_for_status()
            .with_context(|| format!("bad status for {url}"))?;
        resp.text()
            .await
            .with_context(|| format!("reading body of {url}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{method, path, query_param, query_param_is_missing};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn msg(id: u64) -> String {
        format!(
            r#"<div class="tgme_widget_message_wrap"><div class="tgme_widget_message js-widget_message" data-post="testchan/{id}"><div class="tgme_widget_message_text">msg {id}</div><div class="tgme_widget_message_date"><time datetime="2025-01-15T10:30:00+00:00"></time></div></div></div>"#
        )
    }

    fn page(ids: &[u64], before: Option<u64>) -> String {
        let msgs: String = ids.iter().map(|&i| msg(i)).collect();
        let more = before
            .map(|b| format!(r#"<a class="tme_messages_more" data-before="{b}"></a>"#))
            .unwrap_or_default();
        format!("<html><body>{msgs}{more}</body></html>")
    }

    fn scraper_at(base: String) -> Scraper {
        Scraper {
            client: reqwest::Client::new(),
            channel: "testchan".into(),
            delay: Duration::ZERO,
            base,
        }
    }

    #[tokio::test]
    async fn fetch_all_follows_the_before_cursor() {
        let server = MockServer::start().await;
        // First page (no `before`): ids 42, 43 + cursor before=40.
        Mock::given(method("GET"))
            .and(path("/testchan"))
            .and(query_param_is_missing("before"))
            .respond_with(ResponseTemplate::new(200).set_body_string(page(&[42, 43], Some(40))))
            .mount(&server)
            .await;
        // Second page (before=40): id 30, no cursor → stop.
        Mock::given(method("GET"))
            .and(path("/testchan"))
            .and(query_param("before", "40"))
            .respond_with(ResponseTemplate::new(200).set_body_string(page(&[30], None)))
            .mount(&server)
            .await;

        let (msgs, _info) = scraper_at(server.uri()).fetch_all(None).await.unwrap();
        // Collected across both pages, ascending by id, de-duplicated.
        let ids: Vec<u64> = msgs.iter().map(|m| m.id).collect();
        assert_eq!(ids, vec![30, 42, 43]);
    }

    #[tokio::test]
    async fn fetch_all_honours_max_pages() {
        let server = MockServer::start().await;
        // Every page offers a cursor, so only max_pages caps the walk.
        Mock::given(method("GET"))
            .and(path("/testchan"))
            .respond_with(ResponseTemplate::new(200).set_body_string(page(&[42, 43], Some(40))))
            .mount(&server)
            .await;

        let (msgs, _) = scraper_at(server.uri()).fetch_all(Some(1)).await.unwrap();
        assert_eq!(msgs.len(), 2); // one page fetched despite the cursor
    }
}
