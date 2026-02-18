use std::cell::RefCell;
use std::collections::HashMap;
use std::str::FromStr;

use serde::Serialize;

use self::locales::{
    CHINESE_SIMPLIFIED_TRANSLATIONS, CHINESE_TRADITIONAL_HK_TRANSLATIONS,
    CHINESE_TRADITIONAL_TW_TRANSLATIONS, ENGLISH_US_TRANSLATIONS, JAPANESE_TRANSLATIONS,
    KOREAN_TRANSLATIONS,
};
use self::utils::get_user_preferred_language;

mod keys;
mod locales;
mod utils;

#[derive(Clone, Debug, Serialize)]
#[serde(untagged)]
pub enum TranslationValue {
    Text(&'static str),
    Template {
        template: &'static str,
        params: &'static [&'static str],
    },
}

pub type LocaleTranslations = HashMap<&'static str, TranslationValue>;

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum Language {
    #[default]
    EnglishUS,
    // EnglishGB,
    ChineseSimplified,
    ChineseTraditionalTW,
    ChineseTraditionalHK,
    Japanese,
    Korean,
}

impl FromStr for Language {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "en-US" => Ok(Language::EnglishUS),
            "zh-CN" => Ok(Language::ChineseSimplified),
            "zh-HK" => Ok(Language::ChineseTraditionalHK),
            "zh-TW" => Ok(Language::ChineseTraditionalTW),
            "ja-JP" => Ok(Language::Japanese),
            "ko-KR" => Ok(Language::Korean),
            _ => Err(format!("Unsupported language identifier: {s}")),
        }
    }
}

impl Language {
    pub fn native_name(&self) -> &'static str {
        match self {
            Language::EnglishUS => "American English",
            // LanguageVariant::EnglishGB => "British English",
            Language::ChineseSimplified => "简体中文",
            Language::ChineseTraditionalHK => "繁體中文（香港）",
            Language::ChineseTraditionalTW => "繁體中文（台灣）",
            Language::Japanese => "日本語",
            Language::Korean => "한국어",
        }
    }

    pub fn translations(&self) -> &'static LocaleTranslations {
        match self {
            Language::EnglishUS => &ENGLISH_US_TRANSLATIONS,
            Language::ChineseSimplified => &CHINESE_SIMPLIFIED_TRANSLATIONS,
            Language::ChineseTraditionalHK => &CHINESE_TRADITIONAL_HK_TRANSLATIONS,
            Language::ChineseTraditionalTW => &CHINESE_TRADITIONAL_TW_TRANSLATIONS,
            Language::Japanese => &JAPANESE_TRANSLATIONS,
            Language::Korean => &KOREAN_TRANSLATIONS,
        }
    }
}

thread_local! {
    static CURRENT_LANGUAGE: RefCell<Language> = RefCell::new(Language::from_str(&get_user_preferred_language().unwrap_or("en-US".to_string())).unwrap_or(Language::default()));

    // Cache for current language translations to avoid repeated cloning
    static CURRENT_TRANSLATIONS_CACHE: RefCell<Option<(Language, LocaleTranslations)>> =
     const { RefCell::new(None) };
}

pub fn get_current_language() -> Language {
    CURRENT_LANGUAGE.with(|lang| *lang.borrow())
}

// pub fn set_language(language_id: &str) -> Result<(), String> {
//     let language = Language::from_str(language_id)?;
//     CURRENT_LANGUAGE.with(|current_lang| {
//         *current_lang.borrow_mut() = language;
//     });

//     // Invalidate cache when language changes
//     {
//         let mut cache = CURRENT_TRANSLATIONS_CACHE.write().unwrap();
//         *cache = None;
//     }

//     Ok(())
// }

pub fn get_translations() -> LocaleTranslations {
    let current_lang = get_current_language();
    info!(language = current_lang.native_name(), "Current language");

    // Check if we have cached translations for the current language
    {
        if let Some((cached_lang, cached_translations)) =
            CURRENT_TRANSLATIONS_CACHE.with(|cache| cache.borrow().clone())
            && cached_lang == current_lang
        {
            return cached_translations;
        }
    }

    // If not cached or language changed, compute translations

    let mut locale_translations = current_lang.translations().clone();

    for (key, value) in ENGLISH_US_TRANSLATIONS.iter() {
        if !locale_translations.contains_key(*key) {
            debug!(key = key, "Translation missing, using default");
            locale_translations.insert(*key, value.clone());
        }
    }

    // Cache the computed translations
    {
        CURRENT_TRANSLATIONS_CACHE
            .with_borrow_mut(|cache| *cache = Some((current_lang, locale_translations.clone())));
    }

    locale_translations
}
