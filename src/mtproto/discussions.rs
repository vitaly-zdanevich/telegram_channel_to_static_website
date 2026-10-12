//! Bounded snapshots of public channel-post comment threads. Never joins a
//! discussion group, marks messages read, or traverses unrelated group history.

use anyhow::{bail, Context, Result};
use grammers_client::grammers_tl_types as tl;
use grammers_client::types::{Message, Peer, PeerMap};
use grammers_client::Client;
use grammers_session::defs::{PeerId, PeerRef};
use std::collections::{BTreeMap, BTreeSet};
use std::future::Future;
use std::time::Duration;

use crate::config::Settings;
use crate::discussions::{escape_html, safe_link, Archive, Comment, Discussion};
use crate::model::Post;

/// One reply page with the oldest raw message ID, including service messages.
struct ReplyPage {
    comments: Vec<Comment>,
    oldest_id: Option<i32>,
}

/// Collect a complete thread before replacing any previously saved snapshot.
async fn collect_comments<F, Fut>(mut fetch: F) -> Result<Vec<Comment>>
where
    F: FnMut(i32) -> Fut,
    Fut: Future<Output = Result<ReplyPage>>,
{
    let mut comments = BTreeMap::new();
    let mut offset = 0;
    loop {
        let page = fetch(offset).await?;
        let Some(oldest) = page.oldest_id else {
            anyhow::ensure!(page.comments.is_empty(), "reply page has no cursor");
            break;
        };
        anyhow::ensure!(
            oldest > 0 && (offset == 0 || oldest < offset),
            "reply pagination did not advance"
        );
        for comment in page.comments {
            anyhow::ensure!(
                comment.id > 0 && (offset == 0 || comment.id < offset),
                "reply page overlaps its cursor"
            );
            comments.insert(comment.id, comment);
        }
        offset = oldest;
    }
    Ok(comments.into_values().collect())
}

/// Render only recognized formatting, with UTF-16 offsets checked at boundaries.
fn comment_html(text: &str, entities: &[tl::enums::MessageEntity]) -> String {
    use tl::enums::MessageEntity as E;
    let mut boundaries = BTreeMap::from([(0i32, 0usize)]);
    let mut units = 0i32;
    for (byte, ch) in text.char_indices() {
        units += ch.len_utf16() as i32;
        boundaries.insert(units, byte + ch.len_utf8());
    }
    let mut spans = Vec::new();
    for entity in entities {
        let (offset, length, tag, target) = match entity {
            E::Bold(e) => (e.offset, e.length, "b", None),
            E::Italic(e) => (e.offset, e.length, "i", None),
            E::Underline(e) => (e.offset, e.length, "u", None),
            E::Strike(e) => (e.offset, e.length, "s", None),
            E::Code(e) => (e.offset, e.length, "code", None),
            E::Pre(e) => (e.offset, e.length, "pre", None),
            E::Blockquote(e) => (e.offset, e.length, "blockquote", None),
            E::Spoiler(e) => (e.offset, e.length, "details", None),
            E::TextUrl(e) => (e.offset, e.length, "a", Some(e.url.as_str())),
            E::Url(e) => (e.offset, e.length, "a", None),
            _ => continue,
        };
        let Some(end_units) = offset.checked_add(length) else {
            continue;
        };
        let (Some(&start), Some(&end)) = (boundaries.get(&offset), boundaries.get(&end_units))
        else {
            continue;
        };
        if start >= end {
            continue;
        }
        let open = if tag == "a" {
            let url = target.unwrap_or(&text[start..end]);
            if !safe_link(url) {
                continue;
            }
            format!("<a href='{}'>", escape_html(url))
        } else if tag == "details" {
            "<details><summary>…</summary>".to_string()
        } else {
            format!("<{tag}>")
        };
        spans.push(Span {
            start,
            end,
            open,
            close: format!("</{tag}>"),
        });
    }
    spans.sort_by_key(|span| (span.start, std::cmp::Reverse(span.end)));
    let mut accepted: Vec<Span> = Vec::new();
    let mut stack: Vec<(usize, bool)> = Vec::new();
    for span in spans {
        while stack.last().is_some_and(|(end, _)| *end <= span.start) {
            stack.pop();
        }
        // Ignore malformed crossing entities, nested anchors and pathological
        // depth. They must never turn Telegram text into invalid active HTML.
        let anchor = span.close == "</a>";
        if stack.len() >= 64
            || stack.last().is_some_and(|(end, _)| span.end > *end)
            || (anchor && stack.iter().any(|(_, link)| *link))
        {
            continue;
        }
        stack.push((span.end, anchor));
        accepted.push(span);
    }
    let mut html = String::new();
    render_spans(text, &accepted, 0, text.len(), &mut html);
    html
}

