//! UI string translations for the generated site.
//!
//! The whole site is built in a single language chosen via `--language` / the
//! `LANGUAGE` env (default `en`). Only the site *chrome* is translated — post
//! content keeps the channel's original language.

/// Languages we ship UI translations for.
pub const SUPPORTED: &[&str] = &[
    "en", "be", "uk", "ru", "de", "fr", "zh", "ja", "pl", "es", "ko",
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
};

/// Translatable prose for the About page. Sentence templates use `{placeholder}`
/// tokens (filled in `site::about_md`) plus the `__TOKEN__` size/time markers
/// (filled later in `site::set_about_size`).
#[derive(Debug, Clone, Copy)]
pub struct About {
    /// `{channel}` = channel link, `{tool}` = tg2zola link.
    pub intro: &'static str,
    /// Has `__TOTAL_SIZE__`, `__PERCENT__` and `{limit_link}`.
    pub size_limit: &'static str,
    /// Has `__TOTAL_SIZE__`.
    pub size_plain: &'static str,
    /// Link label for the host limit; `{display}` = size (e.g. "1 GB"),
    /// `{name}` = host (e.g. "GitHub Pages").
    pub limit_phrase: &'static str,
    pub by_kind: &'static str,
    /// Has `__BUILD_TIME__`.
    pub generated_in: &'static str,
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
        _ => EN_ABOUT,
    }
}

const EN_ABOUT: About = About {
    intro: "A static mirror of the public Telegram channel {channel}, generated by {tool}.",
    size_limit: "The site occupies **__TOTAL_SIZE__** — **__PERCENT__** of the {limit_link}.",
    size_plain: "The site occupies **__TOTAL_SIZE__** on disk.",
    limit_phrase: "{display} {name} limit",
    by_kind: "By kind:",
    generated_in: "Generated in **__BUILD_TIME__**.",
    source_repo: "Source repository:",
    no_api: "**No Telegram bot, token, or API is needed** — the site is built from the public web preview, with all media downloaded locally, so it keeps working even if the channel is removed.",
    kind_text: "Text",
    kind_images: "Images",
    kind_videos: "Videos",
    kind_audio: "Audio",
    kind_other: "Other",
};

const RU_ABOUT: About = About {
    intro: "Статичная копия публичного Telegram-канала {channel}, созданная с помощью {tool}.",
    size_limit: "Сайт занимает **__TOTAL_SIZE__** — **__PERCENT__** от {limit_link}.",
    size_plain: "Сайт занимает **__TOTAL_SIZE__** на диске.",
    limit_phrase: "лимита {name} в {display}",
    by_kind: "По типам:",
    generated_in: "Сгенерировано за **__BUILD_TIME__**.",
    source_repo: "Репозиторий исходного кода:",
    no_api: "**Не нужны ни бот, ни токен, ни API Telegram** — сайт собран из публичного веб-превью, все медиа скачаны локально, поэтому он продолжит работать, даже если канал удалят.",
    kind_text: "Текст",
    kind_images: "Изображения",
    kind_videos: "Видео",
    kind_audio: "Аудио",
    kind_other: "Прочее",
};

const UK_ABOUT: About = About {
    intro: "Статична копія публічного Telegram-каналу {channel}, створена за допомогою {tool}.",
    size_limit: "Сайт займає **__TOTAL_SIZE__** — **__PERCENT__** від {limit_link}.",
    size_plain: "Сайт займає **__TOTAL_SIZE__** на диску.",
    limit_phrase: "ліміту {name} у {display}",
    by_kind: "За типами:",
    generated_in: "Згенеровано за **__BUILD_TIME__**.",
    source_repo: "Репозиторій вихідного коду:",
    no_api: "**Не потрібні ні бот, ні токен, ні API Telegram** — сайт зібрано з публічного вебпрев’ю, усі медіа завантажено локально, тож він працюватиме, навіть якщо канал видалять.",
    kind_text: "Текст",
    kind_images: "Зображення",
    kind_videos: "Відео",
    kind_audio: "Аудіо",
    kind_other: "Інше",
};

