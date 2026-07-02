# tg2zola

🌐 [English](README.md) · [Беларуская](README.be.md) · [Українська](README.uk.md) · [Русский](README.ru.md) · [Deutsch](README.de.md) · [Français](README.fr.md) · [中文](README.zh.md) · [日本語](README.ja.md) · [Polski](README.pl.md) · [Español](README.es.md) · [한국어](README.ko.md) · **ქართული** · [हिन्दी](README.hi.md)

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

შექმენით **საჯარო Telegram-არხის** სარეზერვო ასლი **თვითკმარი
[Zola](https://www.getzola.org/) სტატიკური საიტის** სახით.

> **მოთხოვნით ხელახლა გენერაცია:** დააწკაპუნეთ ზემოთ მდებარე **daily build** ნიშანს
> → **Run workflow**, რომ ხელახლა ჩამოქაჩოთ, ააწყოთ და განათავსოთ განრიგის ლოდინის
> გარეშე (დეტალები **„ავტომატიზაცია"** განყოფილებაში).

ხელსაწყო კითხულობს საჯარო ვებ-გადახედვას (`https://t.me/s/<channel>`), ჩამოტვირთავს
ყველა მედიას ლოკალურად და ყოველ გაშვებაზე თავიდან აგენერირებს სრულ Zola-ბლოგს.
**Telegram-ის ბოტი, ტოკენი ან API საჭირო არ არის** — იკითხება მხოლოდ საჯარო
ვებგვერდი. შედეგს **არ აქვს Telegram-ზე დამოკიდებულება**: მედია ლოკალურია, არ არის
ჩაშენებები, ხოლო ბმულები არხის *საკუთარ* პოსტებზე გადაიწერება შიდა ფარდობით
ბმულებად — ამიტომ საიტი მუშაობას აგრძელებს, თუნდაც არხი მოგვიანებით წაიშალოს.
ბმულები, რომლებიც სხვა საიტებზე დაწერეთ (მათ შორის სხვა Telegram-არხებზე), რჩება
ჩვეულებრივ ბმულებად. ეს არის სარეზერვო ასლი და არა სარკე.

დაწერილია Rust-ზე: ერთი სტატიკური ბინარი, მარტივი გასაშვები ლოკალურად ან CI-ში.

## რას აკეთებს

- **სრული ისტორია** — გადახედვის `?before=` კურსორით არხს უკან გადაუყვება პირველ
  შეტყობინებამდე და ყოველ გაშვებაზე **თავიდან აგენერირებს ყველა გვერდს**.
- **თვითკმარი მედია** — ჩამოტვირთავს ფოტოებს, ვიდეოებს, აუდიოს (`.ogg/.oga/.mp3`),
  დოკუმენტებსა და სტიკერებს თითოეული პოსტის ბანდლში (რომელიც ამავდროულად ქეშია).
  ფოტოები მისამართდება მათი მდგრადი Telegram-ფაილის id-ით, ამიტომ პოსტის ტექსტის
  რედაქტირება მედიას ხელახლა არასოდეს ჩამოტვირთავს, ხოლო სურათის **ჩანაცვლება**
  (პოსტი მაშინ აღინიშნება როგორც *რედაქტირებული*) ახალ ფაილს იღებს და ძველს შლის.
- **ნაგულისხმევი „ნამდვილი შავი" თემა** — ჩაშენებული შაბლონები მუქ რეჟიმში
  `prefers-color-scheme`-ით `#000`-ად (OLED-ისთვის სასიკეთო), გარე
  დამოკიდებულების გარეშე. გარე თემის დადება შესაძლებელია გარანტირებული
  დაბრუნებით (იხ. განყოფილება **„თემები"**).
- **ჭკვიანი ვიდეო-დამუშავება** (პრიორიტეტის მიხედვით):
  1. მიმაგრებული ვიდეო **+ YouTube-ბმული** → ჩაშენდება YouTube, ვიდეო უარიყოფა;
  2. პირდაპირ ჩამოსატვირთი ვიდეო → ლოკალური `<video>`;
  3. სხვა შემთხვევაში → ინახება **პოსტერ-კადრი** + ხანგრძლივობა (საჯარო გვერდი
     ფაილს არ გასცემს; იხ. **„შეზღუდვები"**).
- **ფორმატირება** — მსხვილი, დახრილი, ხაზგადასმული, კოდი/წინასწარ ფორმატირებული,
  ბმულები და სპოილერები გარდაიქმნება Markdown-ად (Telegram-ის UTF-16 ერთეულების
  წანაცვლებები სწორად მუშავდება).
- **ჰეშთეგები → თეგები** — `#ჰეშთეგები` ხდება Zola-ს `tags` ტაქსონომიის ტერმინები,
  ასე უფასოდ იღებთ თეგების გვერდებს, ხოლო ისინი პოსტში ტექსტად რჩება.
- **დაჯგუფებული პოსტები** — ალბომები ავტომატურად ერთ პოსტად იქცევა; ერთ მომენტში
  გამოქვეყნებული შეტყობინებების სერია (მაგ. რამდენიმეს ერთად გადაგზავნა) ერთიანდება.
- **თვით-ნავიგაცია** — ბმული *იმავე* არხის სხვა შეტყობინებაზე იქცევა ფარდობით
  ბმულად ბლოგში ამ პოსტზე; ბმულები სხვა არხებზე გარედ რჩება.
- **ჩართულობა** — ექსპორტს უკეთებს თითოეული პოსტის **ნახვების რაოდენობას**.
  (რეაქციები/მოწონებები საჯარო გვერდზე ხელმისაწვდომი არ არის — იხ. **„შეზღუდვები"**.)
- **RSS-ფიდი** — სტანდარტული `/rss.xml` **ყველა პოსტით** და სრული შიგთავსით (სრული
  ფიდი და არა მხოლოდ ბოლო ჩანაწერები), გამოცხადებული `<link rel="alternate">`-ით,
  რომ ფიდ-წამკითხველებმა იპოვონ ის ავტომატურად საიტის მისამართიდან. ნაგულისხმევად
  ჩართულია; ითიშება `RSS=false` / `--no-rss`-ით.
- **მდიდარი ბმულის გადახედვები + Mastodon** — ყოველი გვერდი გასცემს Open Graph-ისა
  და Twitter Card-ის თეგებს (სათაური, აღწერა, პოსტის პირველი სურათი), ასე
  გაზიარებული ბმულები ბარათებად ჩანს. დააყენეთ `FEDIVERSE_CREATOR`, რომ Mastodon-ის
  გადახედვებში ავტორის ხელმოწერა დაამატოთ და საიტი თქვენს პროფილში დაადასტუროთ (იხ.
  განყოფილება **„Fediverse"**).
- **ნაგულისხმევად JavaScript-ის გარეშე, ოფლაინისთვის მზად** — მუქი რეჟიმი და
  სპოილერები მხოლოდ CSS-ია, ხოლო ნაგულისხმევი Google-ის ძებნის ველი ჩვეულებრივი
  `<form>`-ია (JS-ის გარეშე). მხოლოდ არა-Google საძიებო სისტემა ამატებს ერთ პატარა
  ჩაშენებულ Enter-დამმუშავებელს. `tg2zola offline <public-საქაღალდე>` აწყობილ საიტს
  ფარდობით ბმულებად გადაწერს **და** აშორებს Zola-ს გვერდების გადამისამართების
  სკრიპტს, ასე რომ ოფლაინ-ასლი იხსნება პირდაპირ `file://`-დან, ყოველგვარი
  JavaScript-ისა და ვებ-სერვერის გარეშე.
- **ლოკალიზებული ინტერფეისი** — საიტის გარსი (უფრო ახალი/უფრო ძველი/ტეგები/შესახებ,
  ძებნის ველი, თარიღები) ჩანს 12 ენიდან ერთ-ერთზე `LANGUAGE` / `--language`-ით
  (en/be/uk/ru/de/fr/zh/ja/pl/es/ko/ka), თვეებისა და კვირის დღეების ლოკალიზებული
  სახელებით. პოსტების შიგთავსი არხის საკუთარ ენაზე რჩება.

## ინსტალაცია

აიღეთ თქვენი არქიტექტურისთვის სტატიკური ბინარი [Releases](../../releases) გვერდიდან
(Linux `amd64` / `arm64`, musl) ან ააწყვეთ წყაროდან:

```sh
cargo build --release
# ბინარი მდებარეობს target/release/tg2zola
```

ასევე გჭირდებათ [`zola`](https://www.getzola.org/documentation/getting-started/installation/)
ბინარი, რომ გენერირებული შიგთავსი HTML-ად აქციოთ.

## გამოყენება

```sh
# Zola-საიტის გენერაცია (პირველ გაშვებაზე ქმნის config-სა + შაბლონებს):
tg2zola --channel durov --site site --init-site

# სტატიკური HTML-ის აწყობა:
zola --root site build       # შედეგი site/public/-ში

# (არჩევით) ვებ-სერვერის გარეშე, პირდაპირ დისკიდან სანახავად:
tg2zola offline site/public  # შემდეგ გახსენით site/public/index.html file://-ით
```

სწრაფი ლოკალური ტესტი (ერთი გვერდი, ~20 შეტყობინება):

```sh
tg2zola --channel <name> --site /tmp/site --init-site --max-pages 1
```

ყველა პარამეტრი მდებარეობს [`tg2zola.toml`](tg2zola.toml)-ში (CLI-ალმები მათ
გადაფარავს):

```sh
tg2zola --config tg2zola.toml
```

გაუშვით `tg2zola --help` ალმების სრული სიისთვის.

## როგორ არის აწყობილი

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

თითოეული პოსტი ხდება Zola-ს გვერდის ბანდლი — `content/posts/<date>-<id>/index.md`
TOML front matter-ით და მედია-ფაილებით გვერდით. `config.toml` და ჩაშენებული
შაბლონები ყოველ გაშვებაზე დეტერმინისტულად თავიდან გენერირდება, ხოლო `write_site`
ხეს აწესრიგებს: გადაწერს მთელ Markdown-ს, ინახავს უკვე ქეშირებულ მედიას და შლის
წაშლილ პოსტებსა და მოძველებულ ფაილებს.

## ავტომატიზაცია (GitHub Actions)

შედის ორი workflow:

- **[`daily.yml`](.github/workflows/daily.yml)** — ეშვება დღეში ერთხელ (და
  მოთხოვნით): აღადგენს წინა საიტს `blog` ტოტიდან (მედია-ქეში) → ჩამოქაჩვა +
  ხელახალი გენერაცია → `zola build` → **განთავსება GitHub Pages-ზე** (გამოქვეყნებული
  შედეგი) → განახლებული საიტის უკან კომიტი **`blog` ტოტში**.
- **[`release.yml`](.github/workflows/release.yml)** — ყოველ უბიძგებულ `v*` თეგზე
  ჯვარედინად აკომპილებს სტატიკურ `amd64` + `arm64` (musl) ბინარებს და ტვირთავს მათ
  GitHub Release-ში.

გამოქვეყნების ჩასართავად: საცავში **Settings → Pages → Build and deployment →
Source: GitHub Actions**. საიდუმლოები საჭირო არ არის — ყველაფერი საჯარო
ჩამოქაჩვაზეა. გამოქვეყნებული საიტი ყოველთვის **GitHub Pages**-ია; `blog` ტოტი
მდგრადი ასლია და არასოდეს ახდენს გავლენას იმაზე, რასაც სტუმრები ხედავენ.

### ახლავე ხელახლა გენერაცია (დღიური გაშვების ლოდინის გარეშე)

`daily.yml`-ს ჩართული აქვს `workflow_dispatch`, ამიტომ შეგიძლიათ მოთხოვნით
გაუშვათ ახალი ჩამოქაჩვა + აწყობა + ხელახალი განთავსება — ის ასრულებს ზუსტად იმავე
ნაბიჯებს, რასაც დაგეგმილი გაშვება:

- **ბრაუზერში:** გახსენით **[Actions → „daily" → Run workflow](../../actions/workflows/daily.yml)**
  და დააწკაპუნეთ მწვანე ღილაკს **Run workflow**. (დაამატეთ ზემოთ მდებარე სტატუსის
  ნიშანი თქვენს README-ში ერთ დაწკაპუნებით წვდომისთვის.)
- **ტერმინალიდან:** `gh workflow run daily.yml` (GitHub CLI), შემდეგ
  `gh run watch` თვალის სადევნებლად.

**რომელი არხი?** არხი არ არის დაკომიტებული, ამიტომ ყოველი განთავსება საკუთარს
აყენებს. დააყენეთ საცავის **ცვლადი** `CHANNEL` (Settings → Secrets and variables →
Actions → Variables) საჯარო არხის სახელით — ეს *ცვლადია*, და არა საიდუმლო, რადგან
არხი საჯაროა. (ან განააქტიურეთ `channel = "…"` თქვენი ფორკის
[`tg2zola.toml`](tg2zola.toml)-ში.) `THEME_REPO` ასევე მუშაობს ცვლადად.

### `blog` ტოტი (არქივი + ქეში)

ყოველი გაშვება გენერირებულ საიტს (Markdown + მედია + ჩაშენებული შაბლონები —
ყველაფერი, აწყობილი `public/`-ისა და ნებისმიერი გარე თემის გარდა) აკომიტებს `blog`
ტოტში, `main`-ში მხოლოდ კოდს ტოვებს. მას ორმაგი დანიშნულება აქვს:

- **ქეში** — შემდეგი გაშვება მისგან აღადგენს მედიას ხელახალი ჩამოტვირთვის ნაცვლად.
- **მდგრადი არქივი** — სრული, ასაწყობი Zola-საიტი, რომელიც შეგიძლიათ დააკლონოთ და
  დაასარკოთ სადმე, რომ ასლი ერთ პლატფორმაზე არ იყოს მიბმული:

```sh
git clone -b blog https://github.com/<you>/<repo> my-archive
zola --root my-archive/site serve          # ოფლაინ დათვალიერება

git -C my-archive remote add gitlab git@gitlab.com:<you>/<repo>.git
git -C my-archive push gitlab blog          # სხვაგან დასარკვა
```

მედია იკომიტება ჩვეულებრივ git-ბლობებად; ძალიან დიდი არხებისთვის განიხილეთ Git LFS
ან ისტორიის დროდადრო შეკუმშვა.

### თემები

ნაგულისხმევია ჩაშენებული **ნამდვილი შავი** თემა — ნულოვანი გარე დამოკიდებულება,
ამიტომ საიტი ყოველთვის იწყობა. გარე [Zola-თემის](https://www.getzola.org/themes/)
გამოსაყენებლად დააყენეთ საცავის **ცვლადი** `THEME_REPO` (Settings → Secrets and
variables → Actions → Variables) მისი git-URL-ით (https, ან ssh deploy-გასაღებით).
workflow მას აკლონებს და მასთან ერთად აწყობს — და **თუ თემა აკლია ან მისი აწყობა
ჩავარდა, ავტომატურად ბრუნდება ჩაშენებულ შაბლონებზე**, ასე რომ თემის პრობლემამ ვერ
შეძლებს ბლოგი ოფლაინში გადაიყვანოს. გაითვალისწინეთ, რომ გარე თემები შიგთავსის
განსაზღვრულ განლაგებას მოელის, ამიტომ ყველა თემა მზამზარეულად თავსებადი არ არის.

რელიზის გამოშვება:

```sh
git tag v0.1.0 && git push origin v0.1.0
```

## კონფიგურაცია

ყველაფერი კონფიგურირდება საცავის **ცვლადით** (Settings → Secrets and variables →
Actions → **Variables**) GitHub Actions-ის ნაკადისთვის, ან ეკვივალენტური
CLI-ალმით / [`tg2zola.toml`](tg2zola.toml)-ის გასაღებით ლოკალურად გაშვებისას. ეს
*ცვლადებია*, და არა საიდუმლოები — ყველაფერი საჯაროა.

| საცავის ცვლადი | CLI-ალამი | ნაგულისხმევი | რას აკეთებს |
|---|---|---|---|
| `CHANNEL` | `--channel` | **სავალდებულო** | სასინქრონიზებელი საჯარო არხი |
| `TITLE` | `--title` | არხის სახელი | ბლოგის სათაური (თავსართი + `<title>`) |
| `ABOUT` | `--about` | აღწერა + სტატისტიკა + საცავის ბმული | „შესახებ" გვერდის ტანის საკუთარი HTML |
| `PAGES` | `--pages` | — | დამატებითი გვერდები: Markdown, თითოეული `# Title` სათაური იწყებს ახალ გვერდს + ნავიგაციის ჩანაწერს |
| `POSTS_PER_PAGE` | `--posts-per-page` | `20` | სრული პოსტი თითო გვერდზე მთავარ ნაკადში |
| `TAGS_FOOTER` | `--tags-footer` | გამორთ. | `true` აჩვენებს პოსტის თეგების ქვედა კოლონტიტულს (თეგები ისედაც დაწკაპუნებადია ტანში) |
| `NEXT_PREV` | `--no-next-prev` | ჩართ. | `false` მალავს წინა/შემდეგი პოსტის ნავიგაციას |
| `TELEGRAM_LINK` | `--no-telegram-link` | ჩართ. | `false` მალავს პოსტის „Telegram-ში ნახვა" ბმულს |
| `RSS` | `--no-rss` | ჩართ. | `false` თიშავს RSS-ფიდს მისამართზე `/rss.xml` (ავტო-აღმოჩენით) |
| `FEDIVERSE_CREATOR` | `--fediverse-creator` | — | Mastodon `@user@instance` → `fediverse:creator` ხელმოწერა + `rel="me"` პროფილის ბმული |
| `SEARCH_ENGINE` | `--search-engine` | `google` | თავსართის ძებნის ველი: `google` (JS-ის გარეშე ფორმა) / `duckduckgo` / `yandex` / `bing` / `none` |
| `SEARCH_URL` | `--search-url` | — | საკუთარი ძებნის URL-პრეფიქსი; მოთხოვნა ემატება Enter-ზე (გადაფარავს ძრავას) |
| `TITLE_MAX_LEN` | `--title-max-len` | `200` | პოსტის სათაურის მაქს. სიგრძე (სიმბოლო); შეკვეცილი სათაური ტანში ინახავს სრულ პირველ წინადადებას |
| `FOOTER` | `--footer` | — | ქვედა კოლონტიტულის შიგთავსი — უბრალო ტექსტი, Markdown ან HTML |
| `PAGES_HOST` | `--pages-host` | ავტო | „შესახებ" გვერდის ზომის ლიმიტის ჰოსტი: `github` / `gitlab` / `none` (URL-დან აღმოჩენილი) |
| `DATE_FORMAT` | `--date-format` | `%Y %B %d` | ნაჩვენები თარიღების strftime-ფორმატი (მაგ. `2025 October 28`; `%Y` მხოლოდ წელი) |
| `LANGUAGE` | `--language` | `en` | საიტის ინტერფეისის ენა (უფრო ახალი/უფრო ძველი/ტეგები/შესახებ/…): `en`/`be`/`uk`/`ru`/`de`/`fr`/`zh`/`ja`/`pl`/`es`/`ko`/`ka` (ქართული). პოსტების შიგთავსი არხის ენაზე რჩება; თარიღები ლოკალიზდება |
| `LINK_UNDERLINE` | `--link-underline` | გამორთ. | `true` ხაზს უსვამს ბმულებს (ნაგულისხმევად: ხაზგასმის გარეშე) |
| `YOUTUBE_FACADE` | `--youtube-facade` | გამორთ. | `true` JS-ის გარეშე დაწკაპუნებით ჩასატვირთი YouTube-მინიატურისთვის (ნაგულისხმევად: პირდაპირი iframe) |
| `GENIUS` | `--no-genius` | ჩართ. | `false` ტოვებს genius.com-ბმულების დამუშავებას (იღებს გვერდს მისი YouTube-ვიდეოსა + ტექსტის ვიჯეტისთვის) |
| `TAGS_TO_PAGES` | `--tags-to-pages` | — | მძიმით გამოყოფილი თეგები, ნაჩვენები `#tag` ბმულებად ზედა ნავიგაციაში (მაგ. `music, batumi, cooking`) |
| `BACKGROUND_DARK_COLOR` | `--background-dark-color` | `#000000` | მუქი რეჟიმის ფონი (ნებისმიერი CSS-ფერი) |
| `BACKGROUND_LIGHT_COLOR` | `--background-light-color` | `#ffffff` | ღია რეჟიმის ფონი |
| `CSS` | `--css` | — | დამატებითი CSS, ჩაშენებულ სტილებზე მიმაგრებული |
| `THEME_REPO` | `--theme` (სახელი) | ჩაშენებული შავი თემა | გარე Zola-თემის git-URL (https/ssh); ჩავარდნისას ავტომატური დაბრუნება |
| `REPO_URL` | `--repo-url` | tg2zola-ს საცავი | „წყაროს კოდის საცავის" ბმული „შესახებ"-ზე (CI ავტომატურად აყენებს თქვენს საცავს) |

**„შესახებ"** გვერდი აჩვენებს არხის **ავატარს** (სრული ზომით), მის აღწერასა და
სტატისტიკას, **ზომას დისკზე** — ტიპების მიხედვით დაყოფითა და, GitHub/GitLab
Pages-ზე, ჰოსტის გამოქვეყნებული საიტის ~1 GB ლიმიტში წილს (ჰოსტის დოკუმენტაციაზე
ბმულით) — პლუს საცავის ბმულს. თავსართი ავატარს აჩვენებს მინიატურადა და favicon-ად;
ჰეშთეგები დაწკაპუნებადია პოსტების ტანში და ქმნის `/tags/<tag>/` გვერდებს.

ერთ `PAGES` ცვლადს შეუძლია რამდენიმე გვერდი განსაზღვროს — თითოეული `# Title`
სათაური იწყებს ახალს:

```
# გვერდის სათაური
გვერდის შიგთავსი Markdown-ში (შეიძლება შეიცავდეს დაუმუშავებელ HTML-ს).

# სხვა გვერდი
მეტი შიგთავსი.
```

→ `/page-title/` და `/another-page/`, თითოეული ნავიგაციაში დაბმული. დამატებითი
დამოკიდებულების გარეშე — Zola ისედაც არენდერებს Markdown-ს.

## Fediverse / Mastodon

ყოველ გვერდს აქვს Open Graph + Twitter Card თეგები, ამიტომ გაზიარებული პოსტის ბმული
ბარათად ჩანს (სათაური, აღწერა, პოსტის პირველი სურათი) Mastodon-ში, Slack-ში,
Discord-ში, X-ში და ა.შ. დააყენეთ `FEDIVERSE_CREATOR` თქვენს `@user@instance`
სახელით, რომ ასევე:

- დაამატოთ **`fediverse:creator`** ხელმოწერა, რომ Mastodon-მა ბმულის გადახედვაში
  აჩვენოს „*by @you@instance*" და თქვენს პროფილზე დააბმულოს;
- გასცეთ **`rel="me"`** ბმული თქვენს პროფილზე, რომ ეს საიტი დაამატოთ თქვენი
  Mastodon-პროფილის მეტამონაცემებში და მიიღოთ **დადასტურებული** (მწვანე) ნიშანი.

**შეუძლიათ ხალხს ბლოგზე გამოწერა Mastodon-*იდან*?** პირდაპირ არა — სტატიკური საიტი
ვერ იქნება ActivityPub-აქტორი (ამას სჭირდება ცოცხალი სერვერი WebFinger +
ActivityPub-ით). ორი გზა, რომ მაინც მისცეთ გამოწერის საშუალება:

- **RSS** — ნებისმიერს შეუძლია `/rss.xml`-ის გამოწერა ფიდ-წამკითხველში დღესვე
  (ბევრ Mastodon-მომხმარებელს აქვს წამკითხველი). ეს ჩაშენებულია და ნაგულისხმევად
  ჩართულია.
- **ხიდი** — მიმართეთ RSS-ფიდი RSS→ActivityPub ხიდზე, როგორიცაა
  [rss-parrot](https://rss-parrot.net/) ან [Bridgy Fed](https://fed.brid.gy/),
  რომლებიც გასცემენ ნამდვილ `@handle`-ს, რომელსაც Mastodon-მომხმარებლები შეძლებენ
  **გამოწერას**, ახალ პოსტებს მათ ლენტში გადასცემენ. საკუთარი სერვერი საჭირო არ
  არის.

ასე რომ: მდიდარი გადახედვები + ავტორის მითითება + პროფილის დადასტურება მუშაობს
მზამზარეულად; ნამდვილი „Mastodon-იდან გამოწერა" ერთი ხიდის მოშორებითაა, RSS-ფიდით
როგორც შესასვლელით.

## შეზღუდვები

საჯარო ვებ-გადახედვა არის ფასი იმისა, რომ **არანაირი ავთენტიფიკაცია** არ სჭირდება:

- **რეაქციები/მოწონებები არ ხდება ხელმისაწვდომი** `t.me/s/`-ით. სამაგიეროდ
  ვაკეთებთ **ნახვების რაოდენობის** ექსპორტს. მონაცემთა მოდელი ტოვებს ადგილს, რომ
  მოგვიანებით დაემატოს ნამდვილი რეაქციები ავთენტიფიცირებული MTProto API-ით (კრეიტი
  [`grammers`](https://codeberg.org/Lonami/grammers)), თუ ოდესმე მოგინდებათ.
- **დიდი ვიდეოები არ ჩამოიტვირთება** — გადახედვა მათთვის მხოლოდ პოსტერ-სურათსა და
  ხანგრძლივობას გასცემს (მოკლე/ავტო-დასაკრავი ვიდეოები *ჩამოიტვირთება*). ნამდვილი
  ფაილის დაარქივებას ასევე MTProto API დასჭირდებოდა.
- **სტიკერ-პაკები არ იბმება** — პაკის სახელს Telegram-ის JavaScript ტვირთავს და ის
  ჩამოქაჩულ HTML-ში არ არის; სტიკერები ინახება ჩვეულებრივ სურათებად.
- **მუსიკალური ფაილები (აუდიო-დოკუმენტები) არ ჩამოიტვირთება** — მათი URL ჩამოქაჩულ
  HTML-ში არ არის (მხოლოდ ხმოვანი ჩანაწერებია, პირდაპირი `.oga`-URL-ებით). დიდი
  ვიდეოებისა და სტიკერების მსგავსად, მათ MTProto API სჭირდებათ. დანართისთვის,
  რომელსაც ვერ ვიღებთ, ვინახავთ მის **ფაილის სახელს** (აღნიშნულს როგორც *არ არის
  დაარქივებული*), რომ იცოდეთ, რომ ის არსებობდა; პოსტი, რომელიც *მხოლოდ* ასეთი
  მითითებაა (ან ერთადერთი ჩამოუტვირთავი ფაილი), გამოტოვებულია და არა ცარიელად
  გამოქვეყნებული.
- **YouTube** იკვრება `youtube.com` iframe-ით (ასე ნახვები ითვლება მაყურებლის
  ისტორიაში) გამოქვეყნებულ HTTPS-საიტზე; `file://`-ით iframe ვერ იტვირთება
  (YouTube-ს origin სჭირდება). `YOUTUBE_FACADE=true` iframe-ს ცვლის JS-ის გარეშე
  დაწკაპუნებით ჩასატვირთი მინიატურით, რომელიც სულ მცირე პოსტერს აჩვენებს `file://`-ით.
- **მხოლოდ საჯარო არხები**, ჩართული ვებ-გადახედვით.

## ლიცენზია

[MIT](LICENSE) © Vitaly Zdanevich