/// Checked byte ranges for an HTML entity; nested ranges are rendered recursively.
struct Span {
    start: usize,
    end: usize,
    open: String,
    close: String,
}

/// Escape all literal slices, including those inside pre/code/link entities.
fn render_spans(text: &str, spans: &[Span], start: usize, end: usize, html: &mut String) {
    let mut cursor = start;
    let mut i = 0;
    while let Some(span) = spans.get(i) {
        html.push_str(&escape_html(&text[cursor..span.start]));
        html.push_str(&span.open);
        let mut next = i + 1;
        while spans.get(next).is_some_and(|child| child.start < span.end) {
            next += 1;
        }
        render_spans(text, &spans[i + 1..next], span.start, span.end, html);
        html.push_str(&span.close);
        cursor = span.end;
        i = next;
    }
    html.push_str(&escape_html(&text[cursor..end]));
}

/// A successful empty snapshot removes comments; a failed fetch changes nothing.
fn apply_snapshot(
    archive: &mut Archive,
    post_id: u64,
    result: Result<Vec<Discussion>>,
) -> Result<()> {
    // Advance on failure too: one inaccessible/rate-limited thread must not
    // permanently prevent the next build from checking older posts.
    archive.next_post_id = Some(post_id.saturating_add(1));
    let threads = result?;
    if threads.is_empty() {
        archive.posts.remove(&post_id);
    } else {
        archive.posts.insert(post_id, threads);
    }
    Ok(())
}

/// Timeouts may have interrupted grammers' automatic flood-wait sleep; do not
/// immediately issue another request, including the subsequent media sync.
pub(super) fn should_stop_requests(error: &anyhow::Error) -> bool {
    error.is::<tokio::time::error::Elapsed>()
        || matches!(error.downcast_ref::<grammers_mtsender::InvocationError>(),
            Some(grammers_mtsender::InvocationError::Rpc(error)) if error.code == 420)
}

