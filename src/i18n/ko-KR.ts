import type { RawDictionary } from ".";

export const dict: RawDictionary = {
  common: {
    message: {
      githubStar:
        "이 애플리케이션이 도움이 되셨다면 GitHub에서 이 프로젝트에 Star를 부탁드립니다:",

      locationPermission:
        "위치 권한이 활성화되지 않았습니다. 수동으로 위치를 활성화하거나 좌표를 수동으로 설정해 주세요.\n\n좌표를 수동으로 설정하시겠습니까?\n'예'를 클릭하면 좌표를 수동으로 설정하고, '아니요'를 클릭하면 위치 권한을 활성화합니다.",

      updateAvailable:
        "새 버전 {{ version }}이(가) 감지되었습니다. 현재 버전은 {{ currentVersion }}입니다. 왼쪽 하단의 업그레이드 버튼을 클릭하여 다운로드 및 설치하세요.",
    },
  },

  settings: {
    unit: {
      hour: "시간",
      second: "초",
      minute: "분",
    },

    button: {
      openLogDirectory: "로그 디렉토리 열기",
      selectDirectory: "디렉토리 선택",
    },

    label: {
      automaticallyRetrieveCoordinates: "좌표 자동 가져오기",
      automaticallySwitchModes: "다크 또는 라이트 모드 자동 전환",
      retrieveCoordinatesInterval: "좌표 획득 간격",
      checkInterval: "확인 간격",
      githubMirrorTemplate: "Github 미러 템플릿",
      launchAtStartup: "시작 시 실행",
      setLockScreenWallpaperSimultaneously: "잠금 화면 배경화면도 동시에 설정",
      sourceCode: "소스 코드",
      themesDirectory: "테마 디렉토리",
      version: "버전",
      language: "언어",
    },

    help: {
      automaticallySwitchModes:
        "라이트/다크 모드를 자동으로 전환하지 않으려면 이 옵션을 비활성화하세요.",
      githubMirror:
        "Github 미러 템플릿은 다운로드를 가속화하는 데 사용됩니다. 일부 국가 및 지역에서는 네트워크 제한으로 인해 Github에 액세스하지 못해 다운로드가 실패할 수 있습니다. 썸네일을 올바르게 로드하고 테마를 다운로드하려면 Github 미러 템플릿을 설정해야 합니다. 이 버튼을 클릭하면 사용 가능한 Github 미러 템플릿을 볼 수 있습니다:",
      launchAtStartup:
        "자동 시작은 백그라운드 프로세스만 실행하며 그래픽 프로그램은 실행되지 않습니다. 또한 메모리를 많이 소모하지 않습니다.",
      manuallySetCoordinates:
        "수동으로 좌표를 설정할 때는 WGS84 좌표계(국제 표준, 중국 사용자는 주의)를 사용해야 합니다. 그렇지 않으면 좌표 오프셋 문제가 발생하여 배경화면 정렬이 부정확해질 수 있습니다.",
      setLockScreenWallpaperSimultaneously:
        "잠금 화면 배경화면도 동시에 설정하지 않으려면 이 옵션을 비활성화하세요.",
      updatedFailed:
        "핫 업데이트를 완료할 수 없습니다. 이 메시지 뒤의 다운로드 버튼을 클릭하여 새 버전을 수동으로 다운로드하세요:",
      retrieveCoordinatesInterval:
        "좌표 획득 간격(단위: 분)은 감지 간격보다 커야 합니다. 컴퓨터 위치가 고정된 경우 간격을 24시간 이상으로 설정할 수 있습니다. 자주 노트북을 가지고 출장을 다니는 경우 1시간 이내로 설정하는 것이 좋습니다.",
    },

    placeholder: {
      latitude: "위도 입력",
      longitude: "경도 입력",
    },

    tooltip: {
      openThemesDirectory: "클릭하여 테마 디렉토리를 엽니다.",
      checkForNewVersion: "클릭하여 새 버전 확인",
    },

    message: {
      changeThemesDirectory:
        "테마 디렉토리를 {{ directory }}(으)로 변경하시겠습니까?",
      checkIntervalUpdated:
        "확인 간격이 {{ interval }}초로 업데이트되었습니다.",
      disableStartupFailed: "시작 시 실행 비활성화 실패: \n{{ error }}",
      githubMirrorTemplateUpdated:
        "Github 미러 템플릿이 {{ template }}(으)로 업데이트되었습니다.",
      invalidNumber: "유효한 숫자를 입력하세요.",
      manualCoordinatesSaved:
        "좌표가 저장되었습니다. 이제 적용할 테마를 선택할 수 있습니다.",
      numberTooLarge: "{{ max }}보다 작은 숫자를 입력하세요.",
      numberTooSmall: "{{ min }}보다 큰 숫자를 입력하세요.",
      SaveManualCoordinatesFailed: "좌표 저장 실패: \n{{ error }}",
      startupFailed: "시작 시 실행 활성화 실패: \n{{ error }}",
      switchAutoModesFailed: "자동 라이트/다크 모드 전환 실패: \n{{ error }}",
      switchToManualCoordinatesFailed: "수동 좌표로 전환 실패: \n{{ error }}",
      movedThemesDirectory:
        "테마 디렉토리가 {{ directory }}(으)로 이동되었습니다.",
      isLatestVersion: "이미 최신 버전을 사용하고 있습니다.",
      checkIntervalUpdateFailed:
        "업데이트 확인 간격 업데이트에 실패했습니다: \n{{ error }}",
      saveRetrieveCoordinatesIntervalFailed:
        "좌표 검색 간격 저장 실패:\n{{ error }}",
    },
  },

  sidebar: {
    tooltip: {
      newVersionAvailable:
        "새 버전을 사용할 수 있습니다! 이 버튼을 클릭하여 업데이트하세요.",
      settings: "설정",
    },
  },

  theme: {
    label: {
      selectMonitor: "모니터 선택",
    },

    button: {
      apply: "적용",
      cancel: "취소",
      download: "다운로드",
      stop: "중지",
    },

    message: {
      applyThemeFailed: "테마 적용 실패: \n{{ error }}",
      downloadCancelled: "다운로드가 취소되었습니다.",
      downloadFailed: "테마 다운로드 실패: \n{{ error }}",
      fileSizeWarning:
        "파일 크기를 가져올 수 없어 다운로드 진행률을 계산할 수 없습니다. 응답 헤더 전달을 지원하는 Github 미러 템플릿으로 전환하세요.",
    },

    title: {
      downloadFailed: "다운로드 실패",
    },
  },

  update: {
    button: {
      install: "설치",
    },
    title: {
      downloadingNewVersion: "새 버전 {{ version }} 다운로드 중",
    },
    message: {
      updateFailed: "업데이트 실패: \n{{ error }}",
    },
  },
};