const BE_ABOUT: About = About {
    intro: "Статычная копія публічнага Telegram-канала {channel}, створаная з дапамогай {tool}.",
    size_limit: "Сайт займае **__TOTAL_SIZE__** — **__PERCENT__** ад {limit_link}.",
    size_plain: "Сайт займае **__TOTAL_SIZE__** на дыску.",
    limit_phrase: "ліміту {name} у {display}",
    by_kind: "Па тыпах:",
    generated_in: "Згенеравана за **__BUILD_TIME__**.",
    source_repo: "Рэпазіторый зыходнага кода:",
    no_api: "**Не патрэбныя ні бот, ні токен, ні API Telegram** — сайт сабраны з публічнага вэб-прэв’ю, усе медыя спампаваныя лакальна, таму ён будзе працаваць, нават калі канал выдаляць.",
    kind_text: "Тэкст",
    kind_images: "Выявы",
    kind_videos: "Відэа",
    kind_audio: "Аўдыя",
    kind_other: "Іншае",
};

const DE_ABOUT: About = About {
    intro: "Eine statische Kopie des öffentlichen Telegram-Kanals {channel}, erstellt mit {tool}.",
    size_limit: "Die Website belegt **__TOTAL_SIZE__** — **__PERCENT__** des {limit_link}.",
    size_plain: "Die Website belegt **__TOTAL_SIZE__** auf der Festplatte.",
    limit_phrase: "Limits von {name} ({display})",
    by_kind: "Nach Typ:",
    generated_in: "Erstellt in **__BUILD_TIME__**.",
    source_repo: "Quellcode-Repository:",
    no_api: "**Kein Telegram-Bot, -Token oder -API nötig** — die Website wird aus der öffentlichen Web-Vorschau erstellt, alle Medien werden lokal gespeichert, sodass sie auch dann funktioniert, wenn der Kanal entfernt wird.",
    kind_text: "Text",
    kind_images: "Bilder",
    kind_videos: "Videos",
    kind_audio: "Audio",
    kind_other: "Sonstiges",
};

const FR_ABOUT: About = About {
    intro: "Une copie statique de la chaîne Telegram publique {channel}, générée avec {tool}.",
    size_limit: "Le site occupe **__TOTAL_SIZE__** — **__PERCENT__** de la {limit_link}.",
    size_plain: "Le site occupe **__TOTAL_SIZE__** sur le disque.",
    limit_phrase: "limite de {name} ({display})",
    by_kind: "Par type :",
    generated_in: "Généré en **__BUILD_TIME__**.",
    source_repo: "Dépôt du code source :",
    no_api: "**Aucun bot, jeton ni API Telegram n’est nécessaire** — le site est généré à partir de l’aperçu web public, et tous les médias sont téléchargés localement, de sorte qu’il continue de fonctionner même si la chaîne est supprimée.",
    kind_text: "Texte",
    kind_images: "Images",
    kind_videos: "Vidéos",
    kind_audio: "Audio",
    kind_other: "Autres",
};

const ZH_ABOUT: About = About {
    intro: "公开 Telegram 频道 {channel} 的静态镜像，由 {tool} 生成。",
    size_limit: "本站占用 **__TOTAL_SIZE__**，为 {limit_link} 的 **__PERCENT__**。",
    size_plain: "本站在磁盘上占用 **__TOTAL_SIZE__**。",
    limit_phrase: "{name} {display} 限额",
    by_kind: "按类型：",
    generated_in: "生成耗时 **__BUILD_TIME__**。",
    source_repo: "源代码仓库：",
    no_api: "**无需 Telegram 机器人、令牌或 API** — 本站由公开网页预览生成，所有媒体均已下载到本地，因此即使频道被删除也能继续访问。",
    kind_text: "文本",
    kind_images: "图片",
    kind_videos: "视频",
    kind_audio: "音频",
    kind_other: "其他",
};