/// Fetch comments independently of the image backfill and preserve good snapshots.
pub(super) async fn sync(client: &Client, peer: &Peer, posts: &[Post], s: &Settings) -> Result<()> {
    // A user session's access to an arbitrary group is not permission to publish
    // it. Only resolve discussion roots attached to this public broadcast channel.
    anyhow::ensure!(
        matches!(peer, Peer::Channel(_)) && peer.username().is_some(),
        "discussions require a public broadcast channel"
    );
    let mut archive = Archive::load(&s.site, &s.channel)?;
    let original = archive.clone();
    let live: BTreeSet<_> = posts.iter().map(|post| post.primary_id).collect();
    archive.posts.retain(|post, _| live.contains(post));
    let mut ordered: Vec<_> = posts.iter().collect();
    ordered.sort_by_key(|post| post.primary_id);
    if let Some(next) = archive.next_post_id {
        let pivot = ordered.partition_point(|post| post.primary_id < next);
        ordered.rotate_left(pivot);
    }
    let peer_ref: PeerRef = peer.into();
    let deadline = tokio::time::Instant::now() + Duration::from_secs(180);
    let mut attempted = 0;
    let mut completed = 0;
    let mut stopped = None;
    'chunks: for chunk in ordered.chunks(100) {
        if attempted >= 100 || tokio::time::Instant::now() >= deadline {
            break;
        }
        let ids: BTreeSet<i32> = chunk
            .iter()
            .flat_map(|post| &post.ids)
            .filter_map(|id| i32::try_from(*id).ok())
            .collect();
        let ids: Vec<_> = ids.into_iter().collect();
        let mut metadata = BTreeMap::new();
        for batch in ids.chunks(100) {
            let request = async {
                tokio::time::sleep(Duration::from_millis(150)).await;
                client.get_messages_by_id(peer_ref, batch).await
            };
            let until = deadline.min(tokio::time::Instant::now() + Duration::from_secs(15));
            let result = tokio::time::timeout_at(until, request)
                .await
                .context("discussion metadata fetch timed out")
                .and_then(|result| result.map_err(anyhow::Error::from));
            match result {
                Ok(messages) => metadata.extend(batch.iter().copied().zip(messages)),
                Err(error) => {
                    tracing::warn!("MTProto: stopped discussion metadata fetch; saved snapshots retained: {error:#}");
                    stopped = Some(error);
                    break 'chunks;
                }
            }
        }
        for post in chunk {
            if attempted >= 100 || tokio::time::Instant::now() >= deadline {
                break 'chunks;
            }
            let has_thread = post.ids.iter().any(|id| {
                i32::try_from(*id)
                    .ok()
                    .and_then(|id| metadata.get(&id))
                    .and_then(Option::as_ref)
                    .is_some_and(|message| discussion_group(message).is_some())
            });
            if has_thread {
                attempted += 1;
            }
            let until = deadline.min(tokio::time::Instant::now() + Duration::from_secs(30));
            let result =
                tokio::time::timeout_at(until, fetch_post(client, peer_ref, post, &metadata))
                    .await
                    .context("discussion fetch timed out")
                    .and_then(|result| result);
            if let Err(error) = apply_snapshot(&mut archive, post.primary_id, result) {
                tracing::warn!(
                    "MTProto: discussion {} kept previous snapshot: {error:#}",
                    post.primary_id
                );
                if should_stop_requests(&error) {
                    stopped = Some(error);
                    break 'chunks;
                }
            } else {
                completed += 1;
            }
        }
    }
    if completed == ordered.len() {
        archive.next_post_id = None;
    }
    let saved = if archive != original {
        archive.save(&s.site)
    } else {
        Ok(())
    };
    if let Some(error) = stopped {
        // Preserve the Telegram error type even if saving fails: otherwise
        // the caller could mistake a flood wait for an ordinary cache failure.
        if let Err(save_error) = saved {
            tracing::warn!("MTProto: could not save completed discussions: {save_error:#}");
        }
        return Err(error);
    }
    saved?;
    tracing::info!(
        "MTProto: checked {completed} post discussion snapshots ({attempted} discussion fetches)"
    );
    Ok(())
}

/// Extract only channel-comment metadata, not ordinary in-channel replies.
fn discussion_group(message: &Message) -> Option<i64> {
    let tl::enums::Message::Message(message) = &message.raw else {
        return None;
    };
    let tl::enums::MessageReplies::Replies(replies) = message.replies.as_ref()?;
    replies.comments.then_some(replies.channel_id).flatten()
}

