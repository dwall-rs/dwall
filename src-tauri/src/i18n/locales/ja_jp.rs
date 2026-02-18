use std::{collections::HashMap, sync::LazyLock};

use crate::i18n::{LocaleTranslations, TranslationValue, keys::*};

pub static JAPANESE_TRANSLATIONS: LazyLock<LocaleTranslations> = LazyLock::new(|| {
    let mut translations = HashMap::new();

    // buttons
    translations.insert(BUTTON_APPLY, TranslationValue::Text("適用"));
    translations.insert(BUTTON_DOWNLOAD, TranslationValue::Text("ダウンロード"));
    translations.insert(BUTTON_INSTALL, TranslationValue::Text("インストール"));
    translations.insert(
        BUTTON_OPEN_LOG_DIRECTORY,
        TranslationValue::Text("ログディレクトリを開く"),
    );
    translations.insert(
        BUTTON_SELECT_FOLDER,
        TranslationValue::Text("フォルダを選択"),
    );
    translations.insert(BUTTON_STOP, TranslationValue::Text("停止"));

    // labels
    translations.insert(
        LABEL_AUTOMATICALLY_RETRIEVE_COORDINATES,
        TranslationValue::Text("座標を自動取得"),
    );
    translations.insert(
        LABEL_AUTOMATICALLY_SWITCH_TO_DARK_MODE,
        TranslationValue::Text("ダークモードに自動切替"),
    );
    translations.insert(LABEL_CHECK_INTERVAL, TranslationValue::Text("確認間隔"));
    translations.insert(
        LABEL_GITHUB_MIRROR_TEMPLATE,
        TranslationValue::Text("GitHubミラーテンプレート"),
    );
    translations.insert(
        LABEL_LAUNCH_AT_STARTUP,
        TranslationValue::Text("起動時に実行"),
    );
    translations.insert(
        LABEL_SET_LOCK_SCREEN_WALLPAPER_SIMULTANEOUSLY,
        TranslationValue::Text("ロック画面の壁紙も同時に設定"),
    );
    translations.insert(LABEL_SOURCE_CODE, TranslationValue::Text("ソースコード"));
    translations.insert(
        LABEL_THEMES_DIRECTORY,
        TranslationValue::Text("テーマディレクトリ"),
    );
    translations.insert(LABEL_VERSION, TranslationValue::Text("バージョン"));

    // tooltips
    translations.insert(
        TOOLTIP_OPEN_THEMES_DIRECTORY,
        TranslationValue::Text("クリックしてテーマディレクトリを開きます"),
    );
    translations.insert(
        TOOLTIP_CHECK_NEW_VERSION,
        TranslationValue::Text("クリックして新しいバージョンを確認"),
    );
    translations.insert(
        TOOLTIP_NEW_VERSION_AVAILABLE,
        TranslationValue::Text(
            "新しいバージョンが利用可能です！このボタンをクリックして更新してください",
        ),
    );
    translations.insert(TOOLTIP_SETTINGS, TranslationValue::Text("設定"));

    // messages
    translations.insert(
        MESSAGE_CHANGE_THEMES_DIRECTORY,
        TranslationValue::Template {
            template: "テーマディレクトリを次に変更しますか：{{newThemesDirectory}}？",
            params: &["newThemesDirectory"],
        },
    );
    translations.insert(
        MESSAGE_DISABLE_STARTUP_FAILED,
        TranslationValue::Template {
            template: "起動時実行の無効化に失敗しました：\n{{error}}",
            params: &["error"],
        },
    );
    translations.insert(
            MESSAGE_DOWNLOAD_FAILED,
            TranslationValue::Template {
                template: "{{error}}\n\n詳細なエラーについては、ログを確認してください：dwall_settings_lib.log",
                params: &["error"],
            },
        );
    translations.insert(
        MESSAGE_INVALID_NUMBER_INPUT,
        TranslationValue::Text("有効な数値を入力してください"),
    );
    translations.insert(
            MESSAGE_LOCATION_PERMISSION,
            TranslationValue::Text(
                "位置情報の権限が有効になっていません。位置情報を手動で有効にするか、座標を手動で設定してください。\n\n座標を手動で設定しますか？\n「はい」をクリックして座標を手動で設定する、「いいえ」をクリックして位置情報を有効にする。",
            ),
        );
    translations.insert(
        MESSAGE_NUMBER_TOO_LARGE,
        TranslationValue::Template {
            template: "{{max}}を超えることはできません",
            params: &["max"],
        },
    );
    translations.insert(
        MESSAGE_NUMBER_TOO_SMALL,
        TranslationValue::Template {
            template: "{{min}}未満にすることはできません",
            params: &["min"],
        },
    );
    translations.insert(
        MESSAGE_STARTUP_FAILED,
        TranslationValue::Template {
            template: "起動に失敗しました：\n{{error}}",
            params: &["error"],
        },
    );
    translations.insert(
        MESSAGE_THEMES_DIRECTORY_MOVED,
        TranslationValue::Template {
            template: "テーマディレクトリが移動されました：{{newThemesDirectory}}",
            params: &["newThemesDirectory"],
        },
    );
    translations.insert(
        MESSAGE_VERSION_IS_THE_LATEST,
        TranslationValue::Text("現在のバージョンは最新です"),
    );

    // titles
    translations.insert(
        TITLE_DOWNLOAD_FAILD,
        TranslationValue::Text("ダウンロード失敗"),
    );
    translations.insert(
        TITLE_DOWNLOADING_NEW_VERSION,
        TranslationValue::Template {
            template: "新しいバージョン {{newVersion}} をダウンロード中...",
            params: &["newVersion"],
        },
    );

    // placeholders
    translations.insert(PLACEHOLDER_LATITUDE, TranslationValue::Text("緯度"));
    translations.insert(PLACEHOLDER_LONGITUDE, TranslationValue::Text("経度"));

    translations
});
