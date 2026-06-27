# tg2zola

🌐 [English](README.md) · [Беларуская](README.be.md) · [Українська](README.uk.md) · [Русский](README.ru.md) · [Deutsch](README.de.md) · **Français** · [中文](README.zh.md) · [日本語](README.ja.md) · [Polski](README.pl.md) · [Español](README.es.md) · [한국어](README.ko.md) · [ქართული](README.ka.md)

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

Sauvegardez une **chaîne Telegram publique** dans un **site statique autonome avec
[Zola](https://www.getzola.org/)**.

> **Régénérer à la demande :** cliquez sur le badge **daily build** ci-dessus →
> **Run workflow** pour re-scraper, reconstruire et redéployer sans attendre la
> planification (détails dans la section **« Automatisation »**).

L'outil lit l'aperçu web public (`https://t.me/s/<channel>`), télécharge tous les
médias en local et régénère un blog Zola complet à chaque exécution. **Aucun bot,
jeton ni API Telegram n'est nécessaire** — il ne lit que la page web publique. Le
résultat n'a **aucune dépendance à Telegram** : les médias sont locaux, il n'y a
pas d'embeds, et les liens vers les *propres* publications de la chaîne sont
réécrits en liens relatifs internes — le site continue donc de fonctionner même si
la chaîne est supprimée plus tard. Les liens que vous avez mis vers d'autres sites
(y compris d'autres chaînes Telegram) sont conservés comme des liens normaux.
C'est une sauvegarde, pas un miroir.

Écrit en Rust : un seul binaire statique, facile à exécuter en local ou en CI.

## Ce qu'il fait

- **Historique complet** — parcourt la chaîne à rebours via le curseur `?before=`
  de l'aperçu jusqu'au tout premier message, et **régénère chaque page** à chaque
  exécution.
- **Médias autonomes** — télécharge photos, vidéos, audio (`.ogg/.oga/.mp3`),
  documents et stickers dans le bundle de chaque publication (qui sert aussi de
  cache). Les photos sont adressées par leur identifiant de fichier Telegram
  stable, donc modifier le texte d'une publication ne re-télécharge jamais ses
  médias, tandis que **remplacer** une image (la publication est alors marquée
  *modifiée*) récupère le nouveau fichier et supprime l'ancien.
- **Thème « noir absolu » par défaut** — templates intégrés stylés `#000` en mode
  sombre via `prefers-color-scheme` (bon pour l'OLED), sans dépendance externe. Un
  thème externe peut se superposer avec un repli garanti (voir la section
  **« Thèmes »**).
- **Gestion vidéo intelligente** (par ordre de priorité) :
  1. vidéo jointe **+ lien YouTube** → intégrer YouTube, abandonner la vidéo ;
  2. vidéo directement téléchargeable → `<video>` local ;
  3. sinon → enregistrer l'**image d'aperçu** + la durée (la page publique
     n'expose pas le fichier ; voir **« Limitations »**).
- **Formatage** — gras, italique, barré, code/pré, liens et spoilers sont
  convertis en Markdown (les décalages d'entités UTF-16 de Telegram sont gérés
  correctement).
- **Hashtags → tags** — les `#hashtags` deviennent des termes de la taxonomie
  `tags` de Zola, vous obtenez donc des pages de tags gratuitement, tout en
  restant du texte dans la publication.
- **Publications groupées** — les albums deviennent automatiquement une seule
  publication ; les rafales de messages publiés au même instant (par ex. en
  transférant plusieurs d'un coup) sont fusionnées.
- **Auto-navigation** — un lien vers un autre message de la *même* chaîne devient
  un lien relatif vers cette publication dans le blog ; les liens vers d'autres
  chaînes restent externes.
- **Engagement** — exporte le **nombre de vues** par publication. (Les
  réactions/j'aime ne sont pas disponibles sur la page publique — voir
  **« Limitations »**.)
- **Flux RSS** — un `/rss.xml` standard avec **toutes les publications** et leur
  contenu complet (un flux complet, pas seulement les éléments récents), annoncé
  via `<link rel="alternate">` pour que les lecteurs le découvrent automatiquement
  depuis l'URL du site. Activé par défaut ; désactivable avec `RSS=false` /
  `--no-rss`.
- **Aperçus de liens riches + Mastodon** — chaque page émet des balises Open Graph
  et Twitter Card (titre, description, première image de la publication), pour que
  les liens partagés s'affichent en cartes. Définissez `FEDIVERSE_CREATOR` pour
  ajouter une signature d'auteur dans les aperçus Mastodon et vérifier le site sur
  votre profil (voir la section **« Fediverse »**).
- **Sans JavaScript par défaut, prêt hors-ligne** — le mode sombre et les spoilers
  sont en CSS uniquement, et la barre de recherche Google par défaut est un simple
  `<form>` (sans JS). Seul un moteur non-Google ajoute un minuscule gestionnaire
  Entrée en ligne. `tg2zola offline <dossier-public>` réécrit le site construit en
  liens relatifs **et** supprime le script de redirection de pagination de Zola,
  pour que la copie hors-ligne s'ouvre directement depuis `file://` sans aucun
  JavaScript ni serveur web.
- **Interface localisée** — l'habillage du site (Plus récent/Plus ancien/Tags/À
  propos, la barre de recherche, les dates) s'affiche dans l'une des 12 langues via
  `LANGUAGE` / `--language` (en/be/uk/ru/de/fr/zh/ja/pl/es/ko/ka), avec les noms de
  mois et de jours localisés. Le contenu des publications reste dans la langue de
  la chaîne.

## Installation

Récupérez un binaire statique pour votre architecture depuis la page
[Releases](../../releases) (Linux `amd64` / `arm64`, musl) ou compilez depuis les
sources :

```sh
cargo build --release
# binaire dans target/release/tg2zola
```

Vous avez aussi besoin du binaire [`zola`](https://www.getzola.org/documentation/getting-started/installation/)
pour transformer le contenu généré en HTML.

## Utilisation

```sh
# Générer le site Zola (crée config + templates au premier lancement) :
tg2zola --channel durov --site site --init-site

# Construire le HTML statique :
zola --root site build       # sortie dans site/public/

# (optionnel) Le rendre consultable SANS serveur web, directement depuis le disque :
tg2zola offline site/public  # puis ouvrez site/public/index.html via file://
```

Test local rapide (une page, ~20 messages) :

```sh
tg2zola --channel <name> --site /tmp/site --init-site --max-pages 1
```

Toutes les options sont dans [`tg2zola.toml`](tg2zola.toml) (les flags CLI le
remplacent) :

```sh
tg2zola --config tg2zola.toml
```

Lancez `tg2zola --help` pour la liste complète des flags.

## Comment c'est câblé

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

Chaque publication devient un bundle de page Zola —
`content/posts/<date>-<id>/index.md` avec un front-matter TOML et ses fichiers
médias à côté. `config.toml` et les templates intégrés sont régénérés de façon
déterministe à chaque exécution, et `write_site` réconcilie l'arbre : réécrit tout
le Markdown, conserve les médias déjà en cache et supprime les publications
effacées et les fichiers obsolètes.

## Automatisation (GitHub Actions)

Deux workflows sont inclus :

- **[`daily.yml`](.github/workflows/daily.yml)** — s'exécute une fois par jour (et
  à la demande) : restaurer le site précédent depuis la branche `blog` (le cache
  des médias) → scraper + régénérer → `zola build` → **déployer sur GitHub Pages**
  (le résultat publié) → committer le site rafraîchi dans la branche **`blog`**.
- **[`release.yml`](.github/workflows/release.yml)** — à chaque tag `v*` poussé,
  compile en croisé des binaires statiques `amd64` + `arm64` (musl) et les
  téléverse dans la Release GitHub.

Pour activer la publication : dans le dépôt, **Settings → Pages → Build and
deployment → Source: GitHub Actions**. Aucun secret requis — tout passe par le
scraping public. Le site publié est toujours **GitHub Pages** ; la branche `blog`
est une copie durable et n'affecte jamais ce que voient les visiteurs.

### Régénérer maintenant (sans attendre l'exécution quotidienne)

`daily.yml` a `workflow_dispatch` activé, vous pouvez donc déclencher un scraping +
build + redéploiement frais à la demande — il exécute exactement les mêmes étapes
que l'exécution planifiée :

- **Dans le navigateur :** ouvrez **[Actions → « daily » → Run workflow](../../actions/workflows/daily.yml)**
  et cliquez sur le bouton vert **Run workflow**. (Ajoutez le badge de statut
  ci-dessus à votre README pour un accès en un clic.)
- **Depuis le terminal :** `gh workflow run daily.yml` (GitHub CLI), puis
  `gh run watch` pour le suivre.

**Quelle chaîne ?** La chaîne n'est pas committée, donc chaque déploiement définit
la sienne. Définissez une **variable** de dépôt `CHANNEL` (Settings → Secrets and
variables → Actions → Variables) avec le nom de la chaîne publique — c'est une
*variable*, pas un secret, car la chaîne est publique. (Ou décommentez
`channel = "…"` dans [`tg2zola.toml`](tg2zola.toml) dans votre fork.) `THEME_REPO`
fonctionne aussi comme variable.

### La branche `blog` (archive + cache)

Chaque exécution committe le site généré (Markdown + médias + templates intégrés —
tout sauf le `public/` construit et un éventuel thème externe) dans une branche
`blog`, laissant `main` avec uniquement le code. Elle a un double rôle :

- **Cache** — l'exécution suivante en restaure les médias au lieu de les
  re-télécharger.
- **Archive durable** — un site Zola complet et constructible que vous pouvez
  cloner et mettre en miroir n'importe où, pour que la sauvegarde ne soit pas liée
  à une seule plateforme :

```sh
git clone -b blog https://github.com/<you>/<repo> my-archive
zola --root my-archive/site serve          # le parcourir hors-ligne

git -C my-archive remote add gitlab git@gitlab.com:<you>/<repo>.git
git -C my-archive push gitlab blog          # le mettre en miroir ailleurs
```

Les médias sont committés en blobs git ordinaires ; pour de très grandes chaînes,
envisagez Git LFS ou un compactage occasionnel de l'historique.

### Thèmes

Le défaut est le thème intégré **noir absolu** — zéro dépendance externe, donc le
site se construit toujours. Pour utiliser un [thème Zola](https://www.getzola.org/themes/)
externe, définissez une **variable** de dépôt `THEME_REPO` (Settings → Secrets and
variables → Actions → Variables) avec son URL git (https, ou ssh avec une clé de
déploiement). Le workflow le clone et construit avec — et **si le thème est absent
ou que sa construction échoue, il revient automatiquement aux templates intégrés**,
donc un problème de thème ne peut jamais mettre le blog hors-ligne. Notez que les
thèmes externes attendent une certaine structure de contenu, donc tous les thèmes
ne sont pas compatibles d'emblée.

Publier une release :

```sh
git tag v0.1.0 && git push origin v0.1.0
```

## Configuration

Tout est configurable via une **variable** de dépôt (Settings → Secrets and
variables → Actions → **Variables**) pour le flux GitHub Actions, ou via le flag
CLI / la clé [`tg2zola.toml`](tg2zola.toml) équivalents en local. Ce sont des
*variables*, pas des secrets — tout est public.

| Variable de dépôt | Flag CLI | Défaut | Ce que ça fait |
|---|---|---|---|
| `CHANNEL` | `--channel` | **requis** | Chaîne publique à synchroniser |
| `TITLE` | `--title` | nom de la chaîne | Titre du blog (en-tête + `<title>`) |
| `ABOUT` | `--about` | description + stats + lien du dépôt | HTML personnalisé pour le corps de la page À propos |
| `PAGES` | `--pages` | — | Pages supplémentaires : Markdown, chaque titre `# Title` commence une nouvelle page + entrée de menu |
| `POSTS_PER_PAGE` | `--posts-per-page` | `20` | Publications complètes par page dans le fil d'accueil |
| `TAGS_FOOTER` | `--tags-footer` | off | `true` affiche le pied de page de tags par publication (les tags sont cliquables dans le corps de toute façon) |
| `NEXT_PREV` | `--no-next-prev` | on | `false` masque la navigation Précédent/Suivant |
| `TELEGRAM_LINK` | `--no-telegram-link` | on | `false` masque le lien « Voir sur Telegram » par publication |
| `RSS` | `--no-rss` | on | `false` désactive le flux RSS à `/rss.xml` (avec autodécouverte) |
| `FEDIVERSE_CREATOR` | `--fediverse-creator` | — | Mastodon `@user@instance` → signature `fediverse:creator` + lien de profil `rel="me"` |
| `SEARCH_ENGINE` | `--search-engine` | `google` | Barre de recherche d'en-tête : `google` (formulaire sans JS) / `duckduckgo` / `yandex` / `bing` / `none` |
| `SEARCH_URL` | `--search-url` | — | Préfixe d'URL de recherche personnalisé ; la requête est ajoutée sur Entrée (remplace le moteur) |
| `TITLE_MAX_LEN` | `--title-max-len` | `200` | Longueur max du titre (caractères) ; un titre tronqué garde sa première phrase complète dans le corps |
| `FOOTER` | `--footer` | — | Contenu du pied de page — texte brut, Markdown ou HTML |
| `PAGES_HOST` | `--pages-host` | auto | Hôte pour la limite de taille de la page À propos : `github` / `gitlab` / `none` (détecté depuis l'URL) |
| `DATE_FORMAT` | `--date-format` | `%Y %B %d` | Format strftime des dates affichées (par ex. `2025 October 28` ; `%Y` pour l'année seule) |
| `LANGUAGE` | `--language` | `en` | Langue de l'interface du site (Plus récent/Plus ancien/Tags/À propos/…) : `en`/`be`/`uk`/`ru`/`de`/`fr`/`zh`/`ja`/`pl`/`es`/`ko`/`ka` (géorgien). Le contenu des publications reste dans la langue de la chaîne ; les dates sont localisées |
| `LINK_UNDERLINE` | `--link-underline` | off | `true` souligne les liens (par défaut : pas de soulignement) |
| `YOUTUBE_FACADE` | `--youtube-facade` | off | `true` pour une miniature YouTube clic-pour-charger sans JS (par défaut : iframe direct) |
| `GENIUS` | `--no-genius` | on | `false` ignore la résolution des liens genius.com (récupère la page pour sa vidéo YouTube + widget de paroles) |
| `TAGS_TO_PAGES` | `--tags-to-pages` | — | Tags séparés par des virgules affichés en liens `#tag` dans la nav supérieure (par ex. `music, batumi, cooking`) |
| `BACKGROUND_DARK_COLOR` | `--background-dark-color` | `#000000` | Fond du mode sombre (toute couleur CSS) |
| `BACKGROUND_LIGHT_COLOR` | `--background-light-color` | `#ffffff` | Fond du mode clair |
| `CSS` | `--css` | — | CSS supplémentaire ajouté à la feuille de style intégrée |
| `THEME_REPO` | `--theme` (nom) | thème noir intégré | URL git d'un thème Zola externe (https/ssh) ; repli automatique en cas d'échec |
| `REPO_URL` | `--repo-url` | dépôt tg2zola | Lien « Dépôt du code source » sur À propos (la CI met automatiquement votre dépôt) |

La page **À propos** montre l'**avatar** de la chaîne (taille réelle), sa
description et ses stats, la **taille sur le disque** — avec la répartition par
type et, sur GitHub/GitLab Pages, la part de la limite ~1 Go du site publié de
l'hôte (avec un lien vers la doc de l'hôte) — plus le lien du dépôt. L'en-tête
montre l'avatar en miniature et en favicon ; les hashtags sont cliquables dans le
corps des publications et produisent des pages `/tags/<tag>/`.

Une seule variable `PAGES` peut définir plusieurs pages — chaque titre `# Title`
en commence une nouvelle :

```
# Titre de la page
Contenu de la page en Markdown (peut contenir du HTML brut).

# Autre page
Plus de contenu.
```

→ `/page-title/` et `/another-page/`, chacune liée dans la nav. Aucune dépendance
supplémentaire — Zola rend déjà le Markdown.

## Fediverse / Mastodon

Chaque page porte des balises Open Graph + Twitter Card, donc un lien de
publication partagé s'affiche en carte (titre, description, première image de la
publication) dans Mastodon, Slack, Discord, X, etc. Définissez `FEDIVERSE_CREATOR`
sur votre identifiant `@user@instance` pour aussi :

- ajouter une signature **`fediverse:creator`**, pour que Mastodon affiche « *by
  @you@instance* » sur l'aperçu du lien et renvoie vers votre profil ;
- émettre un lien **`rel="me"`** vers votre profil, pour que vous puissiez ajouter
  ce site aux métadonnées de votre profil Mastodon et obtenir la coche
  **vérifiée** (verte).

**Les gens peuvent-ils suivre le blog *depuis* Mastodon ?** Pas directement — un
site statique ne peut pas être un acteur ActivityPub (cela demande un serveur actif
parlant WebFinger + ActivityPub). Deux façons de permettre quand même de
s'abonner :

- **RSS** — n'importe qui peut suivre `/rss.xml` dans un lecteur de flux dès
  aujourd'hui (beaucoup d'utilisateurs Mastodon en gardent un). C'est intégré et
  activé par défaut.
- **Une passerelle** — pointez le flux RSS vers une passerelle RSS→ActivityPub
  comme [rss-parrot](https://rss-parrot.net/) ou [Bridgy Fed](https://fed.brid.gy/),
  qui exposent un vrai `@handle` que les utilisateurs Mastodon peuvent **suivre**,
  relayant les nouvelles publications dans leur fil. Aucun serveur personnel requis.

Donc : aperçus riches + attribution d'auteur + vérification de profil fonctionnent
d'emblée ; le vrai « suivi depuis Mastodon » est à une passerelle près, avec le
flux RSS comme entrée.

## Limitations

L'aperçu web public est le compromis pour ne nécessiter **aucune
authentification** :

- **Les réactions/j'aime ne sont pas exposés** par `t.me/s/`. Nous exportons les
  **nombres de vues** à la place. Le modèle de données laisse de la place pour
  ajouter de vraies réactions plus tard via l'API MTProto authentifiée (le crate
  [`grammers`](https://codeberg.org/Lonami/grammers)) si vous le souhaitez un jour.
- **Les grandes vidéos ne sont pas téléchargeables** — l'aperçu ne sert qu'une
  image d'aperçu et la durée pour elles (les vidéos courtes/à lecture automatique
  *sont* téléchargeables). Archiver le fichier réel demanderait aussi l'API
  MTProto.
- **Les packs de stickers ne sont pas liables** — le nom du pack est chargé par le
  JavaScript de Telegram et n'est pas dans le HTML scrapé ; les stickers sont
  enregistrés comme de simples images.
- **Les fichiers musicaux (documents audio) ne sont pas téléchargeables** — leur
  URL n'est pas dans le HTML scrapé (seules les notes vocales, avec des URL `.oga`
  directes, le sont). Comme les grandes vidéos et les stickers, ils demanderaient
  l'API MTProto. Pour une pièce jointe que nous ne pouvons pas récupérer, nous
  gardons son **nom de fichier** (marqué *non archivé*) pour que vous sachiez
  qu'elle existait ; une publication composée *uniquement* d'une telle référence
  (ou d'un seul fichier non téléchargeable) est ignorée plutôt que publiée vide.
- **YouTube** se lit via un iframe `youtube.com` (pour que les lectures comptent
  dans l'historique du spectateur) sur le site HTTPS publié ; via `file://`,
  l'iframe ne peut pas charger (YouTube a besoin d'une origine). `YOUTUBE_FACADE=true`
  remplace l'iframe par une miniature clic-pour-charger sans JS, qui montre au
  moins l'image d'aperçu via `file://`.
- **Chaînes publiques uniquement**, avec l'aperçu web activé.

## Licence

[MIT](LICENSE) © Vitaly Zdanevich
