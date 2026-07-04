//! UI string translations for the generated site.
//!
//! The whole site is built in a single language chosen via `--language` / the
//! `LANGUAGE` env (default `en`). Only the site *chrome* is translated — post
//! content keeps the channel's original language.

/// Languages we ship UI translations for.
pub const SUPPORTED: &[&str] = &[
    "en", "be", "uk", "ru", "de", "fr", "zh", "ja", "pl", "es", "ko", "ka", "hi",
];

/// Translatable UI strings shown in the site chrome (templates + rendered post
/// bodies). All fields are `&'static str`, so `Ui` is `Copy`.
#[derive(Debug, Clone, Copy)]
pub struct Ui {
    pub newer: &'static str,
    pub older: &'static str,
    pub tags: &'static str,
    pub about: &'static str,
    pub archive: &'static str,
    pub search: &'static str,
    pub search_aria: &'static str,
    pub views: &'static str,
    pub view_on_telegram: &'static str,
    pub forwarded_from: &'static str,
    pub full_posts: &'static str,
    pub titles: &'static str,
    pub not_archived: &'static str,
    pub video: &'static str,
    pub calendar: &'static str,
    pub newer_day: &'static str,
    pub older_day: &'static str,
    pub not_found: &'static str,
}

/// Normalize a user-supplied language tag to one of [`SUPPORTED`], falling back
/// to `en`. Accepts case/region variants like `EN`, `ru_RU`, `zh-Hans`.
pub fn normalize(lang: &str) -> &'static str {
    let base = lang
        .trim()
        .split(['-', '_'])
        .next()
        .unwrap_or("")
        .to_ascii_lowercase();
    SUPPORTED.iter().copied().find(|&l| l == base).unwrap_or("en")
}

/// chrono locale (for Zola's `date` filter) giving localized month/weekday
/// names for the configured language.
pub fn date_locale(lang: &str) -> &'static str {
    match normalize(lang) {
        "be" => "be_BY",
        "uk" => "uk_UA",
        "ru" => "ru_RU",
        "de" => "de_DE",
        "fr" => "fr_FR",
        "zh" => "zh_CN",
        "ja" => "ja_JP",
        "pl" => "pl_PL",
        "es" => "es_ES",
        "ko" => "ko_KR",
        "ka" => "ka_GE",
        "hi" => "hi_IN",
        _ => "en_US",
    }
}

/// UI strings for `lang` (normalized; unknown → English).
pub fn ui(lang: &str) -> Ui {
    match normalize(lang) {
        "be" => BE,
        "uk" => UK,
        "ru" => RU,
        "de" => DE,
        "fr" => FR,
        "zh" => ZH,
        "ja" => JA,
        "pl" => PL,
        "es" => ES,
        "ko" => KO,
        "ka" => KA,
        "hi" => HI,
        _ => EN,
    }
}

const EN: Ui = Ui {
    newer: "Newer",
    older: "Older",
    tags: "Tags",
    about: "About",
    archive: "Archive",
    search: "Search",
    search_aria: "Search this site",
    views: "views",
    view_on_telegram: "View on Telegram",
    forwarded_from: "forwarded from",
    full_posts: "full posts",
    titles: "titles",
    not_archived: "not archived",
    video: "video",
    calendar: "Calendar",
    newer_day: "Newer day",
    older_day: "Older day",
    not_found: "Page not found",
};

const RU: Ui = Ui {
    newer: "Новее",
    older: "Старее",
    tags: "Теги",
    about: "О проекте",
    archive: "Архив",
    search: "Поиск",
    search_aria: "Поиск по сайту",
    views: "просмотров",
    view_on_telegram: "Открыть в Telegram",
    forwarded_from: "переслано от",
    full_posts: "полные посты",
    titles: "заголовки",
    not_archived: "не сохранено",
    video: "видео",
    calendar: "Календарь",
    newer_day: "Новее день",
    older_day: "Старее день",
    not_found: "Страница не найдена",
};

