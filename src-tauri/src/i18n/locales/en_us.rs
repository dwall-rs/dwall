use std::collections::HashMap;

use crate::i18n::{keys::*, LocaleTranslations, TranslationMap, TranslationValue};

pub struct EnglishUSTranslations;

impl TranslationMap for EnglishUSTranslations {
    fn get_translations() -> LocaleTranslations {
        let mut translations = HashMap::new();

        // buttons
        translations.insert(BUTTON_APPLY, TranslationValue::Text("Apply"));
        translations.insert(BUTTON_CANCEL, TranslationValue::Text("Cancel"));
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

        // helps
        translations.insert(
            HELP_MANUALLY_SET_COORDINATES,
            TranslationValue::Text("When manually setting coordinates, you must use the WGS84 coordinate system (the international standard, users in China should take note). Otherwise, coordinate offset issues may occur, leading to inaccurate wallpaper alignment."), 
        );

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
            LABEL_SELECT_MONITOR,
            TranslationValue::Text("Select Monitor"),
        );
        translations.insert(
            LABEL_SET_LOCK_SCREEN_WALLPAPER_SIMULTANEOUSLY,
            TranslationValue::Text("Set Lock Screen Wallpaper Simultaneously"),
        );
        translations.insert(LABEL_SOURCE_CODE, TranslationValue::Text("Source Code"));
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
                template: "Failed to disable startup: \n{{error}}",
                params: &["error"],
            },
        );
        translations.insert(
            MESSAGE_DOWNLOAD_FAILED,
            TranslationValue::Template {
                template:
                    "{{error}}\n\nFor specific errors, please check the log: dwall_settings_lib.log",
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
            MESSAGE_MANUAL_COORDINATES_SAVED,
            TranslationValue::Text(
                "Coordinates saved, next you can choose the theme you want to apply.",
            ),
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
            MESSAGE_SAVING_MANUAL_COORDINATES,
            TranslationValue::Template {
                template: "Error saving coordinates: \n{{error}}",
                params: &["error"],
            },
        );
        translations.insert(
            MESSAGE_STARTUP_FAILED,
            TranslationValue::Template {
                template: "Startup failed: \n{{error}}",
                params: &["error"],
            },
        );
        translations.insert(
            MESSAGE_SWITCH_AUTO_LIGHT_DARK_MODE_FAILED,
            TranslationValue::Template {
                template: "Failed to switch auto light/dark mode: \n{{error}}",
                params: &["error"],
            },
        );
        translations.insert(
            MESSAGE_SWITCHING_TO_MANUAL_COORDINATE_CONFIG,
            TranslationValue::Template {
                template: "Error occurred while switching to manual configuration of coordinates: \n{{error}}",
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
            MESSAGE_UPDATE_FAILED,
            TranslationValue::Template {
                template: "Failed to update: \n{{error}}",
                params: &["error"],
            },
        );
        translations.insert(
            MESSAGE_VERSION_IS_THE_LATEST,
            TranslationValue::Text("The current version is already the latest."),
        );

        // units
        translations.insert(UNIT_HOUR, TranslationValue::Text("h"));
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