/// Telegram returns the auto-forwarded root last. Validate its group and sender,
/// not its forward provenance: a channel may itself have forwarded the post
/// from elsewhere. The API request establishes the source-post association.
/// See <https://core.telegram.org/api/discussion#channel-comments>.
fn discussion_root(
    messages: &[tl::enums::Message],
    group: i64,
    channel: PeerId,
) -> Result<&tl::types::Message> {
    let Some(tl::enums::Message::Message(root)) = messages.last() else {
        bail!("discussion root is unavailable");
    };
    anyhow::ensure!(
        root.peer_id == tl::enums::Peer::Channel(tl::types::PeerChannel { channel_id: group }),
        "discussion root belongs to another group"
    );
    anyhow::ensure!(
        root.from_id
            .as_ref()
            .is_some_and(|sender| PeerId::from(sender.clone()) == channel),
        "discussion root belongs to another channel"
    );
    anyhow::ensure!(root.id > 0, "invalid discussion root ID");
    Ok(root)
}

/// Fetch all distinct album roots atomically for one generated post.
async fn fetch_post(
    client: &Client,
    peer: PeerRef,
    post: &Post,
    metadata: &BTreeMap<i32, Option<Message>>,
) -> Result<Vec<Discussion>> {
    let mut threads = BTreeMap::new();
    for id in &post.ids {
        let id = i32::try_from(*id).context("invalid channel message ID")?;
        let message = metadata
            .get(&id)
            .and_then(Option::as_ref)
            .context("missing channel message metadata")?;
        anyhow::ensure!(
            matches!(message.raw, tl::enums::Message::Message(_)),
            "channel message is unavailable"
        );
        let Some(group) = discussion_group(message) else {
            continue;
        };
        tokio::time::sleep(Duration::from_millis(150)).await;
        let tl::enums::messages::DiscussionMessage::Message(discussion) = client
            .invoke(&tl::functions::messages::GetDiscussionMessage {
                peer: peer.into(),
                msg_id: id,
            })
            .await?;
        let root = discussion_root(&discussion.messages, group, peer.id)?;
        let key = (group, root.id);
        if threads.contains_key(&key) {
            continue;
        }
        let peers = PeerMap::new(discussion.users.clone(), discussion.chats.clone());
        let group_peer: PeerRef = peers
            .get(PeerId::channel(group))
            .context("missing discussion group access")?
            .into();
        let root_id = root.id;
        let comments =
            collect_comments(|offset| fetch_page(client, group_peer, root_id, offset)).await?;
        threads.insert(
            key,
            Discussion {
                peer_id: group,
                root_id,
                post_id: id as u64,
                comments,
            },
        );
    }
    Ok(threads.into_values().collect())
}

