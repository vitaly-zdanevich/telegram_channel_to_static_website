# tg2zola

🌐 [English](README.md) · [Беларуская](README.be.md) · [Українська](README.uk.md) · [Русский](README.ru.md) · [Deutsch](README.de.md) · [Français](README.fr.md) · [中文](README.zh.md) · **日本語** · [Polski](README.pl.md) · [Español](README.es.md) · [한국어](README.ko.md) · [ქართული](README.ka.md) · [हिन्दी](README.hi.md)

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

**公開 Telegram チャンネル**を**自己完結型の [Zola](https://www.getzola.org/) 静的サイト**にバックアップします。

> **オンデマンドで再生成：** 上の **daily build** バッジ → **Run workflow** をクリックすると、スケジュールを待たずにスクレイプ・ビルド・再デプロイできます（詳細は **「自動化」** の節）。

このツールは公開ウェブプレビュー（`https://t.me/s/<channel>`）を読み取り、すべてのメディアをローカルにダウンロードして、実行のたびに完全な Zola ブログを再生成します。**Telegram のボット・トークン・API は一切不要** —— 公開ウェブページだけを読みます。出力は **Telegram に依存しません**：メディアはローカルにあり、埋め込みはなく、チャンネル*自身*の投稿へのリンクは内部の相対リンクに書き換えられます —— そのためチャンネルが後で削除されてもサイトは動き続けます。あなたが他サイト（他の Telegram チャンネルを含む）へ書いたリンクは通常のリンクとして保持されます。これはミラーではなくバックアップです。

Rust 製：単一の静的バイナリで、ローカルでも CI でも簡単に実行できます。

## 機能

- **完全な履歴** —— プレビューの `?before=` カーソルでチャンネルを最初のメッセージまでさかのぼり、実行のたびに**すべてのページを再生成**します。
- **自己完結型メディア** —— 写真・動画・音声（`.ogg/.oga/.mp3`）・ドキュメント・ステッカーを各投稿のバンドル（これがキャッシュも兼ねます）にダウンロードします。写真は安定した Telegram ファイル ID でアドレス指定されるため、投稿テキストの編集でメディアが再ダウンロードされることはなく、画像の**差し替え**（投稿は*編集済み*になります）では新しいファイルを取得して古いものを削除します。
- **デフォルトの真っ黒テーマ** —— 組み込みテンプレートはダークモードで `prefers-color-scheme` により `#000`（OLED にやさしい）、外部テーマ依存なし。外部テーマを確実なフォールバック付きで重ねられます（**「テーマ」** の節を参照）。
- **賢い動画処理**（優先順位順）：
  1. 添付動画 **+ YouTube リンク** → YouTube を埋め込み、動画は破棄；
  2. 直接ダウンロード可能な動画 → ローカルの `<video>`；
  3. それ以外 → **ポスターフレーム** + 長さを保存（公開ページはファイルを提供しません；**「制限事項」** を参照）。
- **書式** —— 太字・斜体・打ち消し線・コード/整形済み・リンク・スポイラーは Markdown に変換されます（Telegram の UTF-16 エンティティオフセットを正しく処理）。
- **ハッシュタグ → タグ** —— `#ハッシュタグ` は Zola の `tags` タクソノミー用語になり、無料でタグページが手に入りつつ、投稿内ではテキストとして表示されます。
- **グループ化された投稿** —— アルバムは自動的に 1 つの投稿になり、同じ瞬間に投稿されたメッセージの連続（例：複数を一度に転送）はまとめられます。
- **自己ナビゲーション** —— *同じ*チャンネル内の別メッセージへのリンクは、ブログ内のその投稿への相対リンクになります；他チャンネルへのリンクは外部のままです。
- **エンゲージメント** —— 投稿ごとの**表示回数**をエクスポートします。（リアクション/いいねは公開ページでは取得できません —— **「制限事項」** を参照。）
- **RSS フィード** —— 標準の `/rss.xml` に**すべての投稿**を全文で（最新項目だけでなく完全なフィード）含み、`<link rel="alternate">` で告知するので、フィードリーダーがサイト URL から自動検出します。デフォルトで有効；`RSS=false` / `--no-rss` で無効化。
- **リッチなリンクプレビュー + Mastodon** —— 各ページは Open Graph と Twitter Card タグ（タイトル・説明・投稿の最初の画像）を出力するので、共有リンクはカードとして表示されます。`FEDIVERSE_CREATOR` を設定すると、Mastodon のプレビューに著者表記を追加し、プロフィールでサイトを認証できます（**「Fediverse」** の節を参照）。
- **デフォルトで JavaScript なし、オフライン対応** —— ダークモードとスポイラーは CSS のみ、デフォルトの Google 検索ボックスは素の `<form>`（JS なし）。Google 以外の検索エンジンだけが小さなインラインの Enter ハンドラを追加します。`tg2zola offline <public ディレクトリ>` はビルド済みサイトを相対リンクに書き換え、**さらに** Zola のページネーションリダイレクトスクリプトを除去するので、オフラインコピーは `file://` から直接、JavaScript も Web サーバーもなしで開けます。
- **ローカライズされた UI** —— サイトの外装（新しい/古い/タグ/サイトについて、検索ボックス、日付）は `LANGUAGE` / `--language` で 12 言語のいずれかで表示され（en/be/uk/ru/de/fr/zh/ja/pl/es/ko/ka）、月名・曜日名もローカライズされます。投稿の内容はチャンネル自身の言語のままです。

## インストール

[Releases](../../releases) ページからお使いのアーキテクチャ向けの静的バイナリ（Linux `amd64` / `arm64`、musl）を入手するか、ソースからビルドします：

```sh
cargo build --release
# バイナリは target/release/tg2zola
```

生成された内容を HTML に変換するには [`zola`](https://www.getzola.org/documentation/getting-started/installation/) バイナリも必要です。

## 使い方

```sh
# Zola サイトを生成（初回実行時に config + テンプレートを足場として作成）：
tg2zola --channel durov --site site --init-site

# 静的 HTML をビルド：
zola --root site build       # 出力は site/public/

# （任意）Web サーバーなしで、ディスクから直接閲覧可能にする：
tg2zola offline site/public  # その後 site/public/index.html を file:// で開く
```

手早いローカルテスト（1 ページ、約 20 メッセージ）：

```sh
tg2zola --channel <name> --site /tmp/site --init-site --max-pages 1
```

すべてのオプションは [`tg2zola.toml`](tg2zola.toml) にあります（CLI フラグが上書きします）：

```sh
tg2zola --config tg2zola.toml
```

完全なフラグ一覧は `tg2zola --help` を実行してください。

## 仕組み

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

各投稿は Zola のページバンドルになります —— `content/posts/<date>-<id>/index.md` に TOML フロントマターと、その隣のメディアファイル。`config.toml` と組み込みテンプレートは実行のたびに決定論的に再生成され、`write_site` がツリーを整合させます：すべての Markdown を書き直し、キャッシュ済みメディアを保持し、削除された投稿や古いファイルを取り除きます。

## 自動化（GitHub Actions）

2 つのワークフローが含まれます：

- **[`daily.yml`](.github/workflows/daily.yml)** —— 1 日 1 回（およびオンデマンド）実行：`blog` ブランチ（メディアキャッシュ）から前回のサイトを復元 → スクレイプ + 再生成 → `zola build` → **GitHub Pages にデプロイ**（公開される結果）→ 更新したサイトを **`blog` ブランチ**にコミットして戻す。
- **[`release.yml`](.github/workflows/release.yml)** —— `v*` タグがプッシュされるたびに、静的な `amd64` + `arm64`（musl）バイナリをクロスコンパイルして GitHub Release にアップロードします。

公開を有効にするには：リポジトリで **Settings → Pages → Build and deployment → Source: GitHub Actions**。シークレットは不要 —— すべて公開スクレイプです。公開されるサイトは常に **GitHub Pages**；`blog` ブランチは永続的なコピーであり、訪問者に見えるものには決して影響しません。

### 今すぐ再生成（毎日の実行を待たずに）

`daily.yml` は `workflow_dispatch` が有効なので、オンデマンドで新たなスクレイプ + ビルド + 再デプロイをトリガーできます —— スケジュール実行とまったく同じ手順を実行します：

- **ブラウザで：** **[Actions → 「daily」→ Run workflow](../../actions/workflows/daily.yml)** を開き、緑色の **Run workflow** ボタンをクリック。（上のステータスバッジを README に追加するとワンクリックでアクセスできます。）
- **ターミナルから：** `gh workflow run daily.yml`（GitHub CLI）、続けて `gh run watch` で追跡。

**どのチャンネル？** チャンネルはコミットされないので、各デプロイが自分のものを設定します。リポジトリ**変数** `CHANNEL`（Settings → Secrets and variables → Actions → Variables）に公開チャンネル名を設定します —— チャンネルは公開なので、シークレットではなく*変数*です。（または、フォーク内の [`tg2zola.toml`](tg2zola.toml) の `channel = "…"` をコメント解除します。）`THEME_REPO` も変数として同様に機能します。

### `blog` ブランチ（アーカイブ + キャッシュ）

各実行は、生成したサイト（Markdown + メディア + 組み込みテンプレート —— ビルド済みの `public/` と外部テーマを除くすべて）を `blog` ブランチにコミットし、`main` はコードのみにします。二役を担います：

- **キャッシュ** —— 次の実行は再ダウンロードせずにここからメディアを復元します。
- **永続アーカイブ** —— 完全でビルド可能な Zola サイトで、どこへでもクローンしてミラーでき、バックアップが 1 つのプラットフォームに縛られません：

```sh
git clone -b blog https://github.com/<you>/<repo> my-archive
zola --root my-archive/site serve          # オフラインで閲覧

git -C my-archive remote add gitlab git@gitlab.com:<you>/<repo>.git
git -C my-archive push gitlab blog          # 別の場所にミラー
```

メディアは通常の git blob としてコミットされます；非常に大きなチャンネルでは Git LFS や時々の履歴の圧縮を検討してください。

### テーマ

デフォルトは組み込みの**真っ黒**テーマ —— 外部依存ゼロなので、サイトは常にビルドできます。外部の [Zola テーマ](https://www.getzola.org/themes/)を使うには、リポジトリ**変数** `THEME_REPO`（Settings → Secrets and variables → Actions → Variables）にその git URL（https、またはデプロイキー付きの ssh）を設定します。ワークフローはそれをクローンしてビルドします —— そして**テーマが見つからない、またはビルドに失敗した場合は、自動的に組み込みテンプレートにフォールバックします**ので、テーマの問題でブログがオフラインになることは決してありません。外部テーマは特定のコンテンツレイアウトを前提とするため、すべてのテーマがそのまま使えるわけではない点にご注意ください。

リリースを切る：

```sh
git tag v0.1.0 && git push origin v0.1.0
```

## 設定

すべては GitHub Actions フロー用のリポジトリ**変数**（Settings → Secrets and variables → Actions → **Variables**）で、またはローカル実行時には同等の CLI フラグ / [`tg2zola.toml`](tg2zola.toml) キーで設定できます。これらはシークレットではなく*変数*です —— すべて公開です。

| リポジトリ変数 | CLI フラグ | デフォルト | 機能 |
|---|---|---|---|
| `CHANNEL` | `--channel` | **必須** | 同期する公開チャンネル |
| `TITLE` | `--title` | チャンネル名 | ブログのタイトル（ヘッダー + `<title>`） |
| `ABOUT` | `--about` | 説明 + 統計 + リポジトリリンク | 「サイトについて」ページ本文のカスタム HTML |
| `PAGES` | `--pages` | — | 追加ページ：Markdown、各 `# Title` 見出しが新しいページ + ナビ項目を開始 |
| `POSTS_PER_PAGE` | `--posts-per-page` | `20` | ホームフィードの 1 ページあたりの全文投稿数 |
| `TAGS_FOOTER` | `--tags-footer` | オフ | `true` で投稿ごとのタグフッターを表示（タグは本文内でいずれにせよクリック可能） |
| `NEXT_PREV` | `--no-next-prev` | オン | `false` で前/次の投稿ナビを非表示 |
| `TELEGRAM_LINK` | `--no-telegram-link` | オン | `false` で投稿ごとの「Telegram で見る」リンクを非表示 |
| `RSS` | `--no-rss` | オン | `false` で `/rss.xml` の RSS フィードを無効化（自動検出付き） |
| `FEDIVERSE_CREATOR` | `--fediverse-creator` | — | Mastodon `@user@instance` → `fediverse:creator` 表記 + `rel="me"` プロフィールリンク |
| `SEARCH_ENGINE` | `--search-engine` | `google` | ヘッダー検索ボックス：`google`（JS なしフォーム）/ `duckduckgo` / `yandex` / `bing` / `none` |
| `SEARCH_URL` | `--search-url` | — | カスタム検索 URL プレフィックス；Enter で入力が追加されます（エンジンを上書き） |
| `TITLE_MAX_LEN` | `--title-max-len` | `200` | 投稿タイトルの最大長（文字）；切り詰められたタイトルは本文に最初の文を丸ごと残します |
| `FOOTER` | `--footer` | — | フッターの内容 —— プレーンテキスト・Markdown・HTML |
| `PAGES_HOST` | `--pages-host` | 自動 | 「サイトについて」のサイズ上限のホスト：`github` / `gitlab` / `none`（URL から自動検出） |
| `DATE_FORMAT` | `--date-format` | `%Y %B %d` | 表示日付の strftime 形式（例：`2025 October 28`；`%Y` で年のみ） |
| `LANGUAGE` | `--language` | `en` | サイト UI の言語（新しい/古い/タグ/サイトについて/…）：`en`/`be`/`uk`/`ru`/`de`/`fr`/`zh`/`ja`/`pl`/`es`/`ko`/`ka`（ジョージア語）。投稿の内容はチャンネルの言語のまま；日付はローカライズされます |
| `LINK_UNDERLINE` | `--link-underline` | オフ | `true` でリンクに下線（デフォルト：下線なし） |
| `YOUTUBE_FACADE` | `--youtube-facade` | オフ | `true` で JS なしのクリックで読み込む YouTube サムネイル（デフォルト：直接 iframe） |
| `GENIUS` | `--no-genius` | オン | `false` で genius.com リンクの解決をスキップ（ページを取得して YouTube 動画 + 歌詞ウィジェットを得る） |
| `TAGS_TO_PAGES` | `--tags-to-pages` | — | 上部ナビに `#tag` リンクとして表示するカンマ区切りのタグ（例：`music, batumi, cooking`） |
| `BACKGROUND_DARK_COLOR` | `--background-dark-color` | `#000000` | ダークモードの背景（任意の CSS 色） |
| `BACKGROUND_LIGHT_COLOR` | `--background-light-color` | `#ffffff` | ライトモードの背景 |
| `CSS` | `--css` | — | 組み込みスタイルシートに追記する追加 CSS |
| `THEME_REPO` | `--theme`（名前） | 組み込みの黒テーマ | 外部 Zola テーマの git URL（https/ssh）；失敗時は自動フォールバック |
| `REPO_URL` | `--repo-url` | tg2zola リポジトリ | 「サイトについて」の「ソースコードリポジトリ」リンク（CI が自動であなたのリポジトリに設定） |

**「サイトについて」** ページは、チャンネルの**アバター**（フルサイズ）、その説明と統計、**ディスク上のサイズ** —— 種類別の内訳と、GitHub/GitLab Pages ではホストの公開サイト約 1 GB 上限に対する割合（ホストのドキュメントへリンク）—— に加えてリポジトリリンクを表示します。ヘッダーはアバターをサムネイルと favicon として表示；ハッシュタグは投稿本文でクリック可能で `/tags/<tag>/` ページを生成します。

1 つの `PAGES` 変数で複数のページを定義できます —— 各 `# Title` 見出しが新しいページを開始します：

```
# ページタイトル
Markdown で書かれたページ内容（生の HTML を含められます）。

# 別のページ
さらに内容。
```

→ `/page-title/` と `/another-page/`、それぞれナビにリンクされます。追加依存なし —— Zola はもともと Markdown をレンダリングします。

## Fediverse / Mastodon

各ページは Open Graph + Twitter Card タグを持つので、共有された投稿リンクは Mastodon・Slack・Discord・X などでカードとして表示されます（タイトル・説明・投稿の最初の画像）。`FEDIVERSE_CREATOR` をあなたの `@user@instance` ハンドルに設定すると、さらに：

- **`fediverse:creator`** 表記を追加し、Mastodon がリンクプレビューに「*by @you@instance*」を表示してあなたのプロフィールにリンクします；
- あなたのプロフィールへの **`rel="me"`** リンクを出力するので、この サイトを Mastodon プロフィールのメタデータに追加して**認証済み**（緑）のチェックを得られます。

**人々は Mastodon *から*ブログをフォローできますか？** 直接はできません —— 静的サイトは ActivityPub アクターになれません（それには WebFinger + ActivityPub を話す稼働中のサーバーが必要です）。それでも購読してもらう 2 つの方法：

- **RSS** —— 今日でも誰でもフィードリーダーで `/rss.xml` をフォローできます（多くの Mastodon ユーザーはリーダーを使っています）。これは組み込みでデフォルト有効です。
- **ブリッジ** —— RSS フィードを [rss-parrot](https://rss-parrot.net/) や [Bridgy Fed](https://fed.brid.gy/) のような RSS→ActivityPub ブリッジに向けます。これらは本物の `@handle` を提供し、Mastodon ユーザーが**フォロー**でき、新しい投稿を彼らのタイムラインに中継します。自分のサーバーは不要です。

つまり：リッチなプレビュー + 著者表記 + プロフィール認証はそのまま動作し、本当の「Mastodon からのフォロー」は、RSS フィードを入力とするブリッジ 1 つ分の距離です。

## 制限事項

公開ウェブプレビューは、**認証不要**であることの代償です：

- **リアクション/いいねは公開されません** —— `t.me/s/` は提供しません。代わりに**表示回数**をエクスポートします。データモデルには、必要であれば後で認証済みの MTProto API（[`grammers`](https://codeberg.org/Lonami/grammers) クレート）経由で本物のリアクションを追加する余地があります。
- **大きな動画はダウンロードできません** —— プレビューはそれらにポスター画像と長さしか提供しません（短い/自動再生の動画は*ダウンロードできます*）。実ファイルのアーカイブにも MTProto API が必要です。
- **ステッカーパックはリンクできません** —— パック名は Telegram の JavaScript で読み込まれ、スクレイプした HTML には含まれません；ステッカーは通常の画像として保存されます。
- **音楽ファイル（音声ドキュメント）はダウンロードできません** —— その URL はスクレイプした HTML にありません（直接の `.oga` URL を持つボイスメモのみ）。大きな動画やステッカーと同様、MTProto API が必要です。取得できない添付については、その**ファイル名**を（*未アーカイブ*と記して）保持し、存在したことが分かるようにします；*そのような参照だけ*（または単一のダウンロード不可ファイルだけ）からなる投稿は、空で公開する代わりにスキップされます。
- **YouTube** は公開された HTTPS サイトでは `youtube.com` の iframe で再生されます（再生が視聴者の履歴にカウントされます）；`file://` 経由では iframe を読み込めません（YouTube にはオリジンが必要）。`YOUTUBE_FACADE=true` は iframe を JS なしのクリックで読み込むサムネイルに置き換え、少なくとも `file://` でポスターを表示します。
- **公開チャンネルのみ**、ウェブプレビューが有効なもの。

## ライセンス

[MIT](LICENSE) © Vitaly Zdanevich
