//! Optional MTProto backend (Cargo feature `mtproto`).
//!
//! The public `t.me/s/` preview never exposes voice/audio notes and serves only
//! a size-limited photo. Logging in as a *user* over MTProto (via `grammers`)
//! recovers both. This is strictly opt-in: it needs `TG_API_ID` + `TG_API_HASH`
//! and a session (`TG_SESSION` base64, or a `tg2zola.session` file created once
//! by `tg2zola login`). Without those, [`maybe_enrich`] keeps using the public
//! scraper and may reuse originals already confirmed in GitHub Releases.
#![allow(deprecated)]

use anyhow::{anyhow, bail, Context, Result};
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use base64::Engine;
use grammers_client::grammers_tl_types as tl;
use grammers_client::types::{Media as TlMedia, Peer};
use grammers_client::{Client, SignInError};
use grammers_mtsender::SenderPool;
use grammers_session::defs::{PeerId, PeerRef};
use grammers_session::storages::TlSession;

use crate::config::Settings;
use crate::model::{Media, Post};

/// Default on-disk session file (created by `tg2zola login`).
const SESSION_FILE: &str = "tg2zola.session";

/// Confirmed GitHub Release images, persisted on the `blog` branch only after
/// the workflow uploads every newly staged asset successfully.
const IMAGE_RELEASE_MANIFEST: &str = ".image-releases.json";
/// Candidate manifest written by enrichment and promoted by CI after upload.
const IMAGE_RELEASE_PENDING: &str = ".image-releases.pending.json";
/// Ephemeral inventory produced from GitHub's API before generation.
const IMAGE_RELEASE_INVENTORY: &str = ".image-releases.remote.tsv";
/// Ephemeral tree of `<release-tag>/<asset>` files for the CI upload step.
const IMAGE_RELEASE_STAGING: &str = ".image-releases";
/// GitHub permits 1,000 assets per release. A 500-message range has at most 500
/// media items, leaving headroom for future companion assets.
const IMAGE_RELEASE_BUCKET: i32 = 500;
/// Bound the first historical backfill so a six-hour hosted CI job can publish
/// completed work instead of timing out and restarting every image from zero.
const IMAGE_RELEASE_BACKFILL_PER_RUN: usize = 500;
const IMAGE_RELEASE_MANIFEST_VERSION: u8 = 2;
/// A stuck Telegram media request must not consume the rest of a hosted job.
const IMAGE_DOWNLOAD_TIMEOUT: Duration = Duration::from_secs(120);
/// Likewise, preserve an already-staged batch if walking channel history stalls.
const MESSAGE_ITERATION_TIMEOUT: Duration = Duration::from_secs(120);

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct ReleasedImage {
    message_id: i32,
    media_id: i64,
    declared_bytes: i64,
    tag: String,
    asset: String,
    bytes: u64,
    /// Identity from the current public preview. Records without one can be
    /// reused with live MTProto metadata, but never guessed into a web slot.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    source: Option<ReleasedImageSource>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum ReleasedImageSource {
    Photo { web_key: String },
    Document { filename: String },
}

impl ReleasedImage {
    fn matches(&self, media_id: i64, target: &ImageReleaseTarget) -> bool {
        self.media_id == media_id && self.tag == target.tag && self.asset == target.asset
    }

