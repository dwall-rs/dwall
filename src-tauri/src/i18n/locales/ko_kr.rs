use std::{collections::HashMap, sync::LazyLock};

use crate::i18n::{keys::*, LocaleTranslations, TranslationValue};

pub static KOREAN_TRANSLATIONS: LazyLock<LocaleTranslations> = LazyLock::new(|| {
    let mut translations = HashMap::new();

    // buttons
    translations.insert(BUTTON_APPLY, TranslationValue::Text("적용"));
    translations.insert(BUTTON_CANCEL, TranslationValue::Text("취소"));
    translations.insert(BUTTON_DOWNLOAD, TranslationValue::Text("다운로드"));
    translations.insert(BUTTON_INSTALL, TranslationValue::Text("설치"));
    translations.insert(
        BUTTON_OPEN_LOG_DIRECTORY,
        TranslationValue::Text("로그 폴더 열기"),
    );
    translations.insert(BUTTON_SELECT_FOLDER, TranslationValue::Text("수정"));
    translations.insert(BUTTON_STOP, TranslationValue::Text("중지"));

    // helps
    translations.insert(
        HELP_AUTOMATICALLY_SWITCH_TO_DARK_MODE,
        TranslationValue::Text(
            "자동으로 라이트/다크 모드를 전환하지 않으려면 이 옵션을 비활성화하세요.",
        ),
    );
    translations.insert(
            HELP_GITHUB_MIRROR_TEMPLATE,
            TranslationValue::Text("Github 미러 템플릿은 다운로드를 가속화하는 데 사용됩니다. 일부 국가 및 지역에서는 네트워크 규제로 인해 Github에 정상적으로 접근할 수 없어 다운로드가 실패할 수 있습니다. 썸네일을 정상적으로 로드하고 테마를 다운로드하려면 Github 미러 템플릿을 설정해야 합니다. 이 버튼을 클릭하면 사용 가능한 Github 미러 템플릿을 확인할 수 있습니다."),
        );
    translations.insert(
            HELP_LAUNCH_AT_STARTUP,
            TranslationValue::Text(
                "시작 시 자동 실행은 백그라운드 프로세스만 시작하며 그래픽 프로그램은 시작하지 않아 메모리를 많이 차지하지 않습니다.",
            ),
        );
    translations.insert(
            HELP_MANUALLY_SET_COORDINATES,
            TranslationValue::Text("수동으로 좌표를 설정할 때는 WGS84 좌표계(국제 표준 좌표계, 중국 사용자는 주의 필요)를 사용해야 하며, 그렇지 않으면 좌표 편차 문제가 발생하여 배경화면이 정확하지 않게 일치할 수 있습니다."), 
        );
    translations.insert(
        HELP_SET_LOCK_SCREEN_WALLPAPER_SIMULTANEOUSLY,
        TranslationValue::Text(
            "잠금 화면 배경화면을 동시에 설정하지 않으려면 이 옵션을 비활성화하세요.",
        ),
    );
    translations.insert(
            HELP_UPDATE_FAILED,
            TranslationValue::Text("핫 업데이트를 완료할 수 없습니다. 이 메시지 뒤의 다운로드 버튼을 클릭하여 새 버전을 수동으로 다운로드하세요: "),
        );

    // labels
    translations.insert(
        LABEL_AUTOMATICALLY_RETRIEVE_COORDINATES,
        TranslationValue::Text("좌표 자동 가져오기"),
    );
    translations.insert(
        LABEL_AUTOMATICALLY_SWITCH_TO_DARK_MODE,
        TranslationValue::Text("다크 모드 자동 전환"),
    );
    translations.insert(LABEL_CHECK_INTERVAL, TranslationValue::Text("감지 간격"));
    translations.insert(
        LABEL_GITHUB_MIRROR_TEMPLATE,
        TranslationValue::Text("Github 미러 템플릿"),
    );
    translations.insert(
        LABEL_LAUNCH_AT_STARTUP,
        TranslationValue::Text("시작 시 자동 실행"),
    );
    translations.insert(
        LABEL_SELECT_MONITOR,
        TranslationValue::Text("디스플레이 선택"),
    );
    translations.insert(
        LABEL_SET_LOCK_SCREEN_WALLPAPER_SIMULTANEOUSLY,
        TranslationValue::Text("잠금 화면 배경화면 동시 설정"),
    );
    translations.insert(LABEL_SOURCE_CODE, TranslationValue::Text("소스 코드"));
    translations.insert(LABEL_THEMES_DIRECTORY, TranslationValue::Text("테마 폴더"));
    translations.insert(LABEL_VERSION, TranslationValue::Text("버전"));

    // tooltips
    translations.insert(
        TOOLTIP_OPEN_THEMES_DIRECTORY,
        TranslationValue::Text("클릭하여 테마 폴더 열기"),
    );
    translations.insert(
        TOOLTIP_CHECK_NEW_VERSION,
        TranslationValue::Text("클릭하여 새 버전 확인"),
    );
    translations.insert(
        TOOLTIP_NEW_VERSION_AVAILABLE,
        TranslationValue::Text("새 버전 발견, 업데이트하려면 클릭"),
    );
    translations.insert(TOOLTIP_SETTINGS, TranslationValue::Text("설정"));

    // messages
    translations.insert(
        MESSAGE_APPLY_THEME_FAILED,
        TranslationValue::Template {
            template: "테마 적용 실패: \n{{error}}",
            params: &["error"],
        },
    );
    translations.insert(
        MESSAGE_CHANGE_THEMES_DIRECTORY,
        TranslationValue::Template {
            template: "테마 폴더를 다음으로 수정하시겠습니까: {{newThemesDirectory}}?",
            params: &["newThemesDirectory"],
        },
    );
    translations.insert(
        MESSAGE_CHECK_INTERVAL_UPDATED,
        TranslationValue::Template {
            template: "감지 시간 간격이 다음으로 업데이트되었습니다: {{newInterval}}초",
            params: &["newInterval"],
        },
    );
    translations.insert(
        MESSAGE_DISABLE_STARTUP_FAILED,
        TranslationValue::Template {
            template: "시작 시 자동 실행 비활성화 실패:\n{{error}}",
            params: &["error"],
        },
    );
    translations.insert(
        MESSAGE_DOWNLOAD_CANCELLED,
        TranslationValue::Text("다운로드가 취소되었습니다"),
    );
    translations.insert(
        MESSAGE_DOWNLOAD_FAILED,
        TranslationValue::Template {
            template:
                "{{error}}\n\n자세한 오류 내용은 로그 파일을 확인하세요: dwall_settings_lib.log",
            params: &["error"],
        },
    );
    translations.insert(
            MESSAGE_FILE_SIZE_WARNING,
            TranslationValue::Text(
                "파일 크기를 가져올 수 없어 다운로드 진행 상황을 계산할 수 없습니다. 응답 헤더 전달을 지원하는 Github 미러 템플릿으로 변경하세요",
            ),
        );
    translations.insert(
        MESSAGE_GITHUB_MIRROR_TEMPLATE_UPDATED,
        TranslationValue::Template {
            template: "Github 미러 템플릿이 다음으로 업데이트되었습니다: {{newTemplate}}",
            params: &["newTemplate"],
        },
    );
    translations.insert(
            MESSAGE_GITHUB_STAR,
            TranslationValue::Text(
                "이 프로그램이 도움이 되었다면 Github에서 이 프로젝트에 별점을 주시고 오픈소스 프로젝트를 지원해 주세요: ",
            ),
        );
    translations.insert(
        MESSAGE_INVALID_NUMBER_INPUT,
        TranslationValue::Text("유효한 숫자를 입력하세요"),
    );
    translations.insert(
            MESSAGE_LOCATION_PERMISSION,
            TranslationValue::Text(
                "위치 권한이 활성화되지 않았습니다. 위치를 수동으로 활성화하거나 좌표를 수동으로 설정하세요.\n\n좌표를 수동으로 설정하시겠습니까?\n'예'를 클릭하여 좌표를 수동으로 설정하고, '아니오'를 클릭하여 위치를 수동으로 활성화하세요.",
            ),
        );
    translations.insert(
        MESSAGE_MANUAL_COORDINATES_SAVED,
        TranslationValue::Text(
            "좌표가 저장되었습니다. 이제 적용하고 싶은 테마를 선택할 수 있습니다",
        ),
    );
    translations.insert(
        MESSAGE_NUMBER_TOO_LARGE,
        TranslationValue::Template {
            template: "{{max}}보다 클 수 없습니다",
            params: &["max"],
        },
    );
    translations.insert(
        MESSAGE_NUMBER_TOO_SMALL,
        TranslationValue::Template {
            template: "{{min}}보다 작을 수 없습니다",
            params: &["min"],
        },
    );
    translations.insert(
        MESSAGE_SAVING_MANUAL_COORDINATES,
        TranslationValue::Template {
            template: "좌표 저장 중 오류 발생:\n{{error}}",
            params: &["error"],
        },
    );
    translations.insert(
        MESSAGE_STARTUP_FAILED,
        TranslationValue::Template {
            template: "시작 시 자동 실행 설정 실패:\n{{error}}",
            params: &["error"],
        },
    );
    translations.insert(
        MESSAGE_SWITCH_AUTO_LIGHT_DARK_MODE_FAILED,
        TranslationValue::Template {
            template: "자동 라이트/다크 모드 전환 실패: \n{{error}}",
            params: &["error"],
        },
    );
    translations.insert(
        MESSAGE_SWITCHING_TO_MANUAL_COORDINATE_CONFIG,
        TranslationValue::Template {
            template: "수동 좌표 설정으로 전환 중 오류 발생:\n{{error}}",
            params: &["error"],
        },
    );
    translations.insert(
        MESSAGE_THEMES_DIRECTORY_MOVED,
        TranslationValue::Template {
            template: "테마 폴더가 다음으로 변경되었습니다: {{newThemesDirectory}}",
            params: &["newThemesDirectory"],
        },
    );
    translations.insert(
            MESSAGE_UPDATE_AVAILABLE,
            TranslationValue::Template {
                template: "새 버전 {{version}}이(가) 감지되었습니다. 현재 버전은 {{currentVersion}}입니다. 왼쪽 하단의 업그레이드 버튼을 클릭하여 다운로드 및 설치하세요.",
                params: &["version", "currentVersion"],
            },
        );
    translations.insert(
        MESSAGE_UPDATE_FAILED,
        TranslationValue::Template {
            template: "업그레이드 실패: \n{{error}}",
            params: &["error"],
        },
    );
    translations.insert(
        MESSAGE_VERSION_IS_THE_LATEST,
        TranslationValue::Text("현재가 최신 버전입니다"),
    );

    // units
    translations.insert(UNIT_HOUR, TranslationValue::Text("시간"));
    translations.insert(UNIT_SECOND, TranslationValue::Text("초"));

    // titles
    translations.insert(
        TITLE_DOWNLOAD_FAILD,
        TranslationValue::Text("다운로드 실패"),
    );
    translations.insert(
        TITLE_DOWNLOADING_NEW_VERSION,
        TranslationValue::Template {
            template: "새 버전 {{newVersion}} 다운로드 중...",
            params: &["newVersion"],
        },
    );

    // placeholders
    translations.insert(PLACEHOLDER_LATITUDE, TranslationValue::Text("위도"));
    translations.insert(PLACEHOLDER_LONGITUDE, TranslationValue::Text("경도"));

    translations
});
