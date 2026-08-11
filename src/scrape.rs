//! Walk the public `t.me/s/<channel>` preview backwards through history,
//! following the `?before=<id>` cursor until the channel's first message.

use anyhow::{Context, Result};
use std::collections::{BTreeMap, HashSet};
use std::time::Duration;

use crate::model::RawMessage;
use crate::parse;

/// Total attempts for one Telegram preview page, including the first request.
const PAGE_FETCH_ATTEMPTS: usize = 10;
/// Delay before the first retry; subsequent delays double exponentially.
const PAGE_RETRY_BASE_DELAY: Duration = Duration::from_secs(1);
/// Upper bound for one retry delay, keeping ten attempts within a few minutes.
const PAGE_RETRY_MAX_DELAY: Duration = Duration::from_secs(30);

pub struct Scraper {
    client: reqwest::Client,
    channel: String,
    delay: Duration,
    /// Base backoff between retries of a transient page-fetch failure.
    retry_base_delay: Duration,
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
            retry_base_delay: PAGE_RETRY_BASE_DELAY,
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

        for attempt in 1..=PAGE_FETCH_ATTEMPTS {
            let response = match self.client.get(&url).send().await {
                Ok(response) => response,
                Err(error) if attempt < PAGE_FETCH_ATTEMPTS && retryable_request_error(&error) => {
                    self.wait_before_retry(&url, attempt, &error.to_string())
                        .await;
                    continue;
                }
                Err(error) => return Err(error).with_context(|| format!("requesting {url}")),
            };

            let status = response.status();
            if retryable_status(status) && attempt < PAGE_FETCH_ATTEMPTS {
                self.wait_before_retry(&url, attempt, &format!("HTTP {status}"))
                    .await;
                continue;
            }
            let response = response
                .error_for_status()
                .with_context(|| format!("bad status for {url}"))?;
            match response.text().await {
                Ok(body) => return Ok(body),
                Err(error) if attempt < PAGE_FETCH_ATTEMPTS => {
                    self.wait_before_retry(&url, attempt, &error.to_string())
                        .await;
                }
                Err(error) => {
                    return Err(error).with_context(|| format!("reading body of {url}"));
                }
            }
        }

        unreachable!("the bounded page-fetch loop always returns on its final attempt")
    }

    /// Log a transient failure and wait with exponential backoff before retrying.
    async fn wait_before_retry(&self, url: &str, attempt: usize, reason: &str) {
        let delay = self.retry_delay(attempt);
        tracing::warn!(
            "fetching {url} failed on attempt {attempt}/{PAGE_FETCH_ATTEMPTS}: {reason}; retrying in {} ms",
            delay.as_millis()
        );
        if !delay.is_zero() {
            tokio::time::sleep(delay).await;
        }
    }

    /// Calculate the capped exponential delay after the given failed attempt.
    fn retry_delay(&self, attempt: usize) -> Duration {
        let multiplier = 1_u32 << (attempt - 1);
        self.retry_base_delay
            .saturating_mul(multiplier)
            .min(PAGE_RETRY_MAX_DELAY)
    }
}

/// Whether an HTTP response is likely to succeed when requested again shortly.
fn retryable_status(status: reqwest::StatusCode) -> bool {
    status == reqwest::StatusCode::REQUEST_TIMEOUT
        || status == reqwest::StatusCode::TOO_MANY_REQUESTS
        || status.is_server_error()
}

/// Whether sending a valid page request failed for a likely transient reason.
fn retryable_request_error(error: &reqwest::Error) -> bool {
    error.is_connect() || error.is_timeout() || error.is_request()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::ErrorKind;
    use wiremock::matchers::{method, path, query_param, query_param_is_missing};
    use wiremock::{Mock, MockServer, Request, ResponseTemplate};

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
            retry_base_delay: Duration::ZERO,
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

    /// A transient server failure must not discard an otherwise valid scrape.
    #[tokio::test]
    async fn fetch_all_retries_transient_server_errors() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/testchan"))
            .and(query_param_is_missing("before"))
            .respond_with(ResponseTemplate::new(500))
            .up_to_n_times(1)
            .with_priority(1)
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/testchan"))
            .and(query_param_is_missing("before"))
            .respond_with(ResponseTemplate::new(200).set_body_string(page(&[42], None)))
            .with_priority(2)
            .mount(&server)
            .await;

        let (msgs, _) = scraper_at(server.uri()).fetch_all(Some(1)).await.unwrap();

        assert_eq!(
            msgs.iter().map(|message| message.id).collect::<Vec<_>>(),
            [42]
        );
        assert_eq!(server.received_requests().await.unwrap().len(), 2);
    }

    /// A connection reset while sending a request must be retried like an HTTP 5xx.
    #[tokio::test]
    async fn fetch_all_retries_request_transport_errors() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/testchan"))
            .respond_with_err(|_: &Request| {
                std::io::Error::new(ErrorKind::ConnectionReset, "connection reset")
            })
            .up_to_n_times(1)
            .with_priority(1)
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/testchan"))
            .respond_with(ResponseTemplate::new(200).set_body_string(page(&[42], None)))
            .with_priority(2)
            .mount(&server)
            .await;

        let (msgs, _) = scraper_at(server.uri()).fetch_all(Some(1)).await.unwrap();

        assert_eq!(
            msgs.iter().map(|message| message.id).collect::<Vec<_>>(),
            [42]
        );
        assert_eq!(server.received_requests().await.unwrap().len(), 2);
    }

    /// A permanent client error must fail immediately instead of being retried.
    #[tokio::test]
    async fn fetch_all_does_not_retry_permanent_client_errors() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/testchan"))
            .respond_with(ResponseTemplate::new(404))
            .mount(&server)
            .await;

        let error = scraper_at(server.uri())
            .fetch_all(Some(1))
            .await
            .unwrap_err();

        assert!(format!("{error:#}").contains("404 Not Found"));
        assert_eq!(server.received_requests().await.unwrap().len(), 1);
    }

    /// Persistent transient failures must stop at the bounded attempt count.
    #[tokio::test]
    async fn fetch_all_limits_transient_server_error_retries() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/testchan"))
            .respond_with(ResponseTemplate::new(500))
            .mount(&server)
            .await;

        let error = scraper_at(server.uri())
            .fetch_all(Some(1))
            .await
            .unwrap_err();

        assert!(format!("{error:#}").contains("500 Internal Server Error"));
        assert_eq!(server.received_requests().await.unwrap().len(), 10);
    }

    /// Retry classification includes transient statuses but excludes permanent ones.
    #[test]
    fn retryable_status_only_accepts_transient_failures() {
        for status in [408, 429, 500, 502, 599] {
            assert!(retryable_status(
                reqwest::StatusCode::from_u16(status).unwrap()
            ));
        }
        for status in [400, 401, 403, 404] {
            assert!(!retryable_status(
                reqwest::StatusCode::from_u16(status).unwrap()
            ));
        }
    }

    /// Exponential backoff is capped so ten attempts cannot stall one page excessively.
    #[test]
    fn retry_delay_is_exponential_and_capped() {
        let mut scraper = scraper_at(String::new());
        scraper.retry_base_delay = PAGE_RETRY_BASE_DELAY;

        let delays = (1..PAGE_FETCH_ATTEMPTS)
            .map(|attempt| scraper.retry_delay(attempt).as_secs())
            .collect::<Vec<_>>();

        assert_eq!(delays, [1, 2, 4, 8, 16, 30, 30, 30, 30]);
    }
}