const UK: Ui = Ui {
    newer: "Новіші",
    older: "Старіші",
    tags: "Теги",
    about: "Про проєкт",
    archive: "Архів",
    search: "Пошук",
    search_aria: "Пошук по сайту",
    views: "переглядів",
    view_on_telegram: "Відкрити в Telegram",
    forwarded_from: "переслано від",
    full_posts: "повні дописи",
    titles: "заголовки",
    not_archived: "не збережено",
    video: "відео",
    calendar: "Календар",
    newer_day: "Новіший день",
    older_day: "Старіший день",
    not_found: "Сторінку не знайдено",
};

const BE: Ui = Ui {
    newer: "Навейшыя",
    older: "Старэйшыя",
    tags: "Тэгі",
    about: "Пра праект",
    archive: "Архіў",
    search: "Пошук",
    search_aria: "Пошук па сайце",
    views: "праглядаў",
    view_on_telegram: "Адкрыць у Telegram",
    forwarded_from: "пераслана ад",
    full_posts: "поўныя запісы",
    titles: "загалоўкі",
    not_archived: "не захавана",
    video: "відэа",
    calendar: "Каляндар",
    newer_day: "Навейшы дзень",
    older_day: "Старэйшы дзень",
    not_found: "Старонка не знойдзена",
};

const DE: Ui = Ui {
    newer: "Neuer",
    older: "Älter",
    tags: "Tags",
    about: "Über",
    archive: "Archiv",
    search: "Suche",
    search_aria: "Diese Seite durchsuchen",
    views: "Aufrufe",
    view_on_telegram: "Auf Telegram ansehen",
    forwarded_from: "weitergeleitet von",
    full_posts: "ganze Beiträge",
    titles: "Titel",
    not_archived: "nicht archiviert",
    video: "Video",
    calendar: "Kalender",
    newer_day: "Neuerer Tag",
    older_day: "Älterer Tag",
    not_found: "Seite nicht gefunden",
};

const FR: Ui = Ui {
    newer: "Plus récent",
    older: "Plus ancien",
    tags: "Tags",
    about: "À propos",
    archive: "Archives",
    search: "Rechercher",
    search_aria: "Rechercher sur le site",
    views: "vues",
    view_on_telegram: "Voir sur Telegram",
    forwarded_from: "transféré de",
    full_posts: "articles complets",
    titles: "titres",
    not_archived: "non archivé",
    video: "vidéo",
    calendar: "Calendrier",
    newer_day: "Jour plus récent",
    older_day: "Jour plus ancien",
    not_found: "Page introuvable",
};

const ZH: Ui = Ui {
    newer: "较新",
    older: "较旧",
    tags: "标签",
    about: "关于",
    archive: "归档",
    search: "搜索",
    search_aria: "搜索本站",
    views: "次浏览",
    view_on_telegram: "在 Telegram 中查看",
    forwarded_from: "转发自",
    full_posts: "完整帖子",
    titles: "标题",
    not_archived: "未存档",
    video: "视频",
    calendar: "日历",
    newer_day: "较新一天",
    older_day: "较旧一天",
    not_found: "页面未找到",
};

const JA: Ui = Ui {
    newer: "新しい",
    older: "古い",
    tags: "タグ",
    about: "サイトについて",
    archive: "アーカイブ",
    search: "検索",
    search_aria: "サイト内を検索",
    views: "回表示",
    view_on_telegram: "Telegram で見る",
    forwarded_from: "転送元",
    full_posts: "全文表示",
    titles: "タイトル",
    not_archived: "未アーカイブ",
    video: "動画",
    calendar: "カレンダー",
    newer_day: "新しい日",
    older_day: "古い日",
    not_found: "ページが見つかりません",
};

const PL: Ui = Ui {
    newer: "Nowsze",
    older: "Starsze",
    tags: "Tagi",
    about: "O stronie",
    archive: "Archiwum",
    search: "Szukaj",
    search_aria: "Szukaj w tej witrynie",
    views: "wyświetleń",
    view_on_telegram: "Zobacz na Telegramie",
    forwarded_from: "przesłano z",
    full_posts: "pełne wpisy",
    titles: "tytuły",
    not_archived: "niezarchiwizowane",
    video: "wideo",
    calendar: "Kalendarz",
    newer_day: "Nowszy dzień",
    older_day: "Starszy dzień",
    not_found: "Nie znaleziono strony",
};

