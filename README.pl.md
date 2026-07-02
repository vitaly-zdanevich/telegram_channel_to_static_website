# tg2zola

🌐 [English](README.md) · [Беларуская](README.be.md) · [Українська](README.uk.md) · [Русский](README.ru.md) · [Deutsch](README.de.md) · [Français](README.fr.md) · [中文](README.zh.md) · [日本語](README.ja.md) · **Polski** · [Español](README.es.md) · [한국어](README.ko.md) · [ქართული](README.ka.md) · [हिन्दी](README.hi.md)

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

Utwórz kopię zapasową **publicznego kanału Telegram** jako **samowystarczalną
statyczną stronę z [Zola](https://www.getzola.org/)**.

> **Regeneracja na żądanie:** kliknij odznakę **daily build** powyżej → **Run
> workflow**, aby ponownie zescrapować, zbudować i wdrożyć bez czekania na
> harmonogram (szczegóły w sekcji **„Automatyzacja"**).

Narzędzie czyta publiczny podgląd webowy (`https://t.me/s/<channel>`), pobiera
wszystkie media lokalnie i przy każdym uruchomieniu na nowo generuje pełny blog
Zola. **Nie jest potrzebny żaden bot, token ani API Telegrama** — czytana jest
tylko publiczna strona internetowa. Wynik **nie ma zależności od Telegrama**: media
są lokalne, nie ma osadzeń, a linki do *własnych* wpisów kanału są przepisywane na
wewnętrzne względne linki — dzięki czemu strona działa dalej, nawet jeśli kanał
zostanie później usunięty. Linki, które napisałeś do innych witryn (w tym do innych
kanałów Telegram), są zachowywane jako zwykłe linki. To kopia zapasowa, a nie
lustro.

Napisane w Rust: jeden statyczny plik binarny, łatwy do uruchomienia lokalnie lub
w CI.

## Co robi

- **Pełna historia** — przechodzi kanał wstecz po kursorze `?before=` podglądu aż
  do pierwszej wiadomości i **regeneruje każdą stronę** przy każdym uruchomieniu.
- **Samowystarczalne media** — pobiera zdjęcia, wideo, audio (`.ogg/.oga/.mp3`),
  dokumenty i naklejki do bundla każdego wpisu (który zarazem służy jako pamięć
  podręczna). Zdjęcia są adresowane po ich stabilnym identyfikatorze pliku
  Telegram, więc edycja tekstu wpisu nigdy nie pobiera ponownie jego mediów,
  natomiast **zamiana** obrazu (wpis jest wtedy oznaczany jako *edytowany*) pobiera
  nowy plik i usuwa stary.
- **Domyślny motyw „prawdziwa czerń"** — wbudowane szablony stylizowane na `#000`
  w trybie ciemnym przez `prefers-color-scheme` (przyjazne dla OLED), bez
  zewnętrznych zależności. Motyw zewnętrzny można nałożyć z gwarantowanym
  awaryjnym powrotem (zob. sekcję **„Motywy"**).
- **Inteligentna obsługa wideo** (w kolejności priorytetu):
  1. dołączone wideo **+ link YouTube** → osadź YouTube, porzuć wideo;
  2. wideo do bezpośredniego pobrania → lokalne `<video>`;
  3. w przeciwnym razie → zapisz **klatkę-plakat** + czas trwania (publiczna strona
     nie udostępnia pliku; zob. **„Ograniczenia"**).
- **Formatowanie** — pogrubienie, kursywa, przekreślenie, kod/pre, linki i spoilery
  są konwertowane na Markdown (przesunięcia encji UTF-16 Telegrama są obsługiwane
  poprawnie).
- **Hashtagi → tagi** — `#hashtagi` stają się terminami taksonomii `tags` w Zola,
  więc za darmo dostajesz strony tagów, a one nadal pokazują się jako tekst we
  wpisie.
- **Pogrupowane wpisy** — albumy automatycznie stają się jednym wpisem; serie
  wiadomości opublikowanych w tej samej chwili (np. przy przesyłaniu kilku naraz)
  są scalane.
- **Samonawigacja** — link do innej wiadomości w *tym samym* kanale staje się
  względnym linkiem do tego wpisu w blogu; linki do innych kanałów pozostają
  zewnętrzne.
- **Zaangażowanie** — eksportuje **liczbę wyświetleń** dla każdego wpisu.
  (Reakcje/polubienia nie są dostępne na publicznej stronie — zob.
  **„Ograniczenia"**.)
- **Kanał RSS** — standardowy `/rss.xml` ze **wszystkimi wpisami** i pełną treścią
  (kompletny kanał, nie tylko najnowsze pozycje), ogłoszony przez
  `<link rel="alternate">`, aby czytniki znajdowały go automatycznie po adresie
  strony. Domyślnie włączony; wyłączany przez `RSS=false` / `--no-rss`.
- **Bogate podglądy linków + Mastodon** — każda strona emituje znaczniki Open Graph
  i Twitter Card (tytuł, opis, pierwszy obraz wpisu), dzięki czemu udostępniane
  linki renderują się jako karty. Ustaw `FEDIVERSE_CREATOR`, aby dodać podpis autora
  w podglądach Mastodona i zweryfikować stronę w swoim profilu (zob. sekcję
  **„Fediverse"**).
- **Domyślnie bez JavaScriptu, gotowe offline** — tryb ciemny i spoilery są tylko
  w CSS, a domyślne pole wyszukiwania Google to zwykły `<form>` (bez JS). Tylko
  wyszukiwarka inna niż Google dodaje jeden malutki wbudowany handler Enter.
  `tg2zola offline <katalog-public>` przepisuje zbudowaną stronę na linki względne
  **i** usuwa skrypt przekierowania paginacji Zoli, dzięki czemu kopia offline
  otwiera się wprost z `file://` bez żadnego JavaScriptu i bez serwera WWW.
- **Zlokalizowany interfejs** — interfejs strony (Nowsze/Starsze/Tagi/O stronie,
  pole wyszukiwania, daty) wyświetla się w jednym z 12 języków przez `LANGUAGE` /
  `--language` (en/be/uk/ru/de/fr/zh/ja/pl/es/ko/ka), ze zlokalizowanymi nazwami
  miesięcy i dni tygodnia. Treść wpisów pozostaje w języku kanału.

## Instalacja

Pobierz statyczny plik binarny dla swojej architektury ze strony
[Releases](../../releases) (Linux `amd64` / `arm64`, musl) lub zbuduj ze źródeł:

```sh
cargo build --release
# plik binarny w target/release/tg2zola
```

Potrzebujesz też pliku binarnego [`zola`](https://www.getzola.org/documentation/getting-started/installation/),
aby zamienić wygenerowaną treść na HTML.

## Użycie

```sh
# Wygeneruj stronę Zola (przy pierwszym uruchomieniu tworzy config + szablony):
tg2zola --channel durov --site site --init-site

# Zbuduj statyczny HTML:
zola --root site build       # wynik w site/public/

# (opcjonalnie) Uczyń ją oglądalną BEZ serwera WWW, wprost z dysku:
tg2zola offline site/public  # potem otwórz site/public/index.html przez file://
```

Szybki test lokalny (jedna strona, ~20 wiadomości):

```sh
tg2zola --channel <name> --site /tmp/site --init-site --max-pages 1
```

Wszystkie opcje są w [`tg2zola.toml`](tg2zola.toml) (flagi CLI je nadpisują):

```sh
tg2zola --config tg2zola.toml
```

Uruchom `tg2zola --help`, aby zobaczyć pełną listę flag.

## Jak to jest połączone

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

Każdy wpis staje się bundlem strony Zola — `content/posts/<date>-<id>/index.md` z
front matterem TOML i plikami mediów obok. `config.toml` i wbudowane szablony są
regenerowane deterministycznie przy każdym uruchomieniu, a `write_site` uzgadnia
drzewo: przepisuje cały Markdown, zachowuje już zbuforowane media i usuwa usunięte
wpisy oraz przestarzałe pliki.

## Automatyzacja (GitHub Actions)

Dołączone są dwa workflowy:

- **[`daily.yml`](.github/workflows/daily.yml)** — uruchamia się raz dziennie (i na
  żądanie): przywróć poprzednią stronę z gałęzi `blog` (pamięci podręcznej mediów) →
  scrapuj + regeneruj → `zola build` → **wdróż na GitHub Pages** (opublikowany
  wynik) → zacommituj odświeżoną stronę z powrotem do gałęzi **`blog`**.
- **[`release.yml`](.github/workflows/release.yml)** — przy każdym wypchniętym tagu
  `v*` kompiluje skrośnie statyczne pliki binarne `amd64` + `arm64` (musl) i wysyła
  je do Release na GitHubie.

Aby włączyć publikację: w repozytorium **Settings → Pages → Build and deployment →
Source: GitHub Actions**. Sekrety nie są wymagane — wszystko działa na publicznym
scrapowaniu. Opublikowana strona to zawsze **GitHub Pages**; gałąź `blog` to trwała
kopia i nigdy nie wpływa na to, co widzą odwiedzający.

### Regeneruj teraz (bez czekania na dzienne uruchomienie)

`daily.yml` ma włączone `workflow_dispatch`, więc możesz wyzwolić świeże scrapowanie
+ budowanie + ponowne wdrożenie na żądanie — wykonuje dokładnie te same kroki, co
uruchomienie zaplanowane:

- **W przeglądarce:** otwórz **[Actions → „daily" → Run workflow](../../actions/workflows/daily.yml)**
  i kliknij zielony przycisk **Run workflow**. (Dodaj odznakę statusu powyżej do
  swojego README, aby mieć dostęp jednym kliknięciem.)
- **Z terminala:** `gh workflow run daily.yml` (GitHub CLI), potem `gh run watch`,
  aby śledzić.

**Który kanał?** Kanał nie jest zacommitowany, więc każde wdrożenie ustawia własny.
Ustaw **zmienną** repozytorium `CHANNEL` (Settings → Secrets and variables → Actions
→ Variables) na nazwę publicznego kanału — to *zmienna*, a nie sekret, bo kanał jest
publiczny. (Albo odkomentuj `channel = "…"` w [`tg2zola.toml`](tg2zola.toml) w swoim
forku.) `THEME_REPO` też działa jako zmienna.

### Gałąź `blog` (archiwum + pamięć podręczna)

Każde uruchomienie commituje wygenerowaną stronę (Markdown + media + wbudowane
szablony — wszystko poza zbudowanym `public/` i ewentualnym motywem zewnętrznym) do
gałęzi `blog`, pozostawiając w `main` tylko kod. Pełni podwójną rolę:

- **Pamięć podręczna** — następne uruchomienie przywraca z niej media zamiast
  pobierać je ponownie.
- **Trwałe archiwum** — kompletna, budowalna strona Zola, którą możesz sklonować i
  zmirrorować gdziekolwiek, aby kopia nie była związana z jedną platformą:

```sh
git clone -b blog https://github.com/<you>/<repo> my-archive
zola --root my-archive/site serve          # przeglądaj offline

git -C my-archive remote add gitlab git@gitlab.com:<you>/<repo>.git
git -C my-archive push gitlab blog          # zmirroruj gdzie indziej
```

Media są commitowane jako zwykłe bloby git; przy bardzo dużych kanałach rozważ Git
LFS lub okazjonalne spłaszczanie historii.

### Motywy

Domyślny jest wbudowany motyw **prawdziwa czerń** — zero zewnętrznych zależności,
więc strona zawsze się buduje. Aby użyć zewnętrznego
[motywu Zola](https://www.getzola.org/themes/), ustaw **zmienną** repozytorium
`THEME_REPO` (Settings → Secrets and variables → Actions → Variables) na jego URL
git (https lub ssh z kluczem wdrożeniowym). Workflow klonuje go i buduje z nim — a
**jeśli motyw jest nieobecny lub jego budowa się nie powiedzie, automatycznie wraca
do wbudowanych szablonów**, więc problem z motywem nigdy nie może wyłączyć bloga.
Zwróć uwagę, że motywy zewnętrzne oczekują określonego układu treści, więc nie każdy
motyw jest zgodny od razu.

Wydaj release:

```sh
git tag v0.1.0 && git push origin v0.1.0
```

## Konfiguracja

Wszystko konfiguruje się przez **zmienną** repozytorium (Settings → Secrets and
variables → Actions → **Variables**) dla przepływu GitHub Actions lub przez
równoważną flagę CLI / klucz [`tg2zola.toml`](tg2zola.toml) przy uruchomieniu
lokalnym. To *zmienne*, a nie sekrety — wszystko jest publiczne.

| Zmienna repo | Flaga CLI | Domyślnie | Co robi |
|---|---|---|---|
| `CHANNEL` | `--channel` | **wymagane** | Publiczny kanał do synchronizacji |
| `TITLE` | `--title` | nazwa kanału | Tytuł bloga (nagłówek + `<title>`) |
| `ABOUT` | `--about` | opis + statystyki + link do repo | Własny HTML dla treści strony O stronie |
| `PAGES` | `--pages` | — | Dodatkowe strony: Markdown, każdy nagłówek `# Title` zaczyna nową stronę + wpis w menu |
| `POSTS_PER_PAGE` | `--posts-per-page` | `20` | Pełnych wpisów na stronę w kanale głównym |
| `TAGS_FOOTER` | `--tags-footer` | wył | `true` pokazuje stopkę tagów pod wpisem (tagi i tak są klikalne w treści) |
| `NEXT_PREV` | `--no-next-prev` | wł | `false` ukrywa nawigację Poprzedni/Następny |
| `TELEGRAM_LINK` | `--no-telegram-link` | wł | `false` ukrywa link „Zobacz na Telegramie" pod wpisem |
| `RSS` | `--no-rss` | wł | `false` wyłącza kanał RSS pod `/rss.xml` (z autowykrywaniem) |
| `FEDIVERSE_CREATOR` | `--fediverse-creator` | — | Mastodon `@user@instance` → podpis `fediverse:creator` + link profilu `rel="me"` |
| `SEARCH_ENGINE` | `--search-engine` | `google` | Pole wyszukiwania w nagłówku: `google` (formularz bez JS) / `duckduckgo` / `yandex` / `bing` / `none` |
| `SEARCH_URL` | `--search-url` | — | Własny prefiks URL wyszukiwania; zapytanie dołączane po Enter (nadpisuje wyszukiwarkę) |
| `TITLE_MAX_LEN` | `--title-max-len` | `200` | Maks. długość tytułu wpisu (znaki); skrócony tytuł zachowuje pełne pierwsze zdanie w treści |
| `FOOTER` | `--footer` | — | Treść stopki — zwykły tekst, Markdown lub HTML |
| `PAGES_HOST` | `--pages-host` | auto | Host dla limitu rozmiaru na stronie O stronie: `github` / `gitlab` / `none` (wykrywany z URL) |
| `DATE_FORMAT` | `--date-format` | `%Y %B %d` | Format strftime wyświetlanych dat (np. `2025 October 28`; `%Y` tylko rok) |
| `LANGUAGE` | `--language` | `en` | Język interfejsu strony (Nowsze/Starsze/Tagi/O stronie/…): `en`/`be`/`uk`/`ru`/`de`/`fr`/`zh`/`ja`/`pl`/`es`/`ko`/`ka` (gruziński). Treść wpisów pozostaje w języku kanału; daty są lokalizowane |
| `LINK_UNDERLINE` | `--link-underline` | wył | `true` podkreśla linki (domyślnie: bez podkreślenia) |
| `YOUTUBE_FACADE` | `--youtube-facade` | wył | `true` dla miniatury YouTube klik-aby-załadować bez JS (domyślnie: bezpośredni iframe) |
| `GENIUS` | `--no-genius` | wł | `false` pomija rozwiązywanie linków genius.com (pobiera stronę dla jej wideo YouTube + widżetu tekstu) |
| `TAGS_TO_PAGES` | `--tags-to-pages` | — | Tagi rozdzielone przecinkami pokazywane jako linki `#tag` w górnej nawigacji (np. `music, batumi, cooking`) |
| `BACKGROUND_DARK_COLOR` | `--background-dark-color` | `#000000` | Tło trybu ciemnego (dowolny kolor CSS) |
| `BACKGROUND_LIGHT_COLOR` | `--background-light-color` | `#ffffff` | Tło trybu jasnego |
| `CSS` | `--css` | — | Dodatkowy CSS dołączony do wbudowanego arkusza stylów |
| `THEME_REPO` | `--theme` (nazwa) | wbudowany czarny motyw | URL git zewnętrznego motywu Zola (https/ssh); automatyczny powrót przy błędzie |
| `REPO_URL` | `--repo-url` | repo tg2zola | Link „Repozytorium kodu źródłowego" na O stronie (CI automatycznie ustawia twoje repo) |

Strona **O stronie** pokazuje **awatar** kanału (pełny rozmiar), jego opis i
statystyki, **rozmiar na dysku** — z podziałem na rodzaje i, na GitHub/GitLab
Pages, udział w limicie ~1 GB opublikowanej strony u hosta (z linkiem do
dokumentacji hosta) — plus link do repo. Nagłówek pokazuje awatar jako miniaturę i
favicon; hashtagi są klikalne w treściach wpisów i tworzą strony `/tags/<tag>/`.

Pojedyncza zmienna `PAGES` może definiować kilka stron — każdy nagłówek `# Title`
zaczyna nową:

```
# Tytuł strony
Treść strony w Markdown (może zawierać surowy HTML).

# Inna strona
Więcej treści.
```

→ `/page-title/` i `/another-page/`, każda podlinkowana w nawigacji. Bez dodatkowej
zależności — Zola i tak renderuje Markdown.

## Fediverse / Mastodon

Każda strona niesie znaczniki Open Graph + Twitter Card, więc udostępniony link do
wpisu renderuje się jako karta (tytuł, opis, pierwszy obraz wpisu) w Mastodonie,
Slacku, Discordzie, X itd. Ustaw `FEDIVERSE_CREATOR` na swój uchwyt `@user@instance`,
aby dodatkowo:

- dodać podpis **`fediverse:creator`**, aby Mastodon pokazywał „*by @you@instance*"
  w podglądzie linku i linkował do twojego profilu;
- wyemitować link **`rel="me"`** do twojego profilu, abyś mógł dodać tę stronę do
  metadanych profilu Mastodon i uzyskać **zweryfikowany** (zielony) znaczek.

**Czy ludzie mogą obserwować bloga *z* Mastodona?** Nie bezpośrednio — strona
statyczna nie może być aktorem ActivityPub (to wymaga działającego serwera
mówiącego WebFinger + ActivityPub). Dwa sposoby, by mimo to umożliwić subskrypcję:

- **RSS** — każdy może obserwować `/rss.xml` w czytniku już dziś (wielu
  użytkowników Mastodona ma czytnik). Jest wbudowany i domyślnie włączony.
- **Most** — skieruj kanał RSS na most RSS→ActivityPub, taki jak
  [rss-parrot](https://rss-parrot.net/) lub [Bridgy Fed](https://fed.brid.gy/),
  które udostępniają prawdziwy `@handle`, który użytkownicy Mastodona mogą
  **obserwować**, przekazując nowe wpisy do ich osi czasu. Bez własnego serwera.

Czyli: bogate podglądy + atrybucja autora + weryfikacja profilu działają od ręki;
prawdziwe „obserwowanie z Mastodona" jest o jeden most dalej, z kanałem RSS jako
wejściem.

## Ograniczenia

Publiczny podgląd webowy to kompromis za brak potrzeby **jakiejkolwiek
autoryzacji**:

- **Reakcje/polubienia nie są udostępniane** przez `t.me/s/`. Zamiast tego
  eksportujemy **liczbę wyświetleń**. Model danych zostawia miejsce na dodanie
  prawdziwych reakcji później przez uwierzytelnione API MTProto (crate
  [`grammers`](https://codeberg.org/Lonami/grammers)), jeśli kiedyś ich zechcesz.
- **Duże wideo nie są do pobrania** — podgląd serwuje dla nich tylko obraz-plakat i
  czas trwania (krótkie/automatycznie odtwarzane wideo *są* do pobrania).
  Zarchiwizowanie samego pliku również wymagałoby API MTProto.
- **Pakietów naklejek nie da się podlinkować** — nazwa pakietu jest ładowana przez
  JavaScript Telegrama i nie ma jej w zescrapowanym HTML; naklejki są zapisywane
  jako zwykłe obrazy.
- **Plików muzycznych (dokumentów audio) nie da się pobrać** — ich URL nie ma w
  zescrapowanym HTML (są tylko notatki głosowe, z bezpośrednimi URL `.oga`). Jak
  duże wideo i naklejki, wymagałyby API MTProto. Dla załącznika, którego nie możemy
  pobrać, zachowujemy jego **nazwę pliku** (oznaczoną jako *niezarchiwizowane*),
  abyś wiedział, że istniał; wpis będący *tylko* takim odwołaniem (lub pojedynczym
  niepobieralnym plikiem) jest pomijany, a nie publikowany jako pusty.
- **YouTube** odtwarza się przez iframe `youtube.com` (aby odtworzenia liczyły się
  do historii widza) na opublikowanej stronie HTTPS; przez `file://` iframe nie
  może się załadować (YouTube potrzebuje origin). `YOUTUBE_FACADE=true` zamienia
  iframe na miniaturę klik-aby-załadować bez JS, która przynajmniej pokazuje plakat
  przez `file://`.
- **Tylko kanały publiczne**, z włączonym podglądem webowym.

## Licencja

[MIT](LICENSE) © Vitaly Zdanevich
