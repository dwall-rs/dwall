# Technical comparison

What the three projects are made of, and what follows from that. Feature-level facts
live in [shared-comparison-content.md](./shared-comparison-content.md); this document
covers structure and mechanics. No comparative benchmarks are published here, because
none of the three projects publishes any and this repository does not invent them.

## Process model

**DWALL** — two binaries that never talk directly:

```
dwall.exe        daemon: reads config.toml, computes the solar position, sets wallpaper,
                 light/dark theme and lock screen. Single-threaded, no async runtime, no
                 GUI toolkit linked, no network code. Holds a per-session named mutex.
dwall-settings   Tauri 2 + SolidJS window, started when opened: theme catalogue and
                 downloads, updater, thumbnails.
        └──────── config.toml (the only interface; the window polls state and reads the log tail) ────────┘
```

The split exists to keep the resident process small: the window's toolkit, webview and
HTTP stack are only in memory while it is open. The daemon holds the mutex
`Local\com.thep0y.dwall.daemon`, which is also how the window detects and stops it.

**WinDynamicDesktop** — one WinForms process that is simultaneously the tray icon, the
dialogs and the scheduler; settings live in `settings.json`.

**Auto Dark Mode** — three binaries: a per-session .NET engine with tray icon and
named-pipe server, a WinUI 3 window opened on demand, and a console shell that sends
commands over the pipe; autostart is a Task Scheduler logon task.

## Dependencies that shape behaviour

| | DWALL daemon | DWALL settings | WinDynamicDesktop | Auto Dark Mode |
| --- | --- | --- | --- | --- |
| Runtime | none beyond the OS (Rust, static) | WebView2 | .NET 10, self-contained (~70 MB) | .NET 9 |
| Windows APIs | wallpaper COM, registry, WinRT geolocation, Win32 console | — | Win32/WinForms, LocationIQ over HTTPS | Win32/WinUI, Windows location service |
| Network | none | theme/thumbnail downloads, updater | theme downloads, geocoding, update check | update checks |

No async runtime is linked into the DWALL daemon on purpose: WinRT operations are
awaited with a bounded 50 ms poll helper instead of an executor, so a stuck platform
call cannot hang the process forever.

## File formats

| | DWALL | WinDynamicDesktop | Auto Dark Mode |
| --- | --- | --- | --- |
| Configuration | `config.toml` (`%APPDATA%\dwall\config.toml`), commented default written on first run | `settings.json` | Registry and its own store |
| Theme | directory with `theme.toml` (`schema = 1`), `images/<n>.<ext>`, optional `thumbnails/`; authoritative spec in the [Dwall Maker](https://github.com/dwall-rs/maker) repository | ZIP/`.ddw` with `theme.json` and numbered images | image files per theme; `.theme` files parsed for other personalisation |
| Random state | `random_selection.json`, next to the chosen config file | shuffle setting inside `settings.json` | — |
| Logs | `%LOCALAPPDATA%\com.thep0y.dwall\log\dwall.log`, rotated to `backup\` when it passes 5 MB or on start, five files kept | debug logging toggle | own log file |

Because the config file is the interface, it is also the public contract: `--config`
lets several independent configurations coexist, and `--check-config` validates one
without applying anything.

## Distribution and updates

| | DWALL | WinDynamicDesktop | Auto Dark Mode |
| --- | --- | --- | --- |
| Installer | NSIS, per user, 2.0 MB (0.2.5), no administrator rights | Inno Setup, per user or machine, 62–72 MB | Installer, per user |
| Portable / headless | `Dwall_<version>_x64-daemon.zip` (0.35 MB) usable with no GUI at all | `*_Portable.exe` (the same tray application) | portable ZIP (the same engine) |
| Update | In-app updater replaces the per-user install atomically; `latest.json` points at the NSIS package | Weekly check that notifies and opens the release page; Store builds update through the Store | In-app updater, which the project notes does not cover system-wide installs |
| Store | not listed | free | free |

The daemon archive is a first-class artifact, not a stripped-down copy: it is the same
binary the installer bundles, published with a checksum so a headless machine can be
provisioned without installing anything.

## How DWALL's figures were obtained

- **Binary and archive sizes** — file sizes of a release build
  (`bun run build:daemon`, `cargo build -p dwall --release --features build-script,log-max-level-info`)
  and of the generated archive.
- **Resident memory** — three numbers, measured on release builds with
  `.scripts/measure-daemon-memory.ps1`, which now prints all three so a claim can say
  which one it means:
  - **Private working set** — what Task Manager's Memory column shows, and therefore
    the one to quote when a reader is going to check. A daemon of 0.3.0-alpha.10 that
    had been running for 2.8 days with the lock screen enabled measured 2.12 MB
    (419 handles, 5 threads).
  - **Private bytes** — the memory committed on the process's behalf. Same process:
    7.8 MB. It starts at ≈3 MB with ~270 handles before the first wallpaper change and
    settles at ≈7.5 MB with ~430 handles once the WinRT wallpaper and lock-screen paths
    have run. The difference is one-off, not per change: a daemon that had been running
    for thirteen hours and changed the wallpaper ten times measured the same level as a
    five-minute run that changed it twice. Quote private bytes for what the process
    reserves and the private working set for what it actually occupies.
  - **Working set** — 23 MB, and not a figure of merit: it counts shared system DLLs
    the daemon does not own.
  A single sample is misleading in both directions: the process holds a dozen threads
  while the COM/WinRT machinery starts and 4–7 afterwards.
- **CPU** — not quoted: the daemon computes once per interval (default 15 s) and sleeps
  in between, so any number would depend entirely on the sampling method.

For the other two projects every figure comes from their own published material, and
"not documented" means exactly that.