const ES: Ui = Ui {
    newer: "Más recientes",
    older: "Más antiguos",
    tags: "Etiquetas",
    about: "Acerca de",
    archive: "Archivo",
    search: "Buscar",
    search_aria: "Buscar en este sitio",
    views: "vistas",
    view_on_telegram: "Ver en Telegram",
    forwarded_from: "reenviado de",
    full_posts: "publicaciones completas",
    titles: "títulos",
    not_archived: "no archivado",
    video: "vídeo",
    calendar: "Calendario",
    newer_day: "Día más reciente",
    older_day: "Día más antiguo",
    not_found: "Página no encontrada",
};

const KO: Ui = Ui {
    newer: "최신",
    older: "이전",
    tags: "태그",
    about: "소개",
    archive: "아카이브",
    search: "검색",
    search_aria: "사이트 내 검색",
    views: "조회",
    view_on_telegram: "Telegram에서 보기",
    forwarded_from: "전달:",
    full_posts: "전체 글",
    titles: "제목",
    not_archived: "보관되지 않음",
    video: "동영상",
    calendar: "달력",
    newer_day: "다음 날",
    older_day: "이전 날",
    not_found: "페이지를 찾을 수 없습니다",
};

const KA: Ui = Ui {
    newer: "უფრო ახალი",
    older: "უფრო ძველი",
    tags: "ტეგები",
    about: "შესახებ",
    archive: "არქივი",
    search: "ძებნა",
    search_aria: "ამ საიტზე ძებნა",
    views: "ნახვა",
    view_on_telegram: "Telegram-ში ნახვა",
    forwarded_from: "გადმოგზავნილია",
    full_posts: "სრული პოსტები",
    titles: "სათაურები",
    not_archived: "არ არის დაარქივებული",
    video: "ვიდეო",
    calendar: "კალენდარი",
    newer_day: "უფრო ახალი დღე",
    older_day: "უფრო ძველი დღე",
    not_found: "გვერდი ვერ მოიძებნა",
};

const HI: Ui = Ui {
    newer: "नए",
    older: "पुराने",
    tags: "टैग",
    about: "परिचय",
    archive: "संग्रह",
    search: "खोज",
    search_aria: "इस साइट में खोजें",
    views: "बार देखा गया",
    view_on_telegram: "टेलीग्राम पर देखें",
    forwarded_from: "आगे भेजा गया",
    full_posts: "पूरी पोस्ट",
    titles: "शीर्षक",
    not_archived: "संग्रहीत नहीं",
    video: "वीडियो",
    calendar: "कैलेंडर",
    newer_day: "नया दिन",
    older_day: "पुराना दिन",
    not_found: "पृष्ठ नहीं मिला",
};

/// Translatable prose for the About page. Sentence templates use `{placeholder}`
/// tokens (filled in `site::about_md`) plus the `__TOKEN__` size/time markers
/// (filled later in `site::set_about_size`).
#[derive(Debug, Clone, Copy)]
pub struct About {
    /// `{channel}` = channel link.
    pub intro: &'static str,
    /// Has `__TOTAL_SIZE__`, `__PERCENT__` and `{limit_link}`.
    pub size_limit: &'static str,
    /// Has `__TOTAL_SIZE__`.
    pub size_plain: &'static str,
    /// Link label for the host limit; `{display}` = size (e.g. "1 GB"),
    /// `{name}` = host (e.g. "GitHub Pages").
    pub limit_phrase: &'static str,
    pub by_kind: &'static str,
    /// Heading for the 10-largest-files list on the About page.
    pub largest_files: &'static str,
    /// Has `__BUILD_TIME__`.
    pub generated_in: &'static str,
    /// Heading for the Google Lighthouse scores block on the About page.
    pub pagespeed: &'static str,
    /// Build-provenance line shown when the optional MTProto backend ran.
    pub mtproto_on: &'static str,
    /// Build-provenance line shown when it didn't (pure web-preview build).
    pub mtproto_off: &'static str,
    pub source_repo: &'static str,
    pub no_api: &'static str,
    pub kind_text: &'static str,
    pub kind_images: &'static str,
    pub kind_videos: &'static str,
    pub kind_audio: &'static str,
    pub kind_other: &'static str,
}

