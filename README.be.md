# tg2zola

🌐 [English](README.md) · **Беларуская** · [Українська](README.uk.md) · [Русский](README.ru.md) · [Deutsch](README.de.md) · [Français](README.fr.md) · [中文](README.zh.md) · [日本語](README.ja.md) · [Polski](README.pl.md) · [Español](README.es.md) · [한국어](README.ko.md) · [ქართული](README.ka.md)

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

Рэзервовая копія **публічнага Telegram-канала** ў выглядзе **самадастатковага
статычнага сайта на [Zola](https://www.getzola.org/)**.

> **Перагенерацыя на запыт:** націсніце на бэйдж **daily build** вышэй → **Run
> workflow**, каб нанова спарсіць, сабраць і разгарнуць сайт, не чакаючы раскладу
> (падрабязнасці — у раздзеле **«Аўтаматызацыя»**).

Інструмент парсіць публічную вэб-версію (`https://t.me/s/<channel>`), сцягвае ўсе
медыя лакальна і нанова генеруе паўнавартасны блог на Zola пры кожным запуску.
**Не патрэбныя ні бот, ні токен, ні API Telegram** — чытаецца толькі публічная
вэб-старонка. У выніку **няма залежнасці ад Telegram**: медыя захоўваюцца
лакальна, няма ўбудаваных віджэтаў, а спасылкі на *уласныя* пасты канала
перапісваюцца ва ўнутраныя адносныя спасылкі — таму сайт працуе далей, нават калі
канал пазней выдаляць. Спасылкі, якія вы ставілі на іншыя сайты (у тым ліку іншыя
Telegram-каналы), захоўваюцца як звычайныя спасылкі. Гэта рэзервовая копія, а не
люстэрка.

Напісаны на Rust: адзін статычны бінарнік, зручна запускаць лакальна або ў CI.

## Што ўмее

- **Поўная гісторыя** — праходзіць канал у адваротным парадку па курсоры
  `?before=` вэб-версіі да самага першага паведамлення і **нанова генеруе ўсе
  старонкі** пры кожным запуску.
- **Самадастатковыя медыя** — сцягвае фота, відэа, аўдыя (`.ogg/.oga/.mp3`),
  дакументы і стыкеры ў бандл кожнага паста (які заадно служыць кэшам). Фота
  адрасуюцца па іх стабільным ідэнтыфікатары файла ў Telegram, таму праўка тэксту
  паста ніколі не перасцягвае медыя, а **замена** выявы (пост тады пазначаецца як
  *зменены*) сцягвае новы файл і выдаляе стары.
- **Тэма «сапраўдны чорны» па змаўчанні** — убудаваныя шаблоны са стылем `#000` у
  цёмным рэжыме праз `prefers-color-scheme` (беражліва да OLED), без знешніх
  залежнасцей. Знешнюю тэму можна падключыць паверх з гарантаваным адкатам (гл.
  раздзел **«Тэмы»**).
- **Разумная апрацоўка відэа** (па прыярытэце):
  1. прымацаванае відэа **+ спасылка на YouTube** → убудоўваецца YouTube, відэа
     адкідаецца;
  2. відэа, што сцягваецца напрамую → лакальнае `<video>`;
  3. інакш → захоўваецца **постар-кадр** + працягласць (публічная старонка не
     аддае файл; гл. **«Абмежаванні»**).
- **Фарматаванне** — тоўсты, курсіў, закрэсліванне, код/монашырынны блок, спасылкі
  і спойлеры канвертуюцца ў Markdown (зрухі UTF-16-сутнасцей Telegram
  апрацоўваюцца карэктна).
- **Хэштэгі → тэгі** — `#хэштэгі` становяцца тэрмінамі таксаноміі `tags` у Zola,
  таму вы бясплатна атрымліваеце старонкі тэгаў, а яны застаюцца тэкстам у пасце.
- **Згрупаваныя пасты** — альбомы аўтаматычна становяцца адным пастом; усплёскі
  паведамленняў, апублікаваных у адзін момант (напрыклад, пры перасылцы некалькіх
  адразу), аб'ядноўваюцца.
- **Саманавігацыя** — спасылка на іншае паведамленне ў *тым самым* канале
  становіцца адноснай спасылкай на гэты пост у блогу; спасылкі на іншыя каналы
  застаюцца знешнімі.
- **Уцягнутасць** — экспартуе **лічыльнікі праглядаў** для кожнага паста.
  (Рэакцыі/лайкі недаступныя на публічнай старонцы — гл. **«Абмежаванні»**.)
- **RSS-стужка** — стандартны `/rss.xml` з **усімі пастамі** і поўным змесцівам
  (поўная стужка, а не толькі свежыя запісы), аб'яўлены праз
  `<link rel="alternate">`, каб чыталкі знаходзілі яе аўтаматычна па адрасе сайта.
  Уключана па змаўчанні; адключаецца праз `RSS=false` / `--no-rss`.
- **Багатыя прэв'ю спасылак + Mastodon** — кожная старонка змяшчае тэгі Open Graph
  і Twitter Card (загаловак, апісанне, першая выява паста), таму спасылкі
  паказваюцца карткамі. Задайце `FEDIVERSE_CREATOR`, каб дадаць подпіс аўтара ў
  прэв'ю Mastodon і пацвердзіць сайт у сваім профілі (гл. раздзел **«Fediverse»**).
- **Без JavaScript па змаўчанні, гатовы да афлайну** — цёмны рэжым і спойлеры
  рэалізаваныя толькі на CSS, а радок пошуку Google па змаўчанні — звычайная
  `<form>` (без JS). Толькі не-Google пашукавік дадае адзін малюпасенькі ўбудаваны
  апрацоўшчык Enter. `tg2zola offline <тэчка-public>` перапісвае сабраны сайт на
  адносныя спасылкі **і** прыбірае скрыпт-рэдырэкт пагінацыі Zola, таму
  афлайн-копія адкрываецца проста з `file://` без аніякага JavaScript і без
  вэб-сервера.
- **Лакалізаваны інтэрфейс** — абалонка сайта (Навейшыя/Старэйшыя/Тэгі/Пра праект,
  радок пошуку, даты) паказваецца на любой з 12 моў праз `LANGUAGE` / `--language`
  (en/be/uk/ru/de/fr/zh/ja/pl/es/ko/ka), з лакалізаванымі назвамі месяцаў і дзён
  тыдня. Змесціва пастоў застаецца на мове канала.

## Усталяванне

Вазьміце статычны бінарнік для вашай архітэктуры са старонкі
[Releases](../../releases) (Linux `amd64` / `arm64`, musl) або сабярыце з зыходнікаў:

```sh
cargo build --release
# бінарнік у target/release/tg2zola
```

Таксама патрэбны бінарнік [`zola`](https://www.getzola.org/documentation/getting-started/installation/),
каб ператварыць згенераванае змесціва ў HTML.

## Выкарыстанне

```sh
# Згенераваць сайт Zola (пры першым запуску стварае config + шаблоны):
tg2zola --channel durov --site site --init-site

# Сабраць статычны HTML:
zola --root site build       # вынік у site/public/

# (неабавязкова) Зрабіць прагляд БЕЗ вэб-сервера, проста з дыска:
tg2zola offline site/public  # потым адкрыйце site/public/index.html праз file://
```

Хуткая лакальная праверка (адна старонка, ~20 паведамленняў):

```sh
tg2zola --channel <name> --site /tmp/site --init-site --max-pages 1
```

Усе параметры — у [`tg2zola.toml`](tg2zola.toml) (сцягі CLI перакрываюць яго):

```sh
tg2zola --config tg2zola.toml
```

Запусціце `tg2zola --help` для поўнага спіса сцягоў.

## Як гэта зладжана

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

Кожны пост становіцца бандлам старонкі Zola — `content/posts/<date>-<id>/index.md`
з TOML-фронтматарам і медыяфайламі побач. `config.toml` і ўбудаваныя шаблоны
дэтэрмінавана перагенеруюцца пры кожным запуску, а `write_site` упарадкоўвае
дрэва: перапісвае ўвесь Markdown, захоўвае ўжо закэшаваныя медыя і выдаляе
выдаленыя пасты і састарэлыя файлы.

## Аўтаматызацыя (GitHub Actions)

Уключаны два воркфлоў:

- **[`daily.yml`](.github/workflows/daily.yml)** — запускаецца раз на дзень (і на
  запыт): аднавіць папярэдні сайт з галіны `blog` (кэш медыя) → спарсіць +
  перагенераваць → `zola build` → **разгарнуць на GitHub Pages** (апублікаваны
  вынік) → закаміціць абноўлены сайт назад у галіну **`blog`**.
- **[`release.yml`](.github/workflows/release.yml)** — на кожны запушаны тэг `v*`
  крос-кампілюе статычныя бінарнікі `amd64` + `arm64` (musl) і загружае іх у
  GitHub Release.

Каб уключыць публікацыю: у рэпазіторыі **Settings → Pages → Build and deployment →
Source: GitHub Actions**. Сакрэты не патрэбныя — усё працуе на публічным парсінгу.
Апублікаваны сайт — заўсёды **GitHub Pages**; галіна `blog` — гэта надзейная копія,
якая ніколі не ўплывае на тое, што бачаць наведвальнікі.

### Перагенераваць зараз (не чакаючы штодзённага запуску)

У `daily.yml` уключаны `workflow_dispatch`, таму можна запусціць свежы парсінг +
зборку + разгортванне на запыт — выконваюцца роўна тыя самыя крокі, што і па
раскладзе:

- **У браўзеры:** адкрыйце **[Actions → «daily» → Run workflow](../../actions/workflows/daily.yml)**
  і націсніце зялёную кнопку **Run workflow**. (Дадайце бэйдж статусу вышэй у свой
  README для доступу ў адзін клік.)
- **З тэрмінала:** `gh workflow run daily.yml` (GitHub CLI), потым `gh run watch`,
  каб сачыць за выкананнем.

**Які канал?** Канал не закамічаны, таму кожнае разгортванне задае свой. Задайце
**зменную** рэпазіторыя `CHANNEL` (Settings → Secrets and variables → Actions →
Variables) з імем публічнага канала — гэта *зменная*, а не сакрэт, бо канал
публічны. (Або раскаментуйце `channel = "…"` у [`tg2zola.toml`](tg2zola.toml) у
сваім форку.) `THEME_REPO` таксама працуе як зменная.

### Галіна `blog` (архіў + кэш)

Кожны запуск камітуе згенераваны сайт (Markdown + медыя + убудаваныя шаблоны —
усё, акрамя сабранага `public/` і любой знешняй тэмы) у галіну `blog`, пакідаючы ў
`main` толькі код. Яна выконвае падвойную ролю:

- **Кэш** — наступны запуск аднаўляе медыя з яе замест паўторнага сцягвання.
- **Надзейны архіў** — поўны, прыдатны да зборкі сайт Zola, які можна
  скланіраваць і люстраваць дзе заўгодна, каб копія не была прывязана да адной
  платформы:

```sh
git clone -b blog https://github.com/<you>/<repo> my-archive
zola --root my-archive/site serve          # прагляд афлайн

git -C my-archive remote add gitlab git@gitlab.com:<you>/<repo>.git
git -C my-archive push gitlab blog          # люстраваць у іншае месца
```

Медыя камітуюцца як звычайныя git-блобы; для вельмі вялікіх каналаў разгледзьце
Git LFS або перыядычнае сцісканне гісторыі.

### Тэмы

Па змаўчанні выкарыстоўваецца ўбудаваная тэма **сапраўдны чорны** — без знешніх
залежнасцей, таму сайт заўсёды збіраецца. Каб скарыстаць знешнюю
[тэму Zola](https://www.getzola.org/themes/), задайце **зменную** рэпазіторыя
`THEME_REPO` (Settings → Secrets and variables → Actions → Variables) з яе git-URL
(https або ssh з deploy-ключом). Воркфлоў клануе яе і збірае з ёй — і **калі тэма
адсутнічае або яе зборка падае, адбываецца аўтаматычны адкат на ўбудаваныя
шаблоны**, таму праблема з тэмай ніколі не пакіне блог афлайн. Улічыце, што знешнія
тэмы разлічваюць на пэўную структуру кантэнту, таму не кожная тэма падыходзіць «з
скрыні».

Зрабіць рэліз:

```sh
git tag v0.1.0 && git push origin v0.1.0
```

## Канфігурацыя

Усё наладжваецца праз **зменную** рэпазіторыя (Settings → Secrets and variables →
Actions → **Variables**) для патоку GitHub Actions або праз эквівалентны сцяг CLI /
ключ [`tg2zola.toml`](tg2zola.toml) пры лакальным запуску. Гэта *зменныя*, а не
сакрэты — усё публічна.

| Зменная рэпа | Сцяг CLI | Па змаўчанні | Што робіць |
|---|---|---|---|
| `CHANNEL` | `--channel` | **абавязкова** | Публічны канал для сінхранізацыі |
| `TITLE` | `--title` | імя канала | Загаловак блога (шапка + `<title>`) |
| `ABOUT` | `--about` | апісанне + статыстыка + спасылка на рэпазіторый | Уласны HTML для цела старонкі «Пра праект» |
| `PAGES` | `--pages` | — | Дадатковыя старонкі: Markdown, кожны загаловак `# Title` пачынае новую старонку + пункт меню |
| `POSTS_PER_PAGE` | `--posts-per-page` | `20` | Поўных пастоў на старонцы стужкі |
| `TAGS_FOOTER` | `--tags-footer` | выкл | `true` паказвае футар тэгаў пад пастом (тэгі клікабельныя ў целе ў любым выпадку) |
| `NEXT_PREV` | `--no-next-prev` | укл | `false` хавае навігацыю па пастах «Назад/Наперад» |
| `TELEGRAM_LINK` | `--no-telegram-link` | укл | `false` хавае спасылку «Адкрыць у Telegram» пад пастом |
| `RSS` | `--no-rss` | укл | `false` адключае RSS-стужку па адрасе `/rss.xml` (з аўтавыяўленнем) |
| `FEDIVERSE_CREATOR` | `--fediverse-creator` | — | Mastodon `@user@instance` → подпіс `fediverse:creator` + спасылка профілю `rel="me"` |
| `SEARCH_ENGINE` | `--search-engine` | `google` | Радок пошуку ў шапцы: `google` (форма без JS) / `duckduckgo` / `yandex` / `bing` / `none` |
| `SEARCH_URL` | `--search-url` | — | Уласны прэфікс URL пошуку; запыт дадаецца па Enter (перакрывае рухавік) |
| `TITLE_MAX_LEN` | `--title-max-len` | `200` | Макс. даўжыня загалоўка паста (сімвалы); скарочаны загаловак захоўвае поўны першы сказ у целе |
| `FOOTER` | `--footer` | — | Змесціва футара — звычайны тэкст, Markdown або HTML |
| `PAGES_HOST` | `--pages-host` | аўта | Хост для ліміту памеру на старонцы «Пра праект»: `github` / `gitlab` / `none` (вызначаецца па URL) |
| `DATE_FORMAT` | `--date-format` | `%Y %B %d` | Фармат strftime для паказаных дат (напрыклад, `2025 October 28`; `%Y` — толькі год) |
| `LANGUAGE` | `--language` | `en` | Мова інтэрфейсу сайта (Навейшыя/Старэйшыя/Тэгі/Пра праект/…): `en`/`be`/`uk`/`ru`/`de`/`fr`/`zh`/`ja`/`pl`/`es`/`ko`/`ka` (грузінская). Змесціва пастоў застаецца на мове канала; даты лакалізуюцца |
| `LINK_UNDERLINE` | `--link-underline` | выкл | `true` падкрэслівае спасылкі (па змаўчанні без падкрэслівання) |
| `YOUTUBE_FACADE` | `--youtube-facade` | выкл | `true` для прэв'ю YouTube з клікам-сцягваннем без JS (па змаўчанні прамы iframe) |
| `GENIUS` | `--no-genius` | укл | `false` прапускае разбор спасылак genius.com (сцягванне старонкі дзеля YouTube-відэа + віджэта тэксту песні) |
| `TAGS_TO_PAGES` | `--tags-to-pages` | — | Тэгі праз коску, паказаныя спасылкамі `#tag` у верхнім меню (напрыклад, `music, batumi, cooking`) |
| `BACKGROUND_DARK_COLOR` | `--background-dark-color` | `#000000` | Фон цёмнага рэжыму (любы CSS-колер) |
| `BACKGROUND_LIGHT_COLOR` | `--background-light-color` | `#ffffff` | Фон светлага рэжыму |
| `CSS` | `--css` | — | Дадатковы CSS, дададзены да ўбудаваных стыляў |
| `THEME_REPO` | `--theme` (імя) | убудаваная чорная тэма | Git-URL знешняй тэмы Zola (https/ssh); аўта-адкат пры памылцы |
| `REPO_URL` | `--repo-url` | рэпазіторый tg2zola | Спасылка «Рэпазіторый зыходнага кода» на «Пра праект» (CI ставіць ваш рэпазіторый) |

Старонка **«Пра праект»** паказвае **аватар** канала (поўнага памеру), яго апісанне
і статыстыку, **памер на дыску** — з разбіўкай па тыпах і, на GitHub/GitLab Pages,
долю ад ліміту апублікаванага сайта (~1 ГБ) у хостынгу (са спасылкай на
дакументацыю хостынгу) — плюс спасылку на рэпазіторый. У шапцы аватар паказаны
мініяцюрай і фавіконкай; хэштэгі клікабельныя ў целах пастоў і параджаюць старонкі
`/tags/<tag>/`.

Адна зменная `PAGES` можа задаваць некалькі старонак — кожны загаловак `# Title`
пачынае новую:

```
# Загаловак старонкі
Змесціва старонкі ў Markdown (можа ўтрымліваць звычайны HTML).

# Іншая старонка
Яшчэ змесціва.
```

→ `/page-title/` і `/another-page/`, кожная са спасылкай у меню. Без дадатковых
залежнасцей — Zola і так рэндэрыць Markdown.

## Fediverse / Mastodon

Кожная старонка нясе тэгі Open Graph + Twitter Card, таму спасылка на пост
паказваецца карткай (загаловак, апісанне, першая выява паста) у Mastodon, Slack,
Discord, X і г. д. Задайце `FEDIVERSE_CREATOR` роўным вашаму хэндлу `@user@instance`,
каб таксама:

- дадаць подпіс **`fediverse:creator`**, каб Mastodon паказваў «*by @you@instance*»
  у прэв'ю спасылкі і спасылаўся на ваш профіль;
- выдаць спасылку **`rel="me"`** на ваш профіль, каб вы маглі дадаць гэты сайт у
  метаданыя профілю Mastodon і атрымаць **пацверджаную** (зялёную) галачку.

**Ці могуць людзі падпісацца на блог *з* Mastodon?** Не напрамую — статычны сайт не
можа быць акторам ActivityPub (для гэтага патрэбны жывы сервер з WebFinger +
ActivityPub). Два спосабы ўсё ж даць людзям падпісацца:

- **RSS** — кожны можа падпісацца на `/rss.xml` у чыталцы ўжо сёння (многія
  карыстальнікі Mastodon трымаюць чыталку). Гэта ўбудавана і ўключана па змаўчанні.
- **Мост** — накіруйце RSS-стужку на мост RSS→ActivityPub, напрыклад
  [rss-parrot](https://rss-parrot.net/) або [Bridgy Fed](https://fed.brid.gy/), якія
  даюць сапраўдны `@handle`, на які карыстальнікі Mastodon могуць **падпісацца**,
  рэтранслюючы новыя пасты ў іх стужку. Уласны сервер не патрэбны.

Такім чынам: багатыя прэв'ю + атрыбуцыя аўтара + пацверджанне профілю працуюць «з
скрыні»; сапраўдная «падпіска з Mastodon» — за адзін мост, што выкарыстоўвае
RSS-стужку як уваход.

## Абмежаванні

Публічная вэб-версія — гэта кампраміс за **нулявую аўтэнтыфікацыю**:

- **Рэакцыі/лайкі не аддаюцца** праз `t.me/s/`. Замест гэтага мы экспартуем
  **лічыльнікі праглядаў**. Мадэль даных пакідае месца для дадання сапраўдных
  рэакцый пазней праз аўтэнтыфікаваны API MTProto (крэйт
  [`grammers`](https://codeberg.org/Lonami/grammers)), калі яны вам спатрэбяцца.
- **Вялікія відэа не сцягваюцца** — вэб-версія аддае для іх толькі постар і
  працягласць (кароткія/аўтапрайграваныя відэа *сцягваюцца*). Архівацыя самога
  файла таксама патрабавала б API MTProto.
- **Наборы стыкераў не даюць спасылку** — імя набору загружаецца JavaScript-ам
  Telegram і адсутнічае ў спарсеным HTML; стыкеры захоўваюцца як звычайныя выявы.
- **Музычныя файлы (аўдыядакументы) не сцягваюцца** — іх URL няма ў спарсеным HTML
  (ёсць толькі галасавыя, з прамымі `.oga`-URL). Як вялікім відэа і стыкерам, ім
  патрэбны API MTProto. Для ўкладання, якое мы не можам сцягнуць, мы захоўваем яго
  **імя файла** (з пазнакай *не захавана*), каб вы ведалі, што яно існавала; пост,
  які складаецца *толькі* з такой спасылкі (або з аднаго несцягвальнага файла),
  прапускаецца, а не публікуецца пустым.
- **YouTube** прайграецца праз iframe `youtube.com` (таму прагляды ідуць у гісторыю
  гледача) на апублікаваным HTTPS-сайце; праз `file://` iframe не загружаецца
  (YouTube патрэбны origin). `YOUTUBE_FACADE=true` мяняе iframe на прэв'ю з
  клікам-сцягваннем без JS, якое прынамсі паказвае постар праз `file://`.
- **Толькі публічныя каналы** з уключанай вэб-версіяй.

## Ліцэнзія

[MIT](LICENSE) © Vitaly Zdanevich
