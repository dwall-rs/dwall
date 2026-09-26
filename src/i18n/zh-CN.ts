import type { RawDictionary } from ".";

export const dict: RawDictionary = {
  common: {
    loading: "加载中",
  },

  app: {
    mode: {
      fixed: "固定",
      random: "随机",
    },
    engine: {
      notRunning: "引擎未运行",
      running: "运行中 · 当前",
      daily: "运行中 · 每日洗牌",
      start: "启动引擎",
      stop: "终止引擎",
      confirmStop: "确认终止？",
      altitude: "高度",
      azimuth: "方位",
    },
    appearance: {
      light: "亮色",
      dark: "暗色",
      system: "跟随系统",
      title: "外观",
    },
  },

  library: {
    title: "主题库",
    toggleOpen: "收起主题库",
    toggleClosed: "展开主题库",
    hintFixed: "点击指定给{{ name }}",
    hintRandom: "点击加入 / 移出候选池",
    search: "搜索主题…",
  },

  scope: {
    monitors: "作用域 · 显示器",
    allMonitors: "所有显示器",
    individual: "单独设置",
    unified: "统一所有显示器",
    unifiedOn: "一套主题应用于全部显示器，下方单独设置已锁定。",
    unifiedOff: "关闭后可为每台显示器分别指定主题。",
    randomEmpty: "候选池为空，引擎将无壁纸可抽，请至少保留一套。",
    randomDirty: "候选池 {{ n }} 套 · 已排除 {{ excluded }} · 未保存",
    randomNormal: "候选池 {{ n }} 套 · 取消勾选即从每日洗牌中排除",
  },

  stage: {
    notInstalled: "该主题尚未安装或无壁纸",
    applyTo: "将这套主题写配置到{{ name }}",
    stop: "停止",
    apply: "应用此套",
    altitude: "高度",
    azimuth: "方位",
    matched: "当前匹配",
    match: "匹配",
    stripTitle: "套内壁纸 · 按太阳位置匹配",
    stripHint: "点击预览该太阳位置下的壁纸",
    sunPath: "太阳路径",
    horizon: "0° 地平线",
    east: "东·90°",
    south: "南·180°",
    west: "西·270°",
    randomIntro:
      "每天从候选池洗牌抽取一套，同时应用到所有显示器。编辑器只负责选定池子，抽中哪套由引擎在运行时决定。",
    stats: {
      pool: "候选池 / 套",
      range: "范围",
      all: "全部",
      exclude: "排除 {{ n }}",
      target: "目标显示器",
      period: "切换周期",
      today: "今日主题",
      targetValue: "全部",
      periodValue: "24 小时",
    },
    collageTitle: "候选池拼贴",
    collageNote: "{{ n }} 套参与每日洗牌 · 仅预览池内容，非抽中结果",
    emptyTitle: "候选池为空",
    emptyDesc: "你已排除全部主题。请至少保留一套，让每日洗牌有壁纸可抽。",
  },

  commit: {
    applied: "已应用 {{ theme }}",
    notApplied: "未应用",
    unsaved: "未保存的更改",
    saved: "已保存",
    savedAt: "已保存 {{ time }}",
    discard: "放弃",
    saving: "写入中…",
    save: "保存配置",
  },

  settings: {
    title: "设置",
    subtitle: "preferences",
    unsaved: "未保存",
    saveFailed: "保存失败",
    discard: "放弃",
    save: "保存配置",
    saving: "写入中…",
    intro:
      "偏好控制引擎进程与太阳角匹配的行为。开关即时写入配置；带保存图标的字段需手动确认。",

    appearance: {
      title: "外观与语言",
      appearanceLabel: "外观 / Appearance",
      appearanceDesc:
        "选择亮色、暗色，或跟随系统。跟随系统会实时监听系统外观变化自动切换。",
      languageLabel: "语言 / Language",
      languageDesc: "界面语言，切换后立即生效。",
    },

    engine: {
      title: "引擎与太阳角匹配",
      launchAtStartup: "开机启动",
      launchAtStartupDesc: "仅启动后台引擎进程，不打开图形界面，几乎不占内存。",
      interval: "检查间隔",
      secondsUnit: "秒",
      intervalDesc:
        "引擎每隔 N 秒重算一次太阳高度角，命中对应时段的壁纸。固定模式无固定切换时间，全靠此轮询驱动。",
      autoCoords: "自动获取坐标",
      autoCoordsDesc:
        "经纬度用于计算太阳高度角以匹配壁纸。关闭后可手动填写坐标。",
      manualCoords: "手动坐标",
      manualCoordsDesc: "纬度 -90~90、经度 -180~180、海拔（米）。",
      latitude: "纬度",
      longitude: "经度",
      altitude: "海拔",
      save: "保存",
      lockScreen: "同时设置锁屏壁纸",
      lockScreenDesc: "若不希望锁屏与桌面同步更换，请关闭此项。",
    },

    paths: {
      title: "目录与下载源",
      themesDir: "主题目录",
      themesDirDesc:
        "本地主题与缩略图的存放位置。切换目录会把现有主题迁移过去。",
      selectDir: "选择目录",
      openDir: "打开目录",
      network: "网络设置",
      networkDesc:
        "下载主题或者加载缩略图失败时可能需要配置网络，包括 Github 镜像模板和 SOCKS5 代理。",
      none: "关闭",
      mirror: "镜像",
      socks5: "SOCKS5",
      mirrorTemplate: "Github 镜像模板",
      mirrorDesc:
        "镜像模板用于加速下载。部分国家或地区因网络限制访问 Github 可能失败，需配置镜像模板。点此查看可用模板：",
      viewTemplates: "查看模板列表 ↗",
      address: "地址",
      port: "端口",
      save: "保存",
    },

    about: {
      title: "关于",
      tagline: "Dwall · 太阳位置驱动",
      logoAlt: "Dwall 徽标",
      sourceCode: "源代码",
      sourceSub: "github.com",
      logDir: "日志目录",
      logDirSub: "打开文件夹",
      checkUpdate: "检查更新",
      checking: "检查中…",
      upToDate: "已是最新版本",
      available: "可更新至 {{ version }}",
      download: "下载并安装",
      downloading: "下载中…",
      ready: "重启以完成更新",
      failed: "检查更新失败",
      retry: "重试",
    },
  },
};
