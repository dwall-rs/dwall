# DWALL vs WinDynamicDesktop

Both bring macOS-style dynamic wallpapers to Windows: a location, sunrise and sunset,
and a wallpaper that changes as the day passes. The differences are how the image is
chosen, what else the app changes, and what it costs to keep running.

## At a glance

|            | DWALL                                                       | WinDynamicDesktop                                                                     |
| ---------- | ----------------------------------------------------------- | ------------------------------------------------------------------------------------- |
| Image choice | The frame whose solar altitude/azimuth is closest to the current position (5–24 frames per theme, 15 themes offered) | Day/night images, plus optional sunrise/sunset lists, rotated by sunrise/sunset times or a fixed 24-hour schedule |
| System light/dark theme | Switched, driven by the sun, and switchable off | Not switched — removed in v4 and now a community script                                |
| Location    | Windows Geolocator, or coordinates you enter; works offline with manual coordinates | A place name geocoded through the LocationIQ API, so entering one needs internet      |
| Theme sources | Curated solar-aligned sets (`dwall-rs/dwall-assets`), plus your own directories | ~19 bundled macOS themes, a community site (427 free community themes, some paid), `.ddw` import and a theme creator |
| Theme format | `theme.toml` + images per frame ([Dwall Maker](https://github.com/dwall-rs/maker) spec) | `theme.json` + numbered images in a ZIP/`.ddw`                                        |
| Lock screen | Yes                                                         | Yes                                                                                   |
| Per-monitor themes | Yes                                                  | Yes                                                                                   |
| Automation  | A config file the daemon re-reads when it changes, so a script can rewrite it and the change applies without a restart; a CLI (`--once`, `--check-config`, `--install-autostart`) for scripts and Task Scheduler; no scripting engine inside the app | PowerShell scripts, CLI arguments (`/theme:<N>=<id>`) |
| Architecture | Tiny daemon (always on) + settings window (on demand), config file as the only interface | One WinForms process holding tray, dialogs and scheduler                              |
| Resident cost | Daemon ≈2 MB private working set (3–8 MB private bytes); 0.35 MB archive (0.63 MB binary); no tray icon | Not documented by the project; self-contained ~70 MB executable; tray icon             |
| CPU architectures | x64                                                   | x86, x64, ARM64                                                                       |
| Minimum OS  | Windows 10/11                                               | Windows 7+ (developed for Windows 11)                                                 |
| Languages   | 6                                                           | ~55 offered for translation                                                           |
| Distribution | GitHub releases (installer + daemon archive)               | GitHub releases, Microsoft Store (free), winget, Scoop, Chocolatey                    |
| Licence     | AGPL-3.0-only                                               | MPL-2.0                                                                               |

## Why the frame is chosen by the sun's angle

A theme's frames are drawn for particular daylight: low warm light, mid-morning, noon,
dusk. What makes the sequence look right is where the Sun is when each frame is shown,
and the clock is a poor proxy for that in two ways that both change how a theme reads:

- **The season.** At one place, 09:00 in March and 09:00 in September are different sun
  heights, and the day is a different length; a frame pinned to a clock time shows the
  wrong light for most of the year.
- **The latitude.** Sunrise, noon and sunset fall on the same clock times only near the
  equator; a schedule tuned for one city puts a noon frame under a low sun somewhere else.

DWALL does not consult a clock for this decision at all. `theme.toml` binds every frame to
an altitude and an azimuth, and each cycle applies the frame whose angle is the smallest
distance from the current position — measured on the sky, so a degree of azimuth counts for
`cos(altitude)` degrees of altitude ([custom-themes.md](./custom-themes.md)). The frame and
the light agree in any season at any latitude, with nothing to re-tune.

WinDynamicDesktop works in clock time: its images are distributed between sunrise and
sunset, or over a fixed 24-hour schedule. That is simpler to configure, and it means the sun
behind a given frame drifts with the date and the latitude — reproducing a solar-aligned
look with it means adjusting the schedule as the seasons change.

## Choose DWALL if

- You want the frame chosen by the sun's actual position rather than by a day/night
  schedule, so the light in the picture matches the light outside in any season at any
  latitude, without re-tuning the schedule.
- The always-on process should stay a few megabytes with no notification-area icon.
- You are offline or behind a restrictive network: location can be entered manually and
  theme downloads support mirror templates and SOCKS5.
- You deploy headless from a configuration file.

## Choose WinDynamicDesktop if

- You want the largest ready-made theme catalogue and `.ddw` files from elsewhere.
- You need Windows 7, 32-bit Windows or ARM64.
- You want a Microsoft Store install, or scripting and CLI control of the running app.
- You do not need the system light/dark theme switched for you.

## Both, in one line each

WinDynamicDesktop has the bigger ecosystem, wider platform support and a scripting
language; DWALL matches images to the sun's position frame by frame, switches both
light/dark theme values with the same calculation, and its daemon is 0.63 MB where the
WinDynamicDesktop executable is 62–72 MB — its memory use is not published, so no
memory comparison is claimed here.

## Sources

WinDynamicDesktop facts come from that project's own material — repository and release
metadata, `README.md`, its wiki (frequently asked questions, custom themes, known
issues), `src/JsonConfig.cs`, the installer script and its Microsoft Store listing.
DWALL figures are measured from this repository's release build; see
[technical-comparison.md](./technical-comparison.md) for the method and
[shared-comparison-content.md](./shared-comparison-content.md) for the full matrix.
