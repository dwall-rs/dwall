import type { RawDictionary } from ".";

export const dict: RawDictionary = {
  common: {
    message: {
      githubStar: "如果本應用對你有幫助，請喺 GitHub 上俾本項目點個 Star：",

      locationPermission:
        "位置權限未開啟。請手動開啟位置或手動設定坐標。\n\n是否要手動設定坐標？\n撳「是」手動設定坐標，撳「否」開啟位置權限。",

      updateAvailable:
        "偵測到新版本 {{ version }}，當前版本為 {{ currentVersion }}。請撳左下角嘅升級按鈕下載並安裝。",
    },
  },

  settings: {
    unit: {
      hour: "小時",
      second: "秒",
      minute: "分",
    },

    button: {
      openLogDirectory: "打開日誌目錄",
      selectDirectory: "選擇目錄",
    },

    label: {
      automaticallyRetrieveCoordinates: "自動獲取坐標",
      automaticallySwitchModes: "自動切換深色或淺色模式",
      retrieveCoordinatesInterval: "座標獲取間隔",
      checkInterval: "檢查間隔",
      network: "網絡",
      useSocks5: "使用 SOCKS5 代理",
      socks5: "SOCKS5",
      githubMirrorTemplate: "Github 鏡像模板",
      launchAtStartup: "開機自啟",
      setLockScreenWallpaperSimultaneously: "同時設定鎖定畫面壁紙",
      sourceCode: "源代碼",
      themesDirectory: "主題目錄",
      version: "版本",
      language: "語言",
    },

    help: {
      automaticallySwitchModes:
        "如果你唔希望自動切換淺色/深色模式，請停用此選項。",
      githubMirror:
        "Github 鏡像模板用於加速下載。喺某啲國家同地區，由於網絡限制，訪問 Github 可能會失敗，導致下載失敗。你需要設定 Github 鏡像模板先可以正常載入縮圖同下載主題。撳此按鈕可查看可用嘅 Github 鏡像模板：",
      socks5:
        "僅需輸入SOCKS5代理地址和端口號。如果你沒有有效的SOCKS5代理伺服器，請使用GitHub鏡像模板。",
      launchAtStartup:
        "開機自啟只會啟動背景進程，唔會啟動圖形介面程式，且唔會佔用過多記憶體。",
      manuallySetCoordinates:
        "手動設定坐標時，必須使用 WGS84 坐標系（國際標準，中國用戶請注意）。否則可能出現坐標偏移問題，導致壁紙對齊唔準確。",
      setLockScreenWallpaperSimultaneously:
        "如果你唔希望同時設定鎖定畫面壁紙，請停用此選項。",
      updatedFailed: "無法完成熱更新，請撳本訊息後面嘅下載按鈕手動下載新版本：",
      retrieveCoordinatesInterval:
        "獲取坐標嘅時間間隔（單位：分鐘），呢個值必須大過檢測時間間隔。若果你嘅電腦位置固定，可以將間隔設為24小時或更長；若果你經常帶住電腦出差，建議設為1小時以內。",
    },

    placeholder: {
      latitude: "輸入緯度",
      longitude: "輸入經度",
    },

    tooltip: {
      openThemesDirectory: "撳此打開主題目錄。",
      checkForNewVersion: "撳此檢查新版本",
    },

    message: {
      changeThemesDirectory: "將主題目錄更改為：{{ directory }}？",
      checkIntervalUpdated: "檢查間隔已更新為：{{ interval }} 秒",
      disableStartupFailed: "停用開機自啟失敗：\n{{ error }}",
      githubMirrorTemplateUpdated: "Github 鏡像模板已更新為：{{ template }}",
      invalidNumber: "請輸入有效嘅數字。",
      manualCoordinatesSaved: "坐標已保存，接下來你可以選擇要套用嘅主題。",
      numberTooLarge: "請輸入小於 {{ max }} 嘅數字。",
      numberTooSmall: "請輸入大於 {{ min }} 嘅數字。",
      SaveManualCoordinatesFailed: "保存坐標失敗：\n{{ error }}",
      startupFailed: "啟用開機自啟失敗：\n{{ error }}",
      switchAutoModesFailed: "切換自動淺色/深色模式失敗：\n{{ error }}",
      switchToManualCoordinatesFailed: "切換到手動坐標失敗：\n{{ error }}",
      movedThemesDirectory: "主題目錄已移動到：{{ directory }}",
      isLatestVersion: "你已在使用最新版本。",
      checkIntervalUpdateFailed: "更新檢查間隔失敗：\n{{ error }}",
      saveRetrieveCoordinatesIntervalFailed:
        "儲存座標檢索間隔失敗：\n{{ error }}",
      socks5UpdateFailed: "更新SOCKS5設定失敗：\n{{ error }}",
      clearNetworkFailed: "清除網絡設定失敗：\n{{ error }}",
    },
  },

  sidebar: {
    tooltip: {
      newVersionAvailable: "有新版本可用！撳此按鈕進行更新。",
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
        "由於無法獲取檔案大小，無法計算下載進度。請切換到支援轉發回應標頭嘅 Github 鏡像模板",
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