    fn target(&self, repo: &str) -> Option<ImageReleaseTarget> {
        let ext = self.asset.rsplit_once('.')?.1;
        if !matches!(
            ext,
            "jpg" | "jpeg" | "png" | "gif" | "webp" | "avif" | "bmp"
        ) {
            return None;
        }
        let target = image_release_target(repo, self.message_id, self.media_id, ext);
        (self.tag == target.tag && self.asset == target.asset).then_some(target)
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct ImageReleaseManifest {
    version: u8,
    channel: String,
    images: Vec<ReleasedImage>,
}

#[derive(Debug)]
struct ImageReleaseTarget {
    tag: String,
    asset: String,
    url: String,
}

enum ArchivedImage {
    Local(PathBuf),
    Release(String),
    Deferred,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CappedBackfillAction {
    Process,
    ProcessThenStop,
    Skip,
    Stop,
}

/// Drain only the still-unseen IDs of the grouped post that exhausted the
/// backfill cap. Telegram iterates IDs downward, but grouped IDs need not be
/// contiguous, so unrelated intervening messages are skipped rather than used
/// as a reason to stop early.
struct CappedPostDrain {
    remaining_ids: BTreeSet<i32>,
}

impl CappedPostDrain {
    fn after_message(grouped_ids: &[u64], current_id: i32) -> Option<Self> {
        let remaining_ids = grouped_ids
            .iter()
            .map(|id| *id as i32)
            .filter(|id| *id < current_id)
            .collect::<BTreeSet<_>>();
        (!remaining_ids.is_empty()).then_some(Self { remaining_ids })
    }

    fn action(&mut self, message_id: i32) -> CappedBackfillAction {
        // Any larger expected ID has already been passed by the descending
        // iterator and is unavailable; keep looking for lower grouped IDs.
        self.remaining_ids.retain(|known| *known <= message_id);
        if self.remaining_ids.remove(&message_id) {
            if self.remaining_ids.is_empty() {
                CappedBackfillAction::ProcessThenStop
            } else {
                CappedBackfillAction::Process
            }
        } else if self.remaining_ids.is_empty() {
            CappedBackfillAction::Stop
        } else {
            CappedBackfillAction::Skip
        }
    }
}

impl ArchivedImage {
    fn into_media(self, key: Option<String>) -> Option<Media> {
        match self {
            Self::Local(path) => Some(Media::LocalPhoto { path, key }),
            Self::Release(url) => Some(Media::ReleasePhoto { url }),
            Self::Deferred => None,
        }
    }
}

fn api_id() -> Result<i32> {
    std::env::var("TG_API_ID")
        .context("TG_API_ID not set")?
        .trim()
        .parse()
        .context("TG_API_ID must be an integer")
}

fn api_hash() -> Result<String> {
    Ok(std::env::var("TG_API_HASH")
        .context("TG_API_HASH not set")?
        .trim()
        .to_string())
}

fn session_file() -> PathBuf {
    std::env::var("TG_SESSION_FILE")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(SESSION_FILE))
}

/// Load a session: `TG_SESSION` (base64) if set, else the session file, else new.
fn load_session() -> Result<TlSession> {
    if let Ok(s) = std::env::var("TG_SESSION") {
        let s = s.trim();
        if !s.is_empty() {
            let bytes = base64::engine::general_purpose::STANDARD
                .decode(s)
                .context("TG_SESSION is not valid base64")?;
            return TlSession::load(&bytes).map_err(|e| anyhow!("loading TG_SESSION: {e:?}"));
        }
    }
    let p = session_file();
    if p.exists() {
        TlSession::load_file(&p).with_context(|| format!("loading session {}", p.display()))
    } else {
        Ok(TlSession::new())
    }
}

/// Build a connected client (background runner spawned) plus the shared session
/// handle, which the runner writes auth/DC data into and `login` persists.
fn build_client() -> Result<(Client, Arc<TlSession>)> {
    let session = Arc::new(load_session()?);
    let pool = SenderPool::new(Arc::clone(&session), api_id()?);
    let client = Client::new(&pool);
    let SenderPool { runner, .. } = pool;
    // Detached: drives I/O for the lifetime of this run.
    tokio::spawn(runner.run());
    Ok((client, session))
}

fn prompt(msg: &str) -> Result<String> {
    print!("{msg}");
    std::io::stdout().flush()?;
    let mut s = String::new();
    std::io::stdin().read_line(&mut s)?;
    Ok(s.trim().to_string())
}

/// `tg2zola login` — one-time interactive login. Saves the session file and
/// prints the base64 `TG_SESSION` string for a GitHub Actions secret.
pub async fn login() -> Result<()> {
    let (client, session) = build_client()?;
    if !client.is_authorized().await? {
        let phone = match std::env::var("TG_PHONE") {
            Ok(p) if !p.trim().is_empty() => p.trim().to_string(),
            _ => prompt("Phone number (international, e.g. +12025550123): ")?,
        };
        let hash = api_hash()?;
        let token = client
            .request_login_code(&phone, &hash)
            .await
            .context("requesting login code")?;
        let code = prompt("Login code (sent to you in Telegram): ")?;
        match client.sign_in(&token, &code).await {
            Ok(_) => {}
            Err(SignInError::PasswordRequired(password_token)) => {
                let hint = password_token.hint().unwrap_or("");
                let pw = rpassword::prompt_password(format!("2FA password (hint: {hint}): "))
                    .context("reading 2FA password")?;
                client
                    .check_password(password_token, pw.trim())
                    .await
                    .context("checking 2FA password")?;
            }
            Err(e) => return Err(anyhow!("sign in failed: {e}")),
        }
        println!("Logged in.");
    } else {
        println!("Already authorized (existing session).");
    }

    // NB: grammers' `TlSession::save_to_file` opens write-only without `create`,
    // so it fails when the file doesn't exist yet. Write the bytes ourselves.
    let p = session_file();
    let bytes = session.save();
    std::fs::write(&p, &bytes).with_context(|| format!("saving session to {}", p.display()))?;
    let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
    println!("\nSession saved to {}", p.display());
    println!("\nStore this as the TG_SESSION secret for CI:\n{b64}\n");
    Ok(())
}

/// Best-effort enrichment. Skips silently when MTProto isn't configured; logs a
/// warning (and continues with the web-only result) on any failure — a backup
/// run must never abort because the optional backend had a problem. Returns
/// whether MTProto actually enriched the posts (surfaced on the About page).
pub async fn maybe_enrich(posts: &mut [Post], s: &Settings) -> bool {
    if std::env::var("TG_API_ID").is_err() || std::env::var("TG_API_HASH").is_err() {
        return reuse_released_images(posts, s) > 0;
    }
    let mut extras = String::new();
    if want_photos() {
        extras.push_str(" + original photos");
    }
    if want_videos() {
        extras.push_str(" + videos");
    }
    if want_files() {
        extras.push_str(" + attachments");
    }
    tracing::info!("MTProto: configured — fetching audio{extras}");
    match enrich(posts, s).await {
        Ok(()) => true,
        Err(e) => {
            tracing::warn!("MTProto enrichment skipped: {:#}", e);
            reuse_released_images(posts, s) > 0
        }
    }
}

/// Original photos and image documents are archived by default. Keeping the
/// environment switch as an opt-out lets storage-constrained deployments turn
/// them off without making lossy web-preview images the archival default.
fn want_photos() -> bool {
    let value = std::env::var("MTPROTO_IMAGES").ok();
    photos_enabled(value.as_deref())
}

fn photos_enabled(value: Option<&str>) -> bool {
    !matches!(
        value.map(str::trim),
        Some("0") | Some("false") | Some("no") | Some("off")
    )
}

/// GitHub repository used for Release-backed originals. The existing
/// `VIDEO_RELEASES`/`--no-video-releases` switch remains the backwards-compatible
/// control for all media kept outside the published Pages tree. Restrict this to
/// GitHub Actions, where the bundled workflow can actually upload the staging
/// tree; a local generation keeps its originals in the local site instead.
fn image_release_repo(s: &Settings) -> Option<&str> {
    (s.video_releases
        && s.repo_url.contains("github.com")
        && std::env::var_os("GITHUB_ACTIONS").is_some())
    .then(|| s.repo_url.trim_end_matches('/'))
}

fn image_release_target(
    repo: &str,
    message_id: i32,
    media_id: i64,
    ext: &str,
) -> ImageReleaseTarget {
    let bucket = message_id.div_euclid(IMAGE_RELEASE_BUCKET) * IMAGE_RELEASE_BUCKET;
    let tag = format!("images-{bucket:04}");
    let asset = format!("telegram-image-{message_id}-{media_id}.{ext}");
    let url = format!(
        "{}/releases/download/{tag}/{asset}",
        repo.trim_end_matches('/')
    );
    ImageReleaseTarget { tag, asset, url }
}

type ImageReleaseInventory = BTreeMap<(String, String), u64>;

/// Load the workflow's API-derived view of remote Release assets. The manifest
/// is only a claim; this inventory is the proof that an asset of the expected
/// size currently exists before generated pages may link to it.
fn load_image_release_inventory(site: &Path) -> ImageReleaseInventory {
    let path = site.join(IMAGE_RELEASE_INVENTORY);
    let Ok(raw) = std::fs::read_to_string(&path) else {
        return BTreeMap::new();
    };
    raw.lines()
        .filter_map(|line| {
            let mut fields = line.split('\t');
            let tag = fields.next()?;
            let asset = fields.next()?;
            let bytes = fields.next()?.parse().ok()?;
            (fields.next().is_none() && !tag.is_empty() && !asset.is_empty())
                .then(|| ((tag.to_string(), asset.to_string()), bytes))
        })
        .collect()
}

fn image_is_in_inventory(image: &ReleasedImage, inventory: &ImageReleaseInventory) -> bool {
    inventory
        .get(&(image.tag.clone(), image.asset.clone()))
        .is_some_and(|bytes| *bytes == image.bytes)
}

fn load_image_release_manifest(site: &Path, channel: &str) -> BTreeMap<i32, ReleasedImage> {
    let path = site.join(IMAGE_RELEASE_MANIFEST);
    let Ok(raw) = std::fs::read_to_string(&path) else {
        return BTreeMap::new();
    };
    let manifest: ImageReleaseManifest = match serde_json::from_str(&raw) {
        Ok(manifest) => manifest,
        Err(error) => {
            tracing::warn!("ignoring malformed {}: {error}", path.display());
            return BTreeMap::new();
        }
    };
    if manifest.version != IMAGE_RELEASE_MANIFEST_VERSION || manifest.channel != channel {
        tracing::warn!("ignoring {} for version/channel mismatch", path.display());
        return BTreeMap::new();
    }
    let inventory = load_image_release_inventory(site);
    let total = manifest.images.len();
    let images: BTreeMap<_, _> = manifest
        .images
        .into_iter()
        .filter(|image| image_is_in_inventory(image, &inventory))
        .map(|image| (image.message_id, image))
        .collect();
    if images.len() != total {
        tracing::warn!(
            "ignoring {} image Release record(s) missing from the remote inventory",
            total - images.len()
        );
    }
    images
}

/// Replace only the exact web source recorded when this original was archived.
/// Never append or fall back to position: either would resurrect removed media
/// or shift originals onto the wrong slots in an edited album.
fn apply_released_image(
    media: &mut [Media],
    record: &ReleasedImage,
    target: ImageReleaseTarget,
) -> bool {
    let index = match record.source.as_ref() {
        Some(ReleasedImageSource::Photo { web_key }) => media
            .iter()
            .position(|item| matches!(item, Media::Photo { key: Some(key), .. } if key == web_key)),
        Some(ReleasedImageSource::Document { filename }) => media.iter().position(|item| {
            matches!(item, Media::DocumentRef { filename: current, message_id }
                if crate::media::is_probably_image_doc(current)
                    && current == filename
                    && *message_id == Some(record.message_id as u64))
        }),
        None => None,
    };
    let Some(index) = index else { return false };
    media[index] = Media::ReleasePhoto { url: target.url };
    true
}

fn reuse_released_images(posts: &mut [Post], s: &Settings) -> usize {
    if !want_photos() {
        return 0;
    }
    let Some(repo) = image_release_repo(s) else {
        return 0;
    };
    let releases = load_image_release_manifest(&s.site, &s.channel);
    let mut reused = 0;
    for post in posts {
        let mut records: Vec<_> = post
            .ids
            .iter()
            .filter_map(|id| {
                let message_id = *id as i32;
                releases.get(&message_id).and_then(|record| {
                    record
                        .target(repo)
                        .map(|target| (message_id, record, target))
                })
            })
            .collect();
        records.sort_by_key(|(message_id, _, _)| *message_id);
        for (_, record, target) in records {
            reused += usize::from(apply_released_image(&mut post.media, record, target));
        }
    }
    if reused > 0 {
        tracing::info!(
            "MTProto: reused {reused} original image(s) from GitHub Releases without a live session"
        );
    }
    reused
}

fn write_pending_image_release_manifest(
    site: &Path,
    channel: &str,
    images: &BTreeMap<i32, ReleasedImage>,
) -> Result<()> {
    let manifest = ImageReleaseManifest {
        version: IMAGE_RELEASE_MANIFEST_VERSION,
        channel: channel.to_string(),
        images: images.values().cloned().collect(),
    };
    let path = site.join(IMAGE_RELEASE_PENDING);
    let part = site.join(format!("{IMAGE_RELEASE_PENDING}.part"));
    let mut bytes = serde_json::to_vec_pretty(&manifest)?;
    bytes.push(b'\n');
    std::fs::write(&part, bytes).with_context(|| format!("writing {}", part.display()))?;
    std::fs::rename(&part, &path).with_context(|| format!("promoting {}", path.display()))?;
    Ok(())
}

/// Total bytes of Release-backed original images referenced by the persisted or
/// pending manifest. Used by the About page without re-downloading those assets.
pub fn released_image_bytes(site: &Path) -> u64 {
    if let Ok(raw) = std::fs::read_to_string(site.join(IMAGE_RELEASE_PENDING)) {
        if let Ok(manifest) = serde_json::from_str::<ImageReleaseManifest>(&raw) {
            return manifest.images.iter().map(|image| image.bytes).sum();
        }
    }
    let Ok(raw) = std::fs::read_to_string(site.join(IMAGE_RELEASE_MANIFEST)) else {
        return 0;
    };
    let Ok(manifest) = serde_json::from_str::<ImageReleaseManifest>(&raw) else {
        return 0;
    };
    let inventory = load_image_release_inventory(site);
    manifest
        .images
        .iter()
        .filter(|image| image_is_in_inventory(image, &inventory))
        .map(|image| image.bytes)
        .sum()
}

async fn stage_image_release(source: &Path, destination: &Path) -> Result<()> {
    if let Some(parent) = destination.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    let _ = tokio::fs::remove_file(destination).await;
    if tokio::fs::hard_link(source, destination).await.is_ok() {
        return Ok(());
    }
    let part = destination.with_extension(format!(
        "{}.part",
        destination
            .extension()
            .and_then(|extension| extension.to_str())
            .unwrap_or("image")
    ));
    tokio::fs::copy(source, &part).await?;
    tokio::fs::rename(&part, destination).await?;
    Ok(())
}

/// Download atomically so a cancelled or timed-out request cannot leave a
/// partial file that a later attempt mistakes for a complete cached original.
async fn download_image_to_cache(
    client: &Client,
    media: &TlMedia,
    destination: &Path,
    message_id: i32,
    declared_bytes: i64,
) -> Result<()> {
    if let Ok(metadata) = tokio::fs::metadata(destination).await {
        let bytes = metadata.len();
        if bytes > 0 && (declared_bytes <= 0 || bytes == declared_bytes as u64) {
            return Ok(());
        }
        tracing::warn!(
            "message {message_id}: discarding incomplete image cache ({} bytes, expected {declared_bytes})",
            bytes
        );
        let _ = tokio::fs::remove_file(destination).await;
    }
    let part = destination.with_extension(format!(
        "{}.part",
        destination
            .extension()
            .and_then(|extension| extension.to_str())
            .unwrap_or("image")
    ));
    let _ = tokio::fs::remove_file(&part).await;
    let result =
        tokio::time::timeout(IMAGE_DOWNLOAD_TIMEOUT, client.download_media(media, &part)).await;
    match result {
        Ok(Ok(())) => {}
        Ok(Err(error)) => {
            let _ = tokio::fs::remove_file(&part).await;
            return Err(error)
                .with_context(|| format!("downloading original image from message {message_id}"));
        }
        Err(_) => {
            let _ = tokio::fs::remove_file(&part).await;
            bail!(
                "downloading original image from message {message_id} timed out after {} seconds",
                IMAGE_DOWNLOAD_TIMEOUT.as_secs()
            );
        }
    }
    let downloaded_bytes = tokio::fs::metadata(&part).await?.len();
    if downloaded_bytes == 0 || (declared_bytes > 0 && downloaded_bytes != declared_bytes as u64) {
        let _ = tokio::fs::remove_file(&part).await;
        bail!(
            "message {message_id}: downloaded {downloaded_bytes} image bytes, expected {declared_bytes}"
        );
    }
    tokio::fs::rename(&part, destination)
        .await
        .with_context(|| format!("finishing original image from message {message_id}"))?;
    Ok(())
}

/// Best-effort cache write for optional non-image MTProto media. A failure is
/// isolated to that attachment so it cannot discard an already-staged image
/// batch, and the temporary path prevents a partial file being reused later.
async fn download_optional_media_to_cache(
    client: &Client,
    media: &TlMedia,
    destination: &Path,
    message_id: i32,
    kind: &str,
) -> bool {
    if destination.exists() {
        return true;
    }
    let part = destination.with_extension(format!(
        "{}.part",
        destination
            .extension()
            .and_then(|extension| extension.to_str())
            .unwrap_or("media")
    ));
    let _ = tokio::fs::remove_file(&part).await;
    if let Err(error) = client.download_media(media, &part).await {
        let _ = tokio::fs::remove_file(&part).await;
        tracing::warn!("message {message_id}: optional {kind} skipped: {error}");
        return false;
    }
    if let Err(error) = tokio::fs::rename(&part, destination).await {
        let _ = tokio::fs::remove_file(&part).await;
        tracing::warn!("message {message_id}: could not finish optional {kind}: {error}");
        return false;
    }
    true
}

#[allow(clippy::too_many_arguments)]
async fn archive_image(
    client: &Client,
    media: &TlMedia,
    s: &Settings,
    cache: &Path,
    repo: Option<&str>,
    releases: &mut BTreeMap<i32, ReleasedImage>,
    manifest_dirty: &mut bool,
    release_downloads_left: &mut usize,
    message_id: i32,
    media_id: i64,
    declared_bytes: i64,
    ext: &str,
    source: Option<ReleasedImageSource>,
) -> Result<ArchivedImage> {
    if let Some(repo) = repo {
        if !release_image_size_allowed(declared_bytes) {
            bail!("message {message_id}: original image is at/over the 2 GiB Release limit");
        }
        let target = image_release_target(repo, message_id, media_id, ext);
        if let Some(record) = releases.get_mut(&message_id) {
            if record.matches(media_id, &target) {
                if record.declared_bytes != declared_bytes || record.source != source {
                    record.declared_bytes = declared_bytes;
                    record.source = source;
                    *manifest_dirty = true;
                }
                return Ok(ArchivedImage::Release(target.url));
            }
        }
        // Live MTProto has proved that this message now carries different media.
        // Remove the stale claim even when this run has no backfill slot left.
        if releases.remove(&message_id).is_some() {
            *manifest_dirty = true;
        }
        if *release_downloads_left == 0 {
            return Ok(ArchivedImage::Deferred);
        }

        let cached = cache.join(format!("image-{media_id}.{ext}"));
        download_image_to_cache(client, media, &cached, message_id, declared_bytes).await?;
        let bytes = tokio::fs::metadata(&cached).await?.len();
        if bytes >= MAX_FILE_BYTES as u64 {
            bail!("message {message_id}: downloaded original image is at/over the 2 GiB Release limit");
        }
        let staged = s
            .site
            .join(IMAGE_RELEASE_STAGING)
            .join(&target.tag)
            .join(&target.asset);
        stage_image_release(&cached, &staged)
            .await
            .with_context(|| format!("staging original image from message {message_id}"))?;
        releases.insert(
            message_id,
            ReleasedImage {
                message_id,
                media_id,
                declared_bytes,
                tag: target.tag,
                asset: target.asset,
                bytes,
                source,
            },
        );
        *manifest_dirty = true;
        *release_downloads_left -= 1;
        return Ok(ArchivedImage::Release(target.url));
    }

    let path = cache.join(format!("image-{media_id}.{ext}"));
    download_image_to_cache(client, media, &path, message_id, declared_bytes).await?;
    Ok(ArchivedImage::Local(path))
}

/// On by default: download the *original video* for posts the web preview shows
/// only as a poster (no downloadable file), unless a YouTube/Instagram embed
/// stands in for it. Disable with `MTPROTO_VIDEOS=false` — these can be large, so
/// a video-heavy channel may want it off to stay within the hosting budget.
fn want_videos() -> bool {
    !matches!(
        std::env::var("MTPROTO_VIDEOS").ok().as_deref(),
        Some("0") | Some("false") | Some("no") | Some("off")
    )
}

/// On by default: also download *every other* attachment (pdf, zip, rar, … and
/// images when `MTPROTO_IMAGES` is off) the web preview can't fetch, as a
/// downloadable file. Disable with `MTPROTO_FILES=false`. Large videos stay
/// behind `MTPROTO_VIDEOS` for the hosting budget, so they're never fetched here.
fn want_files() -> bool {
    !matches!(
        std::env::var("MTPROTO_FILES").ok().as_deref(),
        Some("0") | Some("false") | Some("no") | Some("off")
    )
}

/// Telegram attachments above this size are skipped, preserving the historical
/// behavior that accepted an exactly-2-GiB local file. Release-backed original
/// images use the stricter helper below because GitHub requires `< 2 GiB`.
const MAX_FILE_BYTES: i64 = 2 * 1024 * 1024 * 1024;

fn local_file_size_allowed(bytes: i64) -> bool {
    bytes <= MAX_FILE_BYTES
}

fn release_image_size_allowed(bytes: i64) -> bool {
    bytes < MAX_FILE_BYTES
}

/// Per-post-index MTProto audio: (cache path, original filename, label).
type AudioFor = HashMap<usize, Vec<(PathBuf, Option<String>, Option<String>)>>;

fn forwarded_channel_post(header: tl::enums::MessageFwdHeader) -> Option<(i64, i32)> {
    let tl::enums::MessageFwdHeader::Header(header) = header;
    let tl::enums::Peer::Channel(channel) = header.from_id? else {
        return None;
    };
    let post_id = header.channel_post.filter(|id| *id > 0)?;
    Some((channel.channel_id, post_id))
}

fn telegram_post_url(username: &str, post_id: i32) -> Option<String> {
    let username = username.trim().trim_start_matches('@');
    (!username.is_empty() && post_id > 0).then(|| format!("https://t.me/{username}/{post_id}"))
}

fn telegram_post_id(url: &str) -> Option<i32> {
    let after_scheme = url.split("//").nth(1)?;
    let mut segments = after_scheme.split('/');
    let host = segments.next()?.to_ascii_lowercase();
    if !matches!(host.as_str(), "t.me" | "telegram.me" | "telegram.dog") {
        return None;
    }
    segments
        .rfind(|segment| !segment.is_empty() && *segment != "s")?
        .split(['?', '#'])
        .next()?
        .parse()
        .ok()
}

fn pinterest_photo_is_hidden(
    pinterest_enabled: bool,
    keep_image: bool,
    has_pin: bool,
    pin_live: bool,
    pin_dead: bool,
    attached_photos: usize,
) -> bool {
    pinterest_enabled && !keep_image && has_pin && pin_live && !pin_dead && attached_photos == 1
}

/// A confirmed Pinterest widget replaces this sole photo at render time, so
/// downloading and publishing its MTProto original would only waste storage.
fn pinterest_hides_single_photo(post: &Post, s: &Settings) -> bool {
    let attached_photos = post
        .media
        .iter()
        .filter(|media| {
            matches!(
                media,
                Media::Photo { .. } | Media::LocalPhoto { .. } | Media::ReleasePhoto { .. }
            )
        })
        .count();
    pinterest_photo_is_hidden(
        s.pinterest,
        s.pinterest_keep_image,
        post.pinterest.is_some(),
        post.pinterest_live,
        post.pinterest_dead,
        attached_photos,
    )
}

async fn resolve_forward_source(
    client: &Client,
    archive_peer: tl::enums::InputPeer,
    archive_message_id: i32,
    source_channel_id: i64,
) -> Result<Option<String>> {
    let response = client
        .invoke(&tl::functions::channels::GetChannels {
            id: vec![tl::enums::InputChannel::FromMessage(
                tl::types::InputChannelFromMessage {
                    peer: archive_peer,
                    msg_id: archive_message_id,
                    channel_id: source_channel_id,
                },
            )],
        })
        .await?;
    let source_id = PeerId::channel(source_channel_id);
    Ok(response
        .chats()
        .into_iter()
        .map(Peer::from_raw)
        .find(|peer| peer.id() == source_id)
        .and_then(|peer| peer.username().map(str::to_owned)))
}

fn same_filename(a: &str, b: &str) -> bool {
    let normalize = |s: &str| s.split_whitespace().collect::<Vec<_>>().join(" ");
    normalize(a) == normalize(b)
}

fn set_release_source(
    releases: &mut BTreeMap<i32, ReleasedImage>,
    manifest_dirty: &mut bool,
    message_id: i32,
    source: ReleasedImageSource,
) {
    let Some(record) = releases.get_mut(&message_id) else {
        return;
    };
    if record.source.as_ref() != Some(&source) {
        record.source = Some(source);
        *manifest_dirty = true;
    }
}

/// Apply live MTProto photo results while retaining every deferred/error slot.
/// Consuming those placeholders is what keeps a partly archived album aligned.
fn apply_photo_archives(
    media: &mut Vec<Media>,
    mut items: Vec<(i32, ArchivedImage)>,
    releases: &mut BTreeMap<i32, ReleasedImage>,
    manifest_dirty: &mut bool,
) {
    items.sort_by_key(|(message_id, _)| *message_id);
    let mut originals = items.into_iter();
    for item in media.iter_mut() {
        let Media::Photo { key, .. } = item else {
            continue;
        };
        let Some((message_id, image)) = originals.next() else {
            break;
        };
        if matches!(&image, ArchivedImage::Release(_)) {
            if let Some(web_key) = key.clone() {
                set_release_source(
                    releases,
                    manifest_dirty,
                    message_id,
                    ReleasedImageSource::Photo { web_key },
                );
            }
        }
        if let Some(archived) = image.into_media(key.clone()) {
            *item = archived;
        }
    }
    // A live session may expose a photo omitted by the public preview. Show it
    // now, but leave its manifest source empty so fallback never guesses a slot.
    media.extend(originals.filter_map(|(_, image)| image.into_media(None)));
}

/// Apply image documents in Telegram message order. A deferred item still
/// claims its matching placeholder, preventing a later same-named document from
/// shifting into the wrong slot when a backfill boundary splits the group.
fn apply_document_archives(media: &mut Vec<Media>, mut items: Vec<(i32, String, ArchivedImage)>) {
    items.sort_by_key(|(message_id, _, _)| *message_id);
    let mut claimed = std::collections::HashSet::new();
    for (message_id, name, image) in items {
        let linked = media.iter().enumerate().position(|(index, item)| {
            !claimed.contains(&index)
                && matches!(item, Media::DocumentRef { message_id: Some(current), .. }
                    if *current == message_id as u64)
        });
        let exact = || {
            media.iter().enumerate().position(|(index, item)| {
                !claimed.contains(&index)
                    && matches!(item, Media::DocumentRef { filename, .. }
                    if same_filename(filename, &name))
            })
        };
        let fallback = || {
            media.iter().enumerate().position(|(index, item)| {
                !claimed.contains(&index)
                    && matches!(item, Media::DocumentRef { filename, .. }
                        if crate::media::is_probably_image_doc(filename))
            })
        };
        let index = linked.or_else(exact).or_else(fallback);
        if let Some(index) = index {
            claimed.insert(index);
        }
        let Some(archived) = image.into_media(None) else {
            continue;
        };
        match index {
            Some(index) => media[index] = archived,
            None => media.push(archived),
        }
    }
}

async fn enrich(posts: &mut [Post], s: &Settings) -> Result<()> {
    let (client, _session) = build_client()?;
    if !client.is_authorized().await? {
        bail!(
            "no valid session — run `tg2zola login` first (or set TG_SESSION); \
             api_id/api_hash alone can't authenticate"
        );
    }

    let peer = client
        .resolve_username(&s.channel)
        .await
        .with_context(|| format!("resolving @{}", s.channel))?
        .with_context(|| format!("channel @{} not found", s.channel))?;
    let archive_peer: PeerRef = (&peer).into();
    let archive_input_peer: tl::enums::InputPeer = archive_peer.into();

    // message id -> index into `posts` (each post bundles one or more ids).
    let mut id_to_post: HashMap<i32, usize> = HashMap::new();
    for (i, p) in posts.iter().enumerate() {
        for id in &p.ids {
            id_to_post.insert(*id as i32, i);
        }
    }

    let cache = s.site.join(".mtproto-cache");
    tokio::fs::create_dir_all(&cache).await.ok();
    let photos = want_photos();
    let videos = want_videos();
    let files = want_files();
    let release_repo = image_release_repo(s);
    let mut released_images = release_repo
        .map(|_| load_image_release_manifest(&s.site, &s.channel))
        .unwrap_or_default();
    let mut release_manifest_dirty = false;
    let mut release_downloads_left = IMAGE_RELEASE_BACKFILL_PER_RUN;
    let reactions = s.reactions;
    // Reactions before custom-emoji resolution, keyed by post index, plus the
    // custom-emoji document ids to resolve to their `alt` glyph in one batch.
    let mut raw_reactions: HashMap<usize, Vec<(RawReaction, u64)>> = HashMap::new();
    let mut custom_emoji_ids: Vec<i64> = Vec::new();

    // (cache path, original filename, label) per post.
    let mut audio_for: AudioFor = HashMap::new();
    let mut photo_for: HashMap<usize, Vec<(i32, ArchivedImage)>> = HashMap::new();
    let mut video_for: HashMap<usize, Vec<(i32, PathBuf)>> = HashMap::new();
    // Pasted images stored as *documents*: original filename + archive location.
    let mut doc_image_for: HashMap<usize, Vec<(i32, String, ArchivedImage)>> = HashMap::new();
    // Any other attachment (pdf/zip/rar/…) to archive as a download: (name, path).
    let mut doc_file_for: HashMap<usize, Vec<(String, PathBuf)>> = HashMap::new();
    // Source channel id -> public username (or None when it cannot be resolved).
    let mut forward_sources: HashMap<i64, Option<String>> = HashMap::new();
    let (mut n_audio, mut n_photo, mut n_video, mut n_doc_image, mut n_file, mut n_deferred_image) =
        (0usize, 0usize, 0usize, 0usize, 0usize, 0usize);

    let mut iter = client.iter_messages(peer);
    let mut capped_post_drain: Option<CappedPostDrain> = None;
    loop {
        let msg = match tokio::time::timeout(MESSAGE_ITERATION_TIMEOUT, iter.next()).await {
            Ok(Ok(Some(message))) => message,
            Ok(Ok(None)) => break,
            Ok(Err(error)) => {
                tracing::warn!("MTProto: stopped walking channel history after an error: {error}");
                break;
            }
            Err(_) => {
                tracing::warn!(
                    "MTProto: stopped walking channel history after waiting {} seconds; preserving the completed image batch",
                    MESSAGE_ITERATION_TIMEOUT.as_secs()
                );
                break;
            }
        };
        let id = msg.id();
        let mut finish_after_current = false;
        if let Some(drain) = capped_post_drain.as_mut() {
            match drain.action(id) {
                CappedBackfillAction::Process => {}
                CappedBackfillAction::ProcessThenStop => finish_after_current = true,
                CappedBackfillAction::Skip => continue,
                CappedBackfillAction::Stop => break,
            }
        }
        let Some(&pi) = id_to_post.get(&id) else {
            continue;
        };
        // The web preview sometimes shows only an unlinked forwarded-from name.
        // MTProto carries the exact source channel + source message id.
        let needs_forward_url = id == posts[pi].primary_id as i32
            && posts[pi]
                .forwarded_from
                .as_ref()
                .is_some_and(|f| f.url.as_deref().and_then(telegram_post_id).is_none());
        if needs_forward_url {
            if let Some((source_channel_id, source_post_id)) =
                msg.forward_header().and_then(forwarded_channel_post)
            {
                let username = if let Some(cached) = forward_sources.get(&source_channel_id) {
                    cached.clone()
                } else {
                    let resolved = resolve_forward_source(
                        &client,
                        archive_input_peer.clone(),
                        id,
                        source_channel_id,
                    )
                    .await;
                    let username = match resolved {
                        Ok(username) => username,
                        Err(e) => {
                            tracing::debug!("message {id}: forwarded source lookup failed: {e}");
                            None
                        }
                    };
                    forward_sources.insert(source_channel_id, username.clone());
                    username
                };
                if let Some(url) = username
                    .as_deref()
                    .and_then(|u| telegram_post_url(u, source_post_id))
                {
                    if let Some(forward) = posts[pi].forwarded_from.as_mut() {
                        forward.url = Some(url);
                    }
                }
            }
        }
        // Reactions (web preview never exposes them). A grouped post keeps the
        // first message's reactions. Done before the media-only `continue` below
        // so a text-only post's reactions are captured too.
        if reactions && !raw_reactions.contains_key(&pi) {
            let rx = raw_reactions_of(&msg);
            if !rx.is_empty() {
                for (r, _) in &rx {
                    if let RawReaction::Custom(id) = r {
                        custom_emoji_ids.push(*id);
                    }
                }
                raw_reactions.insert(pi, rx);
            }
        }
        let Some(media) = msg.media() else { continue };
        match &media {
            TlMedia::Document(doc) => {
                if !local_file_size_allowed(doc.size()) {
                    tracing::warn!(
                        "message {id}: skipping {:.1} GB attachment (over the 2 GiB limit)",
                        doc.size() as f64 / (1024.0 * 1024.0 * 1024.0)
                    );
                    continue;
                }
                let mime = doc.mime_type().unwrap_or("");
                if mime.starts_with("audio/") {
                    // A YouTube / Apple Podcasts link stands in for the audio —
                    // skip the (often large) download to save space, unless
                    // keep_media is set.
                    if !s.keep_media
                        && ((posts[pi].youtube.is_some() && !posts[pi].youtube_dead)
                            || (posts[pi].apple_podcast.is_some() && !posts[pi].apple_dead)
                            || (posts[pi].yandex_music.is_some() && !posts[pi].yandex_dead))
                    {
                        continue;
                    }
                    let dest = cache.join(format!("{id}.{}", audio_ext(mime)));
                    if !download_optional_media_to_cache(&client, &media, &dest, id, "audio").await
                    {
                        continue;
                    }
                    // Original filename + full (untruncated) title/performer.
                    let orig_name = {
                        let n = doc.name().trim();
                        (!n.is_empty()).then(|| n.to_string())
                    };
                    let label = audio_label(doc.audio_title(), doc.performer());
                    audio_for
                        .entry(pi)
                        .or_default()
                        .push((dest, orig_name, label));
                    n_audio += 1;
                } else if mime.starts_with("video/") {
                    // Videos are handled here (fetched by default unless an embed
                    // replaces them) — never archived as a generic file below.
                    if videos {
                        // Only the *unavailable* videos (shown as a poster) are worth
                        // fetching; a web-downloadable Media::Video already has its file.
                        let has_poster = posts[pi]
                            .media
                            .iter()
                            .any(|m| matches!(m, Media::VideoPoster { .. }));
                        // A live YouTube/Instagram embed stands in for the video — skip
                        // the (large) download unless keep_media is set. Instagram only
                        // counts when its embed is enabled (opt-in).
                        let embed_replaces = !s.keep_media
                            && ((posts[pi].youtube.is_some() && !posts[pi].youtube_dead)
                                || (s.instagram
                                    && posts[pi].instagram.is_some()
                                    && !posts[pi].instagram_dead));
                        if has_poster && !embed_replaces {
                            let dest = cache.join(format!("{id}.{}", video_ext(mime)));
                            if !download_optional_media_to_cache(
                                &client, &media, &dest, id, "video",
                            )
                            .await
                            {
                                continue;
                            }
                            video_for.entry(pi).or_default().push((id, dest));
                            n_video += 1;
                        }
                    }
                } else if let Some(image_extension) = photos
                    .then(|| image_document_ext(mime, doc.name()))
                    .flatten()
                {
                    // A pasted image Telegram stored as a *file* — the web preview
                    // can't download it (shown "(not archived)"), so fetch it and
                    // show it as a photo. Original images are on by default.
                    let name = doc.name().trim().to_string();
                    let source = posts[pi].media.iter().find_map(|item| {
                        let Media::DocumentRef {
                            filename,
                            message_id,
                        } = item
                        else {
                            return None;
                        };
                        (*message_id == Some(id as u64)
                            && crate::media::is_probably_image_doc(filename)
                            && same_filename(filename, &name))
                        .then(|| ReleasedImageSource::Document {
                            filename: filename.clone(),
                        })
                    });
                    let image = match archive_image(
                        &client,
                        &media,
                        s,
                        &cache,
                        release_repo,
                        &mut released_images,
                        &mut release_manifest_dirty,
                        &mut release_downloads_left,
                        id,
                        doc.id(),
                        doc.size(),
                        image_extension,
                        source,
                    )
                    .await
                    {
                        Ok(image) => image,
                        Err(error) => {
                            tracing::warn!(
                                "message {id}: original image document skipped: {error:#}"
                            );
                            ArchivedImage::Deferred
                        }
                    };
                    if matches!(&image, ArchivedImage::Deferred) {
                        n_deferred_image += 1;
                    } else {
                        n_doc_image += 1;
                    }
                    doc_image_for.entry(pi).or_default().push((id, name, image));
                } else if files {
                    // Every other attachment (pdf/zip/rar/…), plus images when
                    // MTPROTO_IMAGES is off — archive it as a downloadable file.
                    let name = doc.name().trim().to_string();
                    let dest = cache.join(format!("{id}.{}", file_ext(&name)));
                    if !download_optional_media_to_cache(&client, &media, &dest, id, "attachment")
                        .await
                    {
                        continue;
                    }
                    doc_file_for.entry(pi).or_default().push((name, dest));
                    n_file += 1;
                }
            }
            TlMedia::Photo(photo) if photos => {
                if pinterest_hides_single_photo(&posts[pi], s) {
                    tracing::debug!(
                        "message {id}: skipping original photo replaced by its Pinterest widget"
                    );
                    continue;
                }
                let image = match archive_image(
                    &client,
                    &media,
                    s,
                    &cache,
                    release_repo,
                    &mut released_images,
                    &mut release_manifest_dirty,
                    &mut release_downloads_left,
                    id,
                    photo.id(),
                    photo.size(),
                    "jpg",
                    None,
                )
                .await
                {
                    Ok(image) => image,
                    Err(error) => {
                        tracing::warn!("message {id}: original photo skipped: {error:#}");
                        ArchivedImage::Deferred
                    }
                };
                if matches!(&image, ArchivedImage::Deferred) {
                    n_deferred_image += 1;
                } else {
                    n_photo += 1;
                }
                photo_for.entry(pi).or_default().push((id, image));
            }
            _ => {}
        }
        if finish_after_current {
            tracing::info!(
                "MTProto: original-image backfill batch is full; finished its grouped post"
            );
            break;
        }
        if release_repo.is_some() && release_downloads_left == 0 && capped_post_drain.is_none() {
            let Some(drain) = CappedPostDrain::after_message(&posts[pi].ids, id) else {
                tracing::info!(
                    "MTProto: original-image backfill batch is full; stopping after its current post"
                );
                break;
            };
            capped_post_drain = Some(drain);
        }
    }

    // Append audio (new media the web never had); MTProto got the real file, so
    // drop the web's redundant "(not archived)" placeholder for the same track.
    for (pi, items) in audio_for {
        posts[pi].media.retain(|m| {
            !matches!(m, Media::DocumentRef { filename, .. } if crate::media::is_probably_audio_doc(filename))
        });
        for (path, name, title) in items {
            posts[pi]
                .media
                .push(Media::LocalAudio { path, name, title });
        }
    }
    // Replace each web Photo with the original, retaining deferred/error slots
    // so a cap boundary can never shift an album's images.
    for (pi, items) in photo_for {
        apply_photo_archives(
            &mut posts[pi].media,
            items,
            &mut released_images,
            &mut release_manifest_dirty,
        );
    }
    // Replace each poster-only video with the fetched original, in id order.
    for (pi, mut items) in video_for {
        items.sort_by_key(|(id, _)| *id);
        let mut originals = items.into_iter().map(|(_, p)| p);
        for m in posts[pi].media.iter_mut() {
            if matches!(m, Media::VideoPoster { .. }) {
                if let Some(path) = originals.next() {
                    *m = Media::LocalVideo { path };
                }
            }
        }
    }
    // Replace each "(not archived)" image document with the fetched original.
    // Message ordering disambiguates duplicate filenames and deferred slots.
    for (pi, items) in doc_image_for {
        apply_document_archives(&mut posts[pi].media, items);
    }
    // Archive every other attachment as a downloadable file, replacing its
    // "(not archived)" reference (matched by filename, else the first
    // non-audio/non-image reference in the post).
    for (pi, items) in doc_file_for {
        for (name, path) in items {
            let media = &mut posts[pi].media;
            let idx = media
                .iter()
                .position(|m| {
                    matches!(m, Media::DocumentRef { filename, .. } if same_filename(filename, &name))
                })
                .or_else(|| {
                    media.iter().position(|m| {
                        matches!(m, Media::DocumentRef { filename, .. }
                            if !crate::media::is_probably_audio_doc(filename)
                                && !crate::media::is_probably_image_doc(filename))
                    })
                });
            match idx {
                Some(i) => media[i] = Media::LocalDocument { path, name },
                None => media.push(Media::LocalDocument { path, name }),
            }
        }
    }
    // Resolve custom-emoji reactions to their unicode `alt` glyph (one batch),
    // then attach each post's reactions.
    let alt = resolve_custom_emoji_alts(&client, &mut custom_emoji_ids).await;
    let mut n_reactions = 0usize;
    for (pi, raw) in raw_reactions {
        let resolved: Vec<(String, u64)> = raw
            .into_iter()
            .filter_map(|(r, count)| match r {
                RawReaction::Glyph(g) => Some((g, count)),
                RawReaction::Custom(id) => alt.get(&id).cloned().map(|g| (g, count)),
            })
            .collect();
        if !resolved.is_empty() {
            posts[pi].reactions = resolved;
            n_reactions += 1;
        }
    }

    tracing::info!(
        "MTProto: {n_audio} audio file(s), {n_photo} original photo(s), \
         {n_doc_image} image file(s), {n_video} video(s), {n_file} attachment(s), \
        {n_reactions} post(s) with reactions"
    );
    if n_deferred_image > 0 {
        tracing::info!(
            "MTProto: deferred {n_deferred_image} original image(s) to a later backfill run"
        );
    }
    if release_manifest_dirty {
        write_pending_image_release_manifest(&s.site, &s.channel, &released_images)?;
        tracing::info!(
            "MTProto: staged original images across {} GitHub Release bucket(s)",
            released_images
                .values()
                .map(|image| image.tag.as_str())
                .collect::<std::collections::HashSet<_>>()
                .len()
        );
    }
    Ok(())
}

/// A label above the player from the audio track's title (+ performer). Used
/// only when the title looks *complete*: many podcast files carry a title tag
/// Telegram/the encoder already truncated with `…`, and the post caption
/// normally has the full title anyway — so a truncated tag is worse than none.
fn audio_label(title: Option<String>, performer: Option<String>) -> Option<String> {
    let t = title
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .filter(|s| !s.ends_with('…') && !s.ends_with("..."))?;
    match performer
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
    {
        Some(p) => Some(format!("{p} — {t}")),
        None => Some(t),
    }
}

/// File extension for an audio MIME type (voice notes are `audio/ogg`).
fn audio_ext(mime: &str) -> &'static str {
    match mime {
        "audio/ogg" | "audio/opus" => "ogg",
        "audio/mpeg" | "audio/mp3" => "mp3",
        "audio/mp4" | "audio/x-m4a" | "audio/aac" => "m4a",
        "audio/wav" | "audio/x-wav" => "wav",
        "audio/flac" => "flac",
        _ => "bin",
    }
}

/// File extension for a video MIME type.
fn video_ext(mime: &str) -> &'static str {
    match mime {
        "video/webm" => "webm",
        "video/quicktime" => "mov",
        _ => "mp4",
    }
}

