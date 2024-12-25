use std::str::FromStr;
use std::sync::LazyLock;
use std::{collections::HashMap, sync::RwLock};

use serde::Serialize;

use self::locales::en_us::EnglishUSTranslations;
use self::locales::zh_cn::ChineseSimplifiedTranslations;
use self::locales::zh_hk::ChineseTraditionalHKTranslations;

mod keys;
mod locales;
mod utils;

static TRANSLATIONS: LazyLock<RwLock<HashMap<&'static str, LocaleTranslations>>> =
    LazyLock::new(|| {
        RwLock::new({
            let mut m = HashMap::new();

            // 英文必须实现所有的翻译键
            m.insert("en-US", EnglishUSTranslations::get_translations());

            m.insert("zh-CN", ChineseSimplifiedTranslations::get_translations());
            m.insert(
                "zh-HK",
                ChineseTraditionalHKTranslations::get_translations(),
            );

            m
        })
    });

#[derive(Debug, Clone, Copy, Default)]
pub enum Language {
    #[default]
    EnglishUS,
    // EnglishGB,
    ChineseSimplified,
    // ChineseTraditionalTW,
    ChineseTraditionalHK,
}

impl FromStr for Language {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "en-US" => Ok(Language::EnglishUS),
            "zh-CN" => Ok(Language::ChineseSimplified),
            "zh-HK" => Ok(Language::ChineseTraditionalHK),
            _ => Err(format!("Unsupported language identifier: {}", s)),
        }
    }
}

impl Language {
    pub fn id(&self) -> &'static str {
        match self {
            Language::EnglishUS => "en-US",
            // LanguageVariant::EnglishGB => "en-GB",
            Language::ChineseSimplified => "zh-CN",
            // LanguageVariant::ChineseTraditionalTW => "zh-TW",
            Language::ChineseTraditionalHK => "zh-HK",
        }
    }

    pub fn native_name(&self) -> &'static str {
        match self {
            Language::EnglishUS => "American English",
            // LanguageVariant::EnglishGB => "British English",
            Language::ChineseSimplified => "简体中文",
            // LanguageVariant::ChineseTraditionalTW => "繁體中文（台灣）",
            Language::ChineseTraditionalHK => "繁體中文（香港）",
        }
    }

    // pub fn country_code(&self) -> &'static str {
    //     match self {
    //         Language::EnglishUS | Language::ChineseSimplified => "US",
    //         // LanguageVariant::EnglishGB => "GB",
    //         Language::ChineseSimplified => "CN",
    //         // LanguageVariant::ChineseTraditionalTW => "TW",
    //         // LanguageVariant::ChineseTraditionalHK => "HK",
    //     }
    // }
}

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

pub trait TranslationMap {
    fn get_translations() -> LocaleTranslations;
}

thread_local! {
    static CURRENT_LANGUAGE: std::cell::RefCell<Language> = std::cell::RefCell::new(Language::from_str(&self::utils::get_user_preferred_language().unwrap_or("en-US".to_string())).unwrap_or(Language::default()));
}

pub fn get_current_language() -> Language {
    CURRENT_LANGUAGE.with(|lang| *lang.borrow())
}

// pub fn set_language(language_id: &str) -> Result<(), String> {
//     let language_id = Language::from_str(language_id)?;
//     CURRENT_LANGUAGE.with(|current_lang| {
//         *current_lang.borrow_mut() = language_id;
//     });
//     Ok(())
// }

#[tauri::command]
pub fn get_translations() -> LocaleTranslations {
    let current_lang = get_current_language();
    info!(language = current_lang.native_name(), "Current language");

    let translations = TRANSLATIONS.read().unwrap();

    let en_us_translations = translations
        .get("en-US")
        .expect("English translations must be defined");

    let mut locale_translations = translations
        .get(&current_lang.id())
        .cloned()
        .unwrap_or_default();

    for (key, value) in en_us_translations.iter() {
        if !locale_translations.contains_key(*key) {
            debug!(key = key, "Translation missing, using default");
            locale_translations.insert(*key, value.clone());
        }
    }

    locale_translations
}
