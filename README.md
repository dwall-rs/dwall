<p align="center">
<img height="64" width="64" src="./crates/dwall-settings/icons/128x128.png" />
<br/>
<a href="https://github.com/dwall-rs/dwall/releases/latest"><img src="https://img.shields.io/github/downloads/dwall-rs/dwall/total.svg?style=flat-square" alt="GitHub releases"/></a>
<img src="https://img.shields.io/badge/Windows%2010-%230079d5.svg?style=for-the-badge&logo=Windows%2010&logoColor=white)" alt="Windows 10" height="20" />
<img src="https://img.shields.io/badge/Windows%2011-%230079d5.svg?style=for-the-badge&logo=Windows%2011&logoColor=white)" alt="Windows 11" height="20" />
<br/>
English | <a href="./README.zh-CN.md">简体中文</a>
</p>

# DWALL

A Windows dynamic-wallpaper app that follows the **sun's position**, not a clock: it
computes solar altitude and azimuth locally and applies the theme frame that matches,
moving the system light/dark theme and the lock screen with it (the theme switch is on by
default and can be turned off). The always-on part is a single-threaded Rust daemon that
holds 3–8 MB of private memory — the executable itself is 0.63 MB — and the settings
window is a separate process that only runs while it is open.

## How it compares

| | DWALL | [WinDynamicDesktop](https://github.com/t1m0thyj/WinDynamicDesktop) | [Auto Dark Mode](https://github.com/AutoDarkMode/Windows-Auto-Night-Mode) |
| --- | --- | --- | --- |
| Frame chosen by | Solar altitude/azimuth match | Day/night and sunrise/sunset times | — (theme switching) |
| System light/dark theme | Switched, by solar position | Not switched | Switched — its core feature |
| Lock screen | Yes | Yes | Not documented |
| Per-monitor themes | Yes | Yes | Yes |
| Scripting / automation | None | PowerShell scripts, CLI | Scripts, conditions, hotkeys |
| Always-on cost | 3–8 MB daemon, no tray icon | Not documented; ~70 MB app with tray icon | Not documented; tray icon |
| Architectures / languages | x64 / 6 | x86, x64, ARM64 / ~55 offered | x64, ARM64 / 31 |
| Distribution | GitHub releases | GitHub, Microsoft Store, winget, Scoop, Chocolatey | GitHub, Microsoft Store, winget |
| Licence | AGPL-3.0-only | MPL-2.0 | GPL-3.0 |

Each fact above is sourced from the other project's own material; the full matrix and
the cases where they are the better choice are in [docs/](./docs/README.md).

## Features

- **Solar scheduling** — frames are matched to the sun's position; the 15-theme catalogue uses 5 to 24 frames per theme.
- **System theme with it** — light/dark is switched on the same calculation (on by default; the switch is in the settings page).
- **Lock screen** — optionally mirrors the wallpaper's theme and frame.
- **Per-monitor themes** — a different theme per display, reapplied when monitors change.
- **Daily random theme** — optionally draw one theme per day from a pool you choose.
- **Adjustable cadence** — evaluate every 1 s to 1 h (default 15 s).
- **Your own themes** — a directory with `theme.toml` and images; see [docs/custom-themes.md](./docs/custom-themes.md).
- **Headless mode** — the same binary runs from a config file, with a CLI; see [docs/headless.md](./docs/headless.md).
- **Restricted networks** — mirror templates and SOCKS5 for theme downloads.
- **Six languages, no telemetry** — English, Simplified/Traditional/Hong Kong Chinese, Japanese, Korean.

## Requirements

64-bit Windows 10 or 11. The settings window needs the WebView2 runtime (present on
current Windows; the installer covers it); the daemon needs neither WebView2 nor
administrator rights. Heavily trimmed Windows images are not supported.

## Install

1. Download the installer (`Dwall.Settings_<version>_x64-setup.exe`) from the
   [latest release](https://github.com/dwall-rs/dwall/releases/latest) and run it — it
   installs per user, so no administrator rights are required.
2. Daemon only, or a managed/headless machine: take
   `Dwall_<version>_x64-daemon.zip` instead and follow [docs/headless.md](./docs/headless.md).

winget and Scoop manifests are prepared but not submitted to those repositories yet.

## Usage

1. Run DWALL and allow location access, or set coordinates manually in the settings page.
2. Pick a theme in the sidebar. Themes that are not downloaded yet show a **Download**
   button and cannot be picked until it finishes; once downloaded, **Apply** uses it.
3. With several monitors, select the display in the monitor selector and repeat step 2
   for each one.
4. Optionally enable the lock-screen wallpaper.
5. DWALL keeps running in the background — there is no tray icon; the settings window is
   started again from the Start menu.

## Screenshots

![home](images/home.avif)

![settings](images/settings.avif)

## Logs

Level is `info`; set `DWALL_LOG` (a plain level such as `debug`, or `dwall=debug`) to
change it. Release builds are compiled without the `debug`/`trace` statements, so only
`info`, `warn` and `error` can appear there. The daemon writes to
`%LOCALAPPDATA%\com.thep0y.dwall\log\dwall.log` and rotates the previous file into
`backup\` on each start; the settings window opens the folder for you.

## Frequently asked questions

**Why are the settings window and the daemon separate processes?**
They started as one process with a tray icon, and the aim is a background cost of a few
megabytes, so the GUI toolkit, webview and HTTP client were moved into a window that
exists only while it is open. The trade-off is no IPC: the two share `config.toml`, and
the window polls the daemon's state instead of being notified. The same reasoning is why
there is no tray icon — a tray process costs more than the daemon it would control.

**Theme downloads fail.** Wallpaper sets are hosted on GitHub, which some networks
cannot reach. Set a mirror template in the settings page, for example
`https://ghproxy.cc/https://github.com/<owner>/<repo>/releases/download/<version>/<asset>`
for a prefixing proxy, or `https://kkgithub.com/<owner>/<repo>/...` for a mirror that
replaces the host. Thumbnails come from the same host and need the same setting.

**The window used the network when I opened it.** It checks for a new version once per
start — through the mirror and proxy configured in the settings — and tells you if there
is one. A check that cannot reach GitHub stays silent rather than showing a failure you
did not ask for, and nothing is downloaded or installed until you press the button in the
About section.

**Can I use my own wallpapers?** Yes — a custom theme is a directory containing a
`theme.toml` manifest (schema 1) plus its images. See
[docs/custom-themes.md](./docs/custom-themes.md). The curated sets remain the easiest
option, since their frames are already aligned with the sun's path.

**Can I contribute a wallpaper set?** Open an issue describing the concept (and where
the images can be fetched); suitable sets can be added to the curated catalogue.

## Contributing and licence

Issues and pull requests are welcome. DWALL is licensed under
[AGPL-3.0-only](./LICENSE); the source is in `crates/` (Rust) and `src/` (frontend),
with [AGENTS.md](./AGENTS.md) describing the layout and conventions.