/// File extension for an image MIME type (pasted-image documents).
fn image_ext(mime: &str) -> &'static str {
    match mime {
        "image/png" => "png",
        "image/gif" => "gif",
        "image/webp" => "webp",
        "image/avif" => "avif",
        "image/bmp" => "bmp",
        _ => "jpg",
    }
}

/// Recognize image documents before deciding whether to render them inline.
///
/// Telegram normally supplies an `image/*` MIME type. The filename argument is
/// retained here because some clients upload image bytes as a generic document.
fn image_document_ext(mime: &str, filename: &str) -> Option<&'static str> {
    if mime.starts_with("image/") {
        return Some(image_ext(mime));
    }
    let extension = filename.rsplit_once('.')?.1.to_ascii_lowercase();
    match extension.as_str() {
        "jpg" => Some("jpg"),
        "jpeg" => Some("jpeg"),
        "png" => Some("png"),
        "gif" => Some("gif"),
        "webp" => Some("webp"),
        "avif" => Some("avif"),
        "bmp" => Some("bmp"),
        _ => None,
    }
}

/// Extension for an attachment's cache file, taken from its filename (lowercased,
/// alphanumeric, ≤8 chars); `bin` when there's no usable extension.
fn file_ext(name: &str) -> String {
    name.rsplit_once('.')
        .map(|(_, e)| e.to_ascii_lowercase())
        .filter(|e| {
            (1..=8).contains(&e.chars().count()) && e.chars().all(|c| c.is_ascii_alphanumeric())
        })
        .unwrap_or_else(|| "bin".to_string())
}

