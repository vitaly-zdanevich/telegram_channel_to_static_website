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
}

impl Scraper {
    pub fn new(client: reqwest::Client, channel: String, delay_ms: u64) -> Self {
        Self {
            client,
            channel,
            delay: Duration::from_millis(delay_ms),
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
            Some(b) => format!("https://t.me/s/{}?before={}", self.channel, b),
            None => format!("https://t.me/s/{}", self.channel),
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
