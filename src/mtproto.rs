//! Optional MTProto backend (Cargo feature `mtproto`).
//!
//! The public `t.me/s/` preview never exposes voice/audio notes and serves only
//! a size-limited photo. Logging in as a *user* over MTProto (via `grammers`)
//! recovers both. This is strictly opt-in: it needs `TG_API_ID` + `TG_API_HASH`
//! and a session (`TG_SESSION` base64, or a `tg2zola.session` file created once
//! by `tg2zola login`). Without those, [`maybe_enrich`] is a no-op and the tool
//! stays the creds-free web scraper.
#![allow(deprecated)]

use anyhow::{anyhow, bail, Context, Result};
use std::collections::HashMap;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Arc;

use base64::Engine;
use grammers_client::types::Media as TlMedia;
use grammers_client::{Client, SignInError};
use grammers_mtsender::SenderPool;
use grammers_session::storages::TlSession;

use crate::config::Settings;
use crate::model::{Media, Post};

/// Default on-disk session file (created by `tg2zola login`).
const SESSION_FILE: &str = "tg2zola.session";

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
        return false; // not configured — stay a pure web scraper
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
            false
        }
    }
}

fn want_photos() -> bool {
    matches!(
        std::env::var("MTPROTO_IMAGES").ok().as_deref(),
        Some("1") | Some("true") | Some("yes") | Some("on")
    )
}

