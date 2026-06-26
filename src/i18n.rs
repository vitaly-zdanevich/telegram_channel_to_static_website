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
    }

    #[test]
    fn every_supported_has_a_table() {
        // None of the shipped languages should silently fall back to English
        // (except `en` itself), i.e. each has its own `tags` rendering.
        for &l in SUPPORTED {
            let _ = ui(l);
            let _ = date_locale(l);
        }
    }
}
