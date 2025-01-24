use std::collections::HashMap;

use crate::i18n::{keys::*, LocaleTranslations, TranslationMap, TranslationValue};

pub struct ChineseTraditionalTWTranslations;

impl TranslationMap for ChineseTraditionalTWTranslations {
    fn get_translations() -> LocaleTranslations {
        let mut translations = HashMap::new();

        // buttons
        translations.insert(BUTTON_APPLY, TranslationValue::Text("應用"));
        translations.insert(BUTTON_DOWNLOAD, TranslationValue::Text("下載"));
        translations.insert(
            BUTTON_OPEN_LOG_DIRECTORY,
            TranslationValue::Text("開啟日誌資料夾"),
        );
        translations.insert(BUTTON_SELECT_FOLDER, TranslationValue::Text("修改"));
        translations.insert(BUTTON_STOP, TranslationValue::Text("停止"));

        // labels
        translations.insert(
            LABEL_AUTOMATICALLY_RETRIEVE_COORDINATES,
            TranslationValue::Text("自動獲取座標"),
        );
        translations.insert(
            LABEL_AUTOMATICALLY_SWITCH_TO_DARK_MODE,
            TranslationValue::Text("自動切換暗色模式"),
        );
        translations.insert(LABEL_CHECK_INTERVAL, TranslationValue::Text("檢測間隔"));
        translations.insert(
            LABEL_GITHUB_MIRROR_TEMPLATE,
            TranslationValue::Text("Github 鏡像範本"),
        );
        translations.insert(LABEL_LAUNCH_AT_STARTUP, TranslationValue::Text("開機啟動"));
        translations.insert(
            LABEL_SET_LOCK_SCREEN_WALLPAPER_SIMULTANEOUSLY,
            TranslationValue::Text("同時設定鎖屏海報"),
        );
        translations.insert(LABEL_SOURCE_CODE, TranslationValue::Text("原始碼"));
        translations.insert(LABEL_THEMES_DIRECTORY, TranslationValue::Text("主題資料夾"));
        translations.insert(LABEL_VERSION, TranslationValue::Text("版本號"));

        // tooltips
        translations.insert(
            TOOLTIP_OPEN_THEMES_DIRECTORY,
            TranslationValue::Text("點擊開啟主題資料夾"),
        );
        translations.insert(
            TOOLTIP_CHECK_NEW_VERSION,
            TranslationValue::Text("點擊檢查新版本"),
        );
        translations.insert(
            TOOLTIP_NEW_VERSION_AVAILABLE,
            TranslationValue::Text("發現新版本，點擊更新"),
        );
        translations.insert(TOOLTIP_SETTINGS, TranslationValue::Text("設定"));

        // messages
        translations.insert(
            MESSAGE_CHANGE_THEMES_DIRECTORY,
            TranslationValue::Template {
                template: "修改主題資料夾為：{{newThemesDirectory}}？",
                params: &["newThemesDirectory"],
            },
        );
        translations.insert(
            MESSAGE_DISABLE_STARTUP_FAILED,
            TranslationValue::Template {
                template: "關閉開機啟動失敗：\n{{error}}",
                params: &["error"],
            },
        );
        translations.insert(
            MESSAGE_DOWNLOAD_FAILED,
            TranslationValue::Template {
                template: "{{error}}\n\n請查看日誌檔案以取得更多錯誤資訊：dwall_settings_lib.log",
                params: &["error"],
            },
        );
        translations.insert(
            MESSAGE_INVALID_NUMBER_INPUT,
            TranslationValue::Text("請輸入有效的數字"),
        );
        translations.insert(
            MESSAGE_LOCATION_PERMISSION,
            TranslationValue::Text("定位權限未開啟，請手動開啟定位或手動設定座標。\n\n是否手動設定座標？\n點擊「是」手動設定座標，點擊「否」結束程式"),
        );
        translations.insert(
            MESSAGE_NUMBER_TOO_LARGE,
            TranslationValue::Template {
                template: "不能大於 {{max}}",
                params: &["max"],
            },
        );
        translations.insert(
            MESSAGE_NUMBER_TOO_SMALL,
            TranslationValue::Template {
                template: "不能小於 {{min}}",
                params: &["min"],
            },
        );
        translations.insert(
            MESSAGE_STARTUP_FAILED,
            TranslationValue::Template {
                template: "設定開機啟動失敗：\n{{error}}",
                params: &["error"],
            },
        );
        translations.insert(
            MESSAGE_THEMES_DIRECTORY_MOVED,
            TranslationValue::Template {
                template: "主題資料夾已改為：{{newThemesDirectory}}",
                params: &["newThemesDirectory"],
            },
        );
        translations.insert(
            MESSAGE_VERSION_IS_THE_LATEST,
            TranslationValue::Text("當前已是最新版本"),
        );

        // titles
        translations.insert(TITLE_DOWNLOAD_FAILD, TranslationValue::Text("下載失敗"));
        translations.insert(
            TITLE_DOWNLOADING_NEW_VERSION,
            TranslationValue::Text("正在下載新版本..."),
        );

        // placeholders
        translations.insert(PLACEHOLDER_LATITUDE, TranslationValue::Text("緯度"));
        translations.insert(PLACEHOLDER_LONGITUDE, TranslationValue::Text("經度"));

        translations
    }
}
