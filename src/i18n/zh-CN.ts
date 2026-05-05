import type { RawDictionary } from ".";

export const dict: RawDictionary = {
  common: {
    message: {
      githubStar: "如果本应用对您有帮助，请在 GitHub 上给本项目点个 Star：",

      locationPermission:
        "位置权限未开启。请手动开启位置或手动设置坐标。\n\n是否要手动设置坐标？\n点击“是”手动设置坐标，点击“否”开启位置权限。",

      updateAvailable:
        "检测到新版本 {{ version }}，当前版本为 {{ currentVersion }}。请点击左下角的升级按钮下载并安装。",
    },
  },

  settings: {
    unit: {
      hour: "小时",
      second: "秒",
    },

    button: {
      openLogDirectory: "打开日志目录",
      selectDirectory: "选择目录",
    },

    label: {
      automaticallyRetrieveCoordinates: "自动获取坐标",
      automaticallySwitchModes: "自动切换深色或浅色模式",
      checkInterval: "检查间隔",
      githubMirrorTemplate: "Github 镜像模板",
      launchAtStartup: "开机自启",
      setLockScreenWallpaperSimultaneously: "同时设置锁屏壁纸",
      sourceCode: "源代码",
      themesDirectory: "主题目录",
      version: "版本",
      language: "语言",
    },

    help: {
      automaticallySwitchModes:
        "如果您不希望自动切换浅色/深色模式，请禁用此选项。",
      githubMirror:
        "Github 镜像模板用于加速下载。在某些国家和地区，由于网络限制，访问 Github 可能会失败，导致下载失败。您需要设置 Github 镜像模板才能正常加载缩略图和下载主题。点击此按钮可查看可用的 Github 镜像模板：",
      launchAtStartup:
        "开机自启仅会启动后台进程，不会启动图形界面程序，且不会占用过多内存。",
      manuallySetCoordinates:
        "手动设置坐标时，必须使用 WGS84 坐标系（国际标准，中国用户请注意）。否则可能出现坐标偏移问题，导致壁纸对齐不准确。",
      setLockScreenWallpaperSimultaneously:
        "如果您不希望同时设置锁屏壁纸，请禁用此选项。",
      updatedFailed:
        "无法完成热更新，请点击本消息后面的下载按钮手动下载新版本：",
    },

    placeholder: {
      latitude: "输入纬度",
      longitude: "输入经度",
    },

    tooltip: {
      openThemesDirectory: "点击打开主题目录。",
      checkForNewVersion: "点击检查新版本",
    },

    message: {
      changeThemesDirectory: "将主题目录更改为：{{ directory }}？",
      checkIntervalUpdated: "检查间隔已更新为：{{ interval }} 秒",
      disableStartupFailed: "禁用开机自启失败：\n{{ error }}",
      githubMirrorTemplateUpdated: "Github 镜像模板已更新为：{{ template }}",
      invalidNumber: "请输入有效的数字。",
      manualCoordinatesSaved: "坐标已保存，接下来您可以选择要应用的主题。",
      numberTooLarge: "请输入小于 {{ max }} 的数字。",
      numberTooSmall: "请输入大于 {{ min }} 的数字。",
      SaveManualCoordinatesFailed: "保存坐标失败：\n{{ error }}",
      startupFailed: "启用开机自启失败：\n{{ error }}",
      switchAutoModesFailed: "切换自动浅色/深色模式失败：\n{{ error }}",
      switchToManualCoordinatesFailed: "切换到手动坐标失败：\n{{ error }}",
      movedThemesDirectory: "主题目录已移动到：{{ directory }}",
      isLatestVersion: "您已在使用最新版本。",
    },
  },

  sidebar: {
    tooltip: {
      newVersionAvailable: "有新版本可用！点击此按钮进行更新。",
      settings: "设置",
    },
  },

  theme: {
    label: {
      selectMonitor: "选择显示器",
    },

    button: {
      apply: "应用",
      cancel: "取消",
      download: "下载",
      stop: "停止",
    },

    message: {
      applyThemeFailed: "应用主题失败：\n{{ error }}",
      downloadCancelled: "下载已取消",
      downloadFailed: "下载主题失败：\n{{ error }}",
      fileSizeWarning:
        "由于无法获取文件大小，无法计算下载进度。请切换到支持转发响应头的 Github 镜像模板",
    },

    title: {
      downloadFailed: "下载失败",
    },
  },

  update: {
    button: {
      install: "安装",
    },
    title: {
      downloadingNewVersion: "正在下载新版本 {{ version }}",
    },
    message: {
      updateFailed: "更新失败：\n{{ error }}",
    },
  },
};
