use std::collections::HashMap;

use crate::i18n::{keys::*, LocaleTranslations, TranslationMap, TranslationValue};

pub struct EnglishUSTranslations;

impl TranslationMap for EnglishUSTranslations {
    fn get_translations() -> LocaleTranslations {
        let mut translations = HashMap::new();

        // buttons
        translations.insert(BUTTON_APPLY, TranslationValue::Text("Apply"));
        translations.insert(BUTTON_DOWNLOAD, TranslationValue::Text("Download"));
        translations.insert(
            BUTTON_OPEN_LOG_DIRECTORY,
            TranslationValue::Text("Open Log Directory"),
        );
        translations.insert(
            BUTTON_SELECT_FOLDER,
            TranslationValue::Text("Select Folder"),
        );
        translations.insert(BUTTON_STOP, TranslationValue::Text("Stop"));

        // labels
        translations.insert(
            LABEL_AUTOMATICALLY_RETRIEVE_COORDINATES,
            TranslationValue::Text("Automatically Retrieve Coordinates"),
        );
        translations.insert(
            LABEL_AUTOMATICALLY_SWITCH_TO_DARK_MODE,
            TranslationValue::Text("Automatically Switch to Dark Mode"),
        );
        translations.insert(
            LABEL_CHECK_INTERVAL,
            TranslationValue::Text("Check Interval"),
        );
        translations.insert(
            LABEL_GITHUB_MIRROR_TEMPLATE,
            TranslationValue::Text("GithubMirrorTemplate"),
        );
        translations.insert(
            LABEL_LAUNCH_AT_STARTUP,
            TranslationValue::Text("Launch at Startup"),
        );
        translations.insert(
            LABEL_SET_LOCK_SCREEN_WALLPAPER_SIMULTANEOUSLY,
            TranslationValue::Text("Set Lock Screen Wallpaper Simultaneously"),
        );
        translations.insert(
            LABEL_THEMES_DIRECTORY,
            TranslationValue::Text("Theme Directory"),
        );
        translations.insert(LABEL_VERSION, TranslationValue::Text("Version"));

        // tooltips
        translations.insert(
            TOOLTIP_OPEN_THEMES_DIRECTORY,
            TranslationValue::Text("Click it to open the themes directory."),
        );
        translations.insert(
            TOOLTIP_CHECK_NEW_VERSION,
            TranslationValue::Text("Click to check for new version"),
        );
        translations.insert(
            TOOLTIP_NEW_VERSION_AVAILABLE,
            TranslationValue::Text("New version available! Click this button to update."),
        );
        translations.insert(TOOLTIP_SETTINGS, TranslationValue::Text("Settings"));

        // messages
        translations.insert(
            MESSAGE_CHANGE_THEMES_DIRECTORY,
            TranslationValue::Template {
                template: "Change the themes directory to: {{newThemesDirectory}}?",
                params: &["newThemesDirectory"],
            },
        );
        translations.insert(
            MESSAGE_DISABLE_STARTUP_FAILED,
            TranslationValue::Template {
                template: "Failed to disable startup: \n${error}",
                params: &["error"],
            },
        );
        translations.insert(
            MESSAGE_DOWNLOAD_FAILED,
            TranslationValue::Template {
                template:
                    "${error}\n\nFor specific errors, please check the log: dwall_settings_lib.log",
                params: &["error"],
            },
        );
        translations.insert(
            MESSAGE_INVALID_NUMBER_INPUT,
            TranslationValue::Text("Please enter a valid number."),
        );
        translations.insert(
            MESSAGE_LOCATION_PERMISSION,
            TranslationValue::Text("The location permission is not turned on. Please manually enable location or manually configure coordinates.\n\nDo you want to manually configure coordinates?\nClick \"Yes\" to manually configure coordinates, or click \"No\" to close the program."),
        );
        translations.insert(
            MESSAGE_NUMBER_TOO_LARGE,
            TranslationValue::Template {
                template: "Cannot exceed {{max}}",
                params: &["max"],
            },
        );
        translations.insert(
            MESSAGE_NUMBER_TOO_SMALL,
            TranslationValue::Template {
                template: "Cannot be less than {{min}}",
                params: &["min"],
            },
        );
        translations.insert(
            MESSAGE_STARTUP_FAILED,
            TranslationValue::Template {
                template: "Startup failed: \n${error}",
                params: &["error"],
            },
        );
        translations.insert(
            MESSAGE_THEMES_DIRECTORY_MOVED,
            TranslationValue::Template {
                template: "The themes directory has been moved to: {{newThemesDirectory}}",
                params: &["newThemesDirectory"],
            },
        );
        translations.insert(
            MESSAGE_VERSION_IS_THE_LATEST,
            TranslationValue::Text("The current version is already the latest."),
        );

        // units
        translations.insert(UNIT_SECOND, TranslationValue::Text("s"));

        // titles
        translations.insert(
            TITLE_DOWNLOAD_FAILD,
            TranslationValue::Text("Download Failed"),
        );
        translations.insert(
            TITLE_DOWNLOADING_NEW_VERSION,
            TranslationValue::Text("Downloading new version..."),
        );

        // placeholders
        translations.insert(PLACEHOLDER_LATITUDE, TranslationValue::Text("latitude"));
        translations.insert(PLACEHOLDER_LONGITUDE, TranslationValue::Text("longitude"));

        translations
    }
}