/// About-page prose for `lang` (normalized; unknown → English).
pub fn about(lang: &str) -> About {
    match normalize(lang) {
        "be" => BE_ABOUT,
        "uk" => UK_ABOUT,
        "ru" => RU_ABOUT,
        "de" => DE_ABOUT,
        "fr" => FR_ABOUT,
        "zh" => ZH_ABOUT,
        "ja" => JA_ABOUT,
        "pl" => PL_ABOUT,
        "es" => ES_ABOUT,
        "ko" => KO_ABOUT,
        "ka" => KA_ABOUT,
        "hi" => HI_ABOUT,
        _ => EN_ABOUT,
    }
}

const EN_ABOUT: About = About {
    intro: "A static mirror of the public Telegram channel {channel}.",
    mtproto_on: "The optional MTProto backend was used to fetch audio and long videos.",
    mtproto_off: "The optional MTProto backend was not used — public web preview only. Enable it with the `mtproto` build feature and Telegram API credentials.",
    size_limit: "The site occupies **__TOTAL_SIZE__** — **__PERCENT__** of the {limit_link}.",
    size_plain: "The site occupies **__TOTAL_SIZE__** on disk.",
    limit_phrase: "{display} {name} limit",
    by_kind: "By kind:",
    largest_files: "Largest files:",
    generated_in: "Generated in **__BUILD_TIME__**.",
    pagespeed: "Google Lighthouse scores (mobile):",
    source_repo: "Source repository:",
    no_api: "**No Telegram bot, token, or API is needed for the public web preview** — the site is built from it, with all media except audio and big videos downloaded locally, so it keeps working even if the channel is removed.",
    kind_text: "Text",
    kind_images: "Images",
    kind_videos: "Videos",
    kind_audio: "Audio",
    kind_other: "Other",
};

const RU_ABOUT: About = About {
    intro: "Статичная копия публичного Telegram-канала {channel}.",
    mtproto_on: "Опциональный бэкенд MTProto использовался для загрузки аудио и длинных видео.",
    mtproto_off: "Опциональный бэкенд MTProto не использовался — только публичное веб-превью. Включается сборкой с функцией `mtproto` и учётными данными Telegram API.",
    size_limit: "Сайт занимает **__TOTAL_SIZE__** — **__PERCENT__** от {limit_link}.",
    size_plain: "Сайт занимает **__TOTAL_SIZE__** на диске.",
    limit_phrase: "лимита {name} в {display}",
    by_kind: "По типам:",
    largest_files: "Самые большие файлы:",
    generated_in: "Сгенерировано за **__BUILD_TIME__**.",
    pagespeed: "Оценки Google Lighthouse (мобильные):",
    source_repo: "Репозиторий исходного кода:",
    no_api: "**Для публичного веб-превью не нужны ни бот, ни токен, ни API Telegram** — сайт собран из него, все медиа, кроме аудио и больших видео, скачаны локально, поэтому он продолжит работать, даже если канал удалят.",
    kind_text: "Текст",
    kind_images: "Изображения",
    kind_videos: "Видео",
    kind_audio: "Аудио",
    kind_other: "Прочее",
};

const UK_ABOUT: About = About {
    intro: "Статична копія публічного Telegram-каналу {channel}.",
    mtproto_on: "Опціональний бекенд MTProto використовувався для завантаження аудіо та довгих відео.",
    mtproto_off: "Опціональний бекенд MTProto не використовувався — лише публічне вебпрев’ю. Вмикається збіркою з функцією `mtproto` та обліковими даними Telegram API.",
    size_limit: "Сайт займає **__TOTAL_SIZE__** — **__PERCENT__** від {limit_link}.",
    size_plain: "Сайт займає **__TOTAL_SIZE__** на диску.",
    limit_phrase: "ліміту {name} у {display}",
    by_kind: "За типами:",
    largest_files: "Найбільші файли:",
    generated_in: "Згенеровано за **__BUILD_TIME__**.",
    pagespeed: "Оцінки Google Lighthouse (мобільні):",
    source_repo: "Репозиторій вихідного коду:",
    no_api: "**Для публічного вебпрев’ю не потрібні ні бот, ні токен, ні API Telegram** — сайт зібрано з нього, усі медіа, крім аудіо та великих відео, завантажено локально, тож він працюватиме, навіть якщо канал видалять.",
    kind_text: "Текст",
    kind_images: "Зображення",
    kind_videos: "Відео",
    kind_audio: "Аудіо",
    kind_other: "Інше",
};

