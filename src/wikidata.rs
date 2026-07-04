//! Optional Wikidata enrichment. Given a QID (via `--wikidata` / `WIKIDATA`),
//! fetch the entity at build time and render a table of its statements on the
//! About page, linking both the properties and their item values back to
//! wikidata.org. Uses only public endpoints — no key, no auth.

use serde_json::Value as J;
use std::collections::{BTreeMap, HashSet};

/// A resolved entity ready to render: its label plus one row per statement.
pub struct Table {
    pub qid: String,
    pub label: String,
    pub rows: Vec<Row>,
}

pub struct Row {
    /// Property id, e.g. `P31`.
    pub prop: String,
    pub prop_label: String,
    pub value: Cell,
}

/// A rendered statement value.
pub enum Cell {
    /// Another Wikidata item — links to `/wiki/<id>`.
    Item { id: String, label: String },
    /// A URL-typed value — links to itself.
    Link { url: String },
    /// A plain literal (string, date, quantity, coordinate, …).
    Text(String),
}

/// Cap on rendered statements, so a heavily-described entity (e.g. a country)
/// doesn't produce a thousand-row table.
const MAX_ROWS: usize = 60;

const WIKI: &str = "https://www.wikidata.org/wiki";

impl Table {
    /// Render the entity as a standalone HTML `<table>` (raw HTML so it survives
    /// Zola's markdown and can be dropped into a post body or the About page).
    /// Both the property keys and item values link back to wikidata.org.
    pub fn to_html(&self, prop_header: &str, value_header: &str) -> String {
        let mut h = format!(
            "<figure class=\"wd\"><figcaption><a href=\"{WIKI}/{qid}\">{label}</a> \
             · <span class=\"wd-qid\">{qid}</span></figcaption>\
             <table><thead><tr><th>{ph}</th><th>{vh}</th></tr></thead><tbody>",
            qid = esc(&self.qid),
            label = esc(&self.label),
            ph = esc(prop_header),
            vh = esc(value_header),
        );
        for r in &self.rows {
            let key = format!(
                "<a href=\"{WIKI}/Property:{p}\">{lbl}</a>",
                p = esc(&r.prop),
                lbl = esc(&r.prop_label)
            );
            let val = match &r.value {
                Cell::Item { id, label } => {
                    format!("<a href=\"{WIKI}/{}\">{}</a>", esc(id), esc(label))
                }
                Cell::Link { url } => {
                    format!("<a href=\"{u}\">{u}</a>", u = esc(url))
                }
                Cell::Text(t) => esc(t),
            };
            h.push_str(&format!("<tr><td>{key}</td><td>{val}</td></tr>"));
        }
        h.push_str("</tbody></table></figure>");
        h
    }
}

/// Wrap a rendered table in a no-JS click-to-expand `<details>` with an emoji
/// summary, for `WIKIDATA_SPOILER`. Collapsed by default; the whole table
/// (links and all) reveals on click, no script involved.
pub fn spoiler(html: &str) -> String {
    format!("<details class=\"wd-spoiler\"><summary>🔎 Wikidata</summary>{html}</details>")
}

/// Minimal HTML escaping for text dropped into the table.
fn esc(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
}

/// `Q42`/`q42`/`Q42 ` → `Q42`; anything that isn't a bare item id → `None`.
fn normalize_qid(raw: &str) -> Option<String> {
    let t = raw.trim().to_ascii_uppercase();
    let t = t.strip_prefix("Q")?;
    (!t.is_empty() && t.bytes().all(|b| b.is_ascii_digit())).then(|| format!("Q{t}"))
}

