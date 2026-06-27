# tg2zola

🌐 [English](README.md) · [Беларуская](README.be.md) · [Українська](README.uk.md) · [Русский](README.ru.md) · **Deutsch** · [Français](README.fr.md) · [中文](README.zh.md) · [日本語](README.ja.md) · [Polski](README.pl.md) · [Español](README.es.md) · [한국어](README.ko.md) · [ქართული](README.ka.md)

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

Sichern Sie einen **öffentlichen Telegram-Kanal** als **eigenständige statische
Website mit [Zola](https://www.getzola.org/)**.

> **Auf Abruf neu generieren:** Klicken Sie oben auf das **daily build**-Badge →
> **Run workflow**, um ohne Warten auf den Zeitplan neu zu scrapen, zu bauen und
> bereitzustellen (Details im Abschnitt **„Automatisierung"**).

Das Tool liest die öffentliche Web-Vorschau (`https://t.me/s/<channel>`), lädt
alle Medien lokal herunter und generiert bei jedem Lauf einen vollständigen
Zola-Blog neu. **Kein Telegram-Bot, -Token oder -API nötig** — es liest nur die
öffentliche Webseite. Das Ergebnis hat **keine Telegram-Abhängigkeit**: Medien
liegen lokal vor, es gibt keine Embeds, und Links auf die *eigenen* Beiträge des
Kanals werden in interne relative Links umgeschrieben — so funktioniert die Seite
weiter, selbst wenn der Kanal später entfernt wird. Links, die Sie auf andere
Seiten gesetzt haben (auch auf andere Telegram-Kanäle), bleiben als normale Links
erhalten. Es ist eine Sicherung, kein Spiegel.

In Rust geschrieben: eine einzige statische Binärdatei, einfach lokal oder in CI
auszuführen.

## Was es kann

- **Vollständige Historie** — durchläuft den Kanal rückwärts über den
  `?before=`-Cursor der Vorschau bis zur ersten Nachricht und **generiert jede
  Seite** bei jedem Lauf neu.
- **Eigenständige Medien** — lädt Fotos, Videos, Audio (`.ogg/.oga/.mp3`),
  Dokumente und Sticker in das Bundle jedes Beitrags (das zugleich als Cache
  dient). Fotos werden über ihre stabile Telegram-Datei-ID adressiert, sodass das
  Bearbeiten des Texts eines Beitrags die Medien nie neu herunterlädt, während das
  **Ersetzen** eines Bildes (der Beitrag gilt dann als *bearbeitet*) die neue Datei
  holt und die alte entfernt.
- **Standardthema „echtes Schwarz"** — eingebaute Templates mit `#000` im
  Dunkelmodus über `prefers-color-scheme` (OLED-freundlich), ohne externe
  Abhängigkeit. Ein externes Thema kann mit garantiertem Fallback darübergelegt
  werden (siehe Abschnitt **„Themes"**).
- **Intelligente Videobehandlung** (in dieser Reihenfolge):
  1. angehängtes Video **+ YouTube-Link** → YouTube einbetten, Video verwerfen;
  2. direkt herunterladbares Video → lokales `<video>`;
  3. sonst → **Posterbild** + Dauer speichern (die öffentliche Seite gibt die
     Datei nicht heraus; siehe **„Einschränkungen"**).
- **Formatierung** — fett, kursiv, durchgestrichen, Code/Pre, Links und Spoiler
  werden in Markdown umgewandelt (Telegrams UTF-16-Entity-Offsets werden korrekt
  behandelt).
- **Hashtags → Tags** — `#Hashtags` werden zu Begriffen der `tags`-Taxonomie in
  Zola, sodass Sie kostenlos Tag-Seiten erhalten, während sie im Beitrag weiterhin
  als Text erscheinen.
- **Gruppierte Beiträge** — Alben werden automatisch zu einem Beitrag; Bündel von
  Nachrichten, die im selben Moment veröffentlicht wurden (z. B. beim Weiterleiten
  mehrerer auf einmal), werden zusammengefasst.
- **Selbstnavigierend** — ein Link auf eine andere Nachricht im *selben* Kanal
  wird zu einem relativen Link auf diesen Beitrag im Blog; Links auf andere Kanäle
  bleiben extern.
- **Engagement** — exportiert **Aufrufzahlen** pro Beitrag. (Reaktionen/Likes sind
  auf der öffentlichen Seite nicht verfügbar — siehe **„Einschränkungen"**.)
- **RSS-Feed** — ein standardmäßiger `/rss.xml` mit **allen Beiträgen** und vollem
  Inhalt (ein vollständiger Feed, nicht nur die neuesten Einträge), über
  `<link rel="alternate">` angekündigt, sodass Feed-Reader ihn automatisch über die
  Seiten-URL finden. Standardmäßig aktiv; deaktivierbar mit `RSS=false` /
  `--no-rss`.
- **Reiche Link-Vorschauen + Mastodon** — jede Seite gibt Open-Graph- und
  Twitter-Card-Tags aus (Titel, Beschreibung, erstes Bild des Beitrags), sodass
  geteilte Links als Karten dargestellt werden. Setzen Sie `FEDIVERSE_CREATOR`, um
  eine Autorenzeile in Mastodon-Vorschauen hinzuzufügen und die Seite in Ihrem
  Profil zu verifizieren (siehe Abschnitt **„Fediverse"**).
- **Standardmäßig kein JavaScript, offline-fähig** — Dunkelmodus und Spoiler sind
  reines CSS, und die Google-Suchbox ist standardmäßig ein einfaches `<form>`
  (kein JS). Nur eine Nicht-Google-Suchmaschine fügt einen winzigen Inline-
  Enter-Handler hinzu. `tg2zola offline <public-Ordner>` schreibt die gebaute Seite
  auf relative Links um **und** entfernt Zolas Paginierungs-Redirect-Skript, sodass
  die Offline-Kopie direkt über `file://` ohne jegliches JavaScript und ohne
  Webserver öffnet.
- **Lokalisierte UI** — die Oberfläche (Neuer/Älter/Tags/Über, die Suchbox, Daten)
  wird in einer von 12 Sprachen über `LANGUAGE` / `--language` dargestellt
  (en/be/uk/ru/de/fr/zh/ja/pl/es/ko/ka), mit lokalisierten Monats- und
  Wochentagsnamen. Der Beitragsinhalt bleibt in der Sprache des Kanals.

## Installation

Holen Sie eine statische Binärdatei für Ihre Architektur von der Seite
[Releases](../../releases) (Linux `amd64` / `arm64`, musl) oder bauen Sie aus dem
Quellcode:

```sh
cargo build --release
# Binärdatei unter target/release/tg2zola
```

Sie benötigen außerdem die Binärdatei [`zola`](https://www.getzola.org/documentation/getting-started/installation/),
um den generierten Inhalt in HTML umzuwandeln.

## Verwendung

```sh
# Die Zola-Seite generieren (beim ersten Lauf werden config + Templates erstellt):
tg2zola --channel durov --site site --init-site

# Das statische HTML bauen:
zola --root site build       # Ausgabe in site/public/

# (optional) Ohne Webserver direkt von der Festplatte ansehbar machen:
tg2zola offline site/public  # dann site/public/index.html über file:// öffnen
```

Schneller lokaler Test (eine Seite, ~20 Nachrichten):

```sh
tg2zola --channel <name> --site /tmp/site --init-site --max-pages 1
```

Alle Optionen stehen in [`tg2zola.toml`](tg2zola.toml) (CLI-Flags überschreiben sie):

```sh
tg2zola --config tg2zola.toml
```

Führen Sie `tg2zola --help` für die vollständige Flag-Liste aus.

## Wie es aufgebaut ist

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

Jeder Beitrag wird zu einem Zola-Page-Bundle —
`content/posts/<date>-<id>/index.md` mit TOML-Front-Matter und den Mediendateien
daneben. `config.toml` und die eingebauten Templates werden bei jedem Lauf
deterministisch neu generiert, und `write_site` gleicht den Baum ab: schreibt das
gesamte Markdown neu, behält bereits zwischengespeicherte Medien und entfernt
gelöschte Beiträge und veraltete Dateien.

## Automatisierung (GitHub Actions)

Zwei Workflows sind enthalten:

- **[`daily.yml`](.github/workflows/daily.yml)** — läuft einmal täglich (und auf
  Abruf): die vorherige Seite aus dem `blog`-Branch (dem Medien-Cache)
  wiederherstellen → scrapen + neu generieren → `zola build` → **auf GitHub Pages
  bereitstellen** (das veröffentlichte Ergebnis) → die aktualisierte Seite zurück
  in den **`blog`-Branch** committen.
- **[`release.yml`](.github/workflows/release.yml)** — bei jedem gepushten
  `v*`-Tag werden statische `amd64`- + `arm64`-Binärdateien (musl)
  cross-kompiliert und in das GitHub-Release hochgeladen.

Zum Aktivieren der Veröffentlichung: im Repo **Settings → Pages → Build and
deployment → Source: GitHub Actions**. Keine Secrets nötig — alles läuft über
öffentliches Scraping. Die veröffentlichte Seite ist immer **GitHub Pages**; der
`blog`-Branch ist eine dauerhafte Kopie und beeinflusst nie, was Besucher sehen.

### Jetzt neu generieren (nicht auf den täglichen Lauf warten)

`daily.yml` hat `workflow_dispatch` aktiviert, sodass Sie auf Abruf ein frisches
Scrapen + Bauen + Bereitstellen auslösen können — es führt genau dieselben
Schritte aus wie der geplante Lauf:

- **Im Browser:** öffnen Sie **[Actions → „daily" → Run workflow](../../actions/workflows/daily.yml)**
  und klicken Sie auf den grünen Knopf **Run workflow**. (Fügen Sie das
  Status-Badge oben zu Ihrer README hinzu, für Zugriff mit einem Klick.)
- **Vom Terminal:** `gh workflow run daily.yml` (GitHub CLI), dann `gh run watch`,
  um den Lauf zu verfolgen.

**Welcher Kanal?** Der Kanal ist nicht eingecheckt, also legt jede Bereitstellung
ihren eigenen fest. Setzen Sie eine Repository-**Variable** `CHANNEL` (Settings →
Secrets and variables → Actions → Variables) auf den öffentlichen Kanalnamen — es
ist eine *Variable*, kein Secret, da der Kanal öffentlich ist. (Oder
kommentieren Sie `channel = "…"` in [`tg2zola.toml`](tg2zola.toml) in Ihrem Fork
ein.) `THEME_REPO` funktioniert ebenfalls als Variable.

### Der `blog`-Branch (Archiv + Cache)

Jeder Lauf committet die generierte Seite (Markdown + Medien + eingebaute
Templates — alles außer dem gebauten `public/` und einem externen Thema) in einen
`blog`-Branch und lässt `main` nur mit Code zurück. Er hat eine Doppelfunktion:

- **Cache** — der nächste Lauf stellt die Medien daraus wieder her, statt sie
  erneut herunterzuladen.
- **Dauerhaftes Archiv** — eine vollständige, baubare Zola-Seite, die Sie überall
  klonen und spiegeln können, sodass die Sicherung nicht an eine Plattform gebunden
  ist:

```sh
git clone -b blog https://github.com/<you>/<repo> my-archive
zola --root my-archive/site serve          # offline durchsehen

git -C my-archive remote add gitlab git@gitlab.com:<you>/<repo>.git
git -C my-archive push gitlab blog          # anderswo spiegeln
```

Medien werden als einfache Git-Blobs committet; für sehr große Kanäle erwägen Sie
Git LFS oder gelegentliches Zusammenfassen der Historie.

### Themes

Standard ist das eingebaute **echte-Schwarz**-Thema — null externe
Abhängigkeiten, sodass die Seite immer baut. Um ein externes
[Zola-Thema](https://www.getzola.org/themes/) zu verwenden, setzen Sie eine
Repository-**Variable** `THEME_REPO` (Settings → Secrets and variables → Actions →
Variables) auf dessen Git-URL (https oder ssh mit Deploy-Key). Der Workflow klont
es und baut damit — und **wenn das Thema fehlt oder sein Build fehlschlägt, fällt
es automatisch auf die eingebauten Templates zurück**, sodass ein Themenproblem den
Blog nie offline nehmen kann. Beachten Sie, dass externe Themes ein bestimmtes
Inhaltslayout erwarten, sodass nicht jedes Thema sofort kompatibel ist.

Ein Release erstellen:

```sh
git tag v0.1.0 && git push origin v0.1.0
```

## Konfiguration

Alles ist über eine Repository-**Variable** (Settings → Secrets and variables →
Actions → **Variables**) für den GitHub-Actions-Ablauf konfigurierbar, oder über
das entsprechende CLI-Flag / den [`tg2zola.toml`](tg2zola.toml)-Schlüssel beim
lokalen Ausführen. Das sind *Variablen*, keine Secrets — alles ist öffentlich.

| Repo-Variable | CLI-Flag | Standard | Was sie tut |
|---|---|---|---|
| `CHANNEL` | `--channel` | **erforderlich** | Zu synchronisierender öffentlicher Kanal |
| `TITLE` | `--title` | Kanalname | Blog-Titel (Kopf + `<title>`) |
| `ABOUT` | `--about` | Beschreibung + Statistik + Repo-Link | Eigenes HTML für den Body der Über-Seite |
| `PAGES` | `--pages` | — | Zusätzliche Seiten: Markdown, jede `# Title`-Überschrift beginnt eine neue Seite + Navi-Eintrag |
| `POSTS_PER_PAGE` | `--posts-per-page` | `20` | Volle Beiträge pro Seite im Home-Feed |
| `TAGS_FOOTER` | `--tags-footer` | aus | `true` zeigt die Tag-Fußzeile pro Beitrag (Tags sind im Body ohnehin anklickbar) |
| `NEXT_PREV` | `--no-next-prev` | an | `false` blendet die Vor/Zurück-Navigation aus |
| `TELEGRAM_LINK` | `--no-telegram-link` | an | `false` blendet den „Auf Telegram ansehen"-Link pro Beitrag aus |
| `RSS` | `--no-rss` | an | `false` deaktiviert den RSS-Feed unter `/rss.xml` (mit Autodiscovery) |
| `FEDIVERSE_CREATOR` | `--fediverse-creator` | — | Mastodon `@user@instance` → `fediverse:creator`-Zeile + `rel="me"`-Profillink |
| `SEARCH_ENGINE` | `--search-engine` | `google` | Suchbox im Kopf: `google` (JS-freies Formular) / `duckduckgo` / `yandex` / `bing` / `none` |
| `SEARCH_URL` | `--search-url` | — | Eigener Such-URL-Präfix; die Eingabe wird per Enter angehängt (überschreibt die Engine) |
| `TITLE_MAX_LEN` | `--title-max-len` | `200` | Max. Beitragstitel-Länge (Zeichen); ein gekürzter Titel behält seinen vollen ersten Satz im Body |
| `FOOTER` | `--footer` | — | Fußzeilen-Inhalt — reiner Text, Markdown oder HTML |
| `PAGES_HOST` | `--pages-host` | auto | Host für das Größenlimit auf der Über-Seite: `github` / `gitlab` / `none` (aus der URL erkannt) |
| `DATE_FORMAT` | `--date-format` | `%Y %B %d` | strftime-Format für angezeigte Daten (z. B. `2025 October 28`; `%Y` nur Jahr) |
| `LANGUAGE` | `--language` | `en` | UI-Sprache der Oberfläche (Neuer/Älter/Tags/Über/…): `en`/`be`/`uk`/`ru`/`de`/`fr`/`zh`/`ja`/`pl`/`es`/`ko`/`ka` (Georgisch). Beitragsinhalt bleibt in der Kanalsprache; Daten werden lokalisiert |
| `LINK_UNDERLINE` | `--link-underline` | aus | `true` unterstreicht Links (Standard: keine Unterstreichung) |
| `YOUTUBE_FACADE` | `--youtube-facade` | aus | `true` für ein JS-freies Klick-zum-Laden-YouTube-Thumbnail (Standard: direkter iframe) |
| `GENIUS` | `--no-genius` | an | `false` überspringt das Auflösen von genius.com-Links (lädt die Seite für ihr YouTube-Video + Lyrics-Widget) |
| `TAGS_TO_PAGES` | `--tags-to-pages` | — | Kommagetrennte Tags als `#tag`-Links in der oberen Navi (z. B. `music, batumi, cooking`) |
| `BACKGROUND_DARK_COLOR` | `--background-dark-color` | `#000000` | Hintergrund im Dunkelmodus (beliebige CSS-Farbe) |
| `BACKGROUND_LIGHT_COLOR` | `--background-light-color` | `#ffffff` | Hintergrund im Hellmodus |
| `CSS` | `--css` | — | Zusätzliches CSS, an das eingebaute Stylesheet angehängt |
| `THEME_REPO` | `--theme` (Name) | eingebautes schwarzes Thema | Git-URL eines externen Zola-Themas (https/ssh); fällt bei Fehler automatisch zurück |
| `REPO_URL` | `--repo-url` | tg2zola-Repo | „Quellcode-Repository"-Link auf Über (CI setzt automatisch Ihr Repo) |

Die **Über**-Seite zeigt den Kanal-**Avatar** (volle Größe), seine Beschreibung
und Statistik, die **Größe auf der Festplatte** — mit der Aufschlüsselung nach Art
und, auf GitHub/GitLab Pages, dem Anteil am ~1-GB-Limit des Hosts für
veröffentlichte Seiten (mit Link zur Host-Doku) — plus den Repo-Link. Der Kopf
zeigt den Avatar als Thumbnail und Favicon; Hashtags sind in Beitragstexten
anklickbar und erzeugen `/tags/<tag>/`-Seiten.

Eine einzige `PAGES`-Variable kann mehrere Seiten definieren — jede
`# Title`-Überschrift beginnt eine neue:

```
# Seitentitel
Seiteninhalt in Markdown (darf rohes HTML enthalten).

# Andere Seite
Mehr Inhalt.
```

→ `/page-title/` und `/another-page/`, jede in der Navi verlinkt. Keine zusätzliche
Abhängigkeit — Zola rendert Markdown ohnehin.

## Fediverse / Mastodon

Jede Seite trägt Open-Graph- + Twitter-Card-Tags, sodass ein geteilter
Beitragslink als Karte dargestellt wird (Titel, Beschreibung, erstes Bild des
Beitrags) in Mastodon, Slack, Discord, X usw. Setzen Sie `FEDIVERSE_CREATOR` auf
Ihren `@user@instance`-Handle, um zusätzlich:

- eine **`fediverse:creator`**-Zeile hinzuzufügen, sodass Mastodon „*by
  @you@instance*" in der Link-Vorschau zeigt und auf Ihr Profil verlinkt;
- einen **`rel="me"`**-Link zu Ihrem Profil auszugeben, sodass Sie diese Seite in
  die Metadaten Ihres Mastodon-Profils aufnehmen und das **verifizierte** (grüne)
  Häkchen erhalten können.

**Können Leute dem Blog *von* Mastodon aus folgen?** Nicht direkt — eine statische
Seite kann kein ActivityPub-Akteur sein (das erfordert einen Live-Server mit
WebFinger + ActivityPub). Zwei Wege, damit Leute trotzdem abonnieren können:

- **RSS** — jeder kann schon heute `/rss.xml` in einem Feed-Reader folgen (viele
  Mastodon-Nutzer haben einen). Das ist eingebaut und standardmäßig aktiv.
- **Eine Bridge** — leiten Sie den RSS-Feed an eine RSS→ActivityPub-Bridge wie
  [rss-parrot](https://rss-parrot.net/) oder [Bridgy Fed](https://fed.brid.gy/),
  die einen echten `@handle` bereitstellen, dem Mastodon-Nutzer **folgen** können,
  und neue Beiträge in ihre Timeline einspeisen. Kein eigener Server nötig.

Also: reiche Vorschauen + Autorenangabe + Profilverifizierung funktionieren sofort;
echtes „Folgen von Mastodon" ist nur eine Bridge entfernt, mit dem RSS-Feed als
Eingabe.

## Einschränkungen

Die öffentliche Web-Vorschau ist der Kompromiss dafür, **keine Authentifizierung**
zu brauchen:

- **Reaktionen/Likes werden nicht ausgegeben** von `t.me/s/`. Wir exportieren
  stattdessen **Aufrufzahlen**. Das Datenmodell lässt Raum, später echte Reaktionen
  über die authentifizierte MTProto-API (das
  [`grammers`](https://codeberg.org/Lonami/grammers)-Crate) hinzuzufügen, falls Sie
  sie je wollen.
- **Große Videos sind nicht herunterladbar** — die Vorschau liefert dafür nur ein
  Posterbild und die Dauer (kurze/automatisch abspielende Videos *sind*
  herunterladbar). Das Archivieren der eigentlichen Datei würde ebenfalls die
  MTProto-API erfordern.
- **Sticker-Packs sind nicht verlinkbar** — der Pack-Name wird von Telegrams
  JavaScript geladen und ist nicht im gescrapten HTML; Sticker werden als einfache
  Bilder gespeichert.
- **Musikdateien (Audio-Dokumente) sind nicht herunterladbar** — ihre URL ist
  nicht im gescrapten HTML (nur Sprachnachrichten, mit direkten `.oga`-URLs). Wie
  große Videos und Sticker bräuchten sie die MTProto-API. Für einen Anhang, den wir
  nicht holen können, behalten wir seinen **Dateinamen** (als *nicht archiviert*
  markiert), damit Sie wissen, dass er existierte; ein Beitrag, der *nur* aus einem
  solchen Verweis (oder einer einzelnen nicht ladbaren Datei) besteht, wird
  übersprungen statt leer veröffentlicht.
- **YouTube** spielt über einen `youtube.com`-iframe (sodass Wiedergaben in den
  Verlauf des Betrachters zählen) auf der veröffentlichten HTTPS-Seite; über
  `file://` kann der iframe nicht laden (YouTube braucht einen Origin).
  `YOUTUBE_FACADE=true` tauscht den iframe gegen ein JS-freies
  Klick-zum-Laden-Thumbnail, das über `file://` zumindest das Posterbild zeigt.
- **Nur öffentliche Kanäle**, mit aktivierter Web-Vorschau.

## Lizenz

[MIT](LICENSE) © Vitaly Zdanevich