/// A reaction before custom-emoji resolution: a ready glyph, or a custom-emoji
/// document id awaiting its `alt` fallback.
enum RawReaction {
    Glyph(String),
    Custom(i64),
}

/// Reaction counts for a message: standard emojis and paid (⭐) become glyphs
/// immediately; custom emojis carry their document id for a later batch lookup.
fn raw_reactions_of(msg: &grammers_client::types::Message) -> Vec<(RawReaction, u64)> {
    let tl::enums::Message::Message(m) = &msg.raw else {
        return Vec::new();
    };
    let Some(tl::enums::MessageReactions::Reactions(r)) = &m.reactions else {
        return Vec::new();
    };
    r.results
        .iter()
        .filter_map(|rc| {
            let tl::enums::ReactionCount::Count(rc) = rc;
            let count = rc.count.max(0) as u64;
            match &rc.reaction {
                tl::enums::Reaction::Emoji(e) => {
                    Some((RawReaction::Glyph(e.emoticon.clone()), count))
                }
                tl::enums::Reaction::Paid => Some((RawReaction::Glyph("⭐".to_string()), count)),
                tl::enums::Reaction::CustomEmoji(c) => {
                    Some((RawReaction::Custom(c.document_id), count))
                }
                tl::enums::Reaction::Empty => None,
            }
        })
        .collect()
}

