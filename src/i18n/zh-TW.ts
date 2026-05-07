import type { RawDictionary } from ".";

export const dict: RawDictionary = {
  common: {
    message: {
      githubStar: "如果本應用對您有幫助，請在 GitHub 上給本專案點個 Star：",

      locationPermission:
        "位置權限未開啟。請手動開啟位置或手動設定座標。\n\n是否要手動設定座標？\n點擊「是」手動設定座標，點擊「否」開啟位置權限。",

      updateAvailable:
        "偵測到新版本 {{ version }}，目前版本為 {{ currentVersion }}。請點擊左下角的升級按鈕下載並安裝。",
    },
  },

  settings: {
    unit: {
      hour: "小時",
      second: "秒",
      minute: "分",
    },

    button: {
      openLogDirectory: "開啟日誌目錄",
      selectDirectory: "選擇目錄",
    },

    label: {
      automaticallyRetrieveCoordinates: "自動取得座標",
      automaticallySwitchModes: "自動切換深色或淺色模式",
      retrieveCoordinatesInterval: "座標擷取間隔",
      checkInterval: "檢查間隔",
      network: "網路",
      useSocks5: "使用 SOCKS5 代理",
      socks5: "SOCKS5",
      githubMirrorTemplate: "Github 鏡像模板",
      launchAtStartup: "開機自啟",
      setLockScreenWallpaperSimultaneously: "同時設定鎖定畫面桌布",
      sourceCode: "原始碼",
      themesDirectory: "主題目錄",
      version: "版本",
      language: "語言",
    },

    help: {
      automaticallySwitchModes:
        "如果您不希望自動切換淺色/深色模式，請停用此選項。",
      githubMirror:
        "Github 鏡像模板用於加速下載。在某些國家和地區，由於網路限制，存取 Github 可能會失敗，導致下載失敗。您需要設定 Github 鏡像模板才能正常載入縮圖和下載主題。點擊此按鈕可查看可用的 Github 鏡像模板：",
      socks5:
        "僅需輸入SOCKS5代理位址和連接埠號。如果你沒有有效的SOCKS5代理伺服器，請使用GitHub鏡像範本。",
      launchAtStartup:
        "開機自啟只會啟動背景程序，不會啟動圖形介面程式，且不會佔用過多記憶體。",
      manuallySetCoordinates:
        "手動設定座標時，必須使用 WGS84 座標系（國際標準，中國用戶請注意）。否則可能出現座標偏移問題，導致桌布對齊不準確。",
      setLockScreenWallpaperSimultaneously:
        "如果您不希望同時設定鎖定畫面桌布，請停用此選項。",
      updatedFailed:
        "無法完成熱更新，請點擊本訊息後面的下載按鈕手動下載新版本：",
      retrieveCoordinatesInterval:
        "獲取座標的時間間隔（單位：分鐘），該值必須大於偵測時間間隔。若您的電腦位置固定，可將間隔設為24小時或更長；若您經常攜帶電腦出差，建議設為1小時以內。",
    },

    placeholder: {
      latitude: "輸入緯度",
      longitude: "輸入經度",
    },

    tooltip: {
      openThemesDirectory: "點擊以開啟主題目錄。",
      checkForNewVersion: "點擊以檢查新版本",
    },

    message: {
      changeThemesDirectory: "將主題目錄更改為：{{ directory }}？",
      checkIntervalUpdated: "檢查間隔已更新為：{{ interval }} 秒",
      disableStartupFailed: "停用開機自啟失敗：\n{{ error }}",
      githubMirrorTemplateUpdated: "Github 鏡像模板已更新為：{{ template }}",
      invalidNumber: "請輸入有效的數字。",
      manualCoordinatesSaved: "座標已儲存，接下來您可以選擇要套用的主題。",
      numberTooLarge: "請輸入小於 {{ max }} 的數字。",
      numberTooSmall: "請輸入大於 {{ min }} 的數字。",
      SaveManualCoordinatesFailed: "儲存座標失敗：\n{{ error }}",
      startupFailed: "啟用開機自啟失敗：\n{{ error }}",
      switchAutoModesFailed: "切換自動淺色/深色模式失敗：\n{{ error }}",
      switchToManualCoordinatesFailed: "切換到手動座標失敗：\n{{ error }}",
      movedThemesDirectory: "主題目錄已移動到：{{ directory }}",
      isLatestVersion: "您已在使用最新版本。",
      checkIntervalUpdateFailed: "更新檢查間隔失敗：\n{{ error }}",
      saveRetrieveCoordinatesIntervalFailed:
        "儲存座標檢索間隔失敗：\n{{ error }}",
      socks5UpdateFailed: "更新SOCKS5設定失敗：\n{{ error }}",
      clearNetworkFailed: "清除網路設定失敗：\n{{ error }}",
    },
  },

  sidebar: {
    tooltip: {
      newVersionAvailable: "有新版本可用！點擊此按鈕進行更新。",
      settings: "設定",
    },
  },

  theme: {
    label: {
      selectMonitor: "選擇顯示器",
    },

    button: {
      apply: "套用",
      cancel: "取消",
      download: "下載",
      stop: "停止",
    },

    message: {
      applyThemeFailed: "套用主題失敗：\n{{ error }}",
      downloadCancelled: "下載已取消",
      downloadFailed: "下載主題失敗：\n{{ error }}",
      fileSizeWarning:
        "由於無法取得檔案大小，無法計算下載進度。請切換到支援轉發回應標頭的 Github 鏡像模板",
    },

    title: {
      downloadFailed: "下載失敗",
    },
  },

  update: {
    button: {
      install: "安装",
    },
    title: {
      downloadingNewVersion: "正在下載新版本 {{ version }}",
    },
    message: {
      updateFailed: "更新失敗：\n{{ error }}",
    },
  },
};
