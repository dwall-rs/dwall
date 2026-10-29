# Shared reference: DWALL, WinDynamicDesktop, Auto Dark Mode

The single place where verified facts about the three projects live. Comparison
documents link here instead of repeating them.

Every claim about another project comes from that project's own repository,
documentation, release metadata or store listing; DWALL figures are measured from
this repository's release build (`bun run build:daemon`; sizes from the file, memory
from the running process: the private working set Task Manager shows, and the private
bytes committed). Nothing here is estimated or inferred from
a project's language or architecture.

## Function

| | DWALL | WinDynamicDesktop | Auto Dark Mode |
| --- | --- | --- | --- |
| Purpose | Wallpaper frames chosen by the sun's position | macOS Mojave Dynamic Desktop port for Windows | Windows dark/light theme on a schedule |
| What drives a change | Solar altitude and azimuth computed locally | Sunrise/sunset times from the location, or a fixed 24-hour schedule — frames sit at clock offsets, so the sun behind a frame drifts with season and latitude | Fixed times, sun times, Night Light, ambient light, hotkeys, scripts |
| System light/dark theme | Both values switched — apps and the system/taskbar — by solar position; a config switch turns it off | Not switched (removed in v4; a community script does it) | Switched — its core feature |
| Lock screen | Yes, optional | Yes (`lockScreenTheme`) | No lock-screen feature documented |
| Per-monitor wallpapers | Yes — a theme per monitor; a monitor with no entry is left without a wallpaper | Yes (`activeThemes` per display) | Yes (per-monitor images per theme) |
| Random/daily variation | Daily random theme from a pool | Shuffle within a theme | Wallpaper slideshow advance |
| Virtual desktops | One wallpaper set across them: Windows applies a desktop's wallpaper to every virtual desktop | Documented limitation: one image set for all | Documented limitation on single-monitor setups |
| Extensibility | Documented theme format (`theme.toml`, [Dwall Maker](https://github.com/dwall-rs/maker)) and a custom-theme directory; a config file the daemon re-reads when it changes, so it can be driven without a restart; no scripting engine | PowerShell scripts, CLI arguments | Custom scripts, hotkeys, conditions |
| CLI | `dwall.exe --check-config`, `--once`, `--install-autostart`, `--uninstall-autostart`, `--config`, `--help`, `--version` | `WinDynamicDesktop.exe /theme:<N>=<id>`, `/theme:<auto\|dark\|light>` | `AutoDarkModeShell.exe` over the engine's named pipe |

## Architecture and stack

| | DWALL | WinDynamicDesktop | Auto Dark Mode |
| --- | --- | --- | --- |
| Processes | Rust daemon (always on) + Tauri/SolidJS settings window (on demand) | One WinForms process: tray, dialogs and scheduler together | .NET engine with tray and named-pipe server (per session) + WinUI 3 window + console CLI |
| Engine/GUI separation | Yes; no IPC, the config file is the interface | No | Yes, via the named pipe |
| Language / framework | Rust; SolidJS + Tauri 2 | C# / .NET 10, WinForms | C# / .NET 9, WinUI 3 + WinForms |
| Background runtime cost | Single-threaded, no async runtime, no GUI toolkit linked, no network code | .NET runtime + WinForms | .NET runtime + WinForms |
| Autostart | `HKCU\...\Run` value, or a scheduled task you create | Installer-provided | Task Scheduler logon task |
| Configuration | `config.toml` | `settings.json` | Registry + its own settings store |
| Licence | AGPL-3.0-only | MPL-2.0 | GPL-3.0 |
| CPU architectures | x64 | x86, x64, ARM64 | x64, ARM64 |
| Interface languages | 6 (en, zh-CN, zh-HK, zh-TW, ja, ko) | ~55 offered for translation; shipped as compiled `.mo` files | 31 |
| Minimum OS | Windows 10/11 (x64) | Windows 7+ (developed for Windows 11); Store build needs Windows 10 19041+ | Windows 10 22H2 or Windows 11 |

## Footprint (DWALL measured, others as published)

| | DWALL | WinDynamicDesktop | Auto Dark Mode |
| --- | --- | --- | --- |
| Resident private memory | ≈2 MB private working set — the figure Task Manager's Memory column shows — with 3–8 MB private bytes committed depending on whether the WinRT wallpaper and lock-screen paths have run (see [technical-comparison.md](./technical-comparison.md)) | Not documented | Not documented |
| Installer / archive | 2.0 MB NSIS installer (0.2.5); 0.35 MB daemon archive | 62–72 MB per architecture (self-contained) | Not documented |
| Main executable | 0.63 MB daemon, 4.1 MB settings | Single ~70 MB executable | Not documented |
| Notification-area icon | None, by design | Yes (the app's control surface) | Yes |

Neither competitor publishes CPU figures; this document does not invent any.

## Distribution and updates

| | DWALL | WinDynamicDesktop | Auto Dark Mode |
| --- | --- | --- | --- |
| GitHub releases | Installer + daemon archive with `.sha256` | Setup and portable per architecture, with checksums | Installer and portable per architecture, with `.sha256` |
| Microsoft Store | No | Yes, free | Yes, free |
| winget | Manifest prepared, not submitted | `t1m0thyj.WinDynamicDesktop` | `ArminOsaj.AutoDarkMode` |
| Scoop / Chocolatey | Manifest prepared, not submitted | Scoop Extras, Chocolatey | Unofficial entries, labelled as such by the project |
| Update path | In-app updater (atomic replacement of the per-user install) | Weekly check that notifies and opens the release page; Store builds update through the Store | Built-in updater; project notes it does not support system-wide installs |

## Network and privacy

| | DWALL | WinDynamicDesktop | Auto Dark Mode |
| --- | --- | --- | --- |
| Telemetry | None in code (`TELEMETRY_ENABLED = false`) | None documented or present in the source | None; the project states settings stay offline |
| Location | Windows Geolocator, or coordinates you enter | Locally entered place name geocoded through the LocationIQ API; Store build uses the Windows location API | Windows location service for sun times, or fixed times |
| Other requests | Theme and thumbnail downloads from GitHub, with mirror templates and SOCKS5 support; the daemon itself makes none | Downloads bundled macOS themes from GitHub/GitLab/Bitbucket; weekly update check | Update checks against GitHub |

## Themes

| | DWALL | WinDynamicDesktop | Auto Dark Mode |
| --- | --- | --- | --- |
| Format | Directory with `theme.toml` (`schema = 1`), images, optional thumbnails; spec owned by [Dwall Maker](https://github.com/dwall-rs/maker) | ZIP/`.ddw` containing `theme.json` and numbered images; documented | Ordinary image files per theme; `.theme` files are parsed for other settings |
| Frames per theme | 5–24 across the 15-theme catalogue, one per monitored sun position | Day/night plus optional sunrise/sunset lists | One image per theme per monitor |
| Source | Curated sets from `dwall-rs/dwall-assets` | Bundled macOS themes (~19) plus a community site with free and paid themes | Your own images, or `.theme` files |
| User-made themes | Yes (`copied into customized_themes_directory`) | Yes, with a documented format and a theme creator | Not a theme-pack format; wallpapers are chosen directly |

## Use-case scenarios

| Scenario | Best fit | Why |
| --- | --- | --- |
| Wallpaper that tracks the sun through 5–24 frames per theme | DWALL | Frame selection is a solar-angle match |
| Windows dark mode plus accent colour, cursors, taskbar | Auto Dark Mode | DWALL only switches the app/system theme value |
| macOS-style dynamic wallpapers with a large community library | WinDynamicDesktop | Community catalogue (427 free community themes) and `.ddw` import |
| Scripted automation and conditions (gaming, battery, ambient light) | Auto Dark Mode | Script engine and condition modules |
| The smallest possible always-on process, no tray icon | DWALL | ≈2 MB private working set (3–8 MB private bytes), no notification-area icon |
| Headless or managed deployment from a config file | DWALL | Portable daemon, CLI, no WebView2 needed for the daemon |
| ARM64 devices | WinDynamicDesktop, Auto Dark Mode | DWALL ships x64 only |
| Non-English interface beyond six languages | WinDynamicDesktop or Auto Dark Mode | ~55 offered / 31 shipped |
