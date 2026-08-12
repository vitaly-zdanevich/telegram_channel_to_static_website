+++
title = "About"
path = "about"
template = "page.html"
+++

*A static mirror of the public Telegram channel [@vitaly_zdanevich_chan](https://t.me/vitaly_zdanevich_chan).*

<https://about.me/zdanevich>  

Software Engineer, from Belarus, from 2022 living in Sakartvello (Georgia country UTC+4). Working with Golang, AWS. Program in Vim. Love Gentoo Linux. Respect free and open software. Contributor to Wikipedia and other projects.

**21** subscribers · **1.53K** images · **246** videos · **35** files · **1.08K** links · **32** audios

🎧 Podcast: [audio](https://vitaly-zdanevich.github.io/telegram_channel_to_static_website/podcast.xml) · [video](https://vitaly-zdanevich.github.io/telegram_channel_to_static_website/video-podcast.xml)

{{ aboutme_photo(src="aboutme.png") }}

[Facebook](http://www.facebook.com/1159185426) · [GitHub](https://github.com/vitaly-zdanevich) · [LinkedIn](http://www.linkedin.com/pub/vitaly-zdanevich/48/480/7b5) · [Pinterest](https://pinterest.com/vitalyzdanevich/) · [X](https://twitter.com/vnon) · [VK](http://vk.com/vitaly.zdanevich) · [Stack Exchange](https://stackexchange.com/users/2114516/vitaly-zdanevich) · [Stack Overflow](https://stackoverflow.com/users/1879101/vitaly-zdanevich) · [Product Hunt](https://www.producthunt.com/@vitaly_zdanevich) · [Reddit](https://www.reddit.com/user/vitaly-zdanevich) · [Quora](https://www.quora.com/profile/Vitaly-Zdanevich) · [Upwork](https://www.upwork.com/freelancers/~017f0e7610b2de42a1) · [YouTube](https://www.youtube.com/@VitalyZdanevich) · [Wikipedia](https://en.wikipedia.org/wiki/User:Vitaly_Zdanevich) · [IMDb](https://imdb.com/user/ur55148955) · [Instagram](https://www.instagram.com/vitalyzdanevich/) · [GitLab](https://gitlab.com/vitaly-zdanevich) · [LastFM](https://www.last.fm/user/vnON) · [Discord](https://discordapp.com/users/vitaly_zdanevich)

<a class="contact-btn" href="https://about.me/zdanevich">✉ Message me on about.me</a>



In the git repository: **258 MB** — **25%** of the [1 GB GitHub Pages limit](https://docs.github.com/en/pages/getting-started-with-github-pages/about-github-pages#usage-limits).

In [GitHub Releases](https://github.com/vitaly-zdanevich/telegram_channel_to_static_website/releases): **12.4 GB** of video and `.tar.xz` files — outside git, so it doesn't count toward the limit.

By kind:

- **Images** 197 MB
- **Audio** 51.3 MB
- **Other** 8.0 MB
- **Text** 2.2 MB

Largest files:

- [16.2 MB — image_2026-07-01_07-36-10.png](@/posts/2026-07-01-1879/index.md "#belarus #cementery #sun #sky #blue #year2015 Source.jpg)")
- [7.7 MB — Exigo1920x1200.exe](@/posts/2026-05-31-1802/index.md "With #llm I added 1920x1200 to #armiesofexigo #game And increased the #camera range.")
- [5.5 MB — Каждые_7_лет_клетки_полностью_регенериру_3.mp3](@/posts/2024-08-25-129/index.md)
- [5.5 MB — Каждые_7_лет_клетки_полностью_регенериру_5.mp3](@/posts/2024-08-25-129/index.md)
- [5.5 MB — Каждые_7_лет_клетки_полностью_регенериру_6.mp3](@/posts/2024-08-25-129/index.md)
- [5.4 MB — Каждые_7_лет_клетки_полностью_регенериру_1.mp3](@/posts/2024-08-25-129/index.md)
- [5.1 MB — Каждые_7_лет_клетки_полностью_регенериру_2.mp3](@/posts/2024-08-25-129/index.md)
- [5.1 MB — complex_numbers_-_39_загробный_мир.mp3](@/posts/2024-04-27-36/index.md "Part of our new opera")
- [4.9 MB — Каждые_7_лет_клетки_полностью_регенериру_7.mp3](@/posts/2024-08-25-129/index.md)
- [4.8 MB — Каждые_7_лет_клетки_полностью_регенериру_4.mp3](@/posts/2024-08-25-129/index.md)

Generated in **1h 16m 27s**.

Last updated **2026-08-12 07:03 UTC** · [build log](https://github.com/vitaly-zdanevich/telegram_channel_to_static_website/actions/runs/31567538350)



Source repository: [https://github.com/vitaly-zdanevich/telegram_channel_to_static_website](https://github.com/vitaly-zdanevich/telegram_channel_to_static_website)

**No Telegram bot, token, or API is needed for the public web preview** — the site is built from it, with all media except audio and big videos downloaded locally, so it keeps working even if the channel is removed.

The optional [MTProto](https://github.com/Lonami/grammers) backend was used to fetch audio and long videos.



<ul class="commits"><li title="Use network-first handling for page navigations so deployed HTML replaces stale service-worker entries, while cached pages remain available offline and static assets stay cache-first.&#10;&#10;Add deterministic service-worker strategy tests and run them in CI. Release version 0.73.6.&#10;&#10;Co-authored-by: OpenAI ChatGPT &lt;noreply@openai.com&gt;"><a href="https://github.com/vitaly-zdanevich/telegram_channel_to_static_website/commit/534d095797ed4283ea00d111209520e6d121c613"><code>534d0957</code></a> Refresh cached pages on navigation <span class="cdate">2026-08-11</span></li><li title="Retry page fetches up to ten times with capped exponential backoff so one temporary Telegram error does not discard a full history scrape. Keep permanent client errors fail-fast and cover status, transport, attempt-limit, and backoff behavior with mock tests.&#10;&#10;Co-authored-by: OpenAI ChatGPT &lt;noreply@openai.com&gt;"><a href="https://github.com/vitaly-zdanevich/telegram_channel_to_static_website/commit/28a7e8f6866de40af3730a7605659b4d3b8439ae"><code>28a7e8f6</code></a> Retry transient Telegram page failures <span class="cdate">2026-08-11</span></li><li title="Keep Release-backed MTProto downloads out of ordinary Git history so scheduled backups do not exceed GitHub blob limits or time out. Remove cache files left by older blog snapshots and cover the workflow contract with a regression test.&#10;&#10;Co-authored-by: OpenAI ChatGPT &lt;noreply@openai.com&gt;"><a href="https://github.com/vitaly-zdanevich/telegram_channel_to_static_website/commit/616d62ae54e65c39b755dbb0fc3b56096f3c7a0d"><code>616d62ae</code></a> Exclude MTProto cache from blog backups <span class="cdate">2026-08-09</span></li><li title="Wrap Pinterest Pin embeds in a local scope and override the two injected white footer containers only in dark mode. Preserve Pin and avatar colours, retain the offline link fallback, and cover the current generated markup with a mock-DOM regression test. Release version 0.73.3.&#10;&#10;Co-authored-by: OpenAI ChatGPT &lt;noreply@openai.com&gt;"><a href="https://github.com/vitaly-zdanevich/telegram_channel_to_static_website/commit/828a3a0341534854308158c2318c41ef158bec40"><code>828a3a03</code></a> Match Pinterest footer to dark theme <span class="cdate">2026-08-09</span></li><li title="Recognize GNU split two-letter suffixes so multipart tar.xz assets are staged in GitHub Releases instead of inflating the Pages artifact. Add remote and MTProto-local regression coverage, including ambiguous suffix rejection, and release version 0.73.2.&#10;&#10;Co-authored-by: OpenAI ChatGPT &lt;noreply@openai.com&gt;"><a href="https://github.com/vitaly-zdanevich/telegram_channel_to_static_website/commit/85a761c9b17eb7bf5531d8569af3a4c69b69b644"><code>85a761c9</code></a> Offload split tar.xz parts to Releases <span class="cdate">2026-08-06</span></li><li title="Apply the current rustfmt output repository-wide so cargo fmt --check is clean.&#10;&#10;Co-authored-by: OpenAI ChatGPT &lt;noreply@openai.com&gt;"><a href="https://github.com/vitaly-zdanevich/telegram_channel_to_static_website/commit/ed6c47a2051a45e28b8e6e3273f1abf9cbdec378"><code>ed6c47a2</code></a> Format Rust sources <span class="cdate">2026-08-06</span></li><li><a href="https://github.com/vitaly-zdanevich/telegram_channel_to_static_website/commit/53970782318f6c1ad072bfc49ad14322931365be"><code>53970782</code></a> Enrich archive metadata and navigation <span class="cdate">2026-07-28</span></li><li title="The MTProto-provenance line qualifies the 'no bot/token/API needed' statement&#10;(audio and big videos do come via MTProto), so it now follows it rather than&#10;preceding it.&#10;&#10;Co-Authored-By: Claude Opus 4.8 &lt;noreply@anthropic.com&gt;"><a href="https://github.com/vitaly-zdanevich/telegram_channel_to_static_website/commit/892d36795610d1d4dad5ba1ad2584ccf941151ef"><code>892d3679</code></a> about: put the MTProto note after the 'no API needed' line (v0.73.1) <span class="cdate">2026-07-11</span></li><li title="New --enex &lt;file&gt; flag: one &lt;note&gt; per post (title, ENML body, tags, created&#10;date), with every media file attached as a base64 &lt;resource&gt; linked by its MD5&#10;en-media hash — importable straight into Evernote. Runs before dedup (media in&#10;bundles). Small pure-Rust md-5 dep for the hashes; reuses the single-file base64&#10;encoder. MD5 + XML-escape tests.&#10;&#10;Co-Authored-By: Claude Opus 4.8 &lt;noreply@anthropic.com&gt;"><a href="https://github.com/vitaly-zdanevich/telegram_channel_to_static_website/commit/204b14b6f63f66873f7331e53e884760acf2eabe"><code>204b14b6</code></a> enex: --enex exports the archive as an Evernote ENEX file (v0.73.0) <span class="cdate">2026-07-11</span></li><li title="New --sqlite &lt;db&gt; flag on generate: write posts, tags, links, reactions and&#10;every media file (as a raw BLOB) into one SQLite database — for preservation and&#10;SELECT-based analytics, and unlike the single-file HTML it's fine for&#10;media-heavy channels (blobs aren't base64-inflated). Runs before dedup, while&#10;media is still in each post's bundle.&#10;&#10;Uses the  crate (bundled SQLite) — deliberately the same one&#10;grammers-session uses, so the two share one native sqlite3 (no links conflict);&#10;verified the mtproto build still links. Schema + blob round-trip test included.&#10;&#10;Co-Authored-By: Claude Opus 4.8 &lt;noreply@anthropic.com&gt;"><a href="https://github.com/vitaly-zdanevich/telegram_channel_to_static_website/commit/37fb59dd7963960515cb7460a2969fe6b06efb8e"><code>37fb59dd</code></a> sqlite: --sqlite exports the whole archive to one SQLite file with blobs (v0.72.0) <span class="cdate">2026-07-09</span></li></ul>
