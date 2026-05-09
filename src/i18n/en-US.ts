export const dict = {
  common: {
    message: {
      githubStar:
        "If this application has helped you, please consider giving this project a star on GitHub:",

      locationPermission:
        "Location permission is not enabled. Please manually enable location or set coordinates manually.\n\nDo you want to set coordinates manually?\nClick 'Yes' to set coordinates manually, click 'No' to enable location.",

      updateAvailable:
        "New version {{ version }} detected, current version is {{ currentVersion }}. Please click the upgrade button in the lower left corner to download and install.",
    },
  },

  settings: {
    unit: {
      hour: "h",
      second: "s",
      minute: "m",
    },

    button: {
      openLogDirectory: "Open Log Directory",
      selectDirectory: "Select Directory",
    },

    label: {
      automaticallyRetrieveCoordinates: "Automatically Retrieve Coordinates",
      retrieveCoordinatesInterval: "Coordinates Retrieval Interval",
      automaticallySwitchModes: "Automatically Switch to Dark or Light Mode",
      checkInterval: "Check Interval",
      network: "Network",
      useSocks5: "Use SOCKS5 Proxy",
      socks5: "SOCKS5",
      githubMirrorTemplate: "Github Mirror Template",
      launchAtStartup: "Launch at Startup",
      setLockScreenWallpaperSimultaneously:
        "Set Lock Screen Wallpaper Simultaneously",
      sourceCode: "Source Code",
      themesDirectory: "Themes Directory",
      version: "Version",
      language: "Language",
    },

    help: {
      automaticallySwitchModes:
        "If you do not want to automatically switch between light and dark modes, please disable this option.",
      retrieveCoordinatesInterval:
        "The interval for retrieving coordinates, in minutes, should be greater than the detection interval. If your computer is always at a fixed location, you can set it to more than 24 hours. If you frequently travel with your computer, it is recommended to set it to less than 1 hour.",
      socks5:
        "Only enter the SOCKS5 proxy address and port number. If you do not have a valid SOCKS5 proxy server, please use the GitHub mirror template.",
      githubMirror:
        "Github mirror template is used to accelerate downloads. In some countries and regions, due to network restrictions, accessing Github may fail, resulting in download failures. You need to set up a Github mirror template to properly load thumbnails and download themes. Click this button to view available Github mirror templates:",
      launchAtStartup:
        "Autostart will only launch the background process, not the graphical program, and will not consume much memory.",
      manuallySetCoordinates:
        "When manually setting coordinates, you must use the WGS84 coordinate system (the international standard, users in China should take note). Otherwise, coordinate offset issues may occur, leading to inaccurate wallpaper alignment.",
      setLockScreenWallpaperSimultaneously:
        "If you do not want to set the lock screen wallpaper simultaneously, please disable this option.",
      updatedFailed:
        "Unable to complete the hot update. Please click the download button behind this message to manually download the new version: ",
    },

    placeholder: {
      latitude: "Enter latitude",
      longitude: "Enter longitude",
    },

    tooltip: {
      openThemesDirectory: "Click it to open the themes directory.",
      checkForNewVersion: "Click it to check for new version",
    },

    message: {
      changeThemesDirectory: "Change the themes directory to: {{ directory }}?",
      checkIntervalUpdated:
        "Check interval has been updated to: {{ interval }} seconds",
      disableStartupFailed: "Failed to disable startup: \n{{ error }}",
      githubMirrorTemplateUpdated:
        "Github mirror template has been updated to: ",
      invalidNumber: "Please enter a valid number.",
      manualCoordinatesSaved:
        "Coordinates saved, next you can choose the theme you want to apply.",
      numberTooLarge: "Please enter a number less than {{ max }}.",
      numberTooSmall: "Please enter a number greater than {{ min }}.",
      SaveManualCoordinatesFailed: "Failed to save coordinates: \n{{ error }}",
      startupFailed: "Failed to enable startup: \n{{ error }}",
      switchAutoModesFailed:
        "Failed to switch auto light/dark mode: \n{{ error }}",
      switchToManualCoordinatesFailed:
        "Failed to switch to manual coordinates: \n{{ error }}",
      movedThemesDirectory:
        "The themes directory has been moved to: {{ directory }}",
      isLatestVersion: "You are already using the latest version.",
      checkIntervalUpdateFailed:
        "Failed to update check interval: \n{{ error }}",
      saveRetrieveCoordinatesIntervalFailed:
        "Failed to save retrieve coordinates interval: \n{{ error }}",
      socks5UpdateFailed: "Failed to update socks5 settings: \n{{ error }}",
      clearNetworkFailed: "Failed to clear network settings: \n{{ error }}",
    },
  },

  sidebar: {
    tooltip: {
      newVersionAvailable:
        "A new version is available! Click this button to update.",
      settings: "Settings",
    },
  },

  theme: {
    button: {
      apply: "Apply",
      cancel: "Cancel",
      download: "Download",
      stop: "Stop",
    },

    label: {
      selectMonitor: "Select Monitor",
    },

    message: {
      applyThemeFailed: "Failed to apply theme: \n{{ error }}",
      downloadCancelled: "Download cancelled",
      downloadFailed: "Failed to download theme: \n{{ error }}",
      fileSizeWarning:
        "Unable to calculate download progress due to failure in getting file size. Please switch to a Github mirror template that supports forwarding response headers",
    },

    title: {
      downloadFailed: "Download Failed",
    },
  },

  update: {
    button: {
      install: "Install",
    },
    title: {
      downloadingNewVersion: "Downloading New Version {{ version }}",
      newVersionDownloaded: "New Version {{version}} Downloaded",
    },
    message: {
      updateFailed: "Failed to update: \n{{ error }}",
    },
  },
};