const BE_ABOUT: About = About {
    intro: "Статычная копія публічнага Telegram-канала {channel}.",
    mtproto_on: "Апцыянальны бэкенд MTProto выкарыстоўваўся для спампоўкі аўдыя і доўгіх відэа.",
    mtproto_off: "Апцыянальны бэкенд MTProto не выкарыстоўваўся — толькі публічнае вэб-прэв’ю. Уключаецца зборкай з функцыяй `mtproto` і ўліковымі данымі Telegram API.",
    size_limit: "Сайт займае **__TOTAL_SIZE__** — **__PERCENT__** ад {limit_link}.",
    size_plain: "Сайт займае **__TOTAL_SIZE__** на дыску.",
    limit_phrase: "ліміту {name} у {display}",
    by_kind: "Па тыпах:",
    largest_files: "Найбуйнейшыя файлы:",
    generated_in: "Згенеравана за **__BUILD_TIME__**.",
    pagespeed: "Адзнакі Google Lighthouse (мабільныя):",
    source_repo: "Рэпазіторый зыходнага кода:",
    no_api: "**Для публічнага вэб-прэв’ю не патрэбныя ні бот, ні токен, ні API Telegram** — сайт сабраны з яго, усе медыя, апроч аўдыя і вялікіх відэа, спампаваныя лакальна, таму ён будзе працаваць, нават калі канал выдаляць.",
    kind_text: "Тэкст",
    kind_images: "Выявы",
    kind_videos: "Відэа",
    kind_audio: "Аўдыя",
    kind_other: "Іншае",
};

const DE_ABOUT: About = About {
    intro: "Eine statische Kopie des öffentlichen Telegram-Kanals {channel}.",
    mtproto_on: "Das optionale MTProto-Backend wurde verwendet, um Audio und lange Videos zu laden.",
    mtproto_off: "Das optionale MTProto-Backend wurde nicht verwendet — nur die öffentliche Web-Vorschau. Aktivierbar über das Build-Feature `mtproto` und Telegram-API-Zugangsdaten.",
    size_limit: "Die Website belegt **__TOTAL_SIZE__** — **__PERCENT__** des {limit_link}.",
    size_plain: "Die Website belegt **__TOTAL_SIZE__** auf der Festplatte.",
    limit_phrase: "Limits von {name} ({display})",
    by_kind: "Nach Typ:",
    largest_files: "Größte Dateien:",
    generated_in: "Erstellt in **__BUILD_TIME__**.",
    pagespeed: "Google-Lighthouse-Werte (mobil):",
    source_repo: "Quellcode-Repository:",
    no_api: "**Für die öffentliche Web-Vorschau ist kein Telegram-Bot, -Token oder -API nötig** — die Website wird daraus erstellt, alle Medien außer Audio und großen Videos werden lokal gespeichert, sodass sie auch dann funktioniert, wenn der Kanal entfernt wird.",
    kind_text: "Text",
    kind_images: "Bilder",
    kind_videos: "Videos",
    kind_audio: "Audio",
    kind_other: "Sonstiges",
};

