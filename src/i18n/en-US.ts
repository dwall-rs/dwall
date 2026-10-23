export const dict = {
  common: {
    loading: "Loading",
    operationFailed: "Operation failed",
  },

  app: {
    mode: {
      fixed: "Fixed",
      random: "Random",
    },
    engine: {
      notRunning: "Engine not running",
      running: "Running · {{ mode }} · now",
      start: "Start engine",
      stop: "Stop engine",
      confirmStop: "Confirm stop?",
      lastError: "Last error:",
      altitude: "alt",
      azimuth: "az",
    },
    appearance: {
      light: "Light",
      dark: "Dark",
      system: "System",
      title: "Appearance",
    },
  },

  library: {
    title: "Themes",
    toggleOpen: "Collapse theme library",
    toggleClosed: "Expand theme library",
    hintFixed: "Click to assign to {{ name }}",
    hintRandom: "Click to add / remove from the pool",
    search: "Search themes…",
    loadFailed: "Could not read the themes",
    empty: "No themes yet",
    emptyDesc: "Put themes in the themes directory, then reload.",
    noMatch: "No theme matches “{{ query }}”",
    reload: "Reload",
  },

  scope: {
    monitors: "Scope · Monitors",
    allMonitors: "All monitors",
    selectAll: "Select all",
    clearAll: "Clear",
    individual: "Per monitor",
    unified: "Unify all monitors",
    unifiedOn:
      "One theme applied to all monitors; per-monitor settings below are locked.",
    unifiedOff: "Turn off to assign a theme to each monitor.",
    randomEmpty:
      "The pool is empty — the engine has no wallpaper to pick. Keep at least one.",
    randomDirty: "Pool {{ n }} themes · {{ excluded }} excluded · unsaved",
    randomNormal:
      "Pool {{ n }} themes · uncheck to exclude from the daily shuffle",
  },

  stage: {
    notInstalled: "This theme is not installed or has no wallpapers",
    download: "Download",
    downloading: "Downloading…",
    cancelDownload: "Cancel",
    downloadFailed: "Download failed:",
    applyTo: "Write this theme to {{ name }}",
    stop: "Stop",
    apply: "Apply this theme",
    altitude: "alt",
    azimuth: "az",
    matched: "Matched",
    match: "Match",
    stripTitle: "Wallpapers · matched by sun position",
    stripHint: "Click to preview the wallpaper at that sun position",
    sunPath: "Sun path",
    horizon: "0° horizon",
    east: "E·90°",
    south: "S·180°",
    west: "W·270°",
    randomSunPath: "Today's theme — {{ name }} — on the sun's path",
    randomIntro:
      "Every day one theme is shuffled out of the pool and applied to all monitors. The editor only picks the pool; which theme is drawn is decided by the engine at runtime.",
    stats: {
      pool: "Pool / themes",
      range: "Range",
      all: "All",
      exclude: "Exclude {{ n }}",
      target: "Target monitors",
      period: "Cycle",
      today: "Today's theme",
      targetValue: "ALL",
      periodValue: "24h",
    },
    collageTitle: "Pool collage",
    collageNote:
      "{{ n }} themes in the daily shuffle · pool preview only, not the drawn result",
    emptyTitle: "Pool is empty",
    emptyDesc:
      "All themes are excluded. Keep at least one theme so the daily shuffle has wallpapers.",
  },

  commit: {
    applied: "Applied {{ theme }}",
    notApplied: "Not applied",
    discard: "Discard",
    saving: "Saving…",
    save: "Save",
  },

  settings: {
    title: "Settings",
    subtitle: "preferences",
    unsaved: "Unsaved",
    saveFailed: "Save failed",
    loadFailed: "Could not read the configuration",
    retry: "Retry",
    discard: "Discard",
    save: "Save configuration",
    saving: "Saving…",
    intro:
      "Preferences control how the engine matches sun angles. Toggles are written immediately; fields with a save icon need confirmation.",

    appearance: {
      title: "Appearance & language",
      appearanceLabel: "Appearance",
      appearanceDesc:
        "Choose light, dark, or follow the system. Following the system switches automatically.",
      titleBar: "Title bar follows the Windows theme",
      titleBarDesc:
        "Off: the title bar is painted with Dwall's background colour, so it matches the window. Matching the colour needs Windows 11; Windows 10 follows the light/dark side only.",
      contentBorder: "Separate the title bar from the content",
      contentBorderDesc:
        "Draws a border along the top of the content area, so the title bar is easy to find — and to grab — when it is the same colour as the window.",
      languageLabel: "Language",
      languageDesc: "Interface language; takes effect immediately.",
    },

    engine: {
      title: "Engine & sun matching",
      launchAtStartup: "Launch at startup",
      launchAtStartupDesc:
        "Starts only the background engine, no window, and uses almost no memory.",
      interval: "Check interval",
      secondsUnit: "s",
      intervalDesc:
        "The engine recomputes the sun altitude every N seconds and picks the wallpaper for that time.",
      autoCoords: "Automatically retrieve coordinates",
      autoCoordsDesc:
        "Coordinates are used to compute the sun altitude. Turn off to enter them manually.",
      manualCoords: "Manual coordinates",
      manualCoordsDesc: "Latitude -90~90, longitude -180~180, altitude (m).",
      latitude: "Latitude",
      longitude: "Longitude",
      altitude: "Altitude",
      save: "Save",
      lockScreen: "Set lock screen wallpaper too",
      lockScreenDesc: "Turn off to keep the lock screen unchanged.",
      colorScheme: "Switch the system light/dark scheme",
      colorSchemeDesc:
        "Follows the sun instead of the clock: light after sunrise, dark after sunset (civil twilight, -6°), with a dead zone around the change.",
    },

    paths: {
      title: "Directories & download",
      themesDir: "Themes directory",
      themesDirDesc:
        "Where local themes and thumbnails are stored. Changing it moves the existing themes.",
      selectDir: "Select directory",
      openDir: "Open folder",
      network: "Network",
      networkDesc:
        "Downloading themes or loading thumbnails may need a GitHub mirror template or a SOCKS5 proxy.",
      none: "Off",
      mirror: "Mirror",
      socks5: "SOCKS5",
      mirrorTemplate: "GitHub mirror template",
      mirrorDesc:
        "The mirror template speeds up downloads. In some regions GitHub is restricted. View available templates:",
      viewTemplates: "View template list ↗",
      address: "Host",
      port: "Port",
      save: "Save",
    },

    about: {
      title: "About",
      tagline: "Dwall · solar-driven",
      logoAlt: "Dwall logo",
      sourceCode: "Source code",
      changelog: "Changelog",
      changelogSub: "Release notes",
      whatsNewFor: "What's new in {{ version }}",
      whatsNew: "What's new",
      sourceSub: "github.com",
      logDir: "Log directory",
      logDirSub: "Open folder",
      checkUpdate: "Check for updates",
      checking: "Checking…",
      updateToastTitle: "Dwall {{ version }} is available",
      updateToastHint: "Open About to download and install it.",
      updateToastAction: "Show me",
      upToDate: "Up to date",
      available: "{{ version }} available",
      download: "Download & install",
      downloading: "Downloading…",
      ready: "Restart to finish",
      failed: "Update check failed",
      retry: "Retry",
    },
  },
};
