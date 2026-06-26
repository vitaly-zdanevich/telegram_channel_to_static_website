# tg2zola

[![daily build](https://github.com/vitaly-zdanevich/telegram_channel_to_static_website/actions/workflows/daily.yml/badge.svg)](https://github.com/vitaly-zdanevich/telegram_channel_to_static_website/actions/workflows/daily.yml)

[![Quality Gate Status](https://sonarcloud.io/api/project_badges/measure?project=vitaly-zdanevich_telegram_channel_to_static_website&metric=alert_status)](https://sonarcloud.io/summary/new_code?id=vitaly-zdanevich_telegram_channel_to_static_website)
[![Coverage](https://sonarcloud.io/api/project_badges/measure?project=vitaly-zdanevich_telegram_channel_to_static_website&metric=coverage)](https://sonarcloud.io/summary/new_code?id=vitaly-zdanevich_telegram_channel_to_static_website)
[![Bugs](https://sonarcloud.io/api/project_badges/measure?project=vitaly-zdanevich_telegram_channel_to_static_website&metric=bugs)](https://sonarcloud.io/summary/new_code?id=vitaly-zdanevich_telegram_channel_to_static_website)
[![Vulnerabilities](https://sonarcloud.io/api/project_badges/measure?project=vitaly-zdanevich_telegram_channel_to_static_website&metric=vulnerabilities)](https://sonarcloud.io/summary/new_code?id=vitaly-zdanevich_telegram_channel_to_static_website)
[![Code Smells](https://sonarcloud.io/api/project_badges/measure?project=vitaly-zdanevich_telegram_channel_to_static_website&metric=code_smells)](https://sonarcloud.io/summary/new_code?id=vitaly-zdanevich_telegram_channel_to_static_website)
[![Duplicated Lines](https://sonarcloud.io/api/project_badges/measure?project=vitaly-zdanevich_telegram_channel_to_static_website&metric=duplicated_lines_density)](https://sonarcloud.io/summary/new_code?id=vitaly-zdanevich_telegram_channel_to_static_website)
[![Maintainability](https://sonarcloud.io/api/project_badges/measure?project=vitaly-zdanevich_telegram_channel_to_static_website&metric=sqale_rating)](https://sonarcloud.io/summary/new_code?id=vitaly-zdanevich_telegram_channel_to_static_website)
[![Reliability](https://sonarcloud.io/api/project_badges/measure?project=vitaly-zdanevich_telegram_channel_to_static_website&metric=reliability_rating)](https://sonarcloud.io/summary/new_code?id=vitaly-zdanevich_telegram_channel_to_static_website)
[![Security](https://sonarcloud.io/api/project_badges/measure?project=vitaly-zdanevich_telegram_channel_to_static_website&metric=security_rating)](https://sonarcloud.io/summary/new_code?id=vitaly-zdanevich_telegram_channel_to_static_website)
[![Lines of Code](https://sonarcloud.io/api/project_badges/measure?project=vitaly-zdanevich_telegram_channel_to_static_website&metric=ncloc)](https://sonarcloud.io/summary/new_code?id=vitaly-zdanevich_telegram_channel_to_static_website)
[![Technical Debt](https://sonarcloud.io/api/project_badges/measure?project=vitaly-zdanevich_telegram_channel_to_static_website&metric=sqale_index)](https://sonarcloud.io/summary/new_code?id=vitaly-zdanevich_telegram_channel_to_static_website)

Back up a **public Telegram channel** into a **self-contained [Zola](https://www.getzola.org/) static website**.

> **Regenerate on demand:** click the **daily build** badge above → **Run
> workflow** to scrape + rebuild + redeploy without waiting for the schedule
> (details in [Automation](#automation-github-actions)).

It scrapes the public web preview (`https://t.me/s/<channel>`), downloads all
media locally, and regenerates a complete Zola blog on every run. **No Telegram
bot, token, or API is needed** — it reads only the public web page. The output has
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
  1. attached video **+ a YouTube link** → embed YouTube, drop the video;
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
- **No JavaScript by default, offline-ready** — dark mode and spoilers are
  CSS-only, and the default Google search box is a plain `<form>` (no JS). Only a
  non-Google search engine adds one tiny inline Enter handler. `tg2zola offline
  <public-dir>` rewrites the built site to relative links **and** strips Zola's
  pagination redirect script, so the offline copy opens straight from `file://`
  with zero JavaScript and no web server.
- **Localized UI** — the site chrome (Newer/Older/Tags/About, the search box,
  dates) renders in any of 11 languages via `LANGUAGE` / `--language`
  (en/be/uk/ru/de/fr/zh/ja/pl/es/ko), with month and weekday names localized
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
| `SEARCH_ENGINE` | `--search-engine` | `google` | Header search box: `google` (JS-free form) / `duckduckgo` / `yandex` / `bing` / `none` |
| `SEARCH_URL` | `--search-url` | — | Custom search URL prefix; the query is appended on Enter (overrides the engine) |
| `TITLE_MAX_LEN` | `--title-max-len` | `200` | Max post-title length (chars); a truncated title keeps its full first sentence in the body |
| `FOOTER` | `--footer` | — | Footer content — plain text, Markdown or HTML |
| `PAGES_HOST` | `--pages-host` | auto | Host for the About-page size limit: `github` / `gitlab` / `none` (auto-detected from the URL) |
| `DATE_FORMAT` | `--date-format` | `%Y %B %d` | strftime format for displayed dates (e.g. `2025 October 28`; `%Y` for year only) |
| `LANGUAGE` | `--language` | `en` | UI language for the site chrome (Newer/Older/Tags/About/…): `en`/`be`/`uk`/`ru`/`de`/`fr`/`zh`/`ja`/`pl`/`es`/`ko`. Post content keeps the channel's own language; dates are localized to match |
| `LINK_UNDERLINE` | `--link-underline` | off | `true` underlines links (default: no underline) |
| `YOUTUBE_FACADE` | `--youtube-facade` | off | `true` for a no-JS click-to-load YouTube thumbnail (default: direct iframe) |
| `GENIUS` | `--no-genius` | on | `false` skips resolving genius.com links (fetches the page for its YouTube video + lyrics widget) |
| `TAGS_TO_PAGES` | `--tags-to-pages` | — | Comma-separated tags shown as `#tag` links in the top nav (e.g. `music, batumi, cooking`) |
| `BACKGROUND_DARK_COLOR` | `--background-dark-color` | `#000000` | Dark-mode background (any CSS color) |
| `BACKGROUND_LIGHT_COLOR` | `--background-light-color` | `#ffffff` | Light-mode background |
| `CSS` | `--css` | — | Extra CSS appended to the built-in stylesheet |
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

## Limitations

The public web preview is the trade-off for needing **zero authentication**:

- **Reactions/likes are not exposed** by `t.me/s/`. We export **view counts**
  instead. The data model leaves room to add real reactions later via the
  authenticated MTProto API (the [`grammers`](https://codeberg.org/Lonami/grammers)
  crate) if you ever want them.
- **Large videos aren't downloadable** — the preview only serves a poster image
  and duration for them (short/auto-play videos *are* downloadable). Archiving
  the actual file would also require the MTProto API.
- **Sticker packs aren't linkable** — the pack name is loaded by Telegram's
  JavaScript and isn't in the scraped HTML; stickers are saved as plain images.
- **Music files (audio documents) aren't downloadable** — their URL isn't in the
  scraped HTML (only voice notes, with direct `.oga` URLs, are). Like large
  videos and stickers, they'd need the MTProto API. For an attachment we can't
  fetch we keep its **filename** (marked *not archived*) so you know it existed;
  a post that is *only* such a reference (or a lone undownloadable file) is
  skipped rather than published empty.
- **YouTube** plays via a `youtube.com` iframe (so plays count toward the
  viewer's history) on the published HTTPS site; over `file://` the iframe can't
  load (YouTube needs an origin). `YOUTUBE_FACADE=true` swaps the iframe for a
  no-JS click-to-load thumbnail, which at least shows the poster over `file://`.
- **Public channels only**, with the web preview enabled.

## License

[MIT](LICENSE) © Vitaly Zdanevich