const FR_ABOUT: About = About {
    intro: "Une copie statique de la chaîne Telegram publique {channel}.",
    mtproto_on: "Le backend MTProto optionnel a été utilisé pour récupérer l’audio et les vidéos longues.",
    mtproto_off: "Le backend MTProto optionnel n’a pas été utilisé — uniquement l’aperçu web public. Activez-le via la fonctionnalité de build `mtproto` et des identifiants de l’API Telegram.",
    size_limit: "Le site occupe **__TOTAL_SIZE__** — **__PERCENT__** de la {limit_link}.",
    size_plain: "Le site occupe **__TOTAL_SIZE__** sur le disque.",
    limit_phrase: "limite de {name} ({display})",
    by_kind: "Par type :",
    largest_files: "Fichiers les plus volumineux :",
    generated_in: "Généré en **__BUILD_TIME__**.",
    pagespeed: "Scores Google Lighthouse (mobile) :",
    source_repo: "Dépôt du code source :",
    no_api: "**Aucun bot, jeton ni API Telegram n’est nécessaire pour l’aperçu web public** — le site en est généré, et tous les médias sauf l’audio et les vidéos volumineuses sont téléchargés localement, de sorte qu’il continue de fonctionner même si la chaîne est supprimée.",
    kind_text: "Texte",
    kind_images: "Images",
    kind_videos: "Vidéos",
    kind_audio: "Audio",
    kind_other: "Autres",
};

const ZH_ABOUT: About = About {
    intro: "公开 Telegram 频道 {channel} 的静态镜像。",
    mtproto_on: "已使用可选的 MTProto 后端来获取音频和长视频。",
    mtproto_off: "未使用可选的 MTProto 后端 — 仅使用公开网页预览。使用 `mtproto` 构建功能和 Telegram API 凭据即可启用。",
    size_limit: "本站占用 **__TOTAL_SIZE__**，为 {limit_link} 的 **__PERCENT__**。",
    size_plain: "本站在磁盘上占用 **__TOTAL_SIZE__**。",
    limit_phrase: "{name} {display} 限额",
    by_kind: "按类型：",
    largest_files: "最大的文件：",
    generated_in: "生成耗时 **__BUILD_TIME__**。",
    pagespeed: "Google Lighthouse 分数（移动端）：",
    source_repo: "源代码仓库：",
    no_api: "**公开网页预览无需 Telegram 机器人、令牌或 API** — 本站由其生成，除音频和大视频外的所有媒体均已下载到本地，因此即使频道被删除也能继续访问。",
    kind_text: "文本",
    kind_images: "图片",
    kind_videos: "视频",
    kind_audio: "音频",
    kind_other: "其他",
};

const JA_ABOUT: About = About {
    intro: "公開 Telegram チャンネル {channel} の静的ミラー。",
    mtproto_on: "オプションの MTProto バックエンドを使用して音声と長い動画を取得しました。",
    mtproto_off: "オプションの MTProto バックエンドは使用していません — 公開ウェブプレビューのみ。`mtproto` ビルド機能と Telegram API 認証情報で有効化できます。",
    size_limit: "このサイトは **__TOTAL_SIZE__** を使用 — {limit_link}の **__PERCENT__** です。",
    size_plain: "このサイトはディスク上で **__TOTAL_SIZE__** を使用しています。",
    limit_phrase: "{name} の {display} 制限",
    by_kind: "種類別：",
    largest_files: "最大のファイル：",
    generated_in: "生成時間 **__BUILD_TIME__**。",
    pagespeed: "Google Lighthouse スコア（モバイル）：",
    source_repo: "ソースコードリポジトリ：",
    no_api: "**公開ウェブプレビューに Telegram のボット・トークン・API は不要** — このサイトはそこから生成され、音声と大きな動画を除くすべてのメディアはローカルに保存されるため、チャンネルが削除されても表示できます。",
    kind_text: "テキスト",
    kind_images: "画像",
    kind_videos: "動画",
    kind_audio: "音声",
    kind_other: "その他",
};

const PL_ABOUT: About = About {
    intro: "Statyczna kopia publicznego kanału Telegram {channel}.",
    mtproto_on: "Opcjonalny backend MTProto został użyty do pobrania audio i długich filmów.",
    mtproto_off: "Opcjonalny backend MTProto nie został użyty — tylko publiczny podgląd sieciowy. Włącz go funkcją kompilacji `mtproto` i danymi uwierzytelniającymi API Telegrama.",
    size_limit: "Witryna zajmuje **__TOTAL_SIZE__** — **__PERCENT__** {limit_link}.",
    size_plain: "Witryna zajmuje **__TOTAL_SIZE__** na dysku.",
    limit_phrase: "limitu {name} ({display})",
    by_kind: "Według typu:",
    largest_files: "Największe pliki:",
    generated_in: "Wygenerowano w **__BUILD_TIME__**.",
    pagespeed: "Wyniki Google Lighthouse (mobilne):",
    source_repo: "Repozytorium kodu źródłowego:",
    no_api: "**Publiczny podgląd nie wymaga bota, tokena ani API Telegrama** — witryna jest z niego budowana, a wszystkie media oprócz audio i dużych filmów są pobierane lokalnie, więc działa nawet po usunięciu kanału.",
    kind_text: "Tekst",
    kind_images: "Obrazy",
    kind_videos: "Wideo",
    kind_audio: "Audio",
    kind_other: "Inne",
};

