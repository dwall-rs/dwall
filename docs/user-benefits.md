# Which tool fits

Feature facts live in [shared-comparison-content.md](./shared-comparison-content.md);
head-to-head reasoning is in [DWALL vs Auto Dark Mode](./dwall-vs-auto-dark-mode.md)
and [DWALL vs WinDynamicDesktop](./dwall-vs-windynamicdesktop.md). This page is the
plain cost/benefit view of DWALL: what it gives you and what it asks in return.

## What DWALL gives you

- **Wallpapers that follow the sun, not a clock.** The frame whose solar altitude and
  azimuth best match the current position is applied, so transitions land where the
  theme author put them regardless of season or latitude.
- **A resident cost you can forget.** One single-threaded daemon, ≈4 MB private memory,
  no notification-area icon, no webview and no HTTP stack while the settings window is
  closed.
- **The system light/dark theme switched with the same calculation**, so taskbar and app
  theme move with the wallpaper instead of at a fixed hour.
- **Per-monitor themes**, a lock screen that mirrors the wallpaper's theme and frame, and
  an optional daily random theme from a pool you choose.
- **Predictable deployment.** One `config.toml`, an NSIS installer that needs no
  administrator rights, a 0.35 MB daemon archive (0.6 MB binary) for headless machines,
  and a CLI (`--check-config`, `--once`, `--install-autostart`) for scripts and
  scheduled tasks.
- **No telemetry and no account.** The daemon makes no network requests at all; theme and
  thumbnail downloads happen in the settings window, and honour a mirror template or a
  SOCKS5 proxy for networks that cannot reach GitHub.

## What it asks in return

- **x64 Windows 10/11 only.** No ARM64, no 32-bit Windows, no Windows 7.
- **Six interface languages** (English, Simplified and Traditional Chinese, Hong Kong
  Chinese, Japanese, Korean), against ~55 and 31 for the alternatives.
- **No scripting and no automation hooks.** If you need "switch when a game starts",
  "obey the ambient-light sensor" or "run this script afterwards", that is Auto Dark
  Mode's territory.
- **It changes the system theme, not the rest of personalisation.** Accent colour,
  mouse cursors and `.theme` files are not touched.
- **A smaller theme catalogue.** Fifteen solar-aligned sets (5 to 24 frames each) are
  offered as downloads, and you can add your own with
  [Dwall Maker](https://github.com/dwall-rs/maker); community libraries with hundreds of
  ready themes exist for WinDynamicDesktop, not here.
- **No Microsoft Store listing.** Updates come from the in-app updater, from winget or
  Scoop once those manifests are merged, or from replacing the daemon archive yourself.
- **One daemon per session.** Starting a second copy is refused rather than run
  alongside, which is deliberate but does mean a portable copy cannot shadow an
  installed one.

## If your priority is…

| Priority | Better fit | Why |
| --- | --- | --- |
| Sun-accurate wallpaper transitions at the lowest possible background cost | DWALL | Frame selection is a solar-angle match; ≈4 MB daemon, no tray. |
| Getting the most out of Windows dark mode (accent colour, cursors, taskbar, conditions, scripts) | [Auto Dark Mode](./dwall-vs-auto-dark-mode.md) | Those areas are its feature set, not a side effect. |
| A large ready-made theme catalogue and `.ddw` files, on any Windows or architecture | [WinDynamicDesktop](./dwall-vs-windynamicdesktop.md) | Community library, ARM64/x86 support, Store install, scripting. |
| Headless or managed deployment from a config file | DWALL | Portable daemon archive, CLI, no WebView2 needed for the daemon. |
| Touch nothing but the wallpaper | [WinDynamicDesktop](./dwall-vs-windynamicdesktop.md) | It no longer switches the system theme at all. |

Running DWALL alongside either alternative is possible — they install side by side and
touch different settings — but both would be setting wallpapers, so pick one for that.
