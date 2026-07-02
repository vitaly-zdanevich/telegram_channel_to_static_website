# tg2zola

🌐 [English](README.md) · [Беларуская](README.be.md) · [Українська](README.uk.md) · [Русский](README.ru.md) · [Deutsch](README.de.md) · [Français](README.fr.md) · [中文](README.zh.md) · [日本語](README.ja.md) · [Polski](README.pl.md) · [Español](README.es.md) · [한국어](README.ko.md) · [ქართული](README.ka.md) · **हिन्दी**

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

किसी **सार्वजनिक Telegram चैनल** का बैकअप एक **स्वयं-निहित [Zola](https://www.getzola.org/) स्टैटिक वेबसाइट** में लें।

> **माँग पर पुनः जनरेट करें:** ऊपर दिए **daily build** बैज पर क्लिक करें → **Run
> workflow**, ताकि शेड्यूल का इंतज़ार किए बिना scrape + rebuild + redeploy हो जाए
> (नीचे स्वचालन अनुभाग में विवरण)।

यह सार्वजनिक वेब प्रीव्यू (`https://t.me/s/<channel>`) को scrape करता है, सभी मीडिया
को स्थानीय रूप से डाउनलोड करता है, और हर बार चलने पर पूरा Zola ब्लॉग फिर से जनरेट
करता है। **किसी Telegram बॉट, टोकन या API की ज़रूरत नहीं** — यह केवल सार्वजनिक वेब
पेज पढ़ता है। आउटपुट में **कोई Telegram निर्भरता नहीं** है: मीडिया स्थानीय है, कोई
embed नहीं है, और चैनल की *अपनी* पोस्ट के लिंक आंतरिक सापेक्ष लिंक में बदल दिए जाते हैं
— इसलिए चैनल बाद में हटा दिए जाने पर भी साइट काम करती रहती है। अन्य साइटों (अन्य
Telegram चैनलों सहित) के लिए आपके लिखे लिंक सामान्य लिंक के रूप में सुरक्षित रहते हैं।
यह एक बैकअप है, मिरर नहीं।

Rust में लिखा गया: एक single static binary, जिसे स्थानीय रूप से या CI में चलाना आसान है।

## यह क्या करता है

- **पूरा इतिहास** — प्रीव्यू के `?before=` कर्सर के ज़रिए चैनल को पहले संदेश तक पीछे की
  ओर पढ़ता है, और हर बार **हर पेज को फिर से जनरेट** करता है।
- **स्वयं-निहित मीडिया** — फ़ोटो, वीडियो, ऑडियो (`.ogg/.oga/.mp3`), दस्तावेज़ और
  स्टिकर हर पोस्ट के bundle में डाउनलोड करता है (जो cache के रूप में भी काम करता है)।
  फ़ोटो उनके स्थिर Telegram file id से content-addressed होती हैं, इसलिए पोस्ट का
  टेक्स्ट संपादित करने पर उसका मीडिया दोबारा डाउनलोड नहीं होता, जबकि किसी छवि को
  **बदलने** पर (तब पोस्ट *edited* दिखती है) नई फ़ाइल लाई जाती है और पुरानी हटा दी जाती है।
- **ट्रू-ब्लैक डिफ़ॉल्ट थीम** — बिल्ट-इन टेम्पलेट `prefers-color-scheme` के ज़रिए डार्क
  मोड में `#000` से स्टाइल किए गए हैं (OLED-अनुकूल), बिना किसी बाहरी थीम निर्भरता के।
  एक बाहरी थीम को गारंटीशुदा fallback के साथ जोड़ा जा सकता है (नीचे थीमिंग अनुभाग देखें)।
- **स्मार्ट वीडियो हैंडलिंग** (प्राथमिकता क्रम में):
  1. संलग्न वीडियो **+ एक जीवित YouTube/Instagram लिंक** → उसे embed करें, वीडियो
     हटा दें (CI डिफ़ॉल्ट, जगह बचाने के लिए; मृत लिंक होने पर वीडियो रखा जाता है)।
     स्थानीय रूप से डिफ़ॉल्ट रूप से सब कुछ डाउनलोड होता है — `KEEP_MEDIA`/`--keep-media`
     इसे बाध्य करता है;
  2. सीधे डाउनलोड करने योग्य वीडियो → स्थानीय `<video>`;
  3. अन्यथा → **पोस्टर फ़्रेम** + अवधि सहेजें (सार्वजनिक पेज फ़ाइल नहीं देता; नीचे
     सीमाएँ अनुभाग देखें)।
- **फ़ॉर्मैटिंग** — bold, italic, strikethrough, code/pre, लिंक और spoiler को Markdown
  में बदला जाता है (Telegram के UTF-16 entity offset सही ढंग से संभाले जाते हैं)।
- **हैशटैग → tags** — `#hashtags` Zola के `tags` taxonomy शब्द बन जाते हैं, इसलिए
  आपको मुफ़्त में tag पेज मिलते हैं, जबकि वे पोस्ट में टेक्स्ट के रूप में भी दिखते हैं।
- **समूहित पोस्ट** — एल्बम अपने आप एक पोस्ट बन जाते हैं; एक ही क्षण में पोस्ट किए गए
  संदेशों के समूह (जैसे एक साथ कई फ़ॉरवर्ड करना) मर्ज कर दिए जाते हैं।
- **स्वयं-नेविगेटिंग** — *उसी* चैनल के किसी अन्य संदेश का लिंक ब्लॉग में उस पोस्ट के
  सापेक्ष लिंक में बदल जाता है; अन्य चैनलों के लिंक बाहरी ही रहते हैं।
- **एंगेजमेंट** — प्रति पोस्ट **व्यू काउंट** निर्यात करता है। (रिएक्शन/लाइक सार्वजनिक
  पेज से उपलब्ध नहीं हैं — नीचे सीमाएँ अनुभाग देखें।)
- **RSS फ़ीड** — पूरे कंटेंट के साथ **हर पोस्ट** का एक मानक `/rss.xml` (पूरी फ़ीड,
  केवल हाल के आइटम नहीं), जिसे `<link rel="alternate">` के ज़रिए विज्ञापित किया जाता है
  ताकि फ़ीड रीडर इसे साइट URL से अपने आप खोज लें। डिफ़ॉल्ट रूप से चालू; `RSS=false` /
  `--no-rss` से बंद करें।
- **रिच लिंक प्रीव्यू + Mastodon** — हर पेज Open Graph और Twitter Card टैग (शीर्षक,
  विवरण, पोस्ट की पहली छवि) उत्सर्जित करता है, इसलिए साझा किए गए लिंक कार्ड के रूप में
  दिखते हैं। Mastodon प्रीव्यू पर लेखक की byline जोड़ने और अपनी प्रोफ़ाइल पर साइट
  सत्यापित करने के लिए `FEDIVERSE_CREATOR` सेट करें (नीचे Fediverse / Mastodon अनुभाग देखें)।
- **डिफ़ॉल्ट रूप से कोई JavaScript नहीं, ऑफ़लाइन-तैयार** — डार्क मोड और spoiler केवल
  CSS हैं, और डिफ़ॉल्ट Google सर्च बॉक्स एक सादा `<form>` है (कोई JS नहीं)। केवल
  गैर-Google सर्च इंजन एक छोटा inline Enter handler जोड़ता है। `tg2zola offline
  <public-dir>` बनी हुई साइट को सापेक्ष लिंक में बदल देता है **और** Zola की pagination
  redirect script हटा देता है, इसलिए ऑफ़लाइन कॉपी बिना किसी JavaScript और वेब सर्वर के
  सीधे `file://` से खुलती है।
- **स्थानीयकृत UI** — साइट chrome (Newer/Older/Tags/About, सर्च बॉक्स, तिथियाँ)
  `LANGUAGE` / `--language` के ज़रिए 13 भाषाओं में से किसी में भी प्रस्तुत होता है
  (en/be/uk/ru/de/fr/zh/ja/pl/es/ko/ka/hi), महीने और सप्ताह के दिनों के नाम भी
  स्थानीयकृत होते हैं। पोस्ट कंटेंट चैनल की अपनी भाषा में रहता है।

## इंस्टॉल

अपने architecture के लिए एक static binary [Releases](../../releases) पेज से लें
(Linux `amd64` / `arm64`, musl), या स्रोत से बनाएँ:

```sh
cargo build --release
# binary target/release/tg2zola पर
```

जनरेट किए गए कंटेंट को HTML में बदलने के लिए आपको
[`zola`](https://www.getzola.org/documentation/getting-started/installation/)
binary की भी ज़रूरत है।

## उपयोग

```sh
# Zola साइट जनरेट करें (पहली बार चलने पर config + templates बनाता है):
tg2zola --channel durov --site site --init-site

# static HTML बनाएँ:
zola --root site build       # आउटपुट site/public/ में

# (वैकल्पिक) बिना किसी वेब सर्वर के, सीधे डिस्क से देखने योग्य बनाएँ:
tg2zola offline site/public  # फिर site/public/index.html को file:// से खोलें
```

त्वरित स्थानीय परीक्षण (एक पेज, ~20 संदेश):

```sh
tg2zola --channel <name> --site /tmp/site --init-site --max-pages 1
```

सभी विकल्प [`tg2zola.toml`](tg2zola.toml) में हैं (CLI flags इन्हें ओवरराइड करते हैं):

```sh
tg2zola --config tg2zola.toml
```

पूरी flag सूची के लिए `tg2zola --help` चलाएँ।

## यह कैसे जुड़ा है

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
        ▼  zola build      (bundle ही cache है)
 site/public/             स्वयं-निहित static साइट
```

हर पोस्ट एक Zola page bundle बन जाती है — `content/posts/<date>-<id>/index.md`,
TOML front matter और उसके साथ उसकी मीडिया फ़ाइलों सहित। `config.toml` और बिल्ट-इन
टेम्पलेट हर बार निश्चित रूप से फिर से जनरेट होते हैं, और `write_site` ट्री को समेटता है:
सभी Markdown फिर से लिखता है, पहले से cache किए मीडिया रखता है, और हटाई गई पोस्ट व
पुरानी फ़ाइलें हटा देता है।

## स्वचालन (GitHub Actions)

दो workflow शामिल हैं:

- **[`daily.yml`](.github/workflows/daily.yml)** — दिन में एक बार (और माँग पर)
  चलता है: `blog` शाखा से पिछली साइट पुनर्स्थापित करें (मीडिया cache) → scrape +
  पुनः जनरेट → `zola build` → **GitHub Pages पर deploy करें** (प्रकाशित परिणाम) →
  ताज़ा साइट को वापस **`blog` शाखा** में commit करें।
- **[`release.yml`](.github/workflows/release.yml)** — हर push किए गए `v*` tag पर,
  static `amd64` + `arm64` (musl) binaries को cross-compile करता है और उन्हें
  GitHub Release पर अपलोड करता है।

प्रकाशन सक्षम करने के लिए: रिपॉज़िटरी में, **Settings → Pages → Build and
deployment → Source: GitHub Actions**। किसी secret की ज़रूरत नहीं — सब कुछ
सार्वजनिक-scrape है। प्रकाशित साइट हमेशा **GitHub Pages** होती है; `blog` शाखा एक
टिकाऊ कॉपी है और यह कभी प्रभावित नहीं करती कि विज़िटर क्या देखते हैं।

### अभी पुनः जनरेट करें (दैनिक रन का इंतज़ार न करें)

`daily.yml` में `workflow_dispatch` सक्षम है, इसलिए आप माँग पर एक ताज़ा scrape +
rebuild + redeploy ट्रिगर कर सकते हैं — यह ठीक वही चरण चलाता है जो शेड्यूल किया रन:

- **ब्राउज़र में:** **[Actions → "daily" → Run workflow](../../actions/workflows/daily.yml)**
  खोलें और हरा **Run workflow** बटन क्लिक करें। (एक-क्लिक पहुँच के लिए ऊपर दिया status
  बैज अपने README में जोड़ें।)
- **टर्मिनल से:** `gh workflow run daily.yml` (GitHub CLI), फिर उसे देखने के लिए
  `gh run watch`।

**कौन-सा चैनल?** चैनल commit नहीं किया जाता, इसलिए हर deployment अपना खुद का सेट
करता है। एक रिपॉज़िटरी **variable** `CHANNEL` (Settings → Secrets and variables →
Actions → Variables) को सार्वजनिक चैनल यूज़रनेम पर सेट करें — यह एक *variable* है,
secret नहीं, क्योंकि चैनल सार्वजनिक है। (या अपने fork में
[`tg2zola.toml`](tg2zola.toml) में `channel = "…"` को uncomment करें।) `THEME_REPO`
भी एक variable के रूप में काम करता है।

### `blog` शाखा (संग्रह + cache)

हर रन जनरेट की गई साइट (Markdown + मीडिया + बिल्ट-इन टेम्पलेट — बनी हुई `public/`
और किसी बाहरी थीम को छोड़कर सब कुछ) को एक `blog` शाखा में commit करता है, जिससे
`main` केवल-कोड रहती है। यह दोहरी भूमिका निभाती है:

- **Cache** — अगला रन दोबारा डाउनलोड करने के बजाय इससे मीडिया पुनर्स्थापित करता है।
- **टिकाऊ संग्रह** — एक पूरा, buildable Zola साइट जिसे आप कहीं भी clone और mirror कर
  सकते हैं, ताकि बैकअप एक ही platform से बँधा न रहे:

```sh
git clone -b blog https://github.com/<you>/<repo> my-archive
zola --root my-archive/site serve          # इसे ऑफ़लाइन ब्राउज़ करें

git -C my-archive remote add gitlab git@gitlab.com:<you>/<repo>.git
git -C my-archive push gitlab blog          # इसे कहीं और mirror करें
```

मीडिया सादे git blobs के रूप में commit होता है; बहुत बड़े चैनलों के लिए Git LFS या
कभी-कभी history squashing पर विचार करें।

### थीमिंग

डिफ़ॉल्ट बिल्ट-इन **true-black** थीम है — शून्य बाहरी निर्भरता, इसलिए साइट हमेशा
build होती है। किसी बाहरी [Zola थीम](https://www.getzola.org/themes/) का उपयोग करने
के लिए, एक रिपॉज़िटरी **variable** `THEME_REPO` (Settings → Secrets and variables →
Actions → Variables) को उसके git URL (https, या deploy key के साथ ssh) पर सेट करें।
workflow उसे clone करके उससे build करता है — और **यदि थीम अनुपस्थित है या उसका build
विफल होता है, तो यह अपने आप बिल्ट-इन टेम्पलेट पर वापस लौट आता है**, इसलिए कोई थीम
समस्या ब्लॉग को कभी ऑफ़लाइन नहीं कर सकती। ध्यान दें कि बाहरी थीमें एक ख़ास कंटेंट
लेआउट की अपेक्षा करती हैं, इसलिए हर थीम drop-in संगत नहीं होती।

एक release बनाएँ:

```sh
git tag v0.1.0 && git push origin v0.1.0
```

## कॉन्फ़िगरेशन

GitHub Actions फ़्लो के लिए सब कुछ एक रिपॉज़िटरी **variable** (Settings → Secrets
and variables → Actions → **Variables**) के ज़रिए कॉन्फ़िगर किया जा सकता है, या
स्थानीय रूप से चलाते समय समकक्ष CLI flag / [`tg2zola.toml`](tg2zola.toml) key से।
ये *variables* हैं, secrets नहीं — यह सब सार्वजनिक है।

| रिपॉज़िटरी variable | CLI flag | डिफ़ॉल्ट | यह क्या करता है |
|---|---|---|---|
| `CHANNEL` | `--channel` | **आवश्यक** | सिंक करने के लिए सार्वजनिक चैनल |
| `TITLE` | `--title` | चैनल यूज़रनेम | ब्लॉग शीर्षक (header + `<title>`) |
| `ABOUT` | `--about` | विवरण + आँकड़े + रिपॉज़िटरी लिंक | About पेज के body के लिए कस्टम HTML |
| `PAGES` | `--pages` | — | अतिरिक्त पेज: Markdown, हर `# Title` शीर्षक एक नया पेज + nav प्रविष्टि शुरू करता है |
| `POSTS_PER_PAGE` | `--posts-per-page` | `20` | होम फ़ीड पर प्रति पेज पूरी पोस्ट |
| `TAGS_FOOTER` | `--tags-footer` | बंद | प्रति-पोस्ट tag footer दिखाने के लिए `true` (tags वैसे भी body में क्लिक-योग्य हैं) |
| `NEXT_PREV` | `--no-next-prev` | चालू | `false` Next/Prev पोस्ट नेविगेशन छिपाता है |
| `TELEGRAM_LINK` | `--no-telegram-link` | चालू | `false` प्रति-पोस्ट "View on Telegram" लिंक छिपाता है |
| `RSS` | `--no-rss` | चालू | `false` `/rss.xml` पर RSS फ़ीड बंद करता है (reader autodiscovery सहित) |
| `FEDIVERSE_CREATOR` | `--fediverse-creator` | — | Mastodon `@user@instance` → `fediverse:creator` byline + `rel="me"` प्रोफ़ाइल लिंक |
| `SEARCH_ENGINE` | `--search-engine` | `google` | Header सर्च बॉक्स: `google` (JS-मुक्त form) / `duckduckgo` / `yandex` / `bing` / `none` |
| `SEARCH_URL` | `--search-url` | — | कस्टम सर्च URL prefix; Enter पर query जोड़ा जाता है (engine को ओवरराइड करता है) |
| `TITLE_MAX_LEN` | `--title-max-len` | `200` | अधिकतम पोस्ट-शीर्षक लंबाई (अक्षर); छोटा किया गया शीर्षक अपना पूरा पहला वाक्य body में रखता है |
| `FOOTER` | `--footer` | — | Footer कंटेंट — सादा टेक्स्ट, Markdown या HTML |
| `PAGES_HOST` | `--pages-host` | स्वतः | About-पेज size limit के लिए host: `github` / `gitlab` / `none` (URL से स्वतः पहचाना जाता है) |
| `DATE_FORMAT` | `--date-format` | `%Y %B %d` | दिखाई गई तिथियों के लिए strftime प्रारूप (जैसे `2025 October 28`; केवल वर्ष के लिए `%Y`) |
| `LANGUAGE` | `--language` | `en` | साइट chrome के लिए UI भाषा (Newer/Older/Tags/About/…): `en`/`be`/`uk`/`ru`/`de`/`fr`/`zh`/`ja`/`pl`/`es`/`ko`/`ka` (जॉर्जियन)/`hi` (हिन्दी)। पोस्ट कंटेंट चैनल की अपनी भाषा में रहता है; तिथियाँ उसके अनुसार स्थानीयकृत होती हैं |
| `DERIVE_TITLES` | `--derive-titles` | बंद | `true` पोस्ट का शीर्षक उसके पहले वाक्य से लेता है; डिफ़ॉल्ट पोस्ट की date/views पंक्ति पर एक क्लिक-योग्य `#id` दिखाता है |
| `STRIP_TITLE` | `--strip-title` | बंद | `DERIVE_TITLES` के साथ, वह पहला वाक्य body से भी हटा देता है ताकि वह दो बार न दिखे |
| `LINK_UNDERLINE` | `--link-underline` | बंद | `true` लिंक को underline करता है (डिफ़ॉल्ट: कोई underline नहीं) |
| `YOUTUBE_FACADE` | `--youtube-facade` | बंद | no-JS click-to-load YouTube thumbnail के लिए `true` (डिफ़ॉल्ट: सीधा iframe) |
| `KEEP_MEDIA` | `--keep-media` | CI: बंद · स्थानीय: चालू | संलग्न **वीडियो/ऑडियो** को रखने (डाउनलोड + दिखाने) के लिए `true`, तब भी जब पोस्ट में YouTube / Apple Podcasts / Instagram लिंक हो। डिफ़ॉल्ट परिवेश-आधारित है: CI (GitHub Actions / GitLab) पर embed संलग्न मीडिया की जगह लेता है ताकि होस्टिंग जगह बचे; स्थानीय मशीन पर पूरे बैकअप के लिए सब कुछ डाउनलोड होता है |
| `GENIUS` | `--no-genius` | चालू | `false` genius.com लिंक हल करना छोड़ देता है (उसके YouTube वीडियो + lyrics widget के लिए पेज लाता है) |
| `LIVENESS` | `--no-liveness` | चालू | `false` YouTube liveness जाँच छोड़ देता है; हटाया गया वीडियो (oEmbed 404) अन्यथा dead embed दिखाने के बजाय अपना स्थानीय मीडिया रखता है |
| `TAGS_TO_PAGES` | `--tags-to-pages` | — | अल्पविराम से अलग किए गए tags जो शीर्ष nav में `#tag` लिंक के रूप में दिखते हैं (जैसे `music, batumi, cooking`) |
| `BACKGROUND_DARK_COLOR` | `--background-dark-color` | `#000000` | डार्क-मोड पृष्ठभूमि (कोई भी CSS रंग) |
| `BACKGROUND_LIGHT_COLOR` | `--background-light-color` | `#ffffff` | लाइट-मोड पृष्ठभूमि |
| `CSS` | `--css` | — | बिल्ट-इन stylesheet में जोड़ा गया अतिरिक्त CSS |
| `THEME_REPO` | `--theme` (नाम) | बिल्ट-इन ब्लैक थीम | बाहरी Zola थीम git URL (https/ssh); विफल होने पर स्वतः वापस लौटता है |
| `REPO_URL` | `--repo-url` | tg2zola रिपॉज़िटरी | About पर "Source repository" लिंक (CI इसे स्वतः आपकी रिपॉज़िटरी पर सेट करता है) |

**About** पेज चैनल **avatar** (पूरा आकार), उसका विवरण और आँकड़े, **डिस्क पर आकार** —
प्रति-प्रकार breakdown के साथ और, GitHub/GitLab Pages पर, host की ~1 GB प्रकाशित-साइट
सीमा का हिस्सा (host डॉक्स से लिंक किया गया) — साथ ही रिपॉज़िटरी लिंक दिखाता है।
Header avatar को thumbnail और favicon के रूप में दिखाता है; hashtags पोस्ट body में
क्लिक-योग्य हैं और `/tags/<tag>/` पेज बनाते हैं।

एक अकेला `PAGES` variable कई पेज परिभाषित कर सकता है — हर `# Title` शीर्षक एक नया
शुरू करता है:

```
# Page title
Markdown में पेज कंटेंट (जिसमें raw HTML हो सकता है)।

# Another page
और कंटेंट।
```

→ `/page-title/` और `/another-page/`, प्रत्येक nav में लिंक किया गया। कोई अतिरिक्त
निर्भरता नहीं — Zola पहले से ही Markdown render करता है।

## Fediverse / Mastodon

हर पेज Open Graph + Twitter Card टैग रखता है, इसलिए एक साझा पोस्ट लिंक Mastodon,
Slack, Discord, X आदि में कार्ड (शीर्षक, विवरण, पोस्ट की पहली छवि) के रूप में दिखता है।
निम्नलिखित के लिए भी `FEDIVERSE_CREATOR` को अपने `@user@instance` handle पर सेट करें:

- एक **`fediverse:creator`** byline जोड़ें, ताकि Mastodon लिंक प्रीव्यू पर "*by
  @you@instance*" दिखाए और आपकी प्रोफ़ाइल से लिंक करे;
- अपनी प्रोफ़ाइल के लिए एक **`rel="me"`** लिंक उत्सर्जित करें, ताकि आप इस साइट को अपनी
  Mastodon प्रोफ़ाइल के metadata में जोड़कर **verified** (हरा) चेकमार्क पा सकें।

**क्या लोग ब्लॉग को *Mastodon से* फ़ॉलो कर सकते हैं?** सीधे नहीं — एक static साइट
ActivityPub actor नहीं हो सकती (उसके लिए WebFinger + ActivityPub बोलने वाला एक जीवित
सर्वर चाहिए)। फिर भी लोगों को सब्सक्राइब करने देने के दो तरीके:

- **RSS** — कोई भी आज `/rss.xml` को किसी फ़ीड reader में फ़ॉलो कर सकता है (कई Mastodon
  उपयोगकर्ता एक रखते हैं)। यह बिल्ट-इन है और डिफ़ॉल्ट रूप से चालू है।
- **एक bridge** — RSS फ़ीड को किसी RSS→ActivityPub bridge जैसे
  [rss-parrot](https://rss-parrot.net/) या [Bridgy Fed](https://fed.brid.gy/) की ओर
  इंगित करें, जो एक असली `@handle` उजागर करते हैं जिसे Mastodon उपयोगकर्ता **फ़ॉलो**
  कर सकते हैं, नई पोस्ट को उनकी timeline में पहुँचाते हुए। अपने किसी सर्वर की ज़रूरत नहीं।

तो: रिच प्रीव्यू + लेखक श्रेय + प्रोफ़ाइल सत्यापन बॉक्स से बाहर काम करते हैं; असली
"Mastodon से फ़ॉलो" RSS फ़ीड को input बनाकर बस एक bridge दूर है।

## वैकल्पिक MTProto बैकएंड

सार्वजनिक वेब प्रीव्यू **voice/audio notes** या **पूर्ण-रिज़ॉल्यूशन फ़ोटो** नहीं दे
सकता (नीचे सीमाएँ अनुभाग देखें)। एक **ऑप्ट-इन** बैकएंड *आपके उपयोगकर्ता खाते* के रूप
में MTProto ([`grammers`](https://codeberg.org/Lonami/grammers) के ज़रिए) पर लॉगिन
करके उन्हें लाता है। यह **डिफ़ॉल्ट रूप से बंद** है — सामान्य build और CI शून्य-क्रेडेंशियल
वेब scraper बने रहते हैं — और केवल एक Cargo feature के साथ compile होता है:

```sh
cargo build --release --features mtproto
```

**1. API क्रेडेंशियल लें।** [my.telegram.org](https://my.telegram.org) → *API
development tools* पर एक app बनाएँ, और **`api_id`** (एक संख्या) व **`api_hash`**
(एक string) नोट करें।

**2. एक बार लॉगिन करें** ताकि एक पुन: उपयोग योग्य session बने — केवल
`api_id`/`api_hash` प्रमाणित नहीं कर सकते, Telegram को एक असली उपयोगकर्ता लॉगिन चाहिए:

```sh
export TG_API_ID=1234567
export TG_API_HASH=0123456789abcdef0123456789abcdef
tg2zola login        # पूछता है: फ़ोन → कोड (Telegram में भेजा गया) → 2FA पासवर्ड
```

यह **`tg2zola.session`** लिखता है और एक base64 **`TG_SESSION`** string प्रिंट करता है।
इसके बाद, रन गैर-इंटरैक्टिव होते हैं।

**3. जनरेट करें** क्रेडेंशियल परिवेश में रखते हुए — एक सामान्य रन तब हर पोस्ट के bundle
में ऑडियो भी लाता है (और `MTPROTO_IMAGES=1` / `MTPROTO_VIDEOS=1` के साथ, मूल-गुणवत्ता की
फ़ोटो / वे पूरे वीडियो जिन्हें प्रीव्यू केवल पोस्टर के रूप में दिखाता है):

```sh
TG_API_ID=$TG_API_ID TG_API_HASH=$TG_API_HASH MTPROTO_IMAGES=1 \
  tg2zola --channel <name> --site site --init-site
```

| Env var | उद्देश्य |
|---|---|
| `TG_API_ID` / `TG_API_HASH` | my.telegram.org से app क्रेडेंशियल (आवश्यक) |
| `TG_SESSION` | `tg2zola login` से base64 session; वैकल्पिक रूप से working dir में एक `tg2zola.session` फ़ाइल उपयोग होती है |
| `MTPROTO_IMAGES` | मूल-गुणवत्ता की फ़ोटो भी लाने के लिए `1`/`true` (ऑडियो हमेशा लाया जाता है) |
| `MTPROTO_VIDEOS` | उन पोस्ट के लिए पूरा वीडियो भी लाने के लिए `1`/`true` जिन्हें वेब प्रीव्यू केवल पोस्टर के रूप में दिखाता है (बड़ी फ़ाइलें — स्थानीय बैकअप के लिए; डिफ़ॉल्ट रूप से बंद) |
| `TG_SESSION_FILE` | session-फ़ाइल पथ ओवरराइड करें (डिफ़ॉल्ट `tg2zola.session`) |

**CI के लिए:** `tg2zola login` को **स्थानीय रूप से** चलाएँ (इंटरैक्टिव चरण Actions में
नहीं चल सकता), फिर `TG_API_ID`, `TG_API_HASH` और प्रिंट किए गए `TG_SESSION` को
**Actions secrets** के रूप में संग्रहीत करें। session की कोई निश्चित समाप्ति नहीं है —
यह तब तक चलती है जब तक आप इसे लॉग आउट न करें, Telegram इसे रद्द न करे, या यह ~6 महीने
अनुपयोगी न रहे — इसलिए एक दैनिक रन इसे अनिश्चित काल तक जीवित रखता है।

> ⚠️ **`TG_SESSION` आपके खाते तक पूर्ण पहुँच है** — इसे पासवर्ड की तरह मानें (एक
> *secret*, सार्वजनिक `CHANNEL` variable के विपरीत)। एक द्वितीयक खाते पर विचार करें
> जो केवल चैनल का सदस्य हो। उपयोगकर्ता-खाता स्वचालन एक Telegram ग्रे क्षेत्र है; इसे
> अपने खुद के चैनल पर हल्की दरों पर उपयोग करें। हर रन पूरे संदेश इतिहास को पढ़ता है।

## सीमाएँ

सार्वजनिक वेब प्रीव्यू **शून्य प्रमाणीकरण** की ज़रूरत के बदले एक trade-off है (इनमें से
कई वैकल्पिक MTProto बैकएंड द्वारा हटा दी जाती हैं):

- **रिएक्शन/लाइक उजागर नहीं होते** `t.me/s/` से। हम इसके बजाय **व्यू काउंट** निर्यात
  करते हैं। डेटा मॉडल में असली रिएक्शन बाद में प्रमाणित MTProto API
  ([`grammers`](https://codeberg.org/Lonami/grammers) crate) के ज़रिए जोड़ने की जगह है,
  यदि आप कभी चाहें।
- **बड़े वीडियो सार्वजनिक प्रीव्यू से डाउनलोड करने योग्य नहीं** — यह उनके लिए केवल एक
  पोस्टर छवि और अवधि देता है (छोटे/auto-play वीडियो *डाउनलोड करने योग्य* हैं)। वैकल्पिक
  MTProto बैकएंड `MTPROTO_VIDEOS=1` के साथ इनके लिए असली फ़ाइल लाता है।
- **स्टिकर पैक लिंक करने योग्य नहीं** — पैक का नाम Telegram के JavaScript से लोड होता है
  और scrape किए गए HTML में नहीं है; स्टिकर सादी छवियों के रूप में सहेजे जाते हैं।
- **संगीत फ़ाइलें (audio documents) डाउनलोड करने योग्य नहीं** `t.me/s/` से — उनका URL
  scrape किए गए HTML में नहीं है (केवल voice notes, सीधे `.oga` URLs के साथ, होते हैं)।
  वैकल्पिक MTProto बैकएंड ऑडियो (voice + संगीत) लाता है; इसके बिना, जिस अटैचमेंट को हम
  नहीं ला सकते उसके लिए हम उसका **फ़ाइलनाम** रखते हैं (*not archived* चिह्नित) ताकि आपको
  पता रहे कि वह मौजूद था; जो पोस्ट *केवल* ऐसा संदर्भ (या एक अकेली गैर-डाउनलोड-योग्य
  फ़ाइल) हो, उसे खाली प्रकाशित करने के बजाय छोड़ दिया जाता है।
- **YouTube** प्रकाशित HTTPS साइट पर एक `youtube.com` iframe के ज़रिए चलता है (इसलिए
  प्ले दर्शक के इतिहास में गिने जाते हैं); `file://` पर iframe लोड नहीं हो सकता (YouTube
  को एक origin चाहिए)। `YOUTUBE_FACADE=true` iframe को एक no-JS click-to-load
  thumbnail से बदल देता है, जो कम-से-कम `file://` पर पोस्टर दिखाता है।
- **केवल सार्वजनिक चैनल**, वेब प्रीव्यू सक्षम के साथ।

## लाइसेंस

[MIT](LICENSE) © Vitaly Zdanevich