/// Opt-in (`MTPROTO_VIDEOS=1`): also download the *original video* for posts the
/// web preview shows only as a poster (large/long videos). Off by default —
/// videos are large, so this is for a full local backup, not the CI budget.
fn want_videos() -> bool {
    matches!(
        std::env::var("MTPROTO_VIDEOS").ok().as_deref(),
        Some("1") | Some("true") | Some("yes") | Some("on")
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

/// Per-post-index MTProto audio: (cache path, original filename, label).
type AudioFor = HashMap<usize, Vec<(PathBuf, Option<String>, Option<String>)>>;

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

    // (cache path, original filename, label) per post.
    let mut audio_for: AudioFor = HashMap::new();
    let mut photo_for: HashMap<usize, Vec<(i32, PathBuf)>> = HashMap::new();
    let mut video_for: HashMap<usize, Vec<(i32, PathBuf)>> = HashMap::new();
    // Pasted images stored as *documents* (Media::DocumentRef): (filename, path).
    let mut doc_image_for: HashMap<usize, Vec<(String, PathBuf)>> = HashMap::new();
    // Any other attachment (pdf/zip/rar/…) to archive as a download: (name, path).
    let mut doc_file_for: HashMap<usize, Vec<(String, PathBuf)>> = HashMap::new();
    let (mut n_audio, mut n_photo, mut n_video, mut n_doc_image, mut n_file) =
        (0usize, 0usize, 0usize, 0usize, 0usize);

    let mut iter = client.iter_messages(peer);
    while let Some(msg) = iter.next().await.context("iterating channel messages")? {
        let id = msg.id();
        let Some(&pi) = id_to_post.get(&id) else {
            continue;
        };
        let Some(media) = msg.media() else { continue };
        match &media {
            TlMedia::Document(doc) => {
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
                    if !dest.exists() {
                        client
                            .download_media(&media, &dest)
                            .await
                            .with_context(|| format!("downloading audio from message {id}"))?;
                    }
                    // Original filename + full (untruncated) title/performer.
                    let orig_name = {
                        let n = doc.name().trim();
                        (!n.is_empty()).then(|| n.to_string())
                    };
                    let label = audio_label(doc.audio_title(), doc.performer());
                    audio_for.entry(pi).or_default().push((dest, orig_name, label));
                    n_audio += 1;
                } else if mime.starts_with("video/") {
                    // Large videos stay behind MTPROTO_VIDEOS (hosting budget), so a
                    // video is never archived as a generic file in the branch below.
                    if videos {
                        // Only the *unavailable* videos (shown as a poster) are worth
                        // fetching; a web-downloadable Media::Video already has its file.
                        let has_poster = posts[pi]
                            .media
                            .iter()
                            .any(|m| matches!(m, Media::VideoPoster { .. }));
                        // A live YouTube/Instagram embed stands in for the video — skip
                        // the (large) download unless keep_media is set.
                        let embed_replaces = !s.keep_media
                            && ((posts[pi].youtube.is_some() && !posts[pi].youtube_dead)
                                || (posts[pi].instagram.is_some() && !posts[pi].instagram_dead));
                        if has_poster && !embed_replaces {
                            let dest = cache.join(format!("{id}.{}", video_ext(mime)));
                            if !dest.exists() {
                                client
                                    .download_media(&media, &dest)
                                    .await
                                    .with_context(|| format!("downloading video from message {id}"))?;
                            }
                            video_for.entry(pi).or_default().push((id, dest));
                            n_video += 1;
                        }
                    }
                } else if mime.starts_with("image/") && photos {
                    // A pasted image Telegram stored as a *file* — the web preview
                    // can't download it (shown "(not archived)"), so fetch it and
                    // show it as a photo. Gated on MTPROTO_IMAGES like real photos.
                    let dest = cache.join(format!("{id}.{}", image_ext(mime)));
                    if !dest.exists() {
                        client
                            .download_media(&media, &dest)
                            .await
                            .with_context(|| format!("downloading image document from message {id}"))?;
                    }
                    let name = doc.name().trim().to_string();
                    doc_image_for.entry(pi).or_default().push((name, dest));
                    n_doc_image += 1;
                } else if files {
                    // Every other attachment (pdf/zip/rar/…), plus images when
                    // MTPROTO_IMAGES is off — archive it as a downloadable file.
                    let name = doc.name().trim().to_string();
                    let dest = cache.join(format!("{id}.{}", file_ext(&name)));
                    if !dest.exists() {
                        client
                            .download_media(&media, &dest)
                            .await
                            .with_context(|| format!("downloading attachment from message {id}"))?;
                    }
                    doc_file_for.entry(pi).or_default().push((name, dest));
                    n_file += 1;
                }
            }
            TlMedia::Photo(_) if photos => {
                let dest = cache.join(format!("{id}.jpg"));
                if !dest.exists() {
                    client
                        .download_media(&media, &dest)
                        .await
                        .with_context(|| format!("downloading photo from message {id}"))?;
                }
                photo_for.entry(pi).or_default().push((id, dest));
                n_photo += 1;
            }
            _ => {}
        }
    }

    // Append audio (new media the web never had); MTProto got the real file, so
    // drop the web's redundant "(not archived)" placeholder for the same track.
    for (pi, items) in audio_for {
        posts[pi].media.retain(|m| {
            !matches!(m, Media::DocumentRef { filename } if crate::media::is_probably_audio_doc(filename))
        });
        for (path, name, title) in items {
            posts[pi].media.push(Media::LocalAudio { path, name, title });
        }
    }
    // Replace each web Photo with the original, matched in message-id order.
    for (pi, mut items) in photo_for {
        items.sort_by_key(|(id, _)| *id);
        let mut originals = items.into_iter().map(|(_, p)| p);
        for m in posts[pi].media.iter_mut() {
            if let Media::Photo { key, .. } = m {
                if let Some(path) = originals.next() {
                    *m = Media::LocalPhoto {
                        path,
                        key: key.clone(),
                    };
                }
            }
        }
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
    // Replace each "(not archived)" image document with the fetched original,
    // matched by filename (else the first image-typed reference in the post).
    for (pi, items) in doc_image_for {
        for (name, path) in items {
            let media = &mut posts[pi].media;
            let idx = media
                .iter()
                .position(|m| matches!(m, Media::DocumentRef { filename } if *filename == name))
                .or_else(|| {
                    media.iter().position(|m| {
                        matches!(m, Media::DocumentRef { filename } if crate::media::is_probably_image_doc(filename))
                    })
                });
            match idx {
                Some(i) => media[i] = Media::LocalPhoto { path, key: None },
                None => media.push(Media::LocalPhoto { path, key: None }),
            }
        }
    }
    // Archive every other attachment as a downloadable file, replacing its
    // "(not archived)" reference (matched by filename, else the first
    // non-audio/non-image reference in the post).
    for (pi, items) in doc_file_for {
        for (name, path) in items {
            let media = &mut posts[pi].media;
            let idx = media
                .iter()
                .position(|m| matches!(m, Media::DocumentRef { filename } if *filename == name))
                .or_else(|| {
                    media.iter().position(|m| {
                        matches!(m, Media::DocumentRef { filename }
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

    tracing::info!(
        "MTProto: {n_audio} audio file(s), {n_photo} original photo(s), \
         {n_doc_image} image file(s), {n_video} video(s), {n_file} attachment(s)"
    );
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
    match performer.map(|s| s.trim().to_string()).filter(|s| !s.is_empty()) {
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

/// Extension for an attachment's cache file, taken from its filename (lowercased,
/// alphanumeric, ≤8 chars); `bin` when there's no usable extension.
fn file_ext(name: &str) -> String {
    name.rsplit_once('.')
        .map(|(_, e)| e.to_ascii_lowercase())
        .filter(|e| (1..=8).contains(&e.chars().count()) && e.chars().all(|c| c.is_ascii_alphanumeric()))
        .unwrap_or_else(|| "bin".to_string())
}
