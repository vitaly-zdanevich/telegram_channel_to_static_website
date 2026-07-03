//! Offline mode + installable PWA (opt-in `--offline` / `OFFLINE`). Ships a
//! service worker (precaches the whole archive on any non-cellular connection,
//! then serves cache-first) and a web app manifest (`display: standalone` hides
//! the browser chrome, so the
//! site is installable). All paths are relative, so it works under any base URL
//! without baking one in.

use anyhow::Result;
use std::fs;
use std::path::Path;

/// The service worker source (kept in a real `.js` file, Biome-linted in CI).
pub const SW_JS: &str = include_str!("sw.js");

/// The web app manifest. Paths are relative to the manifest's own location (the
/// site root), so no base_url is needed. An icon is added when the channel avatar
/// is present.
pub fn manifest_json(title: &str, background: &str, has_avatar: bool) -> String {
    let icons = if has_avatar {
        r#","icons":[{"src":"channel-avatar.jpg","sizes":"512x512","type":"image/jpeg","purpose":"any"}]"#
    } else {
        ""
    };
    format!(
        r#"{{"name":{t},"short_name":{t},"start_url":"./","scope":"./","display":"standalone","background_color":{bg},"theme_color":{bg}{icons}}}"#,
        t = json_str(title),
        bg = json_str(background),
    )
}

fn json_str(s: &str) -> String {
    serde_json::to_string(s).unwrap_or_else(|_| "\"\"".to_string())
}

/// Walk a built `public/` tree and write `asset-manifest.json`: every URL to
/// precache, relative to the site root. Directory `index.html` files become
/// their directory URL so cache keys match page requests. Returns the count. Run
/// after `zola build` (`tg2zola pwa <public>`).
pub fn write_asset_manifest(public: &Path) -> Result<usize> {
    anyhow::ensure!(public.is_dir(), "{} is not a directory", public.display());
    let mut urls = Vec::new();
    collect(public, public, &mut urls)?;
    urls.sort();
    urls.dedup();
    fs::write(public.join("asset-manifest.json"), serde_json::to_string(&urls)?)?;
    Ok(urls.len())
}

fn collect(root: &Path, dir: &Path, urls: &mut Vec<String>) -> Result<()> {
    for entry in fs::read_dir(dir)? {
        let path = entry?.path();
        if path.is_dir() {
            collect(root, &path, urls)?;
            continue;
        }
        let rel = path
            .strip_prefix(root)
            .unwrap_or(&path)
            .to_string_lossy()
            .replace('\\', "/");
        // The SW machinery itself needn't be precached.
        if rel == "asset-manifest.json" || rel == "sw.js" {
            continue;
        }
        let url = match rel.strip_suffix("index.html") {
            // "posts/1/index.html" → "posts/1/"; the root "index.html" → "./".
            Some("") => "./".to_string(),
            Some(dir) => dir.to_string(),
            None => rel,
        };
        urls.push(url);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manifest_is_valid_relative_json() {
        let m = manifest_json("My \"Channel\"", "#000000", true);
        let v: serde_json::Value = serde_json::from_str(&m).expect("valid json");
        assert_eq!(v["display"], "standalone");
        assert_eq!(v["start_url"], "./");
        assert_eq!(v["name"], "My \"Channel\""); // quotes escaped, not broken
        assert_eq!(v["icons"][0]["src"], "channel-avatar.jpg");
        // No avatar → no icons array.
        let no_icon = manifest_json("x", "#fff", false);
        assert!(!no_icon.contains("icons"));
    }

    #[test]
    fn asset_manifest_maps_index_to_dir_urls() {
        let dir = std::env::temp_dir().join(format!("tg2zola-pwa-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("posts/7")).unwrap();
        fs::write(dir.join("index.html"), "x").unwrap();
        fs::write(dir.join("style.css"), "x").unwrap();
        fs::write(dir.join("posts/7/index.html"), "x").unwrap();
        fs::write(dir.join("posts/7/01.mp4"), "x").unwrap();
        fs::write(dir.join("sw.js"), "x").unwrap();

        let n = write_asset_manifest(&dir).unwrap();
        let urls: Vec<String> =
            serde_json::from_str(&fs::read_to_string(dir.join("asset-manifest.json")).unwrap()).unwrap();
        assert_eq!(n, urls.len());
        assert!(urls.contains(&"./".to_string()), "{urls:?}");
        assert!(urls.contains(&"posts/7/".to_string()), "{urls:?}");
        assert!(urls.contains(&"posts/7/01.mp4".to_string()), "{urls:?}");
        assert!(urls.contains(&"style.css".to_string()), "{urls:?}");
        // sw.js and the manifest itself are excluded.
        assert!(!urls.iter().any(|u| u == "sw.js"), "{urls:?}");
        assert!(!urls.iter().any(|u| u.contains("asset-manifest")), "{urls:?}");
        let _ = fs::remove_dir_all(&dir);
    }
}
