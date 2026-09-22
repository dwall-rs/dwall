import type { RawDictionary } from ".";

export const dict: RawDictionary = {
  app: {
    mode: {
      fixed: "고정",
      random: "랜덤",
    },
    engine: {
      notRunning: "엔진 중지됨",
      running: "실행 중 · 현재",
      daily: "실행 중 · 매일 셔플",
      start: "엔진 시작",
      stop: "엔진 종료",
      confirmStop: "종료할까요?",
      altitude: "고도",
      azimuth: "방위",
    },
    appearance: {
      light: "라이트",
      dark: "다크",
      system: "시스템",
      title: "모양",
    },
  },

  library: {
    title: "테마 라이브러리",
    toggleOpen: "테마 라이브러리 접기",
    toggleClosed: "테마 라이브러리 펼치기",
    hintFixed: "클릭하여 {{ name }}에 지정",
    hintRandom: "클릭하여 풀에 추가/제거",
    search: "테마 검색…",
  },

  scope: {
    monitors: "범위 · 모니터",
    allMonitors: "모든 모니터",
    pool: "후보 풀",
    selectedCount: "선택됨 {{ n }}/{{ total }}",
    individual: "개별 설정",
    unified: "모든 모니터 통일",
    unifiedOn:
      "하나의 테마를 모든 모니터에 적용하며 아래 개별 설정은 잠깁니다.",
    unifiedOff: "끄면 모니터마다 테마를 지정할 수 있습니다.",
    searchThemes: "테마 검색…",
    randomEmpty:
      "후보 풀이 비어 있어 엔진이 고를 배경이 없습니다. 최소 하나는 남겨 두세요.",
    randomDirty: "후보 풀 {{ n }}개 · {{ excluded }}개 제외 · 저장 안 됨",
    randomNormal: "후보 풀 {{ n }}개 · 선택 해제 시 매일 셔플에서 제외",
  },

  stage: {
    notInstalled: "이 테마는 설치되지 않았거나 배경이 없습니다",
    applyTo: "이 테마를 {{ name }}에 기록",
    stop: "중지",
    apply: "이 테마 적용",
    altitude: "고도",
    azimuth: "방위",
    matched: "현재 일치",
    match: "일치",
    stripTitle: "포함된 배경 · 태양 위치로 일치",
    stripHint: "클릭하여 해당 태양 위치의 배경 미리보기",
    sunPath: "태양 경로",
    horizon: "0° 지평선",
    minus20: "−20°",
    east: "동·90°",
    south: "남·180°",
    west: "서·270°",
    randomIntro:
      "매일 후보 풀에서 테마 하나를 셔플해 모든 모니터에 적용합니다. 편집기는 풀만 정하고, 어떤 테마가 뽑힐지는 엔진이 런타임에 결정합니다.",
    stats: {
      pool: "후보 풀 / 개",
      range: "범위",
      all: "전체",
      exclude: "{{ n }}개 제외",
      target: "대상 모니터",
      period: "전환 주기",
    },
    collageTitle: "후보 풀 콜라주",
    collageNote:
      "{{ n }}개가 매일 셔플에 참여 · 풀 미리보기일 뿐, 추첨 결과가 아님",
    emptyTitle: "후보 풀이 비어 있음",
    emptyDesc:
      "모든 테마를 제외했습니다. 매일 셔플에 쓸 배경을 최소 하나 남겨 두세요.",
  },

  commit: {
    applied: "적용됨 {{ theme }}",
    notApplied: "적용 안 됨",
    unsaved: "저장되지 않은 변경",
    saved: "저장됨",
    savedAt: "저장됨 {{ time }}",
    discard: "취소",
    saving: "저장 중…",
    save: "설정 저장",
  },

  settings: {
    title: "설정",
    subtitle: "preferences",
    unsaved: "저장 안 됨",
    saveFailed: "저장 실패",
    discard: "취소",
    save: "설정 저장",
    saving: "저장 중…",
    intro:
      "환경설정은 엔진의 태양각 매칭 동작을 제어합니다. 토글은 즉시 기록되고, 저장 아이콘이 있는 항목은 확인이 필요합니다.",

    appearance: {
      title: "모양 및 언어",
      appearanceLabel: "모양",
      appearanceDesc:
        "라이트, 다크 또는 시스템을 선택하세요. 시스템은 자동으로 전환됩니다.",
      languageLabel: "언어",
      languageDesc: "인터페이스 언어이며 즉시 적용됩니다.",
    },

    engine: {
      title: "엔진 및 태양각 매칭",
      launchAtStartup: "시작 시 실행",
      launchAtStartupDesc:
        "백그라운드 엔진만 실행하며 창을 열지 않고 메모리를 거의 쓰지 않습니다.",
      interval: "확인 간격",
      intervalDesc:
        "엔진은 N초마다 태양 고도를 다시 계산하여 해당 시간의 배경을 고릅니다.",
      autoCoords: "좌표 자동 가져오기",
      autoCoordsDesc:
        "위도/경도는 태양 고도 계산에 사용됩니다. 끄면 직접 입력할 수 있습니다.",
      manualCoords: "수동 좌표",
      manualCoordsDesc: "위도 -90~90, 경도 -180~180, 고도(m).",
      latitude: "위도",
      longitude: "경도",
      altitude: "고도",
      save: "저장",
      lockScreen: "잠금 화면 배경도 설정",
      lockScreenDesc: "잠금 화면을 바꾸지 않으려면 끄세요.",
    },

    paths: {
      title: "디렉터리 및 다운로드",
      themesDir: "테마 디렉터리",
      themesDirDesc:
        "로컬 테마와 썸네일 저장 위치. 변경하면 기존 테마가 이동합니다.",
      selectDir: "디렉터리 선택",
      network: "네트워크",
      networkDesc:
        "테마 다운로드나 썸네일 로딩에는 GitHub 미러 템플릿 또는 SOCKS5 프록시가 필요할 수 있습니다.",
      none: "끄기",
      mirror: "미러",
      socks5: "SOCKS5",
      mirrorTemplate: "GitHub 미러 템플릿",
      mirrorDesc:
        "미러 템플릿은 다운로드를 가속합니다. 일부 지역에서는 GitHub 접근이 제한됩니다. 사용 가능한 템플릿:",
      viewTemplates: "템플릿 목록 보기 ↗",
      address: "호스트",
      port: "포트",
      save: "저장",
    },

    about: {
      title: "정보",
      tagline: "Dwall · 태양 위치 기반",
      sourceCode: "소스 코드",
      sourceSub: "github.com",
      logDir: "로그 디렉터리",
      logDirSub: "폴더 열기",
    },
  },
};
