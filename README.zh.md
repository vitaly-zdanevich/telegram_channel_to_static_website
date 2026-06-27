# tg2zola

🌐 [English](README.md) · [Беларуская](README.be.md) · [Українська](README.uk.md) · [Русский](README.ru.md) · [Deutsch](README.de.md) · [Français](README.fr.md) · **中文** · [日本語](README.ja.md) · [Polski](README.pl.md) · [Español](README.es.md) · [한국어](README.ko.md) · [ქართული](README.ka.md)

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

将**公开的 Telegram 频道**备份为**自包含的 [Zola](https://www.getzola.org/) 静态网站**。

> **按需重新生成：** 点击上方的 **daily build** 徽章 → **Run workflow**，即可在不等待计划任务的情况下重新抓取、构建并部署（详见 **「自动化」** 一节）。

本工具读取公开的网页预览（`https://t.me/s/<channel>`），将所有媒体下载到本地，并在每次运行时重新生成完整的 Zola 博客。**无需任何 Telegram 机器人、令牌或 API** —— 它只读取公开的网页。生成结果**不依赖 Telegram**：媒体在本地，没有嵌入内容，指向频道*自身*帖子的链接会被改写为内部相对链接 —— 因此即使该频道日后被删除，网站仍可继续使用。你写入的指向其他站点（包括其他 Telegram 频道）的链接会原样保留为普通链接。这是一份备份，而非镜像。

用 Rust 编写：单个静态二进制文件，便于在本地或 CI 中运行。

## 功能

- **完整历史** —— 通过预览的 `?before=` 游标向后遍历频道，直到第一条消息，并在每次运行时**重新生成每个页面**。
- **自包含媒体** —— 将照片、视频、音频（`.ogg/.oga/.mp3`）、文档和贴纸下载到每个帖子的 bundle 中（它同时充当缓存）。照片按其稳定的 Telegram 文件 id 寻址，因此编辑帖子文本绝不会重新下载其媒体，而**替换**图片（帖子随即标记为*已编辑*）则获取新文件并清除旧文件。
- **默认纯黑主题** —— 内置模板在深色模式下通过 `prefers-color-scheme` 设为 `#000`（对 OLED 友好），无外部主题依赖。可在其上叠加外部主题并保证回退（见 **「主题」** 一节）。
- **智能视频处理**（按优先级）：
  1. 附带视频 **+ YouTube 链接** → 嵌入 YouTube，丢弃视频；
  2. 可直接下载的视频 → 本地 `<video>`；
  3. 否则 → 保存**封面帧** + 时长（公开页面不提供该文件；见 **「限制」**）。
- **格式化** —— 粗体、斜体、删除线、代码/预格式、链接和剧透会转换为 Markdown（正确处理 Telegram 的 UTF-16 实体偏移）。
- **话题标签 → 标签** —— `#话题标签` 会成为 Zola `tags` 分类法的条目，于是你免费获得标签页面，同时它们仍以文本形式显示在帖子中。
- **分组帖子** —— 相册会自动合并为一个帖子；同一时刻发布的一连串消息（例如一次转发多条）会被合并。
- **自导航** —— 指向*同一*频道中另一条消息的链接会变成指向博客中该帖子的相对链接；指向其他频道的链接保持为外部链接。
- **互动数据** —— 导出每个帖子的**浏览量**。（公开页面不提供 reactions/点赞 —— 见 **「限制」**。）
- **RSS 订阅** —— 标准的 `/rss.xml`，包含**所有帖子**及完整内容（一份完整的订阅源，而不仅是最新条目），通过 `<link rel="alternate">` 声明，使阅读器能从网站地址自动发现。默认开启；可用 `RSS=false` / `--no-rss` 关闭。
- **丰富的链接预览 + Mastodon** —— 每个页面都会输出 Open Graph 与 Twitter Card 标签（标题、描述、帖子的第一张图片），使分享的链接渲染为卡片。设置 `FEDIVERSE_CREATOR` 可在 Mastodon 预览中加上作者署名，并在你的个人资料中验证本站（见 **「Fediverse」** 一节）。
- **默认无 JavaScript，支持离线** —— 深色模式和剧透仅用 CSS 实现，默认的 Google 搜索框是普通的 `<form>`（无 JS）。只有非 Google 搜索引擎才会加入一个极小的内联 Enter 处理脚本。`tg2zola offline <public 目录>` 会将构建好的网站改写为相对链接，**并**移除 Zola 的分页重定向脚本，于是离线副本可直接从 `file://` 打开，完全没有 JavaScript，也无需 Web 服务器。
- **本地化界面** —— 网站外壳（较新/较旧/标签/关于、搜索框、日期）可通过 `LANGUAGE` / `--language` 以 12 种语言之一显示（en/be/uk/ru/de/fr/zh/ja/pl/es/ko/ka），月份和星期名称也会本地化。帖子内容保持频道自身的语言。

## 安装

从 [Releases](../../releases) 页面获取适用于你架构的静态二进制文件（Linux `amd64` / `arm64`，musl），或从源码构建：

```sh
cargo build --release
# 二进制文件位于 target/release/tg2zola
```

你还需要 [`zola`](https://www.getzola.org/documentation/getting-started/installation/) 二进制文件，以将生成的内容转换为 HTML。

## 用法

```sh
# 生成 Zola 网站（首次运行时搭建 config + 模板）：
tg2zola --channel durov --site site --init-site

# 构建静态 HTML：
zola --root site build       # 输出在 site/public/

# （可选）无需 Web 服务器，直接从磁盘查看：
tg2zola offline site/public  # 然后通过 file:// 打开 site/public/index.html
```

快速本地测试（一页，约 20 条消息）：

```sh
tg2zola --channel <name> --site /tmp/site --init-site --max-pages 1
```

所有选项都在 [`tg2zola.toml`](tg2zola.toml) 中（CLI 标志会覆盖它）：

```sh
tg2zola --config tg2zola.toml
```

运行 `tg2zola --help` 查看完整标志列表。

## 工作原理

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

每个帖子都会成为一个 Zola 页面 bundle —— `content/posts/<date>-<id>/index.md`，带有 TOML front matter，媒体文件就在旁边。`config.toml` 和内置模板在每次运行时确定性地重新生成，`write_site` 会协调整个目录树：重写全部 Markdown，保留已缓存的媒体，并清除已删除的帖子和过期文件。

## 自动化（GitHub Actions）

包含两个工作流：

- **[`daily.yml`](.github/workflows/daily.yml)** —— 每天运行一次（也可按需）：从 `blog` 分支（媒体缓存）恢复上一次的网站 → 抓取 + 重新生成 → `zola build` → **部署到 GitHub Pages**（已发布的结果）→ 将刷新后的网站提交回 **`blog` 分支**。
- **[`release.yml`](.github/workflows/release.yml)** —— 每当推送 `v*` 标签时，交叉编译静态 `amd64` + `arm64`（musl）二进制文件并上传到 GitHub Release。

启用发布：在仓库中 **Settings → Pages → Build and deployment → Source: GitHub Actions**。无需任何密钥 —— 全部基于公开抓取。已发布的网站始终是 **GitHub Pages**；`blog` 分支是一份持久副本，绝不影响访问者所见。

### 立即重新生成（无需等待每日运行）

`daily.yml` 启用了 `workflow_dispatch`，因此你可以按需触发一次全新的抓取 + 构建 + 重新部署 —— 它执行的步骤与计划运行完全相同：

- **在浏览器中：** 打开 **[Actions → 「daily」→ Run workflow](../../actions/workflows/daily.yml)**，点击绿色的 **Run workflow** 按钮。（把上方的状态徽章加到你的 README 中即可一键访问。）
- **在终端中：** `gh workflow run daily.yml`（GitHub CLI），然后 `gh run watch` 跟踪进度。

**哪个频道？** 频道不会被提交，因此每次部署各自设定。设置仓库**变量** `CHANNEL`（Settings → Secrets and variables → Actions → Variables）为公开频道用户名 —— 它是*变量*而非密钥，因为频道是公开的。（或在你的 fork 中取消注释 [`tg2zola.toml`](tg2zola.toml) 里的 `channel = "…"`。）`THEME_REPO` 同样可作为变量使用。

### `blog` 分支（归档 + 缓存）

每次运行都会把生成的网站（Markdown + 媒体 + 内置模板 —— 除已构建的 `public/` 和任何外部主题之外的一切）提交到 `blog` 分支，使 `main` 仅保留代码。它身兼两职：

- **缓存** —— 下次运行从中恢复媒体，而非重新下载。
- **持久归档** —— 一个完整、可构建的 Zola 网站，你可以克隆并镜像到任何地方，使备份不被绑定到单一平台：

```sh
git clone -b blog https://github.com/<you>/<repo> my-archive
zola --root my-archive/site serve          # 离线浏览

git -C my-archive remote add gitlab git@gitlab.com:<you>/<repo>.git
git -C my-archive push gitlab blog          # 镜像到别处
```

媒体以普通 git blob 形式提交；对于非常大的频道，可考虑 Git LFS 或偶尔压缩历史。

### 主题

默认是内置的**纯黑**主题 —— 零外部依赖，因此网站总能构建成功。要使用外部 [Zola 主题](https://www.getzola.org/themes/)，设置仓库**变量** `THEME_REPO`（Settings → Secrets and variables → Actions → Variables）为其 git URL（https，或带部署密钥的 ssh）。工作流会克隆它并据此构建 —— **若主题缺失或其构建失败，会自动回退到内置模板**，因此主题问题绝不会让博客下线。请注意外部主题期望特定的内容布局，因此并非每个主题都能即插即用。

发布版本：

```sh
git tag v0.1.0 && git push origin v0.1.0
```

## 配置

一切都可通过仓库**变量**（Settings → Secrets and variables → Actions → **Variables**）为 GitHub Actions 流程配置，或在本地运行时通过等价的 CLI 标志 / [`tg2zola.toml`](tg2zola.toml) 键配置。它们是*变量*而非密钥 —— 全部公开。

| 仓库变量 | CLI 标志 | 默认 | 作用 |
|---|---|---|---|
| `CHANNEL` | `--channel` | **必填** | 要同步的公开频道 |
| `TITLE` | `--title` | 频道用户名 | 博客标题（页眉 + `<title>`） |
| `ABOUT` | `--about` | 描述 + 统计 + 仓库链接 | 「关于」页面正文的自定义 HTML |
| `PAGES` | `--pages` | — | 额外页面：Markdown，每个 `# Title` 标题开启一个新页面 + 导航项 |
| `POSTS_PER_PAGE` | `--posts-per-page` | `20` | 首页信息流每页的完整帖子数 |
| `TAGS_FOOTER` | `--tags-footer` | 关 | `true` 显示每帖的标签页脚（标签在正文中本就可点击） |
| `NEXT_PREV` | `--no-next-prev` | 开 | `false` 隐藏上一篇/下一篇导航 |
| `TELEGRAM_LINK` | `--no-telegram-link` | 开 | `false` 隐藏每帖的「在 Telegram 中查看」链接 |
| `RSS` | `--no-rss` | 开 | `false` 关闭 `/rss.xml` 的 RSS 订阅（含自动发现） |
| `FEDIVERSE_CREATOR` | `--fediverse-creator` | — | Mastodon `@user@instance` → `fediverse:creator` 署名 + `rel="me"` 资料链接 |
| `SEARCH_ENGINE` | `--search-engine` | `google` | 页眉搜索框：`google`（无 JS 表单）/ `duckduckgo` / `yandex` / `bing` / `none` |
| `SEARCH_URL` | `--search-url` | — | 自定义搜索 URL 前缀；按 Enter 时追加查询（覆盖搜索引擎） |
| `TITLE_MAX_LEN` | `--title-max-len` | `200` | 帖子标题最大长度（字符）；被截断的标题会在正文保留其完整首句 |
| `FOOTER` | `--footer` | — | 页脚内容 —— 纯文本、Markdown 或 HTML |
| `PAGES_HOST` | `--pages-host` | 自动 | 「关于」页面体积上限的主机：`github` / `gitlab` / `none`（从 URL 自动检测） |
| `DATE_FORMAT` | `--date-format` | `%Y %B %d` | 所显示日期的 strftime 格式（例如 `2025 October 28`；`%Y` 仅年份） |
| `LANGUAGE` | `--language` | `en` | 网站界面语言（较新/较旧/标签/关于/…）：`en`/`be`/`uk`/`ru`/`de`/`fr`/`zh`/`ja`/`pl`/`es`/`ko`/`ka`（格鲁吉亚语）。帖子内容保持频道自身语言；日期会本地化 |
| `LINK_UNDERLINE` | `--link-underline` | 关 | `true` 为链接加下划线（默认无下划线） |
| `YOUTUBE_FACADE` | `--youtube-facade` | 关 | `true` 使用无 JS 的点击加载 YouTube 缩略图（默认直接 iframe） |
| `GENIUS` | `--no-genius` | 开 | `false` 跳过解析 genius.com 链接（会抓取页面以获取其 YouTube 视频 + 歌词小部件） |
| `TAGS_TO_PAGES` | `--tags-to-pages` | — | 以逗号分隔、在顶部导航显示为 `#tag` 链接的标签（例如 `music, batumi, cooking`） |
| `BACKGROUND_DARK_COLOR` | `--background-dark-color` | `#000000` | 深色模式背景（任意 CSS 颜色） |
| `BACKGROUND_LIGHT_COLOR` | `--background-light-color` | `#ffffff` | 浅色模式背景 |
| `CSS` | `--css` | — | 追加到内置样式表的额外 CSS |
| `THEME_REPO` | `--theme`（名称） | 内置黑色主题 | 外部 Zola 主题的 git URL（https/ssh）；失败时自动回退 |
| `REPO_URL` | `--repo-url` | tg2zola 仓库 | 「关于」页面上的「源代码仓库」链接（CI 会自动设为你的仓库） |

**「关于」** 页面显示频道**头像**（原始尺寸）、其描述和统计、**磁盘占用** —— 含按类型的细分，并在 GitHub/GitLab Pages 上显示占主机已发布站点约 1 GB 上限的比例（链接到主机文档）—— 以及仓库链接。页眉将头像显示为缩略图和 favicon；话题标签在帖子正文中可点击，并生成 `/tags/<tag>/` 页面。

单个 `PAGES` 变量可定义多个页面 —— 每个 `# Title` 标题开启一个新页面：

```
# 页面标题
Markdown 编写的页面内容（可包含原始 HTML）。

# 另一个页面
更多内容。
```

→ `/page-title/` 和 `/another-page/`，每个都在导航中链接。无需额外依赖 —— Zola 本就渲染 Markdown。

## Fediverse / Mastodon

每个页面都带有 Open Graph + Twitter Card 标签，因此分享的帖子链接会在 Mastodon、Slack、Discord、X 等中渲染为卡片（标题、描述、帖子的第一张图片）。将 `FEDIVERSE_CREATOR` 设为你的 `@user@instance` 句柄，还可以：

- 添加 **`fediverse:creator`** 署名，使 Mastodon 在链接预览中显示「*by @you@instance*」并链接到你的个人资料；
- 输出指向你个人资料的 **`rel="me"`** 链接，使你可以把本站加入 Mastodon 资料的元数据，获得**已验证**（绿色）对勾。

**人们能*从* Mastodon 关注该博客吗？** 不能直接关注 —— 静态网站无法成为 ActivityPub actor（那需要一个会说 WebFinger + ActivityPub 的实时服务器）。仍有两种方式让人订阅：

- **RSS** —— 任何人今天就能在阅读器中订阅 `/rss.xml`（许多 Mastodon 用户都用阅读器）。它内置且默认开启。
- **桥接** —— 将 RSS 订阅源指向 RSS→ActivityPub 桥接，例如 [rss-parrot](https://rss-parrot.net/) 或 [Bridgy Fed](https://fed.brid.gy/)，它们提供一个真正的 `@handle`，Mastodon 用户可**关注**它，将新帖子转发到他们的时间线。无需你自己的服务器。

因此：丰富预览 + 作者署名 + 资料验证开箱即用；真正的「从 Mastodon 关注」只差一个桥接，以 RSS 订阅源作为输入。

## 限制

公开网页预览是换取**零认证**的代价：

- **不暴露 reactions/点赞** —— `t.me/s/` 不提供。我们改为导出**浏览量**。数据模型留有余地，日后可经由已认证的 MTProto API（[`grammers`](https://codeberg.org/Lonami/grammers) crate）添加真正的 reactions，如果你想要的话。
- **大视频无法下载** —— 预览只为其提供封面图和时长（短/自动播放视频*可*下载）。归档真正的文件同样需要 MTProto API。
- **贴纸包无法链接** —— 贴纸包名称由 Telegram 的 JavaScript 加载，不在抓取到的 HTML 中；贴纸保存为普通图片。
- **音乐文件（音频文档）无法下载** —— 抓取到的 HTML 中没有其 URL（只有语音消息带有直接的 `.oga` URL）。与大视频和贴纸一样，它们需要 MTProto API。对于我们无法获取的附件，我们保留其**文件名**（标记为*未存档*），让你知道它曾存在；仅由这样一个引用（或单个不可下载文件）构成的帖子会被跳过，而不是发布为空帖。
- **YouTube** 在已发布的 HTTPS 站点上通过 `youtube.com` iframe 播放（使播放计入观看者的历史记录）；通过 `file://` 时 iframe 无法加载（YouTube 需要 origin）。`YOUTUBE_FACADE=true` 会把 iframe 换成无 JS 的点击加载缩略图，至少能在 `file://` 下显示封面。
- **仅限公开频道**，且需启用网页预览。

## 许可证

[MIT](LICENSE) © Vitaly Zdanevich