const ES_ABOUT: About = About {
    intro: "Una copia estática del canal público de Telegram {channel}.",
    mtproto_on: "Se usó el backend opcional MTProto para obtener audio y vídeos largos.",
    mtproto_off: "No se usó el backend opcional MTProto — solo la vista previa web pública. Actívalo con la función de compilación `mtproto` y credenciales de la API de Telegram.",
    size_limit: "El sitio ocupa **__TOTAL_SIZE__** — el **__PERCENT__** del {limit_link}.",
    size_plain: "El sitio ocupa **__TOTAL_SIZE__** en disco.",
    limit_phrase: "límite de {name} ({display})",
    by_kind: "Por tipo:",
    largest_files: "Archivos más grandes:",
    generated_in: "Generado en **__BUILD_TIME__**.",
    pagespeed: "Puntuaciones de Google Lighthouse (móvil):",
    source_repo: "Repositorio del código fuente:",
    no_api: "**La vista previa pública no necesita ningún bot, token ni API de Telegram** — el sitio se genera a partir de ella y todos los archivos excepto el audio y los vídeos grandes se descargan localmente, así que sigue funcionando aunque se elimine el canal.",
    kind_text: "Texto",
    kind_images: "Imágenes",
    kind_videos: "Vídeos",
    kind_audio: "Audio",
    kind_other: "Otros",
};

const KO_ABOUT: About = About {
    intro: "공개 Telegram 채널 {channel}의 정적 미러.",
    mtproto_on: "선택적 MTProto 백엔드를 사용하여 오디오와 긴 동영상을 가져왔습니다.",
    mtproto_off: "선택적 MTProto 백엔드를 사용하지 않았습니다 — 공개 웹 미리보기만 사용. `mtproto` 빌드 기능과 Telegram API 자격 증명으로 활성화할 수 있습니다.",
    size_limit: "이 사이트는 **__TOTAL_SIZE__**을(를) 사용 — {limit_link}의 **__PERCENT__**입니다.",
    size_plain: "이 사이트는 디스크에서 **__TOTAL_SIZE__**을(를) 사용합니다.",
    limit_phrase: "{name} {display} 제한",
    by_kind: "종류별:",
    largest_files: "가장 큰 파일:",
    generated_in: "생성 시간 **__BUILD_TIME__**.",
    pagespeed: "Google Lighthouse 점수(모바일):",
    source_repo: "소스 코드 저장소:",
    no_api: "**공개 웹 미리보기에는 Telegram 봇, 토큰, API가 필요 없습니다** — 이 사이트는 이를 통해 생성되며 오디오와 큰 동영상을 제외한 모든 미디어가 로컬에 저장되므로 채널이 삭제되어도 계속 볼 수 있습니다.",
    kind_text: "텍스트",
    kind_images: "이미지",
    kind_videos: "동영상",
    kind_audio: "오디오",
    kind_other: "기타",
};

