# tg2zola

🌐 [English](README.md) · [Беларуская](README.be.md) · **Українська** · [Русский](README.ru.md) · [Deutsch](README.de.md) · [Français](README.fr.md) · [中文](README.zh.md) · [日本語](README.ja.md) · [Polski](README.pl.md) · [Español](README.es.md) · [한국어](README.ko.md) · [ქართული](README.ka.md) · [हिन्दी](README.hi.md)

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

Резервна копія **публічного Telegram-каналу** у вигляді **самодостатнього
статичного сайту на [Zola](https://www.getzola.org/)**.

> **Перегенерація на вимогу:** натисніть на бейдж **daily build** вище → **Run
> workflow**, щоб заново спарсити, зібрати й розгорнути сайт, не чекаючи розкладу
> (подробиці — у розділі **«Автоматизація»**).

Інструмент парсить публічну вебверсію (`https://t.me/s/<channel>`), завантажує
всі медіа локально й заново генерує повноцінний блог на Zola за кожного запуску.
**Не потрібні ні бот, ні токен, ні API Telegram** — читається лише публічна
вебсторінка. У результаті **немає залежності від Telegram**: медіа зберігаються
локально, немає вбудованих віджетів, а посилання на *власні* дописи каналу
переписуються у внутрішні відносні посилання — тож сайт працює далі, навіть якщо
канал згодом видалять. Посилання, які ви ставили на інші сайти (зокрема інші
Telegram-канали), зберігаються як звичайні посилання. Це резервна копія, а не
дзеркало.

Написаний мовою Rust: один статичний бінарник, зручно запускати локально або в CI.

## Що вміє

- **Повна історія** — проходить канал у зворотному порядку за курсором `?before=`
  вебверсії до найпершого повідомлення й **заново генерує всі сторінки** за
  кожного запуску.
- **Самодостатні медіа** — завантажує фото, відео, аудіо (`.ogg/.oga/.mp3`),
  документи та стікери в бандл кожного допису (який заразом слугує кешем). Фото
  адресуються за їхнім стабільним ідентифікатором файлу в Telegram, тож
  редагування тексту допису ніколи не перезавантажує медіа, а **заміна**
  зображення (допис тоді позначається як *змінений*) завантажує новий файл і
  видаляє старий.
- **Тема «справжній чорний» за замовчуванням** — вбудовані шаблони зі стилем
  `#000` у темному режимі через `prefers-color-scheme` (бережно до OLED), без
  зовнішніх залежностей. Зовнішню тему можна під'єднати поверх із гарантованим
  відкотом (див. розділ **«Теми»**).
- **Розумне опрацювання відео** (за пріоритетом):
  1. прикріплене відео **+ посилання на YouTube** → вбудовується YouTube, відео
     відкидається;
  2. відео, що завантажується напряму → локальне `<video>`;
  3. інакше → зберігається **постер-кадр** + тривалість (публічна сторінка не
     віддає файл; див. **«Обмеження»**).
- **Форматування** — жирний, курсив, закреслення, код/моноширинний блок,
  посилання та спойлери конвертуються в Markdown (зміщення UTF-16-сутностей
  Telegram опрацьовуються коректно).
- **Хештеги → теги** — `#хештеги` стають термінами таксономії `tags` у Zola, тож
  ви безкоштовно отримуєте сторінки тегів, а вони лишаються текстом у дописі.
- **Згруповані дописи** — альбоми автоматично стають одним дописом; сплески
  повідомлень, опублікованих в один момент (наприклад, при пересиланні кількох
  одразу), об'єднуються.
- **Самонавігація** — посилання на інше повідомлення в *тому самому* каналі стає
  відносним посиланням на цей допис у блозі; посилання на інші канали лишаються
  зовнішніми.
- **Залученість** — експортує **лічильники переглядів** для кожного допису.
  (Реакції/вподобайки недоступні на публічній сторінці — див. **«Обмеження»**.)
- **RSS-стрічка** — стандартний `/rss.xml` з **усіма дописами** й повним вмістом
  (повна стрічка, а не лише свіжі записи), оголошений через
  `<link rel="alternate">`, щоб читачки знаходили її автоматично за адресою сайту.
  Увімкнено за замовчуванням; вимикається через `RSS=false` / `--no-rss`.
- **Багаті превʼю посилань + Mastodon** — кожна сторінка містить теги Open Graph
  і Twitter Card (заголовок, опис, перше зображення допису), тож посилання
  показуються картками. Задайте `FEDIVERSE_CREATOR`, щоб додати підпис автора в
  превʼю Mastodon і підтвердити сайт у своєму профілі (див. розділ **«Fediverse»**).
- **Без JavaScript за замовчуванням, готовий до офлайну** — темний режим і
  спойлери реалізовані лише на CSS, а рядок пошуку Google за замовчуванням —
  звичайна `<form>` (без JS). Лише не-Google пошуковик додає один крихітний
  вбудований обробник Enter. `tg2zola offline <тека-public>` переписує зібраний
  сайт на відносні посилання **і** прибирає скрипт-редирект пагінації Zola, тож
  офлайн-копія відкривається прямо з `file://` без жодного JavaScript і без
  вебсервера.
- **Локалізований інтерфейс** — оболонка сайту (Новіші/Старіші/Теги/Про проєкт,
  рядок пошуку, дати) показується будь-якою з 12 мов через `LANGUAGE` /
  `--language` (en/be/uk/ru/de/fr/zh/ja/pl/es/ko/ka), з локалізованими назвами
  місяців і днів тижня. Вміст дописів лишається мовою каналу.

## Встановлення

Візьміть статичний бінарник для вашої архітектури зі сторінки
[Releases](../../releases) (Linux `amd64` / `arm64`, musl) або зберіть із джерел:

```sh
cargo build --release
# бінарник у target/release/tg2zola
```

Також потрібен бінарник [`zola`](https://www.getzola.org/documentation/getting-started/installation/),
щоб перетворити згенерований вміст на HTML.

## Використання

```sh
# Згенерувати сайт Zola (за першого запуску створює config + шаблони):
tg2zola --channel durov --site site --init-site

# Зібрати статичний HTML:
zola --root site build       # результат у site/public/

# (необовʼязково) Зробити перегляд БЕЗ вебсервера, прямо з диска:
tg2zola offline site/public  # потім відкрийте site/public/index.html через file://
```

Швидка локальна перевірка (одна сторінка, ~20 повідомлень):

```sh
tg2zola --channel <name> --site /tmp/site --init-site --max-pages 1
```

Усі параметри — у [`tg2zola.toml`](tg2zola.toml) (прапорці CLI перекривають його):

```sh
tg2zola --config tg2zola.toml
```

Запустіть `tg2zola --help` для повного списку прапорців.

## Як це влаштовано

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

Кожен допис стає бандлом сторінки Zola — `content/posts/<date>-<id>/index.md` з
TOML-фронтматтером і медіафайлами поряд. `config.toml` і вбудовані шаблони
детерміновано перегенеруються за кожного запуску, а `write_site` упорядковує
дерево: переписує весь Markdown, зберігає вже закешовані медіа й видаляє вилучені
дописи та застарілі файли.

## Автоматизація (GitHub Actions)

Включено два воркфлоу:

- **[`daily.yml`](.github/workflows/daily.yml)** — запускається раз на день (і на
  вимогу): відновити попередній сайт із гілки `blog` (кеш медіа) → спарсити +
  перегенерувати → `zola build` → **розгорнути на GitHub Pages** (опублікований
  результат) → закомітити оновлений сайт назад у гілку **`blog`**.
- **[`release.yml`](.github/workflows/release.yml)** — на кожен запушений тег
  `v*` крос-компілює статичні бінарники `amd64` + `arm64` (musl) і завантажує їх
  у GitHub Release.

Щоб увімкнути публікацію: у репозиторії **Settings → Pages → Build and deployment
→ Source: GitHub Actions**. Секрети не потрібні — усе працює на публічному
парсингу. Опублікований сайт — завжди **GitHub Pages**; гілка `blog` — це надійна
копія, що ніколи не впливає на те, що бачать відвідувачі.

### Перегенерувати зараз (не чекаючи щоденного запуску)

У `daily.yml` увімкнено `workflow_dispatch`, тож можна запустити свіжий парсинг +
збірку + розгортання на вимогу — виконуються рівно ті самі кроки, що й за
розкладом:

- **У браузері:** відкрийте **[Actions → «daily» → Run workflow](../../actions/workflows/daily.yml)**
  і натисніть зелену кнопку **Run workflow**. (Додайте бейдж статусу вище у свій
  README для доступу в один клік.)
- **З термінала:** `gh workflow run daily.yml` (GitHub CLI), потім
  `gh run watch`, щоб стежити за виконанням.

**Який канал?** Канал не закомічено, тож кожне розгортання задає свій. Задайте
**змінну** репозиторію `CHANNEL` (Settings → Secrets and variables → Actions →
Variables) з імʼям публічного каналу — це *змінна*, а не секрет, бо канал
публічний. (Або розкоментуйте `channel = "…"` у [`tg2zola.toml`](tg2zola.toml) у
своєму форку.) `THEME_REPO` теж працює як змінна.

### Гілка `blog` (архів + кеш)

Кожен запуск комітить згенерований сайт (Markdown + медіа + вбудовані шаблони —
усе, крім зібраного `public/` і будь-якої зовнішньої теми) у гілку `blog`,
лишаючи в `main` лише код. Вона виконує подвійну роль:

- **Кеш** — наступний запуск відновлює медіа з неї замість повторного
  завантаження.
- **Надійний архів** — повний, придатний до збірки сайт Zola, який можна
  склонувати й дзеркалити будь-де, щоб копія не була привʼязана до однієї
  платформи:

```sh
git clone -b blog https://github.com/<you>/<repo> my-archive
zola --root my-archive/site serve          # перегляд офлайн

git -C my-archive remote add gitlab git@gitlab.com:<you>/<repo>.git
git -C my-archive push gitlab blog          # дзеркалити деінде
```

Медіа комітяться як звичайні git-блоби; для дуже великих каналів розгляньте Git
LFS або періодичне стиснення історії.

### Теми

За замовчуванням використовується вбудована тема **справжній чорний** — без
зовнішніх залежностей, тож сайт завжди збирається. Щоб використати зовнішню
[тему Zola](https://www.getzola.org/themes/), задайте **змінну** репозиторію
`THEME_REPO` (Settings → Secrets and variables → Actions → Variables) з її git-URL
(https або ssh із deploy-ключем). Воркфлоу клонує її й збирає з нею — і **якщо
тема відсутня або її збірка падає, відбувається автоматичний відкіт на вбудовані
шаблони**, тож проблема з темою ніколи не лишить блог офлайн. Зважте, що зовнішні
теми розраховують на певну структуру контенту, тож не кожна тема підходить «з
коробки».

Зробити реліз:

```sh
git tag v0.1.0 && git push origin v0.1.0
```

## Конфігурація

Усе налаштовується через **змінну** репозиторію (Settings → Secrets and variables
→ Actions → **Variables**) для потоку GitHub Actions або через еквівалентний
прапорець CLI / ключ [`tg2zola.toml`](tg2zola.toml) під час локального запуску. Це
*змінні*, а не секрети — усе публічне.

| Змінна репо | Прапорець CLI | За замовчуванням | Що робить |
|---|---|---|---|
| `CHANNEL` | `--channel` | **обовʼязково** | Публічний канал для синхронізації |
| `TITLE` | `--title` | імʼя каналу | Заголовок блогу (шапка + `<title>`) |
| `ABOUT` | `--about` | опис + статистика + посилання на репозиторій | Власний HTML для тіла сторінки «Про проєкт» |
| `PAGES` | `--pages` | — | Додаткові сторінки: Markdown, кожен заголовок `# Title` починає нову сторінку + пункт меню |
| `POSTS_PER_PAGE` | `--posts-per-page` | `20` | Повних дописів на сторінці стрічки |
| `TAGS_FOOTER` | `--tags-footer` | вимк | `true` показує футер тегів під дописом (теги клікабельні в тілі в будь-якому разі) |
| `NEXT_PREV` | `--no-next-prev` | увімк | `false` ховає навігацію по дописах «Назад/Вперед» |
| `TELEGRAM_LINK` | `--no-telegram-link` | увімк | `false` ховає посилання «Відкрити в Telegram» під дописом |
| `RSS` | `--no-rss` | увімк | `false` вимикає RSS-стрічку за адресою `/rss.xml` (з автовиявленням) |
| `FEDIVERSE_CREATOR` | `--fediverse-creator` | — | Mastodon `@user@instance` → підпис `fediverse:creator` + посилання профілю `rel="me"` |
| `SEARCH_ENGINE` | `--search-engine` | `google` | Рядок пошуку в шапці: `google` (форма без JS) / `duckduckgo` / `yandex` / `bing` / `none` |
| `SEARCH_URL` | `--search-url` | — | Власний префікс URL пошуку; запит додається по Enter (перекриває рушій) |
| `TITLE_MAX_LEN` | `--title-max-len` | `200` | Макс. довжина заголовка допису (символи); скорочений заголовок зберігає повне перше речення в тілі |
| `FOOTER` | `--footer` | — | Вміст футера — звичайний текст, Markdown або HTML |
| `PAGES_HOST` | `--pages-host` | авто | Хост для ліміту розміру на сторінці «Про проєкт»: `github` / `gitlab` / `none` (визначається за URL) |
| `DATE_FORMAT` | `--date-format` | `%Y %B %d` | Формат strftime для показаних дат (наприклад, `2025 October 28`; `%Y` — лише рік) |
| `LANGUAGE` | `--language` | `en` | Мова інтерфейсу сайту (Новіші/Старіші/Теги/Про проєкт/…): `en`/`be`/`uk`/`ru`/`de`/`fr`/`zh`/`ja`/`pl`/`es`/`ko`/`ka` (грузинська). Вміст дописів лишається мовою каналу; дати локалізуються |
| `LINK_UNDERLINE` | `--link-underline` | вимк | `true` підкреслює посилання (за замовчуванням без підкреслення) |
| `YOUTUBE_FACADE` | `--youtube-facade` | вимк | `true` для превʼю YouTube з кліком-завантаженням без JS (за замовчуванням прямий iframe) |
| `GENIUS` | `--no-genius` | увімк | `false` пропускає розбір посилань genius.com (завантаження сторінки заради YouTube-відео + віджета тексту пісні) |
| `TAGS_TO_PAGES` | `--tags-to-pages` | — | Теги через кому, показані посиланнями `#tag` у верхньому меню (наприклад, `music, batumi, cooking`) |
| `BACKGROUND_DARK_COLOR` | `--background-dark-color` | `#000000` | Тло темного режиму (будь-який CSS-колір) |
| `BACKGROUND_LIGHT_COLOR` | `--background-light-color` | `#ffffff` | Тло світлого режиму |
| `CSS` | `--css` | — | Додатковий CSS, доданий до вбудованих стилів |
| `THEME_REPO` | `--theme` (імʼя) | вбудована чорна тема | Git-URL зовнішньої теми Zola (https/ssh); авто-відкіт за помилки |
| `REPO_URL` | `--repo-url` | репозиторій tg2zola | Посилання «Репозиторій вихідного коду» на «Про проєкт» (CI ставить ваш репозиторій) |

Сторінка **«Про проєкт»** показує **аватар** каналу (повного розміру), його опис і
статистику, **розмір на диску** — з розбивкою за типами і, на GitHub/GitLab Pages,
частку від ліміту опублікованого сайту (~1 ГБ) у хостингу (з посиланням на
документацію хостингу) — плюс посилання на репозиторій. У шапці аватар показано
мініатюрою та фавіконкою; хештеги клікабельні в тілах дописів і породжують
сторінки `/tags/<tag>/`.

Одна змінна `PAGES` може задавати кілька сторінок — кожен заголовок `# Title`
починає нову:

```
# Заголовок сторінки
Вміст сторінки в Markdown (може містити звичайний HTML).

# Інша сторінка
Ще вміст.
```

→ `/page-title/` і `/another-page/`, кожна з посиланням у меню. Без додаткових
залежностей — Zola і так рендерить Markdown.

## Fediverse / Mastodon

Кожна сторінка несе теги Open Graph + Twitter Card, тож посилання на допис
показується карткою (заголовок, опис, перше зображення допису) у Mastodon, Slack,
Discord, X тощо. Задайте `FEDIVERSE_CREATOR` рівним вашому хендлу `@user@instance`,
щоб також:

- додати підпис **`fediverse:creator`**, щоб Mastodon показував
  «*by @you@instance*» у превʼю посилання й посилався на ваш профіль;
- видати посилання **`rel="me"`** на ваш профіль, щоб ви могли додати цей сайт у
  метадані профілю Mastodon і отримати **підтверджену** (зелену) галочку.

**Чи можуть люди підписатися на блог *із* Mastodon?** Не напряму — статичний сайт
не може бути актором ActivityPub (для цього потрібен живий сервер з WebFinger +
ActivityPub). Два способи все ж дати людям підписатися:

- **RSS** — будь-хто може підписатися на `/rss.xml` у читачці вже сьогодні
  (багато користувачів Mastodon тримають читачку). Це вбудовано й увімкнено за
  замовчуванням.
- **Міст** — спрямуйте RSS-стрічку на міст RSS→ActivityPub, наприклад
  [rss-parrot](https://rss-parrot.net/) або [Bridgy Fed](https://fed.brid.gy/),
  які надають справжній `@handle`, на який користувачі Mastodon можуть
  **підписатися**, ретранслюючи нові дописи в їхню стрічку. Власний сервер не
  потрібен.

Отже: багаті превʼю + атрибуція автора + підтвердження профілю працюють «з
коробки»; справжня «підписка з Mastodon» — за один міст, що використовує
RSS-стрічку як вхід.

## Обмеження

Публічна вебверсія — це компроміс за **нульову автентифікацію**:

- **Реакції/вподобайки не віддаються** через `t.me/s/`. Натомість ми експортуємо
  **лічильники переглядів**. Модель даних лишає місце для додавання справжніх
  реакцій згодом через автентифікований API MTProto (крейт
  [`grammers`](https://codeberg.org/Lonami/grammers)), якщо вони вам знадобляться.
- **Великі відео не завантажуються** — вебверсія віддає для них лише постер і
  тривалість (короткі/автовідтворювані відео *завантажуються*). Архівація самого
  файлу теж потребувала б API MTProto.
- **Набори стікерів не дають посилання** — імʼя набору завантажується JavaScript-ом
  Telegram і відсутнє у спарсеному HTML; стікери зберігаються як звичайні
  картинки.
- **Музичні файли (аудіодокументи) не завантажуються** — їхнього URL немає у
  спарсеному HTML (є лише голосові, з прямими `.oga`-URL). Як великим відео й
  стікерам, їм потрібен API MTProto. Для вкладення, яке ми не можемо завантажити,
  ми зберігаємо його **імʼя файлу** (з позначкою *не збережено*), щоб ви знали, що
  воно існувало; допис, що складається *лише* з такого посилання (або з одного
  незавантажуваного файлу), пропускається, а не публікується порожнім.
- **YouTube** відтворюється через iframe `youtube.com` (тож перегляди йдуть в
  історію глядача) на опублікованому HTTPS-сайті; через `file://` iframe не
  завантажується (YouTube потрібен origin). `YOUTUBE_FACADE=true` міняє iframe на
  превʼю з кліком-завантаженням без JS, яке принаймні показує постер через
  `file://`.
- **Лише публічні канали** з увімкненою вебверсією.

## Ліцензія

[MIT](LICENSE) © Vitaly Zdanevich