/// Fetch the entity and its statements, resolving property/item labels in the
/// site language (falling back to the id). `None` on any network/parse failure.
pub async fn fetch(client: &reqwest::Client, raw_qid: &str, lang: &str) -> Option<Table> {
    let qid = normalize_qid(raw_qid)?;
    let url = format!("https://www.wikidata.org/wiki/Special:EntityData/{qid}.json");
    let json: J = client
        .get(&url)
        .send()
        .await
        .and_then(|r| r.error_for_status())
        .ok()?
        .json()
        .await
        .ok()?;
    let entity = json.get("entities")?.get(&qid)?;
    let (mut rows, mut ids) = parse_entity(entity);
    if rows.is_empty() {
        return None;
    }
    // Resolve labels for every property and item id referenced (plus the entity
    // itself), in one or a few batched calls.
    ids.insert(qid.clone());
    let labels = fetch_labels(client, &ids, lang).await;
    let label_of = |id: &str| labels.get(id).cloned().unwrap_or_else(|| id.to_string());
    for r in &mut rows {
        r.prop_label = label_of(&r.prop);
        if let Cell::Item { id, label } = &mut r.value {
            *label = label_of(id);
        }
    }
    Some(Table { label: label_of(&qid), qid, rows })
}

/// Parse an entity's `claims` into rows (first statement per property) plus the
/// set of property/item ids whose labels we still need to resolve.
fn parse_entity(entity: &J) -> (Vec<Row>, HashSet<String>) {
    let mut ids = HashSet::new();
    // BTreeMap keyed by the numeric part → stable, human-friendly P1, P2, … order.
    let mut ordered: BTreeMap<u64, Row> = BTreeMap::new();
    let Some(claims) = entity.get("claims").and_then(J::as_object) else {
        return (Vec::new(), ids);
    };
    for (prop, statements) in claims {
        let Some(main) = statements
            .as_array()
            .and_then(|a| a.first())
            .and_then(|s| s.get("mainsnak"))
        else {
            continue;
        };
        if main.get("snaktype").and_then(J::as_str) != Some("value") {
            continue; // "novalue" / "somevalue" — nothing to show
        }
        let Some(cell) = snak_cell(main) else { continue };
        if let Cell::Item { id, .. } = &cell {
            ids.insert(id.clone());
        }
        ids.insert(prop.clone());
        if let Some(n) = prop.strip_prefix('P').and_then(|d| d.parse::<u64>().ok()) {
            ordered.insert(
                n,
                Row { prop: prop.clone(), prop_label: prop.clone(), value: cell },
            );
        }
    }
    let rows: Vec<Row> = ordered.into_values().take(MAX_ROWS).collect();
    (rows, ids)
}

/// Turn a mainsnak's datavalue into a renderable cell.
fn snak_cell(main: &J) -> Option<Cell> {
    let datatype = main.get("datatype").and_then(J::as_str).unwrap_or("");
    let dv = main.get("datavalue")?;
    let value = dv.get("value")?;
    match dv.get("type").and_then(J::as_str)? {
        "wikibase-entityid" => {
            let id = value.get("id").and_then(J::as_str)?.to_string();
            Some(Cell::Item { label: id.clone(), id })
        }
        "string" => {
            let s = value.as_str()?.to_string();
            if datatype == "url" {
                Some(Cell::Link { url: s })
            } else {
                Some(Cell::Text(s))
            }
        }
        "monolingualtext" => Some(Cell::Text(value.get("text")?.as_str()?.to_string())),
        "time" => Some(Cell::Text(format_time(value.get("time")?.as_str()?))),
        "quantity" => {
            let amount = value.get("amount")?.as_str()?.trim_start_matches('+').to_string();
            Some(Cell::Text(amount))
        }
        "globecoordinate" => {
            let lat = value.get("latitude")?.as_f64()?;
            let lon = value.get("longitude")?.as_f64()?;
            Some(Cell::Text(format!("{lat:.5}, {lon:.5}")))
        }
        _ => None,
    }
}