/// One raw API page, filtered to this exact group/thread with no side effects.
async fn fetch_page(client: &Client, peer: PeerRef, root: i32, offset: i32) -> Result<ReplyPage> {
    tokio::time::sleep(Duration::from_millis(150)).await;
    let result = client
        .invoke(&tl::functions::messages::GetReplies {
            peer: peer.into(),
            msg_id: root,
            offset_id: offset,
            offset_date: 0,
            add_offset: 0,
            limit: 100,
            max_id: 0,
            min_id: 0,
            hash: 0,
        })
        .await?;
    let (messages, users, chats) = match result {
        tl::enums::messages::Messages::Messages(result) => {
            (result.messages, result.users, result.chats)
        }
        tl::enums::messages::Messages::Slice(result) => {
            (result.messages, result.users, result.chats)
        }
        tl::enums::messages::Messages::ChannelMessages(result) => {
            (result.messages, result.users, result.chats)
        }
        tl::enums::messages::Messages::NotModified(_) => bail!("unexpected unchanged reply page"),
    };
    let peers = PeerMap::new(users, chats);
    let mut comments = Vec::new();
    let mut oldest_id: Option<i32> = None;
    for raw in messages {
        let message = Message::from_raw(client, raw, Some(peer), &peers);
        oldest_id = Some(oldest_id.map_or(message.id(), |old| old.min(message.id())));
        let tl::enums::Message::Message(raw) = &message.raw else {
            continue;
        };
        if raw.id == root {
            continue;
        }
        anyhow::ensure!(
            message.peer_id() == peer.id,
            "reply belongs to another group"
        );
        let Some(tl::enums::MessageReplyHeader::Header(reply)) = &raw.reply_to else {
            bail!("comment has no thread identity");
        };
        anyhow::ensure!(
            reply.reply_to_top_id.or(reply.reply_to_msg_id) == Some(root),
            "reply belongs to another thread"
        );
        let author = match message.sender() {
            Some(Peer::User(user)) => user.full_name(),
            Some(peer) => peer.name().unwrap_or("Unknown author").to_string(),
            None => raw
                .post_author
                .clone()
                .unwrap_or_else(|| "Unknown author".into()),
        };
        comments.push(Comment {
            id: raw.id,
            author: if author.is_empty() {
                "Deleted account".into()
            } else {
                author
            },
            date: message.date().to_rfc3339(),
            body_html: comment_html(&raw.message, raw.entities.as_deref().unwrap_or(&[])),
            reply_to: reply.reply_to_msg_id,
            has_media: raw.media.as_ref().is_some_and(|media| {
                !matches!(
                    media,
                    tl::enums::MessageMedia::Empty | tl::enums::MessageMedia::WebPage(_)
                )
            }),
        });
    }
    Ok(ReplyPage {
        comments,
        oldest_id,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Synthetic errors test policy without a session or live Telegram traffic.
    #[tokio::test]
    async fn rate_limits_and_timeouts_stop_requests_but_bad_threads_do_not() {
        let rpc = |code, name: &str| {
            anyhow::Error::from(grammers_mtsender::InvocationError::Rpc(
                grammers_mtsender::RpcError::from(tl::types::RpcError {
                    error_code: code,
                    error_message: name.into(),
                }),
            ))
            .context("fetching discussion")
        };
        assert!(should_stop_requests(&rpc(420, "FLOOD_WAIT_120")));
        assert!(!should_stop_requests(&rpc(400, "MSG_ID_INVALID")));
        let timeout = tokio::time::timeout(Duration::ZERO, std::future::pending::<()>())
            .await
            .unwrap_err();
        assert!(should_stop_requests(
            &anyhow::Error::from(timeout).context("fetching discussion")
        ));

        let mut archive = Archive::empty("channel");
        let saved = vec![Discussion {
            peer_id: 200,
            root_id: 10,
            post_id: 42,
            comments: vec![comment(11, "saved")],
        }];
        archive.posts.insert(42, saved.clone());
        let failure =
            apply_snapshot(&mut archive, 42, Err(rpc(420, "FLOOD_WAIT_120"))).unwrap_err();
        assert!(should_stop_requests(&failure));
        assert_eq!(archive.next_post_id, Some(43));
        assert_eq!(archive.posts.get(&42), Some(&saved));
    }

    /// Service-only pages still advance the cursor; they are not empty threads.
    #[tokio::test]
    async fn pagination_continues_past_a_service_only_page() {
        let result = collect_comments(|offset| {
            std::future::ready(Ok(match offset {
                0 => page(Vec::new(), Some(20)),
                20 => page(vec![comment(11, "older")], Some(11)),
                11 => page(Vec::new(), None),
                _ => panic!("unexpected offset"),
            }))
        })
        .await
        .unwrap();
        assert_eq!(result, vec![comment(11, "older")]);
    }

    /// Raw channel/group fixture without any Telegram account or client session.
    fn root_message(id: i32, group: i64, sender: i64) -> tl::types::Message {
        tl::types::Message {
            out: false,
            mentioned: false,
            media_unread: false,
            silent: false,
            post: false,
            from_scheduled: false,
            legacy: false,
            edit_hide: false,
            pinned: false,
            noforwards: false,
            invert_media: false,
            offline: false,
            video_processing_pending: false,
            paid_suggested_post_stars: false,
            paid_suggested_post_ton: false,
            id,
            from_id: Some(tl::types::PeerChannel { channel_id: sender }.into()),
            from_boosts_applied: None,
            peer_id: tl::types::PeerChannel { channel_id: group }.into(),
            saved_peer_id: None,
            fwd_from: None,
            via_bot_id: None,
            via_business_bot_id: None,
            reply_to: None,
            date: 0,
            message: "Root".into(),
            media: None,
            reply_markup: None,
            entities: None,
            views: None,
            forwards: None,
            replies: None,
            edit_date: None,
            post_author: None,
            grouped_id: None,
            reactions: None,
            restriction_reason: None,
            ttl_period: None,
            quick_reply_shortcut_id: None,
            effect: None,
            factcheck: None,
            report_delivery_until_date: None,
            paid_message_stars: None,
            suggested_post: None,
        }
    }

    /// Forward provenance can belong to another channel even though the linked
    /// public channel is the sender of the automatically forwarded root.
    #[test]
    fn discussion_root_accepts_original_forward_from_another_channel() {
        let mut root = root_message(10, 200, 100);
        root.fwd_from = Some(
            tl::types::MessageFwdHeader {
                imported: false,
                saved_out: false,
                from_id: Some(tl::types::PeerChannel { channel_id: 300 }.into()),
                from_name: None,
                date: 0,
                channel_post: Some(999),
                post_author: None,
                saved_from_peer: None,
                saved_from_msg_id: None,
                saved_from_id: None,
                saved_from_name: None,
                saved_date: None,
                psa_type: None,
            }
            .into(),
        );
        let messages = [root_message(11, 200, 400).into(), root.clone().into()];
        assert_eq!(
            discussion_root(&messages, 200, PeerId::channel(100)).unwrap(),
            &root
        );
    }

    /// Only the final API message is the root; do not search newer comments for
    /// a plausible replacement when the root's identity is invalid.
    #[test]
    fn discussion_root_rejects_other_groups_senders_and_missing_roots() {
        let channel = PeerId::channel(100);
        for wrong in [root_message(10, 201, 100), root_message(10, 200, 101)] {
            let messages = [root_message(11, 200, 100).into(), wrong.into()];
            assert!(discussion_root(&messages, 200, channel).is_err());
        }
        assert!(discussion_root(&[], 200, channel).is_err());
        let unavailable = [tl::types::MessageEmpty {
            id: 10,
            peer_id: None,
        }
        .into()];
        assert!(discussion_root(&unavailable, 200, channel).is_err());
        let root = root_message(10, 200, 100);
        assert_eq!(
            discussion_root(&[root.clone().into()], 200, channel).unwrap(),
            &root
        );
    }

    /// A text-only comment fixture, independent of real accounts and Telegram.
    fn comment(id: i32, body: &str) -> Comment {
        Comment {
            id,
            author: "Alice".into(),
            date: "2026-10-12T10:00:00Z".into(),
            body_html: body.into(),
            reply_to: None,
            has_media: false,
        }
    }

    /// Mock pages retain old raw message IDs even when no visible comment exists.
    fn page(comments: Vec<Comment>, oldest_id: Option<i32>) -> ReplyPage {
        ReplyPage {
            comments,
            oldest_id,
        }
    }

    #[tokio::test]
    async fn pagination_fetches_all_pages_in_chronological_order() {
        let mut calls = Vec::new();
        let result = collect_comments(|offset| {
            calls.push(offset);
            std::future::ready(Ok(match offset {
                0 => page(vec![comment(13, "last"), comment(12, "middle")], Some(12)),
                12 => page(vec![comment(11, "first")], Some(11)),
                11 => page(Vec::new(), None),
                _ => panic!("unexpected offset"),
            }))
        })
        .await
        .unwrap();
        assert_eq!(calls, [0, 12, 11]);
        assert_eq!(
            result.iter().map(|c| c.id).collect::<Vec<_>>(),
            [11, 12, 13]
        );
    }

    #[tokio::test]
    async fn incomplete_pagination_never_replaces_the_saved_discussion() {
        let mut archive = Archive::empty("channel");
        let old = vec![Discussion {
            peer_id: 200,
            root_id: 10,
            post_id: 42,
            comments: vec![comment(11, "saved")],
        }];
        archive.posts.insert(42, old.clone());
        let result = collect_comments(|offset| {
            std::future::ready(if offset == 0 {
                Ok(page(vec![comment(12, "partial")], Some(12)))
            } else {
                Err(anyhow::anyhow!("mock timeout"))
            })
        })
        .await
        .map(|comments| {
            vec![Discussion {
                peer_id: 200,
                root_id: 10,
                post_id: 42,
                comments,
            }]
        });
        assert!(apply_snapshot(&mut archive, 42, result).is_err());
        assert_eq!(archive.posts.get(&42), Some(&old));
    }

    #[tokio::test]
    async fn pagination_rejects_a_nonadvancing_server_cursor() {
        let result =
            collect_comments(|_| std::future::ready(Ok(page(vec![comment(12, "same")], Some(12)))))
                .await;
        assert!(result.is_err());
    }

    #[test]
    fn complete_snapshots_replace_edits_and_deletions_and_clear_zero_comments() {
        let mut archive = Archive::empty("channel");
        archive.posts.insert(
            42,
            vec![Discussion {
                peer_id: 200,
                root_id: 10,
                post_id: 42,
                comments: vec![comment(11, "old"), comment(12, "deleted")],
            }],
        );
        let edited = vec![Discussion {
            peer_id: 200,
            root_id: 10,
            post_id: 42,
            comments: vec![comment(11, "edited")],
        }];
        apply_snapshot(&mut archive, 42, Ok(edited.clone())).unwrap();
        assert_eq!(archive.posts.get(&42), Some(&edited));
        apply_snapshot(&mut archive, 42, Ok(Vec::new())).unwrap();
        assert!(!archive.posts.contains_key(&42));
    }

    #[test]
    fn literal_comment_html_is_escaped_and_utf16_formatting_keeps_emoji() {
        assert_eq!(
            comment_html("<script>&'\"\n", &[]),
            "&lt;script&gt;&amp;&#39;&quot;\n"
        );
        let entities = vec![tl::types::MessageEntityBold {
            offset: 2,
            length: 4,
        }
        .into()];
        assert_eq!(
            comment_html("😀bold <b>", &entities),
            "😀<b>bold</b> &lt;b&gt;"
        );
        let invalid = vec![tl::types::MessageEntityBold {
            offset: 1,
            length: 3,
        }
        .into()];
        assert_eq!(comment_html("😀bold", &invalid), "😀bold");
    }

    #[test]
    fn comment_links_escape_attributes_and_disallow_active_url_schemes() {
        let link = |url: &str| {
            vec![tl::types::MessageEntityTextUrl {
                offset: 0,
                length: 4,
                url: url.into(),
            }
            .into()]
        };
        assert_eq!(comment_html("link", &link("javascript:alert(1)")), "link");
        assert_eq!(
            comment_html("link", &link("https://example.test/?q='&x=1")),
            "<a href='https://example.test/?q=&#39;&amp;x=1'>link</a>"
        );
    }

    #[test]
    fn nested_and_crossing_entities_never_produce_misnested_html() {
        let nested = vec![
            tl::types::MessageEntityBold {
                offset: 0,
                length: 4,
            }
            .into(),
            tl::types::MessageEntityItalic {
                offset: 1,
                length: 2,
            }
            .into(),
        ];
        assert_eq!(comment_html("abcd", &nested), "<b>a<i>bc</i>d</b>");
        let crossing = vec![
            tl::types::MessageEntityBold {
                offset: 0,
                length: 3,
            }
            .into(),
            tl::types::MessageEntityItalic {
                offset: 1,
                length: 3,
            }
            .into(),
        ];
        assert_eq!(comment_html("abcd", &crossing), "<b>abc</b>d");
    }
}
