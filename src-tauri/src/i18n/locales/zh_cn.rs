use std::collections::HashMap;

use crate::i18n::{keys::*, LocaleTranslations, TranslationMap, TranslationValue};

pub struct ChineseSimplifiedTranslations;

impl TranslationMap for ChineseSimplifiedTranslations {
    fn get_translations() -> LocaleTranslations {
        let mut translations = HashMap::new();

        // buttons
        translations.insert(BUTTON_APPLY, TranslationValue::Text("应用"));
        translations.insert(BUTTON_DOWNLOAD, TranslationValue::Text("下载"));
        translations.insert(
            BUTTON_OPEN_LOG_DIRECTORY,
            TranslationValue::Text("打开日志文件夹"),
        );
        translations.insert(BUTTON_SELECT_FOLDER, TranslationValue::Text("修改"));
        translations.insert(BUTTON_STOP, TranslationValue::Text("停止"));

        // labels
        translations.insert(
            LABEL_AUTOMATICALLY_RETRIEVE_COORDINATES,
            TranslationValue::Text("自动获取坐标"),
        );
        translations.insert(
            LABEL_AUTOMATICALLY_SWITCH_TO_DARK_MODE,
            TranslationValue::Text("自动切换暗色模式"),
        );
        translations.insert(LABEL_CHECK_INTERVAL, TranslationValue::Text("检测间隔"));
        translations.insert(
            LABEL_GITHUB_MIRROR_TEMPLATE,
            TranslationValue::Text("Github 镜像模板"),
        );
        translations.insert(LABEL_LAUNCH_AT_STARTUP, TranslationValue::Text("开机自启"));
        translations.insert(
            LABEL_SET_LOCK_SCREEN_WALLPAPER_SIMULTANEOUSLY,
            TranslationValue::Text("同时设置锁屏壁纸"),
        );
        translations.insert(LABEL_THEMES_DIRECTORY, TranslationValue::Text("主题文件夹"));
        translations.insert(LABEL_VERSION, TranslationValue::Text("版本号"));

        // tooltips
        translations.insert(
            TOOLTIP_OPEN_THEMES_DIRECTORY,
            TranslationValue::Text("点击打开主题文件夹"),
        );
        translations.insert(
            TOOLTIP_CHECK_NEW_VERSION,
            TranslationValue::Text("点击检查新版本"),
        );
        translations.insert(
            TOOLTIP_NEW_VERSION_AVAILABLE,
            TranslationValue::Text("发现新版本，点击更新"),
        );
        translations.insert(TOOLTIP_SETTINGS, TranslationValue::Text("设置"));

        // messages
        translations.insert(
            MESSAGE_CHANGE_THEMES_DIRECTORY,
            TranslationValue::Template {
                template: "修改主题文件夹为：{{newThemesDirectory}}？",
                params: &["newThemesDirectory"],
            },
        );
        translations.insert(
            MESSAGE_DISABLE_STARTUP_FAILED,
            TranslationValue::Template {
                template: "关闭开机自启失败：\n{{error}}",
                params: &["error"],
            },
        );
        translations.insert(
            MESSAGE_DOWNLOAD_FAILED,
            TranslationValue::Template {
                template: "{{error}}\n\n具体错误请查看日志文件：dwall_settings_lib.log",
                params: &["error"],
            },
        );
        translations.insert(
            MESSAGE_INVALID_NUMBER_INPUT,
            TranslationValue::Text("请输入有效的数字"),
        );
        translations.insert(
            MESSAGE_LOCATION_PERMISSION,
            TranslationValue::Text("定位权限未打开，请手动开启定位或手动配置坐标。\n\n是否手动配置坐标？\n点击“是”手动配置坐标，点击“否”关闭程序"),
        );
        translations.insert(
            MESSAGE_MANUAL_COORDINATES_SAVED,
            TranslationValue::Text("坐标已保存，接下来可以选择想要应用的主题了"),
        );
        translations.insert(
            MESSAGE_NUMBER_TOO_LARGE,
            TranslationValue::Template {
                template: "不能大于 {{max}}",
                params: &["max"],
            },
        );
        translations.insert(
            MESSAGE_NUMBER_TOO_SMALL,
            TranslationValue::Template {
                template: "不能小于 {{min}}",
                params: &["min"],
            },
        );
        translations.insert(
            MESSAGE_SAVING_MANUAL_COORDINATES,
            TranslationValue::Template {
                template: "保存坐标时出错：\n{{error}}",
                params: &["error"],
            },
        );
        translations.insert(
            MESSAGE_STARTUP_FAILED,
            TranslationValue::Template {
                template: "设置开机自启失败：\n{{error}}",
                params: &["error"],
            },
        );
        translations.insert(
            MESSAGE_SWITCHING_TO_MANUAL_COORDINATE_CONFIG,
            TranslationValue::Template {
                template: "切换至手动配置坐标时出错：\n{{error}}",
                params: &["error"],
            },
        );
        translations.insert(
            MESSAGE_THEMES_DIRECTORY_MOVED,
            TranslationValue::Template {
                template: "主题文件夹已改为：{{newThemesDirectory}}",
                params: &["newThemesDirectory"],
            },
        );
        translations.insert(
            MESSAGE_VERSION_IS_THE_LATEST,
            TranslationValue::Text("当前已是最新版本"),
        );

        // titles
        translations.insert(TITLE_DOWNLOAD_FAILD, TranslationValue::Text("下载失败"));
        translations.insert(
            TITLE_DOWNLOADING_NEW_VERSION,
            TranslationValue::Text("正在下载新版本..."),
        );

        // placeholders
        translations.insert(PLACEHOLDER_LATITUDE, TranslationValue::Text("纬度"));
        translations.insert(PLACEHOLDER_LONGITUDE, TranslationValue::Text("经度"));

        translations
    }
}