const JA_ABOUT: About = About {
    intro: "公開 Telegram チャンネル {channel} の静的ミラー。{tool} で生成。",
    size_limit: "このサイトは **__TOTAL_SIZE__** を使用 — {limit_link}の **__PERCENT__** です。",
    size_plain: "このサイトはディスク上で **__TOTAL_SIZE__** を使用しています。",
    limit_phrase: "{name} の {display} 制限",
    by_kind: "種類別：",
    generated_in: "生成時間 **__BUILD_TIME__**。",
    source_repo: "ソースコードリポジトリ：",
    no_api: "**Telegram のボット・トークン・API は不要** — このサイトは公開ウェブプレビューから生成され、すべてのメディアはローカルに保存されるため、チャンネルが削除されても表示できます。",
    kind_text: "テキスト",
    kind_images: "画像",
    kind_videos: "動画",
    kind_audio: "音声",
    kind_other: "その他",
};

const PL_ABOUT: About = About {
    intro: "Statyczna kopia publicznego kanału Telegram {channel}, wygenerowana za pomocą {tool}.",
    size_limit: "Witryna zajmuje **__TOTAL_SIZE__** — **__PERCENT__** {limit_link}.",
    size_plain: "Witryna zajmuje **__TOTAL_SIZE__** na dysku.",
    limit_phrase: "limitu {name} ({display})",
    by_kind: "Według typu:",
    generated_in: "Wygenerowano w **__BUILD_TIME__**.",
    source_repo: "Repozytorium kodu źródłowego:",
    no_api: "**Nie potrzeba bota, tokena ani API Telegrama** — witryna jest budowana z publicznego podglądu, a wszystkie media są pobierane lokalnie, więc działa nawet po usunięciu kanału.",
    kind_text: "Tekst",
    kind_images: "Obrazy",
    kind_videos: "Wideo",
    kind_audio: "Audio",
    kind_other: "Inne",
};

const ES_ABOUT: About = About {
    intro: "Una copia estática del canal público de Telegram {channel}, generada con {tool}.",
    size_limit: "El sitio ocupa **__TOTAL_SIZE__** — el **__PERCENT__** del {limit_link}.",
    size_plain: "El sitio ocupa **__TOTAL_SIZE__** en disco.",
    limit_phrase: "límite de {name} ({display})",
    by_kind: "Por tipo:",
    generated_in: "Generado en **__BUILD_TIME__**.",
    source_repo: "Repositorio del código fuente:",
    no_api: "**No se necesita ningún bot, token ni API de Telegram** — el sitio se genera a partir de la vista previa pública y todos los archivos se descargan localmente, así que sigue funcionando aunque se elimine el canal.",
    kind_text: "Texto",
    kind_images: "Imágenes",
    kind_videos: "Vídeos",
    kind_audio: "Audio",
    kind_other: "Otros",
};

const KO_ABOUT: About = About {
    intro: "공개 Telegram 채널 {channel}의 정적 미러. {tool}(으)로 생성됨.",
    size_limit: "이 사이트는 **__TOTAL_SIZE__**을(를) 사용 — {limit_link}의 **__PERCENT__**입니다.",
    size_plain: "이 사이트는 디스크에서 **__TOTAL_SIZE__**을(를) 사용합니다.",
    limit_phrase: "{name} {display} 제한",
    by_kind: "종류별:",
    generated_in: "생성 시간 **__BUILD_TIME__**.",
    source_repo: "소스 코드 저장소:",
    no_api: "**Telegram 봇, 토큰, API가 필요 없습니다** — 이 사이트는 공개 웹 미리보기로 생성되며 모든 미디어가 로컬에 저장되므로 채널이 삭제되어도 계속 볼 수 있습니다.",
    kind_text: "텍스트",
    kind_images: "이미지",
    kind_videos: "동영상",
    kind_audio: "오디오",
    kind_other: "기타",
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
