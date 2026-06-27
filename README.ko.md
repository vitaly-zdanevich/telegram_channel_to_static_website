# tg2zola

🌐 [English](README.md) · [Беларуская](README.be.md) · [Українська](README.uk.md) · [Русский](README.ru.md) · [Deutsch](README.de.md) · [Français](README.fr.md) · [中文](README.zh.md) · [日本語](README.ja.md) · [Polski](README.pl.md) · [Español](README.es.md) · **한국어** · [ქართული](README.ka.md)

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

**공개 Telegram 채널**을 **자체 완결형 [Zola](https://www.getzola.org/) 정적 사이트**로 백업합니다.

> **온디맨드 재생성:** 위의 **daily build** 배지 → **Run workflow** 를 클릭하면 일정 작업을 기다리지 않고 스크래핑·빌드·재배포할 수 있습니다(자세한 내용은 **「자동화」** 절).

이 도구는 공개 웹 미리보기(`https://t.me/s/<channel>`)를 읽고, 모든 미디어를 로컬에 내려받아 실행할 때마다 완전한 Zola 블로그를 재생성합니다. **Telegram 봇, 토큰, API가 전혀 필요 없습니다** —— 공개 웹 페이지만 읽습니다. 결과물은 **Telegram에 의존하지 않습니다**: 미디어는 로컬에 있고, 임베드가 없으며, 채널 *자신*의 게시물로 가는 링크는 내부 상대 링크로 다시 작성됩니다 —— 그래서 채널이 나중에 삭제되어도 사이트는 계속 동작합니다. 다른 사이트(다른 Telegram 채널 포함)로 직접 작성한 링크는 일반 링크로 보존됩니다. 이것은 미러가 아니라 백업입니다.

Rust로 작성: 단일 정적 바이너리로, 로컬이나 CI에서 쉽게 실행됩니다.

## 기능

- **전체 기록** —— 미리보기의 `?before=` 커서로 채널을 첫 메시지까지 거슬러 올라가며, 실행할 때마다 **모든 페이지를 재생성**합니다.
- **자체 완결형 미디어** —— 사진, 동영상, 오디오(`.ogg/.oga/.mp3`), 문서, 스티커를 각 게시물의 번들(캐시 역할도 겸함)에 내려받습니다. 사진은 안정적인 Telegram 파일 id로 주소가 지정되므로 게시물 텍스트를 편집해도 미디어가 다시 내려받아지지 않으며, 이미지를 **교체**하면(게시물은 그때 *편집됨*으로 표시) 새 파일을 가져오고 옛 파일을 정리합니다.
- **기본 순흑 테마** —— 내장 템플릿이 다크 모드에서 `prefers-color-scheme`를 통해 `#000`(OLED 친화적)로 스타일링되며 외부 테마 의존성이 없습니다. 보장된 폴백과 함께 외부 테마를 덧입힐 수 있습니다(**「테마」** 절 참조).
- **스마트 동영상 처리**(우선순위 순):
  1. 첨부 동영상 **+ YouTube 링크** → YouTube 임베드, 동영상 폐기;
  2. 직접 내려받을 수 있는 동영상 → 로컬 `<video>`;
  3. 그 외 → **포스터 프레임** + 길이 저장(공개 페이지가 파일을 제공하지 않음; **「제한 사항」** 참조).
- **서식** —— 굵게, 기울임, 취소선, 코드/사전 서식, 링크, 스포일러가 Markdown으로 변환됩니다(Telegram의 UTF-16 엔티티 오프셋을 올바르게 처리).
- **해시태그 → 태그** —— `#해시태그`는 Zola의 `tags` 분류 용어가 되어 태그 페이지를 공짜로 얻으면서도 게시물에서는 여전히 텍스트로 표시됩니다.
- **묶인 게시물** —— 앨범은 자동으로 하나의 게시물이 되고, 같은 순간에 게시된 메시지 묶음(예: 여러 개를 한 번에 전달)은 병합됩니다.
- **자가 내비게이션** —— *같은* 채널의 다른 메시지로 가는 링크는 블로그 내 해당 게시물로 가는 상대 링크가 됩니다; 다른 채널로 가는 링크는 외부로 유지됩니다.
- **참여도** —— 게시물별 **조회수**를 내보냅니다.(리액션/좋아요는 공개 페이지에서 제공되지 않음 —— **「제한 사항」** 참조.)
- **RSS 피드** —— 표준 `/rss.xml`에 **모든 게시물**을 전문으로 담고(최신 항목만이 아니라 완전한 피드), `<link rel="alternate">`로 알려 피드 리더가 사이트 URL에서 자동으로 발견하게 합니다. 기본 켜짐; `RSS=false` / `--no-rss`로 끕니다.
- **풍부한 링크 미리보기 + Mastodon** —— 모든 페이지가 Open Graph와 Twitter Card 태그(제목, 설명, 게시물의 첫 이미지)를 내보내므로 공유된 링크가 카드로 렌더링됩니다. `FEDIVERSE_CREATOR`를 설정하면 Mastodon 미리보기에 작성자 표기를 더하고 프로필에서 사이트를 인증할 수 있습니다(**「Fediverse」** 절 참조).
- **기본적으로 JavaScript 없음, 오프라인 대응** —— 다크 모드와 스포일러는 CSS만으로 구현되고, 기본 Google 검색 상자는 평범한 `<form>`(JS 없음)입니다. Google이 아닌 검색 엔진만 아주 작은 인라인 Enter 핸들러를 추가합니다. `tg2zola offline <public 디렉터리>`는 빌드된 사이트를 상대 링크로 다시 쓰고 **또한** Zola의 페이지네이션 리디렉션 스크립트를 제거하므로, 오프라인 사본을 `file://`에서 곧바로, JavaScript도 웹 서버도 없이 열 수 있습니다.
- **현지화된 UI** —— 사이트 외형(최신/이전/태그/소개, 검색 상자, 날짜)이 `LANGUAGE` / `--language`를 통해 12개 언어 중 하나로 표시되며(en/be/uk/ru/de/fr/zh/ja/pl/es/ko/ka), 월·요일 이름도 현지화됩니다. 게시물 내용은 채널 고유의 언어를 유지합니다.

## 설치

[Releases](../../releases) 페이지에서 사용 중인 아키텍처용 정적 바이너리(Linux `amd64` / `arm64`, musl)를 받거나 소스에서 빌드하세요:

```sh
cargo build --release
# 바이너리는 target/release/tg2zola
```

생성된 콘텐츠를 HTML로 바꾸려면 [`zola`](https://www.getzola.org/documentation/getting-started/installation/) 바이너리도 필요합니다.

## 사용법

```sh
# Zola 사이트 생성(첫 실행 시 config + 템플릿을 스캐폴딩):
tg2zola --channel durov --site site --init-site

# 정적 HTML 빌드:
zola --root site build       # 출력은 site/public/

# (선택) 웹 서버 없이 디스크에서 곧바로 볼 수 있게:
tg2zola offline site/public  # 그런 다음 site/public/index.html을 file://로 열기
```

빠른 로컬 테스트(한 페이지, 약 20개 메시지):

```sh
tg2zola --channel <name> --site /tmp/site --init-site --max-pages 1
```

모든 옵션은 [`tg2zola.toml`](tg2zola.toml)에 있습니다(CLI 플래그가 이를 재정의):

```sh
tg2zola --config tg2zola.toml
```

전체 플래그 목록은 `tg2zola --help`를 실행하세요.

## 구성 방식

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

각 게시물은 Zola 페이지 번들이 됩니다 —— `content/posts/<date>-<id>/index.md`에 TOML 프런트매터와 그 옆의 미디어 파일. `config.toml`과 내장 템플릿은 실행할 때마다 결정론적으로 재생성되며, `write_site`가 트리를 조정합니다: 모든 Markdown을 다시 쓰고, 이미 캐시된 미디어를 유지하며, 삭제된 게시물과 오래된 파일을 정리합니다.

## 자동화(GitHub Actions)

두 개의 워크플로가 포함됩니다:

- **[`daily.yml`](.github/workflows/daily.yml)** —— 하루에 한 번(및 온디맨드) 실행: `blog` 브랜치(미디어 캐시)에서 이전 사이트 복원 → 스크래핑 + 재생성 → `zola build` → **GitHub Pages에 배포**(게시되는 결과) → 새로 고친 사이트를 **`blog` 브랜치**에 다시 커밋.
- **[`release.yml`](.github/workflows/release.yml)** —— `v*` 태그가 푸시될 때마다 정적 `amd64` + `arm64`(musl) 바이너리를 교차 컴파일하여 GitHub Release에 업로드합니다.

게시를 활성화하려면: 저장소에서 **Settings → Pages → Build and deployment → Source: GitHub Actions**. 시크릿이 필요 없습니다 —— 모두 공개 스크래핑입니다. 게시되는 사이트는 항상 **GitHub Pages**이며; `blog` 브랜치는 영구 사본으로 방문자가 보는 것에 결코 영향을 주지 않습니다.

### 지금 재생성(매일 실행을 기다리지 않고)

`daily.yml`은 `workflow_dispatch`가 켜져 있어 온디맨드로 새 스크래핑 + 빌드 + 재배포를 트리거할 수 있습니다 —— 예약 실행과 정확히 같은 단계를 수행합니다:

- **브라우저에서:** **[Actions → 「daily」→ Run workflow](../../actions/workflows/daily.yml)**를 열고 녹색 **Run workflow** 버튼을 클릭하세요.(위의 상태 배지를 README에 추가하면 한 번의 클릭으로 접근할 수 있습니다.)
- **터미널에서:** `gh workflow run daily.yml`(GitHub CLI), 이어서 `gh run watch`로 따라가기.

**어느 채널?** 채널은 커밋되지 않으므로 각 배포가 자신의 것을 설정합니다. 저장소 **변수** `CHANNEL`(Settings → Secrets and variables → Actions → Variables)을 공개 채널 사용자명으로 설정하세요 —— 채널은 공개이므로 시크릿이 아니라 *변수*입니다.(또는 포크의 [`tg2zola.toml`](tg2zola.toml)에서 `channel = "…"`의 주석을 해제하세요.) `THEME_REPO`도 변수로 동일하게 작동합니다.

### `blog` 브랜치(아카이브 + 캐시)

각 실행은 생성된 사이트(Markdown + 미디어 + 내장 템플릿 —— 빌드된 `public/`과 외부 테마를 제외한 전부)를 `blog` 브랜치에 커밋하여 `main`은 코드만 남깁니다. 두 가지 역할을 합니다:

- **캐시** —— 다음 실행이 다시 내려받지 않고 여기서 미디어를 복원합니다.
- **영구 아카이브** —— 어디로든 클론하고 미러링할 수 있는 완전하고 빌드 가능한 Zola 사이트로, 백업이 하나의 플랫폼에 묶이지 않습니다:

```sh
git clone -b blog https://github.com/<you>/<repo> my-archive
zola --root my-archive/site serve          # 오프라인으로 둘러보기

git -C my-archive remote add gitlab git@gitlab.com:<you>/<repo>.git
git -C my-archive push gitlab blog          # 다른 곳에 미러링
```

미디어는 일반 git blob으로 커밋됩니다; 매우 큰 채널의 경우 Git LFS나 가끔의 히스토리 압축을 고려하세요.

### 테마

기본값은 내장 **순흑** 테마입니다 —— 외부 의존성이 전혀 없어 사이트가 항상 빌드됩니다. 외부 [Zola 테마](https://www.getzola.org/themes/)를 사용하려면 저장소 **변수** `THEME_REPO`(Settings → Secrets and variables → Actions → Variables)를 그 git URL(https, 또는 배포 키가 있는 ssh)로 설정하세요. 워크플로가 이를 클론하여 함께 빌드합니다 —— 그리고 **테마가 없거나 그 빌드가 실패하면 자동으로 내장 템플릿으로 폴백**하므로, 테마 문제로 블로그가 오프라인이 되는 일은 결코 없습니다. 외부 테마는 특정 콘텐츠 레이아웃을 기대하므로 모든 테마가 그대로 호환되지는 않는다는 점에 유의하세요.

릴리스 만들기:

```sh
git tag v0.1.0 && git push origin v0.1.0
```

## 설정

모든 것은 GitHub Actions 흐름의 경우 저장소 **변수**(Settings → Secrets and variables → Actions → **Variables**)로, 또는 로컬 실행 시 동등한 CLI 플래그 / [`tg2zola.toml`](tg2zola.toml) 키로 설정할 수 있습니다. 이는 시크릿이 아니라 *변수*입니다 —— 전부 공개입니다.

| 저장소 변수 | CLI 플래그 | 기본값 | 하는 일 |
|---|---|---|---|
| `CHANNEL` | `--channel` | **필수** | 동기화할 공개 채널 |
| `TITLE` | `--title` | 채널 사용자명 | 블로그 제목(헤더 + `<title>`) |
| `ABOUT` | `--about` | 설명 + 통계 + 저장소 링크 | 「소개」 페이지 본문의 사용자 지정 HTML |
| `PAGES` | `--pages` | — | 추가 페이지: Markdown, 각 `# Title` 제목이 새 페이지 + 내비 항목을 시작 |
| `POSTS_PER_PAGE` | `--posts-per-page` | `20` | 홈 피드의 페이지당 전문 게시물 수 |
| `TAGS_FOOTER` | `--tags-footer` | 꺼짐 | `true`면 게시물별 태그 푸터 표시(태그는 어차피 본문에서 클릭 가능) |
| `NEXT_PREV` | `--no-next-prev` | 켜짐 | `false`면 이전/다음 게시물 내비 숨김 |
| `TELEGRAM_LINK` | `--no-telegram-link` | 켜짐 | `false`면 게시물별 「Telegram에서 보기」 링크 숨김 |
| `RSS` | `--no-rss` | 켜짐 | `false`면 `/rss.xml`의 RSS 피드 비활성화(자동 검색 포함) |
| `FEDIVERSE_CREATOR` | `--fediverse-creator` | — | Mastodon `@user@instance` → `fediverse:creator` 표기 + `rel="me"` 프로필 링크 |
| `SEARCH_ENGINE` | `--search-engine` | `google` | 헤더 검색 상자: `google`(JS 없는 폼) / `duckduckgo` / `yandex` / `bing` / `none` |
| `SEARCH_URL` | `--search-url` | — | 사용자 지정 검색 URL 접두사; Enter 시 질의가 추가됨(엔진을 재정의) |
| `TITLE_MAX_LEN` | `--title-max-len` | `200` | 게시물 제목 최대 길이(문자); 잘린 제목은 본문에 첫 문장을 온전히 유지 |
| `FOOTER` | `--footer` | — | 푸터 내용 —— 일반 텍스트, Markdown 또는 HTML |
| `PAGES_HOST` | `--pages-host` | 자동 | 「소개」 페이지 크기 한도의 호스트: `github` / `gitlab` / `none`(URL에서 자동 감지) |
| `DATE_FORMAT` | `--date-format` | `%Y %B %d` | 표시 날짜의 strftime 형식(예: `2025 October 28`; `%Y`는 연도만) |
| `LANGUAGE` | `--language` | `en` | 사이트 UI 언어(최신/이전/태그/소개/…): `en`/`be`/`uk`/`ru`/`de`/`fr`/`zh`/`ja`/`pl`/`es`/`ko`/`ka`(조지아어). 게시물 내용은 채널 고유 언어 유지; 날짜는 현지화됨 |
| `LINK_UNDERLINE` | `--link-underline` | 꺼짐 | `true`면 링크에 밑줄(기본: 밑줄 없음) |
| `YOUTUBE_FACADE` | `--youtube-facade` | 꺼짐 | `true`면 JS 없는 클릭하여 로드하는 YouTube 썸네일(기본: 직접 iframe) |
| `GENIUS` | `--no-genius` | 켜짐 | `false`면 genius.com 링크 해석 건너뜀(YouTube 동영상 + 가사 위젯을 위해 페이지를 가져옴) |
| `TAGS_TO_PAGES` | `--tags-to-pages` | — | 상단 내비에 `#tag` 링크로 표시할 쉼표 구분 태그(예: `music, batumi, cooking`) |
| `BACKGROUND_DARK_COLOR` | `--background-dark-color` | `#000000` | 다크 모드 배경(임의의 CSS 색) |
| `BACKGROUND_LIGHT_COLOR` | `--background-light-color` | `#ffffff` | 라이트 모드 배경 |
| `CSS` | `--css` | — | 내장 스타일시트에 덧붙이는 추가 CSS |
| `THEME_REPO` | `--theme`(이름) | 내장 검정 테마 | 외부 Zola 테마의 git URL(https/ssh); 실패 시 자동 폴백 |
| `REPO_URL` | `--repo-url` | tg2zola 저장소 | 「소개」의 「소스 코드 저장소」 링크(CI가 자동으로 당신의 저장소로 설정) |

**「소개」** 페이지는 채널 **아바타**(전체 크기), 그 설명과 통계, **디스크 사용량** —— 종류별 분석과, GitHub/GitLab Pages에서는 호스트의 게시 사이트 약 1 GB 한도 중 차지하는 비율(호스트 문서로 링크) —— 에 더해 저장소 링크를 보여 줍니다. 헤더는 아바타를 썸네일과 파비콘으로 표시하고; 해시태그는 게시물 본문에서 클릭 가능하며 `/tags/<tag>/` 페이지를 생성합니다.

하나의 `PAGES` 변수로 여러 페이지를 정의할 수 있습니다 —— 각 `# Title` 제목이 새 페이지를 시작합니다:

```
# 페이지 제목
Markdown으로 작성한 페이지 내용(원시 HTML 포함 가능).

# 다른 페이지
더 많은 내용.
```

→ `/page-title/`와 `/another-page/`, 각각 내비에 링크됩니다. 추가 의존성 없음 —— Zola가 이미 Markdown을 렌더링합니다.

## Fediverse / Mastodon

모든 페이지가 Open Graph + Twitter Card 태그를 지니므로, 공유된 게시물 링크는 Mastodon, Slack, Discord, X 등에서 카드로 렌더링됩니다(제목, 설명, 게시물의 첫 이미지). `FEDIVERSE_CREATOR`를 당신의 `@user@instance` 핸들로 설정하면 추가로:

- **`fediverse:creator`** 표기를 더해 Mastodon이 링크 미리보기에 「*by @you@instance*」를 표시하고 당신의 프로필로 링크하게 하며;
- 당신의 프로필로 가는 **`rel="me"`** 링크를 내보내, 이 사이트를 Mastodon 프로필 메타데이터에 추가하고 **인증된**(녹색) 체크를 받을 수 있습니다.

**사람들이 Mastodon*에서* 블로그를 팔로우할 수 있나요?** 직접은 안 됩니다 —— 정적 사이트는 ActivityPub 액터가 될 수 없습니다(그러려면 WebFinger + ActivityPub를 말하는 실시간 서버가 필요). 그래도 구독하게 하는 두 가지 방법:

- **RSS** —— 누구나 오늘 당장 피드 리더에서 `/rss.xml`을 팔로우할 수 있습니다(많은 Mastodon 사용자가 리더를 씁니다). 내장되어 기본 켜짐입니다.
- **브리지** —— RSS 피드를 [rss-parrot](https://rss-parrot.net/)나 [Bridgy Fed](https://fed.brid.gy/) 같은 RSS→ActivityPub 브리지로 향하게 하세요. 이들은 Mastodon 사용자가 **팔로우**할 수 있는 진짜 `@handle`을 제공하여 새 게시물을 그들의 타임라인으로 중계합니다. 당신만의 서버가 필요 없습니다.

요컨대: 풍부한 미리보기 + 작성자 표기 + 프로필 인증은 곧바로 동작하고; 진짜 「Mastodon에서 팔로우」는 RSS 피드를 입력으로 쓰는 브리지 하나만큼 떨어져 있습니다.

## 제한 사항

공개 웹 미리보기는 **인증이 전혀 필요 없음**의 대가입니다:

- **리액션/좋아요는 노출되지 않습니다** —— `t.me/s/`가 제공하지 않습니다. 대신 **조회수**를 내보냅니다. 데이터 모델은 원한다면 나중에 인증된 MTProto API([`grammers`](https://codeberg.org/Lonami/grammers) 크레이트)로 진짜 리액션을 추가할 여지를 남겨 둡니다.
- **큰 동영상은 내려받을 수 없습니다** —— 미리보기는 그것들에 포스터 이미지와 길이만 제공합니다(짧은/자동 재생 동영상은 내려받을 수 *있습니다*). 실제 파일을 아카이브하려면 역시 MTProto API가 필요합니다.
- **스티커 팩은 링크할 수 없습니다** —— 팩 이름은 Telegram의 JavaScript로 로드되어 스크래핑한 HTML에 없습니다; 스티커는 일반 이미지로 저장됩니다.
- **음악 파일(오디오 문서)은 내려받을 수 없습니다** —— 그 URL이 스크래핑한 HTML에 없습니다(직접 `.oga` URL을 가진 음성 메모만 있음). 큰 동영상이나 스티커처럼 MTProto API가 필요합니다. 가져올 수 없는 첨부에 대해서는 그 **파일 이름**을(*보관되지 않음*으로 표시) 보존하여 존재했음을 알 수 있게 합니다; *그러한 참조만*(또는 단일 다운로드 불가 파일만)으로 이루어진 게시물은 빈 채로 게시되는 대신 건너뜁니다.
- **YouTube**는 게시된 HTTPS 사이트에서 `youtube.com` iframe으로 재생됩니다(재생이 시청자의 기록에 집계됨); `file://`에서는 iframe이 로드될 수 없습니다(YouTube에 origin이 필요). `YOUTUBE_FACADE=true`는 iframe을 JS 없는 클릭하여 로드 썸네일로 바꿔 적어도 `file://`에서 포스터를 보여 줍니다.
- **공개 채널만**, 웹 미리보기가 켜진 채널.

## 라이선스

[MIT](LICENSE) © Vitaly Zdanevich
