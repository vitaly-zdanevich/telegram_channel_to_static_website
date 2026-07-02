# tg2zola

🌐 [English](README.md) · [Беларуская](README.be.md) · [Українська](README.uk.md) · **Русский** · [Deutsch](README.de.md) · [Français](README.fr.md) · [中文](README.zh.md) · [日本語](README.ja.md) · [Polski](README.pl.md) · [Español](README.es.md) · [한국어](README.ko.md) · [ქართული](README.ka.md) · [हिन्दी](README.hi.md)

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

Резервная копия **публичного Telegram-канала** в виде **самодостаточного статического сайта на [Zola](https://www.getzola.org/)**.

> **Перегенерация по запросу:** нажмите на бейдж **daily build** выше → **Run
> workflow**, чтобы заново спарсить, собрать и развернуть сайт, не дожидаясь
> расписания (подробности — в разделе **«Автоматизация»**).

Инструмент парсит публичную веб-версию (`https://t.me/s/<channel>`), скачивает
все медиа локально и заново генерирует полноценный блог на Zola при каждом
запуске. **Не нужны ни бот, ни токен, ни API Telegram** — читается только
публичная веб-страница. В результате **нет зависимости от Telegram**: медиа
хранятся локально, нет встраиваемых виджетов, а ссылки на *собственные* посты
канала переписываются во внутренние относительные ссылки — поэтому сайт
продолжает работать, даже если канал позже удалят. Ссылки, которые вы ставили на
другие сайты (включая другие Telegram-каналы), сохраняются как обычные ссылки.
Это резервная копия, а не зеркало.

Написан на Rust: один статический бинарник, удобно запускать локально или в CI.

## Что умеет

- **Полная история** — проходит канал в обратном порядке по курсору `?before=`
  веб-версии до самого первого сообщения и **заново генерирует все страницы** при
  каждом запуске.
- **Самодостаточные медиа** — скачивает фото, видео, аудио (`.ogg/.oga/.mp3`),
  документы и стикеры в бандл каждого поста (который заодно служит кешем). Фото
  адресуются по их стабильному идентификатору файла в Telegram, поэтому правка
  текста поста никогда не перекачивает медиа, а **замена** изображения (пост тогда
  помечается как *изменённый*) скачивает новый файл и удаляет старый.
- **Тема «истинно чёрная» по умолчанию** — встроенные шаблоны со стилем `#000` в
  тёмном режиме через `prefers-color-scheme` (бережно к OLED), без внешних
  зависимостей. Внешнюю тему можно подключить поверх с гарантированным откатом
  (см. раздел **«Темы»**).
- **Умная обработка видео** (в порядке приоритета):
  1. прикреплённое видео **+ ссылка на YouTube** → встраивается YouTube, видео
     отбрасывается;
  2. напрямую скачиваемое видео → локальное `<video>`;
  3. иначе → сохраняется **постер-кадр** + длительность (публичная страница не
     отдаёт файл; см. **«Ограничения»**).
- **Форматирование** — жирный, курсив, зачёркивание, код/моноширинный блок,
  ссылки и спойлеры конвертируются в Markdown (смещения UTF-16-сущностей Telegram
  обрабатываются корректно).
- **Хештеги → теги** — `#хештеги` становятся терминами таксономии `tags` в Zola,
  так что вы бесплатно получаете страницы тегов, при этом они остаются текстом в
  посте.
- **Сгруппированные посты** — альбомы автоматически становятся одним постом;
  всплески сообщений, опубликованных в один момент (например, при пересылке
  нескольких сразу), объединяются.
- **Самонавигация** — ссылка на другое сообщение в *том же* канале превращается в
  относительную ссылку на этот пост в блоге; ссылки на другие каналы остаются
  внешними.
- **Вовлечённость** — экспортирует **счётчики просмотров** по каждому посту.
  (Реакции/лайки недоступны на публичной странице — см. **«Ограничения»**.)
- **RSS-лента** — стандартный `/rss.xml` со **всеми постами** и полным
  содержимым (полная лента, а не только свежие записи), объявленный через
  `<link rel="alternate">`, чтобы читалки находили её автоматически по адресу
  сайта. Включено по умолчанию; отключается через `RSS=false` / `--no-rss`.
- **Богатые превью ссылок + Mastodon** — каждая страница содержит теги Open Graph
  и Twitter Card (заголовок, описание, первое изображение поста), поэтому ссылки
  отображаются карточками. Задайте `FEDIVERSE_CREATOR`, чтобы добавить подпись
  автора в превью Mastodon и подтвердить сайт в своём профиле (см. раздел
  **«Fediverse»**).
- **Без JavaScript по умолчанию, готов к офлайну** — тёмный режим и спойлеры
  реализованы только на CSS, а строка поиска Google по умолчанию — обычная
  `<form>` (без JS). Только не-Google поисковик добавляет один крошечный
  встроенный обработчик Enter. `tg2zola offline <папка-public>` переписывает
  собранный сайт на относительные ссылки **и** удаляет скрипт-редирект пагинации
  Zola, поэтому офлайн-копия открывается прямо из `file://` без какого-либо
  JavaScript и без веб-сервера.
- **Локализованный интерфейс** — оболочка сайта (Новее/Старее/Теги/О проекте,
  строка поиска, даты) отображается на любом из 12 языков через `LANGUAGE` /
  `--language` (en/be/uk/ru/de/fr/zh/ja/pl/es/ko/ka), с локализованными названиями
  месяцев и дней недели. Содержимое постов остаётся на языке канала.

## Установка

Возьмите статический бинарник для вашей архитектуры со страницы
[Releases](../../releases) (Linux `amd64` / `arm64`, musl) или соберите из
исходников:

```sh
cargo build --release
# бинарник в target/release/tg2zola
```

Также нужен бинарник [`zola`](https://www.getzola.org/documentation/getting-started/installation/),
чтобы превратить сгенерированное содержимое в HTML.

## Использование

```sh
# Сгенерировать сайт Zola (при первом запуске создаёт config + шаблоны):
tg2zola --channel durov --site site --init-site

# Собрать статический HTML:
zola --root site build       # результат в site/public/

# (необязательно) Сделать просмотр БЕЗ веб-сервера, прямо с диска:
tg2zola offline site/public  # затем откройте site/public/index.html через file://
```

Быстрая локальная проверка (одна страница, ~20 сообщений):

```sh
tg2zola --channel <name> --site /tmp/site --init-site --max-pages 1
```

Все параметры — в [`tg2zola.toml`](tg2zola.toml) (флаги CLI перекрывают его):

```sh
tg2zola --config tg2zola.toml
```

Запустите `tg2zola --help` для полного списка флагов.

## Как это устроено

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

Каждый пост становится бандлом страницы Zola — `content/posts/<date>-<id>/index.md`
с TOML-фронтматтером и медиафайлами рядом. `config.toml` и встроенные шаблоны
детерминированно перегенерируются при каждом запуске, а `write_site` приводит
дерево в порядок: переписывает весь Markdown, сохраняет уже закешированные медиа
и удаляет удалённые посты и устаревшие файлы.

## Автоматизация (GitHub Actions)

Включены два воркфлоу:

- **[`daily.yml`](.github/workflows/daily.yml)** — запускается раз в день (и по
  запросу): восстановить предыдущий сайт из ветки `blog` (кеш медиа) → спарсить +
  перегенерировать → `zola build` → **развернуть на GitHub Pages** (опубликованный
  результат) → закоммитить обновлённый сайт обратно в ветку **`blog`**.
- **[`release.yml`](.github/workflows/release.yml)** — на каждый запушенный тег
  `v*` кросс-компилирует статические бинарники `amd64` + `arm64` (musl) и
  загружает их в GitHub Release.

Чтобы включить публикацию: в репозитории **Settings → Pages → Build and deployment
→ Source: GitHub Actions**. Секреты не нужны — всё работает на публичном парсинге.
Опубликованный сайт — всегда **GitHub Pages**; ветка `blog` — это надёжная копия,
которая никогда не влияет на то, что видят посетители.

### Перегенерировать сейчас (не дожидаясь ежедневного запуска)

У `daily.yml` включён `workflow_dispatch`, поэтому можно запустить свежий парсинг
+ сборку + развёртывание по запросу — выполняются ровно те же шаги, что и по
расписанию:

- **В браузере:** откройте **[Actions → «daily» → Run workflow](../../actions/workflows/daily.yml)**
  и нажмите зелёную кнопку **Run workflow**. (Добавьте бейдж статуса выше в свой
  README для доступа в один клик.)
- **Из терминала:** `gh workflow run daily.yml` (GitHub CLI), затем
  `gh run watch`, чтобы следить за выполнением.

**Какой канал?** Канал не закоммичен, поэтому каждое развёртывание задаёт свой.
Задайте **переменную** репозитория `CHANNEL` (Settings → Secrets and variables →
Actions → Variables) с именем публичного канала — это *переменная*, а не секрет,
поскольку канал публичный. (Или раскомментируйте `channel = "…"` в
[`tg2zola.toml`](tg2zola.toml) в своём форке.) `THEME_REPO` тоже работает как
переменная.

### Ветка `blog` (архив + кеш)

Каждый запуск коммитит сгенерированный сайт (Markdown + медиа + встроенные
шаблоны — всё, кроме собранного `public/` и любой внешней темы) в ветку `blog`,
оставляя в `main` только код. Она выполняет двойную роль:

- **Кеш** — следующий запуск восстанавливает медиа из неё вместо повторного
  скачивания.
- **Надёжный архив** — полный, собираемый сайт Zola, который можно склонировать и
  зеркалировать где угодно, чтобы копия не была привязана к одной платформе:

```sh
git clone -b blog https://github.com/<you>/<repo> my-archive
zola --root my-archive/site serve          # просмотр офлайн

git -C my-archive remote add gitlab git@gitlab.com:<you>/<repo>.git
git -C my-archive push gitlab blog          # зеркалировать в другое место
```

Медиа коммитятся как обычные git-блобы; для очень больших каналов рассмотрите Git
LFS или периодическое сжатие истории.

### Темы

По умолчанию используется встроенная **истинно чёрная** тема — без внешних
зависимостей, поэтому сайт всегда собирается. Чтобы использовать внешнюю
[тему Zola](https://www.getzola.org/themes/), задайте **переменную** репозитория
`THEME_REPO` (Settings → Secrets and variables → Actions → Variables) с её
git-URL (https или ssh с deploy-ключом). Воркфлоу клонирует её и собирает с ней —
и **если тема отсутствует или её сборка падает, происходит автоматический откат на
встроенные шаблоны**, так что проблема с темой никогда не оставит блог офлайн.
Учтите, что внешние темы рассчитывают на определённую структуру контента, поэтому
не каждая тема подходит «из коробки».

Сделать релиз:

```sh
git tag v0.1.0 && git push origin v0.1.0
```

## Конфигурация

Всё настраивается через **переменную** репозитория (Settings → Secrets and
variables → Actions → **Variables**) для потока GitHub Actions либо через
эквивалентный флаг CLI / ключ [`tg2zola.toml`](tg2zola.toml) при локальном
запуске. Это *переменные*, а не секреты — всё публично.

| Переменная репо | Флаг CLI | По умолчанию | Что делает |
|---|---|---|---|
| `CHANNEL` | `--channel` | **обязательно** | Публичный канал для синхронизации |
| `TITLE` | `--title` | имя канала | Заголовок блога (шапка + `<title>`) |
| `ABOUT` | `--about` | описание + статистика + ссылка на репозиторий | Свой HTML для тела страницы «О проекте» |
| `PAGES` | `--pages` | — | Доп. страницы: Markdown, каждый заголовок `# Title` начинает новую страницу + пункт меню |
| `POSTS_PER_PAGE` | `--posts-per-page` | `20` | Полных постов на странице ленты |
| `TAGS_FOOTER` | `--tags-footer` | выкл | `true` показывает футер тегов под постом (теги кликабельны в теле в любом случае) |
| `NEXT_PREV` | `--no-next-prev` | вкл | `false` скрывает навигацию по постам «Назад/Вперёд» |
| `TELEGRAM_LINK` | `--no-telegram-link` | вкл | `false` скрывает ссылку «Открыть в Telegram» под постом |
| `RSS` | `--no-rss` | вкл | `false` отключает RSS-ленту по адресу `/rss.xml` (с автообнаружением читалками) |
| `FEDIVERSE_CREATOR` | `--fediverse-creator` | — | Mastodon `@user@instance` → подпись `fediverse:creator` + ссылка профиля `rel="me"` |
| `SEARCH_ENGINE` | `--search-engine` | `google` | Строка поиска в шапке: `google` (форма без JS) / `duckduckgo` / `yandex` / `bing` / `none` |
| `SEARCH_URL` | `--search-url` | — | Свой префикс URL поиска; запрос добавляется по Enter (перекрывает движок) |
| `TITLE_MAX_LEN` | `--title-max-len` | `200` | Макс. длина заголовка поста (символы); усечённый заголовок сохраняет полное первое предложение в теле |
| `FOOTER` | `--footer` | — | Содержимое футера — обычный текст, Markdown или HTML |
| `PAGES_HOST` | `--pages-host` | авто | Хост для лимита размера на странице «О проекте»: `github` / `gitlab` / `none` (определяется по URL) |
| `DATE_FORMAT` | `--date-format` | `%Y %B %d` | Формат strftime для отображаемых дат (например, `2025 October 28`; `%Y` — только год) |
| `LANGUAGE` | `--language` | `en` | Язык интерфейса сайта (Новее/Старее/Теги/О проекте/…): `en`/`be`/`uk`/`ru`/`de`/`fr`/`zh`/`ja`/`pl`/`es`/`ko`/`ka` (грузинский). Содержимое постов остаётся на языке канала; даты локализуются |
| `LINK_UNDERLINE` | `--link-underline` | выкл | `true` подчёркивает ссылки (по умолчанию без подчёркивания) |
| `YOUTUBE_FACADE` | `--youtube-facade` | выкл | `true` для превью YouTube с кликом-загрузкой без JS (по умолчанию прямой iframe) |
| `GENIUS` | `--no-genius` | вкл | `false` пропускает разбор ссылок genius.com (загрузка страницы ради YouTube-видео + виджета текста песни) |
| `TAGS_TO_PAGES` | `--tags-to-pages` | — | Теги через запятую, показываемые ссылками `#tag` в верхнем меню (например, `music, batumi, cooking`) |
| `BACKGROUND_DARK_COLOR` | `--background-dark-color` | `#000000` | Фон тёмного режима (любой CSS-цвет) |
| `BACKGROUND_LIGHT_COLOR` | `--background-light-color` | `#ffffff` | Фон светлого режима |
| `CSS` | `--css` | — | Доп. CSS, добавляемый к встроенным стилям |
| `THEME_REPO` | `--theme` (имя) | встроенная чёрная тема | Git-URL внешней темы Zola (https/ssh); авто-откат при ошибке |
| `REPO_URL` | `--repo-url` | репозиторий tg2zola | Ссылка «Репозиторий исходного кода» на «О проекте» (CI ставит ваш репозиторий) |

Страница **«О проекте»** показывает **аватар** канала (в полном размере), его
описание и статистику, **размер на диске** — с разбивкой по типам и, на
GitHub/GitLab Pages, долю от лимита опубликованного сайта (~1 ГБ) у хостинга (со
ссылкой на документацию хостинга) — плюс ссылку на репозиторий. В шапке аватар
показан миниатюрой и фавиконкой; хештеги кликабельны в телах постов и порождают
страницы `/tags/<tag>/`.

Одна переменная `PAGES` может задавать несколько страниц — каждый заголовок
`# Title` начинает новую:

```
# Заголовок страницы
Содержимое страницы в Markdown (может содержать обычный HTML).

# Другая страница
Ещё содержимое.
```

→ `/page-title/` и `/another-page/`, каждая со ссылкой в меню. Без дополнительных
зависимостей — Zola и так рендерит Markdown.

## Fediverse / Mastodon

Каждая страница несёт теги Open Graph + Twitter Card, поэтому ссылка на пост
отображается карточкой (заголовок, описание, первое изображение поста) в Mastodon,
Slack, Discord, X и т. д. Задайте `FEDIVERSE_CREATOR` равным вашему хэндлу
`@user@instance`, чтобы также:

- добавить подпись **`fediverse:creator`**, чтобы Mastodon показывал
  «*by @you@instance*» в превью ссылки и ссылался на ваш профиль;
- выдать ссылку **`rel="me"`** на ваш профиль, чтобы вы могли добавить этот сайт в
  метаданные профиля Mastodon и получить **подтверждённую** (зелёную) галочку.

**Могут ли люди подписаться на блог *из* Mastodon?** Не напрямую — статический
сайт не может быть актором ActivityPub (для этого нужен живой сервер с
WebFinger + ActivityPub). Два способа всё же дать людям подписаться:

- **RSS** — любой может подписаться на `/rss.xml` в читалке уже сегодня (многие
  пользователи Mastodon держат читалку). Это встроено и включено по умолчанию.
- **Мост** — направьте RSS-ленту на мост RSS→ActivityPub, например
  [rss-parrot](https://rss-parrot.net/) или [Bridgy Fed](https://fed.brid.gy/),
  которые предоставляют настоящий `@handle`, на который пользователи Mastodon
  могут **подписаться**, ретранслируя новые посты в их ленту. Собственный сервер
  не нужен.

Итого: богатые превью + атрибуция автора + подтверждение профиля работают «из
коробки»; настоящая «подписка из Mastodon» — в одном мосте, использующем RSS-ленту
как вход.

## Ограничения

Публичная веб-версия — это компромисс за **нулевую аутентификацию**:

- **Реакции/лайки не отдаются** через `t.me/s/`. Вместо этого мы экспортируем
  **счётчики просмотров**. Модель данных оставляет место для добавления настоящих
  реакций позже через аутентифицированный API MTProto (крейт
  [`grammers`](https://codeberg.org/Lonami/grammers)), если они вам понадобятся.
- **Большие видео не скачиваются** — веб-версия отдаёт для них только постер и
  длительность (короткие/автовоспроизводимые видео *скачиваются*). Архивация
  самого файла тоже потребовала бы API MTProto.
- **Наборы стикеров не дают ссылку** — имя набора загружается JavaScript-ом
  Telegram и отсутствует в спарсенном HTML; стикеры сохраняются как обычные
  картинки.
- **Музыкальные файлы (аудиодокументы) не скачиваются** — их URL нет в спарсенном
  HTML (есть только голосовые, с прямыми `.oga`-URL). Как большим видео и
  стикерам, им нужен API MTProto. Для вложения, которое мы не можем скачать, мы
  сохраняем его **имя файла** (с пометкой *не сохранено*), чтобы вы знали, что оно
  существовало; пост, состоящий *только* из такой ссылки (или из одного
  нескачиваемого файла), пропускается, а не публикуется пустым.
- **YouTube** проигрывается через iframe `youtube.com` (поэтому просмотры идут в
  историю зрителя) на опубликованном HTTPS-сайте; через `file://` iframe не
  загружается (YouTube нужен origin). `YOUTUBE_FACADE=true` меняет iframe на
  превью с кликом-загрузкой без JS, которое хотя бы показывает постер через
  `file://`.
- **Только публичные каналы** с включённой веб-версией.

## Лицензия

[MIT](LICENSE) © Vitaly Zdanevich
