# tg2zola

🌐 **English** · [Беларуская](README.be.md) · [Українська](README.uk.md) · [Русский](README.ru.md) · [Deutsch](README.de.md) · [Français](README.fr.md) · [中文](README.zh.md) · [日本語](README.ja.md) · [Polski](README.pl.md) · [Español](README.es.md) · [한국어](README.ko.md) · [ქართული](README.ka.md) · [हिन्दी](README.hi.md)

[![daily build](https://github.com/vitaly-zdanevich/telegram_channel_to_static_website/actions/workflows/daily.yml/badge.svg)](https://github.com/vitaly-zdanevich/telegram_channel_to_static_website/actions/workflows/daily.yml)

[![Quality Gate Status](https://sonarcloud.io/api/project_badges/measure?project=vitaly-zdanevich_telegram_channel_to_static_website&metric=alert_status)](https://sonarcloud.io/summary/new_code?id=vitaly-zdanevich_telegram_channel_to_static_website)
[![Coverage](https://sonarcloud.io/api/project_badges/measure?project=vitaly-zdanevich_telegram_channel_to_static_website&metric=coverage)](https://sonarcloud.io/summary/new_code?id=vitaly-zdanevich_telegram_channel_to_static_website)
[![Bugs](https://sonarcloud.io/api/project_badges/measure?project=vitaly-zdanevich_telegram_channel_to_static_website&metric=bugs)](https://sonarcloud.io/summary/new_code?id=vitaly-zdanevich_telegram_channel_to_static_website)
[![Vulnerabilities](https://sonarcloud.io/api/project_badges/measure?project=vitaly-zdanevich_telegram_channel_to_static_website&metric=vulnerabilities)](https://sonarcloud.io/summary/new_code?id=vitaly-zdanevich_telegram_channel_to_static_website)
[![Code Smells](https://sonarcloud.io/api/project_badges/measure?project=vitaly-zdanevich_telegram_channel_to_static_website&metric=code_smells)](https://sonarcloud.io/summary/new_code?id=vitaly-zdanevich_telegram_channel_to_static_website)
[![Duplicated Lines](https://sonarcloud.io/api/project_badges/measure?project=vitaly-zdanevich_telegram_channel_to_static_website&metric=duplicated_lines_density)](https://sonarcloud.io/summary/new_code?id=vitaly-zdanevich_telegram_channel_to_static_website)
[![Maintainability](https://sonarcloud.io/api/project_badges/measure?project=vitaly-zdanevich_telegram_channel_to_static_website&metric=sqale_rating)](https://sonarcloud.io/summary/new_code?id=vitaly-zdanevich_telegram_channel_to_static_website)
[![Reliability](https://sonarcloud.io/api/project_badges/measure?project=vitaly-zdanevich_telegram_channel_to_static_website&metric=reliability_rating)](https://sonarcloud.io/summary/new_code?id=vitaly-zdanevich_telegram_channel_to_static_website)

Google Lighthouse (mobile, updated by the daily run):
[![Lighthouse Performance](https://img.shields.io/endpoint?url=https%3A%2F%2Fvitaly-zdanevich.github.io%2Ftelegram_channel_to_static_website%2Flighthouse-performance.json)](https://pagespeed.web.dev/analysis?url=https://vitaly-zdanevich.github.io/telegram_channel_to_static_website/)
[![Lighthouse Accessibility](https://img.shields.io/endpoint?url=https%3A%2F%2Fvitaly-zdanevich.github.io%2Ftelegram_channel_to_static_website%2Flighthouse-accessibility.json)](https://pagespeed.web.dev/analysis?url=https://vitaly-zdanevich.github.io/telegram_channel_to_static_website/)
[![Lighthouse Best Practices](https://img.shields.io/endpoint?url=https%3A%2F%2Fvitaly-zdanevich.github.io%2Ftelegram_channel_to_static_website%2Flighthouse-best-practices.json)](https://pagespeed.web.dev/analysis?url=https://vitaly-zdanevich.github.io/telegram_channel_to_static_website/)
[![Lighthouse SEO](https://img.shields.io/endpoint?url=https%3A%2F%2Fvitaly-zdanevich.github.io%2Ftelegram_channel_to_static_website%2Flighthouse-seo.json)](https://pagespeed.web.dev/analysis?url=https://vitaly-zdanevich.github.io/telegram_channel_to_static_website/)
[![Security](https://sonarcloud.io/api/project_badges/measure?project=vitaly-zdanevich_telegram_channel_to_static_website&metric=security_rating)](https://sonarcloud.io/summary/new_code?id=vitaly-zdanevich_telegram_channel_to_static_website)
[![Lines of Code](https://sonarcloud.io/api/project_badges/measure?project=vitaly-zdanevich_telegram_channel_to_static_website&metric=ncloc)](https://sonarcloud.io/summary/new_code?id=vitaly-zdanevich_telegram_channel_to_static_website)
[![Technical Debt](https://sonarcloud.io/api/project_badges/measure?project=vitaly-zdanevich_telegram_channel_to_static_website&metric=sqale_index)](https://sonarcloud.io/summary/new_code?id=vitaly-zdanevich_telegram_channel_to_static_website)

Back up a **public Telegram channel** into a **self-contained [Zola](https://www.getzola.org/) static website**.

> **Regenerate on demand:** click the **daily build** badge above → **Run
> workflow** to scrape + rebuild + redeploy without waiting for the schedule
> (details in [Automation](#automation-github-actions)).

It scrapes the public web preview (`https://t.me/s/<channel>`), downloads all
media locally, and regenerates a complete Zola blog on every run. **No Telegram
bot, token, or API is needed** for that — it reads only the public web page (the
optional [MTProto backend](#optional-mtproto-backend) does log in with API
credentials to add audio/photos/videos). The output has
**no Telegram dependency**: media is local, there are no embeds, and links to the
channel's *own* posts are rewritten to internal relative links — so the site
keeps working even if the channel is later removed. Links you wrote to other
sites (including other Telegram channels) are preserved as normal links. It's a
backup, not a mirror.

Written in Rust: a single static binary, easy to run locally or in CI.

## What it does

- **Full history** — walks the channel backwards via the preview's `?before=`
  cursor until the first message, and **regenerates every page** each run.
- **Self-contained media** — downloads photos, videos, audio (`.ogg/.oga/.mp3`),
  documents and stickers into each post's bundle (which doubles as the cache).
  Photos are content-addressed by their stable Telegram file id, so editing a
  post's text never re-downloads its media, while **replacing** an image (the
  post then shows as *edited*) fetches the new file and prunes the old one.
- **True-black default theme** — built-in templates styled `#000` in dark mode
  via `prefers-color-scheme` (OLED-friendly), with no external theme dependency.
  An external theme can be layered on with a guaranteed fallback (see [Theming](#theming)).
- **Smart video handling** (in priority order):
  1. attached video **+ a live YouTube link** (or an Instagram link with
     `INSTAGRAM`/`--instagram` enabled) → embed it, drop the video (the CI
     default, to save space; a dead link keeps the video). Locally everything is
     downloaded by default — `KEEP_MEDIA`/`--keep-media` forces it;
  2. directly downloadable video → local `<video>`;
  3. otherwise → save the **poster frame** + duration (the public page doesn't
     expose the file; see [Limitations](#limitations)).
- **Formatting** — bold, italic, strikethrough, code/pre, links and spoilers are
  converted to Markdown (Telegram's UTF-16 entity offsets handled correctly).
- **Hashtags → tags** — `#hashtags` become Zola `tags` taxonomy terms, so you get
  tag pages for free, while still showing as text in the post.
- **Grouped posts** — albums are one post automatically; bursts of messages
  posted at the same instant (e.g. forwarding several at once) are merged.
- **Self-navigating** — a link to another message in the *same* channel becomes a
  relative link to that post in the blog; links to other channels stay external.
- **Engagement** — exports per-post **view counts**. (Reactions/likes aren't
  available from the public page — see [Limitations](#limitations).)
- **RSS feed** — a standard `/rss.xml` of **every post** with full content (a
  complete feed, not just recent items), advertised via a `<link rel="alternate">`
  so feed readers auto-discover it from the site URL. On by default; disable
  with `RSS=false` / `--no-rss`.
- **Rich link previews + Mastodon** — every page emits Open Graph and Twitter
  Card tags (title, description, the post's first image), so shared links render
  as cards. Set `FEDIVERSE_CREATOR` to add an author byline on Mastodon previews
  and verify the site on your profile (see [Fediverse](#fediverse--mastodon)).
- **Minimal JavaScript, offline-ready** — dark mode and spoilers are CSS-only.
  Search is the only JS: the default Elasticlunr is client-side and self-contained,
  so it **works in the offline `file://` copy too** — `tg2zola offline <public-dir>`
  rewrites the built site to relative links and keeps the local search scripts
  (relativized) while stripping the network-dependent ones (Zola's pagination
  redirect, embed loaders), so it opens straight from `file://` with no web
  server. For a fully JavaScript-free site, set `SEARCH_ENGINE=google` (a plain
  `<form>`) or `none`.
- **Localized UI** — the site chrome (Newer/Older/Tags/About, the search box,
  dates) renders in any of 13 languages via `LANGUAGE` / `--language`
  (en/be/uk/ru/de/fr/zh/ja/pl/es/ko/ka/hi), with month and weekday names localized
  too. Post content keeps the channel's own language.

## Install

Grab a static binary for your architecture from the
[Releases](../../releases) page (Linux `amd64` / `arm64`, musl), or build from
source:

```sh
cargo build --release
# binary at target/release/tg2zola
```

You also need the [`zola`](https://www.getzola.org/documentation/getting-started/installation/)
binary to turn the generated content into HTML.

## Usage

```sh
# Generate the Zola site (scaffolds config + templates on first run):
tg2zola --channel durov --site site --init-site

# Build the static HTML:
zola --root site build       # output in site/public/

# (optional) Make it viewable with NO web server, straight from disk:
tg2zola offline site/public  # then open site/public/index.html via file://
```

Quick local test (one page, ~20 messages):

```sh
tg2zola --channel <name> --site /tmp/site --init-site --max-pages 1
```

All options live in [`tg2zola.toml`](tg2zola.toml) (CLI flags override it):

```sh
tg2zola --config tg2zola.toml
```

Run `tg2zola --help` for the full flag list.

## How it's wired

```
t.me/s/<channel>          scrape + paginate        (scrape.rs / parse.rs)
        │  HTML
        ▼
   RawMessage[]           HTML → Markdown           (html2md.rs)
        │
        ▼
     Post[]               group albums + bursts     (group.rs)
        │
        ▼
 site/content/posts/…     reconcile bundles +        (render.rs / site.rs
        │                 cache-aware media download   + media.rs)
        ▼  zola build      (the bundle is the cache)
 site/public/             self-contained static site
```

Each post becomes a Zola page bundle — `content/posts/<date>-<id>/index.md` with
TOML front matter and its media files alongside it. `config.toml` and the
built-in templates are regenerated deterministically every run, and `write_site`
reconciles the tree: rewrite all Markdown, keep already-cached media, and prune
deleted posts and stale files.

## Automation (GitHub Actions)

Two workflows are included:

- **[`daily.yml`](.github/workflows/daily.yml)** — runs once a day (and on
  demand): restore the previous site from the `blog` branch (the media cache) →
  scrape + regenerate → `zola build` → **deploy to GitHub Pages** (the published
  result) → commit the refreshed site back to the **`blog` branch**.
- **[`release.yml`](.github/workflows/release.yml)** — on every pushed `v*` tag,
  cross-compiles static `amd64` + `arm64` (musl) binaries and uploads them to the
  GitHub Release.

To enable publishing: in the repo, **Settings → Pages → Build and deployment →
Source: GitHub Actions**. No secrets are required — everything is public-scrape.
The published site is always **GitHub Pages**; the `blog` branch is a durable
copy and never affects what visitors see.

### Regenerate now (don't wait for the daily run)

`daily.yml` has `workflow_dispatch` enabled, so you can trigger a fresh scrape +
rebuild + redeploy on demand — it runs exactly the same steps as the scheduled
run:

- **In the browser:** open **[Actions → "daily" → Run workflow](../../actions/workflows/daily.yml)**
  and click the green **Run workflow** button. (Add the status badge above to
  your README for one-click access.)
- **From the terminal:** `gh workflow run daily.yml` (GitHub CLI), then
  `gh run watch` to follow it.

**Which channel?** The channel isn't committed, so each deployment sets its own.
Set a repository **variable** `CHANNEL` (Settings → Secrets and variables → Actions
→ Variables) to the public channel username — it's a *variable*, not a secret,
since the channel is public. (Or uncomment `channel = "…"` in
[`tg2zola.toml`](tg2zola.toml) in your fork.) `THEME_REPO` works as a variable too.

### The `blog` branch (archive + cache)

Each run commits the generated site (Markdown + media + built-in templates —
everything except the built `public/` and any external theme) to a `blog`
branch, leaving `main` code-only. It does double duty:

- **Cache** — the next run restores media from it instead of re-downloading.
- **Durable archive** — a complete, buildable Zola site you can clone and mirror
  anywhere, so the backup isn't locked to one platform:

```sh
git clone -b blog https://github.com/<you>/<repo> my-archive
zola --root my-archive/site serve          # browse it offline

git -C my-archive remote add gitlab git@gitlab.com:<you>/<repo>.git
git -C my-archive push gitlab blog          # mirror it elsewhere
```

Media is committed as plain git blobs; for very large channels consider Git LFS
or occasional history squashing.

### Theming

The default is the built-in **true-black** theme — zero external dependencies, so
the site always builds. To use an external [Zola theme](https://www.getzola.org/themes/),
set a repository **variable** `THEME_REPO` (Settings → Secrets and variables →
Actions → Variables) to its git URL (https, or ssh with a deploy key). The
workflow clones it and builds with it — and **if the theme is missing or its
build fails, it automatically falls back to the built-in templates**, so a theme
problem can never take the blog offline. Note that external themes expect a
particular content layout, so not every theme is drop-in compatible.

Cut a release:

```sh
git tag v0.1.0 && git push origin v0.1.0
```

## Configuration

Everything is configurable via a repository **variable** (Settings → Secrets and
variables → Actions → **Variables**) for the GitHub Actions flow, or the
equivalent CLI flag / [`tg2zola.toml`](tg2zola.toml) key when running locally.
These are *variables*, not secrets — all of it is public.

| Repo variable | CLI flag | Default | What it does |
|---|---|---|---|
| `CHANNEL` | `--channel` | **required** | Public channel to sync |
| `TITLE` | `--title` | channel username | Blog title (header + `<title>`) |
| `ABOUT` | `--about` | description + stats + repo link | Custom HTML for the About page body |
| `PAGES` | `--pages` | — | Extra pages: Markdown, each `# Title` heading starts a new page + nav entry |
| `POSTS_PER_PAGE` | `--posts-per-page` | `20` | Full posts per page on the home feed |
| `TAGS_FOOTER` | `--tags-footer` | off | `true` to show the per-post tag footer (tags are clickable in the body regardless) |
| `NEXT_PREV` | `--no-next-prev` | on | `false` hides the Next/Prev post navigation |
| `TELEGRAM_LINK` | `--no-telegram-link` | on | `false` hides the per-post "View on Telegram" link |
| `RSS` | `--no-rss` | on | `false` disables the RSS feed at `/rss.xml` (with reader autodiscovery) |
| `FEDIVERSE_CREATOR` | `--fediverse-creator` | — | Mastodon `@user@instance` → `fediverse:creator` byline + `rel="me"` profile link |
| `SEARCH_ENGINE` | `--search-engine` | `elasticlunr` | Header search box: `google` (JS-free form) / `duckduckgo` / `yandex` / `bing` / `elasticlunr` ([Elasticlunr](http://elasticlunr.com/) — Zola's built-in client-side full-text search over the post content — needs JS, bundled with no CDN; the offline copy strips it, so it works on the deployed site only) / `none` |
| `SEARCH_URL` | `--search-url` | — | Custom search URL prefix; the query is appended on Enter (overrides the engine) |
| `TITLE_MAX_LEN` | `--title-max-len` | `200` | Max post-title length (chars); a truncated title keeps its full first sentence in the body |
| `FOOTER` | `--footer` | — | Footer content — plain text, Markdown or HTML |
| `PAGES_HOST` | `--pages-host` | auto | Host for the About-page size limit: `github` / `gitlab` / `none` (auto-detected from the URL) |
| `DATE_FORMAT` | `--date-format` | `%Y %B %d` | strftime format for displayed dates (e.g. `2025 October 28`; `%Y` for year only) |
| `LANGUAGE` | `--language` | `en` | UI language for the site chrome (Newer/Older/Tags/About/…): `en`/`be`/`uk`/`ru`/`de`/`fr`/`zh`/`ja`/`pl`/`es`/`ko`/`ka` (Georgian)/`hi` (Hindi). Post content keeps the channel's own language; dates are localized to match |
| `DERIVE_TITLES` | `--derive-titles` | off | `true` derives a post title from its first sentence; default shows a clickable `#id` on the post's date/views line instead |
| `STRIP_TITLE` | `--strip-title` | off | With `DERIVE_TITLES`, also remove that first sentence from the body so it isn't shown twice |
| `LINK_UNDERLINE` | `--link-underline` | off | `true` underlines links (default: no underline) |
| `YOUTUBE_FACADE` | `--youtube-facade` | off | `true` for a no-JS click-to-load YouTube thumbnail (default: direct iframe) |
| `CAROUSEL` | `--carousel` | off | `true` shows a multi-image post as a swipeable **carousel** (one image at a time, with arrows + dots) instead of a vertical stack. The swipe is native CSS `scroll-snap`; arrows/dots need JavaScript. Default is the stack (no JS) |
| `KEEP_MEDIA` | `--keep-media` | CI: off · local: on | `true` to keep (download + show) attached **video/audio** even when the post links YouTube / Apple Podcasts / Instagram. The default is environment-aware: on CI (GitHub Actions / GitLab) the embed replaces the attached media to save hosting space; on a local machine everything is downloaded for a complete backup |
| `GENIUS` | `--no-genius` | on | `false` skips resolving genius.com links (for their YouTube video + lyrics-widget id) |
| `GENIUS_TOKEN` (secret) | — | — | Genius API [Client Access Token](https://genius.com/api-clients). When set, genius links resolve via the **API** (works in CI) instead of scraping — the genius web pages are Cloudflare-blocked on datacenter IPs. Store it as an Actions **secret**, not a variable |
| `SPOTIFY` | `--spotify` | off | `true` to replace a Spotify link with the Spotify player (opt-in — it plays a ~30s preview for non-Premium listeners) |
| `INSTAGRAM` | `--instagram` | off | `true` to embed a live Instagram post in place of an attached video (opt-in — the widget needs JavaScript and loads from instagram.com; otherwise the attached video is kept) |
| `PINTEREST` | `--no-pinterest` | on | `false` stops replacing a Pinterest pin link with the embedded pin |
| `PINTEREST_SAVE` | `--pinterest-save` | off | `true` adds a Pinterest **"Save" hover button** to the site's own images so visitors can pin them to their boards (opt-in; needs JavaScript, loads pinit.js) |
| `LIVENESS` | `--no-liveness` | on | `false` skips the liveness checks (YouTube / Apple Podcasts / Yandex / Instagram / Spotify / Pinterest). Otherwise a removed item keeps its local media, or shows the plain link, instead of a dead/broken embed |
| `REACTIONS` | `--no-reactions` | on | show per-emoji reaction counts (👍 42 · ❤️ 10). Paid "stars" reactions show ⭐; custom emojis show their standard-emoji fallback. **Requires the [MTProto backend](#optional-mtproto-backend)** — `t.me/s/` never exposes reactions. `false` hides them |
| `ABOUT_ME` | `--no-about-me` | on | if the channel description links to an [about.me](https://about.me) profile, pull its photo, bio, social links and a contact button onto the About page (about.me blocks iframing, so the contact is a link). `false` disables it |
| `PAGESPEED` | `--no-pagespeed` | on | `false` skips fetching Google Lighthouse (mobile) scores for the deployed site. When on, the four category scores are shown on the About page and written as shields.io badge endpoints (`lighthouse-*.json`) for the README. Needs a live `base_url`; measures the previous deploy |
| `PAGESPEED_API_KEY` (secret) | — | — | Optional [PageSpeed Insights API key](https://developers.google.com/speed/docs/insights/v5/get-started) to raise the rate limit (one call/day works without it) |
| `PWA` | `--no-pwa` | on | installable **PWA** — a web app manifest (`display: standalone`, so the site can be installed and hides the address bar) plus a service worker that caches pages/media as you browse. On by default; needs JavaScript, built-in templates only |
| `OFFLINE` | `--offline` | off | `true` makes the service worker **precache the whole archive** (audio, video, attachments) on the first visit over **any non-cellular connection** (Wi-Fi or wired — it skips only cellular / Data Saver), so the site works fully offline (not just visited pages) |
| `VIDEO_RELEASES` | `--no-video-releases` | on | videos are uploaded to **this repo's GitHub Releases** and played from there, so they don't count against the Pages **1 GB** quota (release assets are separate storage — 2 GB/file, CDN-backed). `false` keeps videos inline. Needs a `github.com` repo + the workflow's release-upload step |
| `TAGS_TO_PAGES` | `--tags-to-pages` | — | Comma-separated tags shown as `#tag` links in the top nav (e.g. `music, batumi, cooking`) |
| `BACKGROUND_DARK_COLOR` | `--background-dark-color` | `#000000` | Dark-mode background (any CSS color) |
| `BACKGROUND_LIGHT_COLOR` | `--background-light-color` | `#ffffff` | Light-mode background |
| `CSS` | `--css` | — | Extra CSS appended to the built-in stylesheet |
| `FONT` | `--font` | system | Body `font-family` — a local/system font stack, e.g. `Georgia, serif` |
| `GOOGLE_FONT` | `--google-font` | — | A [Google Fonts](https://fonts.google.com) family to load and use for the body, e.g. `Inter` (an external request to fonts.googleapis.com; wins over `FONT`) |
| `GOOGLE_ANALYTICS` | `--google-analytics` | — | Google Analytics measurement ID (e.g. `G-XXXXXXX`) — injects `gtag.js`. Adds JavaScript + a third-party request |
| `YANDEX_METRICA` | `--yandex-metrica` | — | Yandex Metrica counter ID (a number) — injects the Metrica tag (+ a `<noscript>` pixel). Adds JavaScript + a third-party request |
| `THEME_REPO` | `--theme` (name) | built-in black theme | External Zola theme git URL (https/ssh); auto-falls-back if it fails |
| `REPO_URL` | `--repo-url` | tg2zola repo | "Source repository" link on About (CI auto-sets it to your repo) |

The **About** page shows the channel **avatar** (full size), its description and
stats, the **on-disk size** — with the per-kind breakdown and, on GitHub/GitLab
Pages, the share of the host's ~1 GB published-site limit (linked to the host
docs) — plus the repo link. The header shows the avatar as a thumbnail and a
favicon; hashtags are clickable in post bodies and produce `/tags/<tag>/` pages.

A single `PAGES` variable can define several pages — each `# Title` heading
starts a new one:

```
# Page title
Page content in Markdown (which may contain raw HTML).

# Another page
More content.
```

→ `/page-title/` and `/another-page/`, each linked in the nav. No extra
dependency — Zola already renders Markdown.

## Fediverse / Mastodon

Every page carries Open Graph + Twitter Card tags, so a shared post link renders
as a card (title, description, the post's first image) in Mastodon, Slack,
Discord, X, etc. Set `FEDIVERSE_CREATOR` to your `@user@instance` handle to also:

- add a **`fediverse:creator`** byline, so Mastodon shows "*by @you@instance*" on
  the link preview and links to your profile;
- emit a **`rel="me"`** link to your profile, so you can add this site to your
  Mastodon profile's metadata and get the **verified** (green) checkmark.

**Can people follow the blog *from* Mastodon?** Not directly — a static site
can't be an ActivityPub actor (that needs a live server speaking WebFinger +
ActivityPub). Two ways to let people subscribe anyway:

- **RSS** — anyone can follow `/rss.xml` in a feed reader today (many Mastodon
  users keep one). This is built in and on by default.
- **A bridge** — point the RSS feed at an RSS→ActivityPub bridge such as
  [rss-parrot](https://rss-parrot.net/) or [Bridgy Fed](https://fed.brid.gy/),
  which expose a real `@handle` that Mastodon users can **follow**, relaying new
  posts into their timeline. No server of your own required.

So: rich previews + author attribution + profile verification work out of the
box; true "follow from Mastodon" is one bridge away, using the RSS feed as input.

## Optional MTProto backend

The public web preview can't hand out **voice/audio notes** or **full-resolution
photos** (see [Limitations](#limitations)). An **opt-in** backend logs in as
*your user account* over MTProto (via [`grammers`](https://codeberg.org/Lonami/grammers))
to fetch them. It's **off by default** — the normal build and CI stay the
zero-credential web scraper — and is compiled in only with a Cargo feature:

```sh
cargo build --release --features mtproto
```

**1. Get API credentials.** Create an app at
[my.telegram.org](https://my.telegram.org) → *API development tools*, and note the
**`api_id`** (a number) and **`api_hash`** (a string).

**2. Log in once** to mint a reusable session — `api_id`/`api_hash` alone can't
authenticate, Telegram requires a real user login:

```sh
export TG_API_ID=1234567
export TG_API_HASH=0123456789abcdef0123456789abcdef
tg2zola login        # prompts: phone → code (sent in Telegram) → 2FA password
```

This writes **`tg2zola.session`** and prints a base64 **`TG_SESSION`** string.
From then on, runs are non-interactive.

**3. Generate** with the credentials in the environment — a normal run then also
pulls audio into each post's bundle (and, with `MTPROTO_IMAGES=1` / `MTPROTO_VIDEOS=1`,
original-quality photos / the full videos the preview shows only as a poster):

```sh
TG_API_ID=$TG_API_ID TG_API_HASH=$TG_API_HASH MTPROTO_IMAGES=1 \
  tg2zola --channel <name> --site site --init-site
```

| Env var | Purpose |
|---|---|
| `TG_API_ID` / `TG_API_HASH` | App credentials from my.telegram.org (required) |
| `TG_SESSION` | base64 session from `tg2zola login`; alternatively a `tg2zola.session` file in the working dir is used |
| `MTPROTO_IMAGES` | `1`/`true` to also fetch original-quality photos **and pasted images Telegram stored as files** (shown *not archived* on the web preview); audio is always fetched |
| `MTPROTO_VIDEOS` | downloads the full video for posts the web preview shows only as a poster **when no YouTube/Instagram embed replaces it** — **on by default**; set `false`/`0` to disable (these can be large) |
| `MTPROTO_FILES` | archives **every other attachment** (pdf, zip, rar, … — any file type) as a download; **on by default**, set `false`/`0` to disable |
| `TG_SESSION_FILE` | override the session-file path (default `tg2zola.session`) |

**For CI:** run `tg2zola login` **locally** (the interactive step can't run in
Actions), then store `TG_API_ID`, `TG_API_HASH` and the printed `TG_SESSION` as
**Actions secrets**. The bundled [`daily.yml`](.github/workflows/daily.yml) then
compiles the `mtproto` feature and runs it automatically whenever `TG_SESSION` is
set (set the `MTPROTO_IMAGES` / `MTPROTO_VIDEOS` repository *variables* to `1` for
original photos / big videos). The session has no fixed expiry — it lasts until
you log it out, Telegram revokes it, or it goes unused for ~6 months — so a daily
run keeps it alive indefinitely.

> ⚠️ **`TG_SESSION` is full access to your account** — treat it like a password
> (a *secret*, unlike the public `CHANNEL` variable). Consider a secondary
> account that's just a member of the channel. User-account automation is a
> Telegram grey area; use it on your own channel at gentle rates. Each run walks
> the full message history.

## Limitations

The public web preview is the trade-off for needing **zero authentication**
(several of these are lifted by the optional [MTProto backend](#optional-mtproto-backend)):

- **Reactions/likes are not exposed** by `t.me/s/`. We export **view counts**
  instead. The data model leaves room to add real reactions later via the
  authenticated MTProto API (the [`grammers`](https://codeberg.org/Lonami/grammers)
  crate) if you ever want them.
- **Large videos aren't downloadable** *from the public preview* — it serves only
  a poster image and duration for them (short/auto-play videos *are* downloadable).
  The optional [MTProto backend](#optional-mtproto-backend) with `MTPROTO_VIDEOS=1`
  fetches the real file for these.
- **Sticker packs aren't linkable** — the pack name is loaded by Telegram's
  JavaScript and isn't in the scraped HTML; stickers are saved as plain images.
- **Music files (audio documents) aren't downloadable** from `t.me/s/` — their
  URL isn't in the scraped HTML (only voice notes, with direct `.oga` URLs, are).
  The optional [MTProto backend](#optional-mtproto-backend) fetches audio
  (voice + music), archives **any other attachment** (pdf, zip, rar, …) as a
  download by default (`MTPROTO_FILES`), and with `MTPROTO_IMAGES` shows pasted
  images inline; without it, for an attachment we can't fetch we keep its
  **filename** (marked *not archived*) so you know it existed; a post that is
  *only* such a reference (or a lone undownloadable file) is skipped rather than
  published empty.
- **YouTube** plays via a `youtube.com` iframe (so plays count toward the
  viewer's history) on the published HTTPS site; over `file://` the iframe can't
  load (YouTube needs an origin). `YOUTUBE_FACADE=true` swaps the iframe for a
  no-JS click-to-load thumbnail, which at least shows the poster over `file://`.
- **Public channels only**, with the web preview enabled.

## License

[MIT](LICENSE) © Vitaly Zdanevich