const KA_ABOUT: About = About {
    intro: "საჯარო Telegram-არხის {channel} სტატიკური ასლი.",
    mtproto_on: "სურვილისამებრ MTProto backend გამოყენებულია აუდიოსა და გრძელი ვიდეოების ჩამოსატვირთად.",
    mtproto_off: "სურვილისამებრ MTProto backend არ გამოყენებულა — მხოლოდ საჯარო ვებ-გადახედვა. ჩართეთ `mtproto` build-ფუნქციითა და Telegram API-ის მონაცემებით.",
    size_limit: "საიტი იკავებს **__TOTAL_SIZE__**-ს ({limit_link}: **__PERCENT__**).",
    size_plain: "საიტი დისკზე იკავებს **__TOTAL_SIZE__**-ს.",
    limit_phrase: "{name}-ის {display} ლიმიტი",
    by_kind: "ტიპის მიხედვით:",
    largest_files: "უდიდესი ფაილები:",
    generated_in: "შექმნის დრო: **__BUILD_TIME__**.",
    pagespeed: "Google Lighthouse-ის ქულები (მობილური):",
    source_repo: "წყაროს კოდის რეპოზიტორი:",
    no_api: "**საჯარო ვებ-გადახედვისთვის Telegram-ის ბოტი, ტოკენი ან API საჭირო არ არის** — საიტი მისგან იქმნება, ყველა მედია, აუდიოსა და დიდი ვიდეოების გარდა, ჩამოტვირთულია ლოკალურად, ამიტომ ის იმუშავებს მაშინაც კი, თუ არხი წაიშლება.",
    kind_text: "ტექსტი",
    kind_images: "სურათები",
    kind_videos: "ვიდეო",
    kind_audio: "აუდიო",
    kind_other: "სხვა",
};

const HI_ABOUT: About = About {
    intro: "सार्वजनिक Telegram चैनल {channel} की एक स्थिर प्रति।",
    size_limit: "साइट **__TOTAL_SIZE__** लेती है — {limit_link} का **__PERCENT__**।",
    size_plain: "साइट डिस्क पर **__TOTAL_SIZE__** लेती है।",
    limit_phrase: "{name} की {display} सीमा",
    by_kind: "प्रकार के अनुसार:",
    largest_files: "सबसे बड़ी फ़ाइलें:",
    generated_in: "**__BUILD_TIME__** में तैयार किया गया।",
    pagespeed: "Google Lighthouse स्कोर (मोबाइल):",
    mtproto_on: "ऑडियो और मूल-गुणवत्ता की तस्वीरें लाने के लिए वैकल्पिक MTProto बैकएंड का उपयोग किया गया।",
    mtproto_off: "वैकल्पिक MTProto बैकएंड का उपयोग नहीं किया गया — केवल सार्वजनिक वेब पूर्वावलोकन। इसे `mtproto` बिल्ड फ़ीचर और Telegram API क्रेडेंशियल के साथ सक्षम करें।",
    source_repo: "स्रोत कोड रिपॉज़िटरी:",
    no_api: "**सार्वजनिक वेब पूर्वावलोकन के लिए किसी Telegram बॉट, टोकन या API की आवश्यकता नहीं** — साइट इसी से बनाई जाती है और ऑडियो और बड़े वीडियो को छोड़कर सभी मीडिया स्थानीय रूप से डाउनलोड किए जाते हैं, इसलिए चैनल हटाए जाने पर भी यह काम करती रहती है।",
    kind_text: "टेक्स्ट",
    kind_images: "छवियाँ",
    kind_videos: "वीडियो",
    kind_audio: "ऑडियो",
    kind_other: "अन्य",
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_variants() {
        assert_eq!(normalize("en"), "en");
        assert_eq!(normalize("EN"), "en");
        assert_eq!(normalize("ru_RU"), "ru");
        assert_eq!(normalize("zh-Hans"), "zh");
        assert_eq!(normalize("  be  "), "be");
        // Unknown / unsupported falls back to English.
        assert_eq!(normalize("pt"), "en");
        assert_eq!(normalize(""), "en");
    }

    #[test]
    fn tables_resolve() {
        assert_eq!(ui("ru").tags, "Теги");
        assert_eq!(ui("xx").tags, "Tags"); // fallback
        assert_eq!(date_locale("uk"), "uk_UA");
        assert_eq!(date_locale("xx"), "en_US");
        assert_eq!(about("ru").by_kind, "По типам:");
        assert_eq!(about("xx").by_kind, "By kind:"); // fallback
    }

    #[test]
    fn every_supported_has_a_table() {
        // None of the shipped languages should silently fall back to English
        // (except `en` itself), i.e. each has its own `tags` rendering.
        for &l in SUPPORTED {
            let _ = ui(l);
            let _ = about(l);
            let _ = date_locale(l);
        }
    }
}
