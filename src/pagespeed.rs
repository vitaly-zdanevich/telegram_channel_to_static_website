//! Google PageSpeed Insights (Lighthouse) scores for the deployed site: shown on
//! the About page and published as shields.io badge endpoints for the README.
//!
//! Opt-in (`--pagespeed` / `PAGESPEED=1`). The scan runs against the site's
//! `base_url` — i.e. the *currently live* deploy — so in the daily workflow the
//! scores reflect the previous run (a one-day lag, which is fine for a health
//! readout). No API key is required at this volume; set `PAGESPEED_API_KEY` for
//! higher rate limits.

use serde::Deserialize;

/// Mobile Lighthouse category scores (0–100). Each is optional — a category can
/// be missing from the response, and the whole fetch is best-effort.
#[derive(Debug, Clone, Copy, Default)]
pub struct Scores {
    pub performance: Option<u8>,
    pub accessibility: Option<u8>,
    pub best_practices: Option<u8>,
    pub seo: Option<u8>,
}

impl Scores {
    /// `(label, score)` for each category that came back, in display order.
    pub fn entries(&self) -> Vec<(&'static str, u8)> {
        [
            ("Performance", self.performance),
            ("Accessibility", self.accessibility),
            ("Best Practices", self.best_practices),
            ("SEO", self.seo),
        ]
        .into_iter()
        .filter_map(|(name, v)| v.map(|v| (name, v)))
        .collect()
    }
}

#[derive(Deserialize)]
struct PsiResp {
    #[serde(rename = "lighthouseResult")]
    lighthouse_result: Option<LhResult>,
}

#[derive(Deserialize)]
struct LhResult {
    categories: Categories,
}

#[derive(Deserialize)]
struct Categories {
    performance: Option<Cat>,
    accessibility: Option<Cat>,
    #[serde(rename = "best-practices")]
    best_practices: Option<Cat>,
    seo: Option<Cat>,
}

#[derive(Deserialize)]
struct Cat {
    score: Option<f64>,
}

/// Lighthouse category scores are 0–1 floats; present them as 0–100 integers.
fn pct(cat: &Option<Cat>) -> Option<u8> {
    cat.as_ref()
        .and_then(|c| c.score)
        .map(|s| (s * 100.0).round().clamp(0.0, 100.0) as u8)
}

fn parse(body: &str) -> Option<Scores> {
    let resp: PsiResp = serde_json::from_str(body).ok()?;
    let cats = resp.lighthouse_result?.categories;
    Some(Scores {
        performance: pct(&cats.performance),
        accessibility: pct(&cats.accessibility),
        best_practices: pct(&cats.best_practices),
        seo: pct(&cats.seo),
    })
}

/// Fetch mobile Lighthouse scores for `url` via the PSI API. `None` on any error
/// (network, non-200, unparseable) — a health readout must never fail the build.
pub async fn fetch(client: &reqwest::Client, url: &str) -> Option<Scores> {
    let key = std::env::var("PAGESPEED_API_KEY").ok();
    let mut params: Vec<(&str, &str)> = vec![
        ("url", url),
        ("strategy", "mobile"),
        ("category", "performance"),
        ("category", "accessibility"),
        ("category", "best-practices"),
        ("category", "seo"),
    ];
    if let Some(k) = key.as_deref().filter(|k| !k.is_empty()) {
        params.push(("key", k));
    }
    let resp = client
        .get("https://www.googleapis.com/pagespeedonline/v5/runPagespeed")
        .query(&params)
        .timeout(std::time::Duration::from_secs(120))
        .send()
        .await
        .ok()?;
    if !resp.status().is_success() {
        tracing::info!("PageSpeed: API returned {} — skipping scores", resp.status());
        return None;
    }
    let body = resp.text().await.ok()?;
    parse(&body)
}

/// shields.io endpoint color for a Lighthouse score (Lighthouse's own bands).
pub fn badge_color(score: u8) -> &'static str {
    if score >= 90 {
        "brightgreen"
    } else if score >= 50 {
        "orange"
    } else {
        "red"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_category_scores_to_percent() {
        let body = r#"{"lighthouseResult":{"categories":{
            "performance":{"score":0.97},
            "accessibility":{"score":1.0},
            "best-practices":{"score":0.92},
            "seo":{"score":0.8}
        }}}"#;
        let s = parse(body).expect("parsed");
        assert_eq!(s.performance, Some(97));
        assert_eq!(s.accessibility, Some(100));
        assert_eq!(s.best_practices, Some(92));
        assert_eq!(s.seo, Some(80));
        assert_eq!(s.entries().len(), 4);
    }

    #[test]
    fn missing_pieces_are_none_not_errors() {
        let s = parse(r#"{"lighthouseResult":{"categories":{"performance":{"score":0.5}}}}"#).unwrap();
        assert_eq!(s.performance, Some(50));
        assert_eq!(s.seo, None);
        assert_eq!(s.entries(), vec![("Performance", 50)]);
        // A response without lighthouseResult yields nothing.
        assert!(parse(r#"{"error":"bad"}"#).is_none());
        assert!(parse("not json").is_none());
    }

    #[test]
    fn badge_colors_track_lighthouse_bands() {
        assert_eq!(badge_color(95), "brightgreen");
        assert_eq!(badge_color(90), "brightgreen");
        assert_eq!(badge_color(75), "orange");
        assert_eq!(badge_color(40), "red");
    }
}