/// `+1952-03-11T00:00:00Z` → `1952-03-11`; a year-precision `+2001-00-00T…` →
/// `2001`. Best-effort: on anything unexpected, return the date part as-is.
fn format_time(t: &str) -> String {
    let core = t.trim_start_matches('+');
    let date = core.split('T').next().unwrap_or(core);
    match date.split('-').collect::<Vec<_>>().as_slice() {
        [y, m, d] if *m == "00" || *d == "00" => y.to_string(),
        _ => date.to_string(),
    }
}

/// Batch-resolve labels for `ids` in `lang` (falling back to English, then the
/// id). Chunks to stay within the API's 50-ids-per-request limit.
async fn fetch_labels(
    client: &reqwest::Client,
    ids: &HashSet<String>,
    lang: &str,
) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    let all: Vec<&String> = ids.iter().collect();
    for chunk in all.chunks(50) {
        let joined = chunk.iter().map(|s| s.as_str()).collect::<Vec<_>>().join("|");
        let url = format!(
            "https://www.wikidata.org/w/api.php?action=wbgetentities&ids={joined}\
             &props=labels&languages={lang}|en&format=json"
        );
        let Ok(json) = async {
            client
                .get(&url)
                .send()
                .await
                .and_then(|r| r.error_for_status())?
                .json::<J>()
                .await
        }
        .await
        else {
            continue;
        };
        let Some(entities) = json.get("entities").and_then(J::as_object) else {
            continue;
        };
        for (id, ent) in entities {
            let labels = ent.get("labels");
            let pick = |k: &str| {
                labels
                    .and_then(|l| l.get(k))
                    .and_then(|l| l.get("value"))
                    .and_then(J::as_str)
                    .map(str::to_string)
            };
            if let Some(v) = pick(lang).or_else(|| pick("en")) {
                out.insert(id.clone(), v);
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_qids() {
        assert_eq!(normalize_qid(" q42 ").as_deref(), Some("Q42"));
        assert_eq!(normalize_qid("Q5").as_deref(), Some("Q5"));
        assert_eq!(normalize_qid("P31"), None);
        assert_eq!(normalize_qid("Q"), None);
        assert_eq!(normalize_qid("douglas"), None);
    }

    #[test]
    fn formats_times() {
        assert_eq!(format_time("+1952-03-11T00:00:00Z"), "1952-03-11");
        assert_eq!(format_time("+2001-00-00T00:00:00Z"), "2001");
    }

    #[test]
    fn parses_claims_into_ordered_rows() {
        let entity = serde_json::json!({
            "claims": {
                "P31": [{ "mainsnak": { "snaktype": "value", "datatype": "wikibase-item",
                    "datavalue": { "type": "wikibase-entityid", "value": { "id": "Q5" } } } }],
                "P856": [{ "mainsnak": { "snaktype": "value", "datatype": "url",
                    "datavalue": { "type": "string", "value": "https://example.org" } } }],
                "P1477": [{ "mainsnak": { "snaktype": "value", "datatype": "string",
                    "datavalue": { "type": "string", "value": "Douglas Noel Adams" } } }],
                "P569": [{ "mainsnak": { "snaktype": "value", "datatype": "time",
                    "datavalue": { "type": "time", "value": { "time": "+1952-03-11T00:00:00Z" } } } }],
                "P1000": [{ "mainsnak": { "snaktype": "somevalue" } }]
            }
        });
        let (rows, ids) = parse_entity(&entity);
        // Ordered by property number: P31, P569, P856, P1477. somevalue skipped.
        let props: Vec<&str> = rows.iter().map(|r| r.prop.as_str()).collect();
        assert_eq!(props, ["P31", "P569", "P856", "P1477"]);
        assert!(ids.contains("Q5") && ids.contains("P31"));
        assert!(matches!(&rows[0].value, Cell::Item { id, .. } if id == "Q5"));
        assert!(matches!(&rows[2].value, Cell::Link { url } if url == "https://example.org"));
        assert!(matches!(&rows[1].value, Cell::Text(t) if t == "1952-03-11"));
    }
}