/// Resolve custom-emoji document ids to their unicode `alt` glyph, in batches.
/// Best-effort: ids that fail to resolve (or have no alt) are simply dropped.
async fn resolve_custom_emoji_alts(client: &Client, ids: &mut Vec<i64>) -> HashMap<i64, String> {
    let mut out = HashMap::new();
    if ids.is_empty() {
        return out;
    }
    ids.sort_unstable();
    ids.dedup();
    for chunk in ids.chunks(200) {
        let req = tl::functions::messages::GetCustomEmojiDocuments {
            document_id: chunk.to_vec(),
        };
        match client.invoke(&req).await {
            Ok(docs) => {
                for doc in docs {
                    let tl::enums::Document::Document(doc) = doc else {
                        continue;
                    };
                    for attr in &doc.attributes {
                        if let tl::enums::DocumentAttribute::CustomEmoji(ce) = attr {
                            if !ce.alt.is_empty() {
                                out.insert(doc.id, ce.alt.clone());
                            }
                        }
                    }
                }
            }
            Err(e) => tracing::info!("MTProto: custom-emoji reaction lookup failed: {e}"),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn forward_header(
        from_id: Option<tl::enums::Peer>,
        channel_post: Option<i32>,
    ) -> tl::enums::MessageFwdHeader {
        tl::enums::MessageFwdHeader::Header(tl::types::MessageFwdHeader {
            imported: false,
            saved_out: false,
            from_id,
            from_name: None,
            date: 0,
            channel_post,
            post_author: None,
            saved_from_peer: None,
            saved_from_msg_id: None,
            saved_from_id: None,
            saved_from_name: None,
            saved_date: None,
            psa_type: None,
        })
    }

    #[test]
    fn extracts_forwarded_channel_message() {
        let header = forward_header(
            Some(tl::enums::Peer::Channel(tl::types::PeerChannel {
                channel_id: 42,
            })),
            Some(534),
        );
        assert_eq!(forwarded_channel_post(header), Some((42, 534)));
        assert_eq!(
            telegram_post_url("@durov", 534).as_deref(),
            Some("https://t.me/durov/534")
        );
        assert_eq!(telegram_post_id("https://t.me/durov/534"), Some(534));
        assert_eq!(telegram_post_id("https://t.me/durov"), None);
    }

    #[test]
    fn forward_without_public_channel_post_stays_unlinked() {
        let user = forward_header(
            Some(tl::enums::Peer::User(tl::types::PeerUser { user_id: 42 })),
            None,
        );
        assert_eq!(forwarded_channel_post(user), None);
        assert_eq!(telegram_post_url("", 534), None);
        assert_eq!(telegram_post_url("durov", 0), None);
    }

    #[test]
    fn attachment_filename_matching_collapses_whitespace() {
        assert!(same_filename(
            "F 1-75-0012 0003.jpg",
            "F 1-75-0012  0003.jpg"
        ));
        assert!(!same_filename("0003.jpg", "0005.jpg"));
    }

    #[test]
    fn generic_mime_png_document_is_still_an_inline_image() {
        assert_eq!(
            image_document_ext("application/octet-stream", "image_2026-07-01_07-36-10.png"),
            Some("png")
        );
        assert_eq!(
            image_document_ext("application/octet-stream", "archive.zip"),
            None
        );
    }

    #[test]
    fn original_photos_default_on_with_explicit_opt_out() {
        for value in [
            None,
            Some(""),
            Some("1"),
            Some("true"),
            Some("yes"),
            Some("on"),
            Some("unexpected"),
        ] {
            assert!(photos_enabled(value), "{value:?} should enable originals");
        }
        for value in [
            Some("0"),
            Some("false"),
            Some(" false "),
            Some("no"),
            Some("off"),
        ] {
            assert!(!photos_enabled(value), "{value:?} should disable originals");
        }
    }

    #[test]
    fn image_release_targets_are_sharded_and_immutable() {
        let low = image_release_target("https://github.com/o/r", 499, 9001, "jpg");
        let next = image_release_target("https://github.com/o/r", 500, 9002, "png");

        assert_eq!(low.tag, "images-0000");
        assert_eq!(low.asset, "telegram-image-499-9001.jpg");
        assert_eq!(
            low.url,
            "https://github.com/o/r/releases/download/images-0000/telegram-image-499-9001.jpg"
        );
        assert_eq!(next.tag, "images-0500");
        assert_eq!(next.asset, "telegram-image-500-9002.png");
    }

    #[test]
    fn only_release_images_use_the_strict_two_gibibyte_limit() {
        assert!(release_image_size_allowed(MAX_FILE_BYTES - 1));
        assert!(!release_image_size_allowed(MAX_FILE_BYTES));
        assert!(local_file_size_allowed(MAX_FILE_BYTES));
        assert!(!local_file_size_allowed(MAX_FILE_BYTES + 1));
    }

    #[test]
    fn release_manifest_reuses_only_the_same_telegram_media() {
        let target = image_release_target("https://github.com/o/r", 1367, 42, "jpg");
        let record = ReleasedImage {
            message_id: 1367,
            media_id: 42,
            declared_bytes: 123,
            tag: target.tag.clone(),
            asset: target.asset.clone(),
            bytes: 123,
            source: Some(ReleasedImageSource::Photo {
                web_key: "photo-key".into(),
            }),
        };

        assert!(record.matches(42, &target));
        assert!(!record.matches(43, &target));
        let replacement = image_release_target("https://github.com/o/r", 1367, 43, "jpg");
        assert!(!record.matches(42, &replacement));
    }

    #[test]
    fn image_release_manifest_round_trips_and_counts_bytes() {
        let dir = std::env::temp_dir().join(format!(
            "tg2zola-image-release-manifest-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let mut images = BTreeMap::new();
        let target = image_release_target("https://github.com/o/r", 1367, 42, "jpg");
        images.insert(
            1367,
            ReleasedImage {
                message_id: 1367,
                media_id: 42,
                declared_bytes: 123,
                tag: target.tag.clone(),
                asset: target.asset.clone(),
                bytes: 456,
                source: Some(ReleasedImageSource::Photo {
                    web_key: "photo-key".into(),
                }),
            },
        );

        write_pending_image_release_manifest(&dir, "channel", &images).unwrap();
        assert_eq!(released_image_bytes(&dir), 456);
        std::fs::rename(
            dir.join(IMAGE_RELEASE_PENDING),
            dir.join(IMAGE_RELEASE_MANIFEST),
        )
        .unwrap();
        assert!(load_image_release_manifest(&dir, "channel").is_empty());
        assert_eq!(released_image_bytes(&dir), 0);
        std::fs::write(
            dir.join(IMAGE_RELEASE_INVENTORY),
            format!("{}\t{}\t456\n", target.tag, target.asset),
        )
        .unwrap();
        let loaded = load_image_release_manifest(&dir, "channel");
        assert_eq!(loaded.get(&1367).unwrap().media_id, 42);
        assert_eq!(released_image_bytes(&dir), 456);
        assert!(load_image_release_manifest(&dir, "other").is_empty());
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn fallback_replaces_only_exact_web_sources_and_never_appends() {
        let photo_target = image_release_target("https://github.com/o/r", 1367, 42, "jpg");
        let photo = ReleasedImage {
            message_id: 1367,
            media_id: 42,
            declared_bytes: 123,
            tag: photo_target.tag.clone(),
            asset: photo_target.asset.clone(),
            bytes: 123,
            source: Some(ReleasedImageSource::Photo {
                web_key: "kept-photo".into(),
            }),
        };
        let document_target = image_release_target("https://github.com/o/r", 1368, 43, "png");
        let document = ReleasedImage {
            message_id: 1368,
            media_id: 43,
            declared_bytes: 124,
            tag: document_target.tag.clone(),
            asset: document_target.asset.clone(),
            bytes: 124,
            source: Some(ReleasedImageSource::Document {
                filename: "pasted image.png".into(),
            }),
        };
        let mut media = vec![
            Media::Photo {
                url: "https://cdn.example/replaced.jpg".into(),
                key: Some("replacement-photo".into()),
            },
            Media::Photo {
                url: "https://cdn.example/kept.jpg".into(),
                key: Some("kept-photo".into()),
            },
            Media::DocumentRef {
                filename: "pasted image.png".into(),
                message_id: Some(1369),
            },
            Media::DocumentRef {
                filename: "pasted image.png".into(),
                message_id: Some(1368),
            },
        ];

        assert!(apply_released_image(&mut media, &photo, photo_target));
        assert!(apply_released_image(&mut media, &document, document_target));
        assert!(
            matches!(&media[0], Media::Photo { key: Some(key), .. } if key == "replacement-photo")
        );
        assert!(matches!(&media[1], Media::ReleasePhoto { .. }));
        assert!(matches!(
            &media[2],
            Media::DocumentRef {
                filename,
                message_id: Some(1369)
            } if filename == "pasted image.png"
        ));
        assert!(matches!(&media[3], Media::ReleasePhoto { .. }));

        let mut renamed_document = vec![Media::DocumentRef {
            filename: "pasted  image.png".into(),
            message_id: Some(1368),
        }];
        let target = image_release_target("https://github.com/o/r", 1368, 43, "png");
        assert!(!apply_released_image(
            &mut renamed_document,
            &document,
            target
        ));

        let mut absent = Vec::new();
        let target = image_release_target("https://github.com/o/r", 1367, 42, "jpg");
        assert!(!apply_released_image(&mut absent, &photo, target));
        assert!(
            absent.is_empty(),
            "fallback must not resurrect removed media"
        );
    }

    #[test]
    fn deferred_album_slots_do_not_shift_release_images() {
        let target = image_release_target("https://github.com/o/r", 12, 42, "jpg");
        let mut releases = BTreeMap::from([(
            12,
            ReleasedImage {
                message_id: 12,
                media_id: 42,
                declared_bytes: 123,
                tag: target.tag,
                asset: target.asset,
                bytes: 123,
                source: None,
            },
        )]);
        let mut media = vec![
            Media::Photo {
                url: "https://cdn.example/a.jpg".into(),
                key: Some("a".into()),
            },
            Media::Photo {
                url: "https://cdn.example/b.jpg".into(),
                key: Some("b".into()),
            },
            Media::Photo {
                url: "https://cdn.example/c.jpg".into(),
                key: Some("c".into()),
            },
        ];
        let mut dirty = false;

        apply_photo_archives(
            &mut media,
            vec![
                (13, ArchivedImage::Deferred),
                (12, ArchivedImage::Release("https://release/b.jpg".into())),
                (11, ArchivedImage::Deferred),
            ],
            &mut releases,
            &mut dirty,
        );

        assert!(matches!(&media[0], Media::Photo { key: Some(key), .. } if key == "a"));
        assert!(matches!(&media[1], Media::ReleasePhoto { url } if url.ends_with("/b.jpg")));
        assert!(matches!(&media[2], Media::Photo { key: Some(key), .. } if key == "c"));
        assert_eq!(
            releases.get(&12).unwrap().source,
            Some(ReleasedImageSource::Photo {
                web_key: "b".into()
            })
        );
        assert!(dirty);
    }

    #[test]
    fn capped_backfill_skips_intervening_messages_until_group_is_complete() {
        let mut drain = CappedPostDrain::after_message(&[1787, 1797], 1797)
            .expect("the lower grouped ID remains after the cap is reached");

        assert_eq!(drain.action(1796), CappedBackfillAction::Skip);
        assert_eq!(drain.action(1790), CappedBackfillAction::Skip);
        assert_eq!(drain.action(1787), CappedBackfillAction::ProcessThenStop);
        assert_eq!(drain.action(1786), CappedBackfillAction::Stop);
    }

    #[test]
    fn duplicate_document_names_follow_linked_message_ids() {
        let mut media = vec![
            Media::DocumentRef {
                filename: "image.png".into(),
                message_id: Some(12),
            },
            Media::DocumentRef {
                filename: "image.png".into(),
                message_id: Some(11),
            },
        ];

        apply_document_archives(
            &mut media,
            vec![
                (
                    12,
                    "image.png".into(),
                    ArchivedImage::Release("https://release/newer.png".into()),
                ),
                (
                    11,
                    "image.png".into(),
                    ArchivedImage::Release("https://release/older.png".into()),
                ),
            ],
        );

        assert!(matches!(&media[0], Media::ReleasePhoto { url } if url.ends_with("/newer.png")));
        assert!(matches!(&media[1], Media::ReleasePhoto { url } if url.ends_with("/older.png")));
    }

    #[test]
    fn confirmed_pinterest_widget_skips_only_its_single_hidden_photo() {
        assert!(pinterest_photo_is_hidden(true, false, true, true, false, 1));
        assert!(!pinterest_photo_is_hidden(true, true, true, true, false, 1));
        assert!(!pinterest_photo_is_hidden(
            true, false, true, false, false, 1
        ));
        assert!(!pinterest_photo_is_hidden(true, false, true, true, true, 1));
        assert!(!pinterest_photo_is_hidden(
            true, false, true, true, false, 2
        ));
    }
}
