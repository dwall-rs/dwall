# DWALL vs Auto Dark Mode

Both are free, open-source Windows apps that run a per-user background engine, switch
the system light/dark theme and can set per-monitor wallpapers. They differ in what
drives the switch and how much of Windows they touch.

## At a glance

|            | DWALL                                                  | Auto Dark Mode                                                                      |
| ---------- | ------------------------------------------------------ | ----------------------------------------------------------------------------------- |
| Primary job | Wallpaper frames chosen by the sun's position          | Windows dark/light theme on a schedule                                              |
| Switch signal | Solar altitude/azimuth computed locally (refraction, altitude, polar latitudes) or manual coordinates | Fixed times, sun times from Windows location, Night Light state, ambient-light sensor, hotkeys, scripts |
| Wallpaper  | Theme = frames bound to sun positions, matched continuously; per-monitor; optional lock screen | One image per monitor per theme, solid colour or slideshow; no lock-screen feature documented |
| Also changes | Both light/dark theme values — apps and the system/taskbar — and the lock-screen wallpaper; both are switches, so it can be left setting wallpapers only | App/system theme, accent colour, mouse cursors, `.theme` files, taskbar/title bars, grayscale |
| Extensibility | A documented theme format (`theme.toml`, Dwall Maker) with custom theme directories, and a config file the daemon re-reads when it changes; no scripts or condition modules | Custom scripts, conditions (gaming, battery, Night Light), hotkeys                  |
| Stack      | Rust daemon (no async runtime) + Tauri/SolidJS GUI     | .NET 9: WinUI 3 GUI, WinForms-based engine, console CLI shell                        |
| Footprint  | Daemon ≈2 MB private working set (3–8 MB private bytes), 0.63 MB exe; no tray icon | Not documented by the project; tray icon and named-pipe command server               |
| CPU architectures | x64                                              | x64, ARM64                                                                          |
| Languages  | 6                                                      | 31                                                                                   |
| Distribution | Installer + portable daemon archive; no Store listing | GitHub, Microsoft Store (free), winget `ArminOsaj.AutoDarkMode`; Chocolatey/Scoop entries are unofficial |
| Requirements | Windows 10/11 x64; WebView2 for the GUI              | Windows 10 22H2 or newer, Windows 11                                                 |
| CLI        | `dwall.exe --check-config`, `--once`, `--install-autostart`, `--uninstall-autostart`, `--config`, `--help`, `--version` | `AutoDarkModeShell.exe`, commands over the engine's named pipe |
| Licence    | AGPL-3.0-only                                          | GPL-3.0                                                                              |

## Choose DWALL if

- You want the wallpaper itself to follow the sun, not a two-state theme flip.
- The resident cost should be a few megabytes, with no tray icon.
- You deploy headless: a portable daemon, a config file, no GUI and no WebView2.

## Choose Auto Dark Mode if

- Your goal is Windows personalisation: accent colour, cursors, taskbar, `.theme` files.
- You need scheduling beyond the sun — Night Light, ambient light, battery or gaming
  conditions, hotkeys, custom scripts.
- You need ARM64, a Microsoft Store install, or a language other than the six DWALL ships.

## Both, in one line each

Auto Dark Mode covers more of Windows and is scriptable; DWALL deliberately narrows to
sun-driven wallpaper transitions so the always-on process stays small. DWALL switches the
system theme as well, but only from the solar position; if you want that switch on your own
schedule — fixed times, Night Light, an ambient-light sensor — use Auto Dark Mode or
Windows' own schedule.

## Sources

Auto Dark Mode facts above come from that project's own material — repository and
release metadata (github.com/AutoDarkMode/Windows-Auto-Night-Mode), its
`docs/backend-flow.md`, `PRIVACY.md`, wiki pages on known limitations and
troubleshooting, and its Microsoft Store listing. DWALL figures are measured from this
repository's release build; see [technical-comparison.md](./technical-comparison.md)
for how.
