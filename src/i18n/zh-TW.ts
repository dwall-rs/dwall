import type { RawDictionary } from ".";

export const dict: RawDictionary = {
  common: {
    loading: "載入中",
  },

  app: {
    mode: {
      fixed: "固定",
      random: "隨機",
    },
    engine: {
      notRunning: "引擎未執行",
      running: "執行中 · 目前",
      daily: "執行中 · 每日洗牌",
      start: "啟動引擎",
      stop: "終止引擎",
      confirmStop: "確認終止？",
      altitude: "高度",
      azimuth: "方位",
    },
    appearance: {
      light: "亮色",
      dark: "暗色",
      system: "跟隨系統",
      title: "外觀",
    },
  },

  library: {
    title: "主題庫",
    toggleOpen: "收合主題庫",
    toggleClosed: "展開主題庫",
    hintFixed: "點擊指定給{{ name }}",
    hintRandom: "點擊加入 / 移出候選池",
    search: "搜尋主題…",
  },

  scope: {
    monitors: "作用域 · 顯示器",
    allMonitors: "所有顯示器",
    pool: "候選池",
    selectedCount: "已選 {{ n }}/{{ total }}",
    individual: "個別設定",
    unified: "統一所有顯示器",
    unifiedOn: "一套主題套用於全部顯示器，下方個別設定已鎖定。",
    unifiedOff: "關閉後可為每台顯示器分別指定主題。",
    searchThemes: "搜尋主題…",
    randomEmpty: "候選池為空，引擎將無桌布可抽，請至少保留一套。",
    randomDirty: "候選池 {{ n }} 套 · 已排除 {{ excluded }} · 未儲存",
    randomNormal: "候選池 {{ n }} 套 · 取消勾選即從每日洗牌中排除",
  },

  stage: {
    notInstalled: "該主題尚未安裝或無桌布",
    applyTo: "將這套主題寫入設定到{{ name }}",
    stop: "停止",
    apply: "套用此套",
    altitude: "高度",
    azimuth: "方位",
    matched: "目前符合",
    match: "符合",
    stripTitle: "套內桌布 · 依太陽位置符合",
    stripHint: "點擊預覽該太陽位置下的桌布",
    sunPath: "太陽路徑",
    horizon: "0° 地平線",
    east: "東·90°",
    south: "南·180°",
    west: "西·270°",
    randomIntro:
      "每天從候選池洗牌抽取一套，同時套用到所有顯示器。編輯器只負責選定池子，抽中哪套由引擎在執行時決定。",
    stats: {
      pool: "候選池 / 套",
      range: "範圍",
      all: "全部",
      exclude: "排除 {{ n }}",
      target: "目標顯示器",
      period: "切換週期",
      targetValue: "全部",
      periodValue: "24 小時",
    },
    collageTitle: "候選池拼貼",
    collageNote: "{{ n }} 套參與每日洗牌 · 僅預覽池內容，非抽中結果",
    emptyTitle: "候選池為空",
    emptyDesc: "你已排除全部主題。請至少保留一套，讓每日洗牌有桌布可抽。",
  },

  commit: {
    applied: "已套用 {{ theme }}",
    notApplied: "未套用",
    unsaved: "未儲存的變更",
    saved: "已儲存",
    savedAt: "已儲存 {{ time }}",
    discard: "放棄",
    saving: "寫入中…",
    save: "儲存設定",
  },

  settings: {
    title: "設定",
    subtitle: "preferences",
    unsaved: "未儲存",
    saveFailed: "儲存失敗",
    discard: "放棄",
    save: "儲存設定",
    saving: "寫入中…",
    intro:
      "偏好控制引擎程序與太陽角符合的行為。開關即時寫入設定；帶儲存圖示的欄位需手動確認。",

    appearance: {
      title: "外觀與語言",
      appearanceLabel: "外觀 / Appearance",
      appearanceDesc:
        "選擇亮色、暗色，或跟隨系統。跟隨系統會即時監聽系統外觀變化自動切換。",
      languageLabel: "語言 / Language",
      languageDesc: "介面語言，切換後立即生效。",
    },

    engine: {
      title: "引擎與太陽角符合",
      launchAtStartup: "開機啟動",
      launchAtStartupDesc:
        "僅啟動背景引擎程序，不開啟圖形介面，幾乎不佔記憶體。",
      interval: "檢查間隔",
      secondsUnit: "秒",
      intervalDesc:
        "引擎每隔 N 秒重算一次太陽高度角，命中對應時段的桌布。固定模式無固定切換時間，全靠此輪詢驅動。",
      autoCoords: "自動取得座標",
      autoCoordsDesc:
        "經緯度用於計算太陽高度角以符合桌布。關閉後可手動填寫座標。",
      manualCoords: "手動座標",
      manualCoordsDesc: "緯度 -90~90、經度 -180~180、海拔（公尺）。",
      latitude: "緯度",
      longitude: "經度",
      altitude: "海拔",
      save: "儲存",
      lockScreen: "同時設定鎖定畫面桌布",
      lockScreenDesc: "若不希望鎖定畫面與桌面同步更換，請關閉此項。",
    },

    paths: {
      title: "目錄與下載來源",
      themesDir: "主題目錄",
      themesDirDesc: "本機主題與縮圖的存放位置。切換目錄會把現有主題遷移過去。",
      selectDir: "選擇目錄",
      openDir: "開啟目錄",
      network: "網路設定",
      networkDesc:
        "下載主題或載入縮圖失敗時可能需要設定網路，包括 Github 鏡像範本和 SOCKS5 代理。",
      none: "關閉",
      mirror: "鏡像",
      socks5: "SOCKS5",
      mirrorTemplate: "Github 鏡像範本",
      mirrorDesc:
        "鏡像範本用於加速下載。部分國家或地區因網路限制存取 Github 可能失敗，需設定鏡像範本。點此查看可用範本：",
      viewTemplates: "查看範本清單 ↗",
      address: "位址",
      port: "連接埠",
      save: "儲存",
    },

    about: {
      title: "關於",
      tagline: "Dwall · 太陽位置驅動",
      logoAlt: "Dwall 標誌",
      sourceCode: "原始碼",
      sourceSub: "github.com",
      logDir: "日誌目錄",
      logDirSub: "開啟資料夾",
      checkUpdate: "檢查更新",
      checking: "檢查中…",
      upToDate: "已是最新版本",
      available: "可更新至 {{ version }}",
      download: "下載並安裝",
      downloading: "下載中…",
      ready: "重新啟動以完成更新",
      failed: "檢查更新失敗",
      retry: "重試",
    },
  },
};
