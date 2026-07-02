# tg2zola

🌐 [English](README.md) · [Беларуская](README.be.md) · [Українська](README.uk.md) · [Русский](README.ru.md) · [Deutsch](README.de.md) · [Français](README.fr.md) · [中文](README.zh.md) · [日本語](README.ja.md) · [Polski](README.pl.md) · **Español** · [한국어](README.ko.md) · [ქართული](README.ka.md) · [हिन्दी](README.hi.md)

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

Haz una copia de seguridad de un **canal público de Telegram** en un **sitio
estático autónomo con [Zola](https://www.getzola.org/)**.

> **Regenerar bajo demanda:** haz clic en la insignia **daily build** de arriba →
> **Run workflow** para volver a hacer scraping, compilar y desplegar sin esperar a
> la programación (detalles en la sección **«Automatización»**).

La herramienta lee la vista previa web pública (`https://t.me/s/<channel>`),
descarga todos los medios localmente y regenera un blog Zola completo en cada
ejecución. **No se necesita ningún bot, token ni API de Telegram** — solo lee la
página web pública. El resultado **no tiene dependencia de Telegram**: los medios
son locales, no hay incrustaciones, y los enlaces a las publicaciones *propias* del
canal se reescriben como enlaces relativos internos — así el sitio sigue
funcionando aunque el canal se elimine más adelante. Los enlaces que escribiste a
otros sitios (incluidos otros canales de Telegram) se conservan como enlaces
normales. Es una copia de seguridad, no un espejo.

Escrito en Rust: un único binario estático, fácil de ejecutar en local o en CI.

## Qué hace

- **Historial completo** — recorre el canal hacia atrás mediante el cursor
  `?before=` de la vista previa hasta el primer mensaje, y **regenera cada página**
  en cada ejecución.
- **Medios autónomos** — descarga fotos, vídeos, audio (`.ogg/.oga/.mp3`),
  documentos y stickers en el bundle de cada publicación (que a la vez sirve de
  caché). Las fotos se direccionan por su id de archivo estable de Telegram, así
  que editar el texto de una publicación nunca vuelve a descargar sus medios,
  mientras que **reemplazar** una imagen (la publicación queda entonces como
  *editada*) obtiene el archivo nuevo y elimina el viejo.
- **Tema «negro absoluto» por defecto** — plantillas integradas con estilo `#000`
  en modo oscuro vía `prefers-color-scheme` (bueno para OLED), sin dependencia
  externa. Se puede superponer un tema externo con repliegue garantizado (ver la
  sección **«Temas»**).
- **Manejo inteligente de vídeo** (por orden de prioridad):
  1. vídeo adjunto **+ enlace de YouTube** → incrustar YouTube, descartar el vídeo;
  2. vídeo descargable directamente → `<video>` local;
  3. en otro caso → guardar el **fotograma de portada** + la duración (la página
     pública no expone el archivo; ver **«Limitaciones»**).
- **Formato** — negrita, cursiva, tachado, código/pre, enlaces y spoilers se
  convierten a Markdown (los desplazamientos de entidades UTF-16 de Telegram se
  manejan correctamente).
- **Hashtags → etiquetas** — los `#hashtags` se convierten en términos de la
  taxonomía `tags` de Zola, así obtienes páginas de etiquetas gratis, sin dejar de
  mostrarse como texto en la publicación.
- **Publicaciones agrupadas** — los álbumes son una sola publicación
  automáticamente; las ráfagas de mensajes publicados en el mismo instante (p. ej.
  al reenviar varios a la vez) se fusionan.
- **Autonavegación** — un enlace a otro mensaje del *mismo* canal se convierte en
  un enlace relativo a esa publicación en el blog; los enlaces a otros canales
  siguen siendo externos.
- **Interacción** — exporta el **número de vistas** por publicación. (Las
  reacciones/me gusta no están disponibles en la página pública — ver
  **«Limitaciones»**.)
- **Feed RSS** — un `/rss.xml` estándar con **todas las publicaciones** y contenido
  completo (un feed completo, no solo los elementos recientes), anunciado mediante
  `<link rel="alternate">` para que los lectores lo descubran automáticamente desde
  la URL del sitio. Activado por defecto; se desactiva con `RSS=false` / `--no-rss`.
- **Vistas previas de enlaces enriquecidas + Mastodon** — cada página emite
  etiquetas Open Graph y Twitter Card (título, descripción, primera imagen de la
  publicación), de modo que los enlaces compartidos se muestran como tarjetas.
  Define `FEDIVERSE_CREATOR` para añadir una firma de autor en las vistas previas de
  Mastodon y verificar el sitio en tu perfil (ver la sección **«Fediverse»**).
- **Sin JavaScript por defecto, listo sin conexión** — el modo oscuro y los
  spoilers son solo CSS, y la caja de búsqueda de Google por defecto es un simple
  `<form>` (sin JS). Solo un motor que no sea Google añade un diminuto manejador de
  Enter en línea. `tg2zola offline <carpeta-public>` reescribe el sitio compilado a
  enlaces relativos **y** elimina el script de redirección de paginación de Zola,
  de modo que la copia sin conexión se abre directamente desde `file://` sin nada de
  JavaScript ni servidor web.
- **Interfaz localizada** — la interfaz del sitio (Más recientes/Más
  antiguos/Etiquetas/Acerca de, la caja de búsqueda, las fechas) se muestra en uno
  de 12 idiomas vía `LANGUAGE` / `--language` (en/be/uk/ru/de/fr/zh/ja/pl/es/ko/ka),
  con los nombres de meses y días de la semana localizados. El contenido de las
  publicaciones permanece en el idioma del canal.

## Instalación

Toma un binario estático para tu arquitectura desde la página
[Releases](../../releases) (Linux `amd64` / `arm64`, musl) o compila desde el
código fuente:

```sh
cargo build --release
# binario en target/release/tg2zola
```

También necesitas el binario [`zola`](https://www.getzola.org/documentation/getting-started/installation/)
para convertir el contenido generado en HTML.

## Uso

```sh
# Generar el sitio Zola (crea config + plantillas en la primera ejecución):
tg2zola --channel durov --site site --init-site

# Compilar el HTML estático:
zola --root site build       # salida en site/public/

# (opcional) Hacerlo visible SIN servidor web, directamente desde el disco:
tg2zola offline site/public  # luego abre site/public/index.html vía file://
```

Prueba local rápida (una página, ~20 mensajes):

```sh
tg2zola --channel <name> --site /tmp/site --init-site --max-pages 1
```

Todas las opciones están en [`tg2zola.toml`](tg2zola.toml) (los flags de CLI lo
anulan):

```sh
tg2zola --config tg2zola.toml
```

Ejecuta `tg2zola --help` para la lista completa de flags.

## Cómo está conectado

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

Cada publicación se convierte en un bundle de página de Zola —
`content/posts/<date>-<id>/index.md` con front matter TOML y sus archivos de medios
al lado. `config.toml` y las plantillas integradas se regeneran de forma
determinista en cada ejecución, y `write_site` reconcilia el árbol: reescribe todo
el Markdown, conserva los medios ya en caché y elimina las publicaciones borradas y
los archivos obsoletos.

## Automatización (GitHub Actions)

Se incluyen dos flujos de trabajo:

- **[`daily.yml`](.github/workflows/daily.yml)** — se ejecuta una vez al día (y
  bajo demanda): restaurar el sitio anterior desde la rama `blog` (la caché de
  medios) → scraping + regenerar → `zola build` → **desplegar en GitHub Pages** (el
  resultado publicado) → committear el sitio actualizado de vuelta a la rama
  **`blog`**.
- **[`release.yml`](.github/workflows/release.yml)** — en cada etiqueta `v*`
  enviada, compila de forma cruzada binarios estáticos `amd64` + `arm64` (musl) y
  los sube a la Release de GitHub.

Para habilitar la publicación: en el repo, **Settings → Pages → Build and
deployment → Source: GitHub Actions**. No se requieren secretos — todo es scraping
público. El sitio publicado es siempre **GitHub Pages**; la rama `blog` es una
copia duradera y nunca afecta a lo que ven los visitantes.

### Regenerar ahora (sin esperar a la ejecución diaria)

`daily.yml` tiene `workflow_dispatch` activado, así que puedes lanzar un scraping +
compilación + redespliegue fresco bajo demanda — ejecuta exactamente los mismos
pasos que la ejecución programada:

- **En el navegador:** abre **[Actions → «daily» → Run workflow](../../actions/workflows/daily.yml)**
  y haz clic en el botón verde **Run workflow**. (Añade la insignia de estado de
  arriba a tu README para acceso con un clic.)
- **Desde la terminal:** `gh workflow run daily.yml` (GitHub CLI), luego
  `gh run watch` para seguirla.

**¿Qué canal?** El canal no está committeado, así que cada despliegue fija el suyo.
Define una **variable** de repositorio `CHANNEL` (Settings → Secrets and variables →
Actions → Variables) con el nombre del canal público — es una *variable*, no un
secreto, ya que el canal es público. (O descomenta `channel = "…"` en
[`tg2zola.toml`](tg2zola.toml) en tu fork.) `THEME_REPO` también funciona como
variable.

### La rama `blog` (archivo + caché)

Cada ejecución committea el sitio generado (Markdown + medios + plantillas
integradas — todo excepto el `public/` compilado y cualquier tema externo) a una
rama `blog`, dejando `main` solo con código. Cumple una doble función:

- **Caché** — la siguiente ejecución restaura los medios desde ella en lugar de
  volver a descargarlos.
- **Archivo duradero** — un sitio Zola completo y compilable que puedes clonar y
  replicar en cualquier sitio, para que la copia no quede atada a una sola
  plataforma:

```sh
git clone -b blog https://github.com/<you>/<repo> my-archive
zola --root my-archive/site serve          # explóralo sin conexión

git -C my-archive remote add gitlab git@gitlab.com:<you>/<repo>.git
git -C my-archive push gitlab blog          # replícalo en otro sitio
```

Los medios se committean como blobs git normales; para canales muy grandes
considera Git LFS o un aplastamiento ocasional del historial.

### Temas

El valor por defecto es el tema integrado **negro absoluto** — cero dependencias
externas, así que el sitio siempre compila. Para usar un
[tema Zola](https://www.getzola.org/themes/) externo, define una **variable** de
repositorio `THEME_REPO` (Settings → Secrets and variables → Actions → Variables)
con su URL git (https, o ssh con una clave de despliegue). El flujo lo clona y
compila con él — y **si el tema falta o su compilación falla, vuelve
automáticamente a las plantillas integradas**, de modo que un problema de tema nunca
puede dejar el blog fuera de línea. Ten en cuenta que los temas externos esperan
una estructura de contenido concreta, así que no todos los temas son compatibles de
inmediato.

Crear una release:

```sh
git tag v0.1.0 && git push origin v0.1.0
```

## Configuración

Todo es configurable mediante una **variable** de repositorio (Settings → Secrets
and variables → Actions → **Variables**) para el flujo de GitHub Actions, o el flag
de CLI / la clave de [`tg2zola.toml`](tg2zola.toml) equivalentes al ejecutar en
local. Son *variables*, no secretos — todo es público.

| Variable del repo | Flag de CLI | Por defecto | Qué hace |
|---|---|---|---|
| `CHANNEL` | `--channel` | **obligatorio** | Canal público a sincronizar |
| `TITLE` | `--title` | nombre del canal | Título del blog (cabecera + `<title>`) |
| `ABOUT` | `--about` | descripción + estadísticas + enlace al repo | HTML propio para el cuerpo de la página Acerca de |
| `PAGES` | `--pages` | — | Páginas extra: Markdown, cada encabezado `# Title` inicia una nueva página + entrada de menú |
| `POSTS_PER_PAGE` | `--posts-per-page` | `20` | Publicaciones completas por página en el feed de inicio |
| `TAGS_FOOTER` | `--tags-footer` | off | `true` muestra el pie de etiquetas por publicación (las etiquetas son clicables en el cuerpo igualmente) |
| `NEXT_PREV` | `--no-next-prev` | on | `false` oculta la navegación Anterior/Siguiente |
| `TELEGRAM_LINK` | `--no-telegram-link` | on | `false` oculta el enlace «Ver en Telegram» por publicación |
| `RSS` | `--no-rss` | on | `false` desactiva el feed RSS en `/rss.xml` (con autodescubrimiento) |
| `FEDIVERSE_CREATOR` | `--fediverse-creator` | — | Mastodon `@user@instance` → firma `fediverse:creator` + enlace de perfil `rel="me"` |
| `SEARCH_ENGINE` | `--search-engine` | `google` | Caja de búsqueda en la cabecera: `google` (formulario sin JS) / `duckduckgo` / `yandex` / `bing` / `none` |
| `SEARCH_URL` | `--search-url` | — | Prefijo de URL de búsqueda propio; la consulta se añade al pulsar Enter (anula el motor) |
| `TITLE_MAX_LEN` | `--title-max-len` | `200` | Longitud máx. del título (caracteres); un título truncado conserva su primera frase completa en el cuerpo |
| `FOOTER` | `--footer` | — | Contenido del pie — texto plano, Markdown o HTML |
| `PAGES_HOST` | `--pages-host` | auto | Host para el límite de tamaño de la página Acerca de: `github` / `gitlab` / `none` (detectado desde la URL) |
| `DATE_FORMAT` | `--date-format` | `%Y %B %d` | Formato strftime para las fechas mostradas (p. ej. `2025 October 28`; `%Y` solo el año) |
| `LANGUAGE` | `--language` | `en` | Idioma de la interfaz del sitio (Más recientes/Más antiguos/Etiquetas/Acerca de/…): `en`/`be`/`uk`/`ru`/`de`/`fr`/`zh`/`ja`/`pl`/`es`/`ko`/`ka` (georgiano). El contenido de las publicaciones permanece en el idioma del canal; las fechas se localizan |
| `LINK_UNDERLINE` | `--link-underline` | off | `true` subraya los enlaces (por defecto: sin subrayado) |
| `YOUTUBE_FACADE` | `--youtube-facade` | off | `true` para una miniatura de YouTube de clic-para-cargar sin JS (por defecto: iframe directo) |
| `GENIUS` | `--no-genius` | on | `false` omite resolver enlaces de genius.com (descarga la página por su vídeo de YouTube + widget de letras) |
| `TAGS_TO_PAGES` | `--tags-to-pages` | — | Etiquetas separadas por comas mostradas como enlaces `#tag` en la nav superior (p. ej. `music, batumi, cooking`) |
| `BACKGROUND_DARK_COLOR` | `--background-dark-color` | `#000000` | Fondo del modo oscuro (cualquier color CSS) |
| `BACKGROUND_LIGHT_COLOR` | `--background-light-color` | `#ffffff` | Fondo del modo claro |
| `CSS` | `--css` | — | CSS extra añadido a la hoja de estilos integrada |
| `THEME_REPO` | `--theme` (nombre) | tema negro integrado | URL git de un tema Zola externo (https/ssh); repliegue automático si falla |
| `REPO_URL` | `--repo-url` | repo de tg2zola | Enlace «Repositorio del código fuente» en Acerca de (la CI pone automáticamente tu repo) |

La página **Acerca de** muestra el **avatar** del canal (tamaño completo), su
descripción y estadísticas, el **tamaño en disco** — con el desglose por tipo y, en
GitHub/GitLab Pages, la parte del límite de ~1 GB del sitio publicado del host (con
enlace a la documentación del host) — más el enlace al repo. La cabecera muestra el
avatar como miniatura y favicon; los hashtags son clicables en los cuerpos de las
publicaciones y producen páginas `/tags/<tag>/`.

Una sola variable `PAGES` puede definir varias páginas — cada encabezado `# Title`
inicia una nueva:

```
# Título de la página
Contenido de la página en Markdown (puede contener HTML en crudo).

# Otra página
Más contenido.
```

→ `/page-title/` y `/another-page/`, cada una enlazada en la nav. Sin dependencia
extra — Zola ya renderiza Markdown.

## Fediverse / Mastodon

Cada página lleva etiquetas Open Graph + Twitter Card, así que un enlace de
publicación compartido se muestra como tarjeta (título, descripción, primera imagen
de la publicación) en Mastodon, Slack, Discord, X, etc. Define `FEDIVERSE_CREATOR`
con tu identificador `@user@instance` para además:

- añadir una firma **`fediverse:creator`**, para que Mastodon muestre «*by
  @you@instance*» en la vista previa del enlace y enlace a tu perfil;
- emitir un enlace **`rel="me"`** a tu perfil, para que puedas añadir este sitio a
  los metadatos de tu perfil de Mastodon y obtener la marca **verificada** (verde).

**¿Puede la gente seguir el blog *desde* Mastodon?** No directamente — un sitio
estático no puede ser un actor de ActivityPub (eso requiere un servidor activo que
hable WebFinger + ActivityPub). Dos formas de permitir suscribirse de todos modos:

- **RSS** — cualquiera puede seguir `/rss.xml` en un lector de feeds hoy mismo
  (muchos usuarios de Mastodon tienen uno). Está integrado y activado por defecto.
- **Un puente** — apunta el feed RSS a un puente RSS→ActivityPub como
  [rss-parrot](https://rss-parrot.net/) o [Bridgy Fed](https://fed.brid.gy/), que
  exponen un `@handle` real que los usuarios de Mastodon pueden **seguir**,
  retransmitiendo las nuevas publicaciones a su línea de tiempo. Sin servidor
  propio.

Así que: vistas previas enriquecidas + atribución de autor + verificación de perfil
funcionan de inmediato; el verdadero «seguir desde Mastodon» está a un puente de
distancia, usando el feed RSS como entrada.

## Limitaciones

La vista previa web pública es el compromiso por no necesitar **ninguna
autenticación**:

- **Las reacciones/me gusta no se exponen** por `t.me/s/`. Exportamos los
  **números de vistas** en su lugar. El modelo de datos deja espacio para añadir
  reacciones reales más adelante vía la API MTProto autenticada (el crate
  [`grammers`](https://codeberg.org/Lonami/grammers)) si alguna vez las quieres.
- **Los vídeos grandes no se pueden descargar** — la vista previa solo sirve una
  imagen de portada y la duración para ellos (los vídeos cortos/de reproducción
  automática *sí* se descargan). Archivar el archivo real también requeriría la API
  MTProto.
- **Los packs de stickers no son enlazables** — el nombre del pack lo carga el
  JavaScript de Telegram y no está en el HTML scrapeado; los stickers se guardan
  como imágenes normales.
- **Los archivos de música (documentos de audio) no se pueden descargar** — su URL
  no está en el HTML scrapeado (solo las notas de voz, con URL `.oga` directas).
  Como los vídeos grandes y los stickers, necesitarían la API MTProto. Para un
  adjunto que no podemos obtener conservamos su **nombre de archivo** (marcado como
  *no archivado*) para que sepas que existió; una publicación que es *solo* tal
  referencia (o un único archivo no descargable) se omite en lugar de publicarse
  vacía.
- **YouTube** se reproduce vía un iframe `youtube.com` (para que las
  reproducciones cuenten en el historial del espectador) en el sitio HTTPS
  publicado; por `file://` el iframe no puede cargar (YouTube necesita un origen).
  `YOUTUBE_FACADE=true` cambia el iframe por una miniatura de clic-para-cargar sin
  JS, que al menos muestra la portada por `file://`.
- **Solo canales públicos**, con la vista previa web activada.

## Licencia

[MIT](LICENSE) © Vitaly Zdanevich
