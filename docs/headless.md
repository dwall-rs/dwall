# Running the daemon on its own (headless)

`dwall.exe` does not need the settings window: it reads `config.toml`, applies the
wallpaper matching the current solar position, and can start at sign-in. Use it on
machines where the GUI is unnecessary — kiosks, LTSC/Server images, VM templates —
or when configuration is managed by hand, dotfiles or MDM/GPO. It needs no WebView2
runtime and no administrator rights: all state is per-user.

The daemon reads theme directories and nothing else. Downloading themes, mirrors and
thumbnails live in the settings app, so a headless install deploys themes itself.

## Getting the binary

Download `Dwall_<version>_x64-daemon.zip` (or `Dwall_nightly_x64-daemon.zip` for
nightly) from the [releases page](https://github.com/dwall-rs/dwall/releases) and
unpack it anywhere. The archive contains `dwall.exe`, the licence, this document,
the theme format and `SHA256SUM.txt`; verify a download against the `.sha256` asset
published next to it.

Alternatives: copy `dwall.exe` from an existing installation (`%ProgramFiles%\Dwall
Settings\dwall.exe`), or build it:

```sh
bun run build:daemon   # cargo build -p dwall --release --features build-script,log-max-level-info
```

## Configuration

| | |
| --- | --- |
| Default path | `%APPDATA%\dwall\config.toml` |
| Override | `--config <PATH>` (also `--config=<PATH>`) |
| Validate | `--check-config` |

The first run writes a **commented default file** there, so there is always
something to edit; every key is optional and falls back to its default. It covers
wallpaper mode (`fixed` per monitor, or `random` with an optional pool drawn daily),
the theme directories, the update interval in seconds (default 15, range 1–3600),
image format, system light/dark switching (on by default), lock-screen wallpaper and
the position source (automatic via Windows Geolocator, or manual coordinates).

`--config` also moves `random_selection.json` next to the chosen file, so
independent configurations coexist. The settings window always uses the default
path, so it does not see a daemon started with `--config` elsewhere.

## Commands

| Command | Meaning |
| --- | --- |
| *(none)* | Run the daemon in the foreground. |
| `--config <PATH>` | Use a configuration file other than the default. |
| `--check-config` | Validate the configuration, print a summary, apply nothing. |
| `--once` | Apply one update cycle and exit. |
| `--install-autostart` | Register this executable to start at sign-in. |
| `--uninstall-autostart` | Remove that registration. |
| `-h`/`--help`, `-V`/`--version` | Usage / version. |

| Exit code | Meaning |
| --- | --- |
| 0 | Success, including "a daemon is already running". |
| 1 | Fatal error while running. |
| 2 | Unusable command line. |
| 3 | Invalid or unreadable configuration. |
| 4 | Auto-start registration failed. |
| 5 | `--once` refused: a daemon already runs. |

### Invoking it from a shell

`dwall.exe` is a **GUI-subsystem** executable, so auto-start never flashes a console
window — and `cmd`/PowerShell therefore return to the prompt without waiting, which
means a bare `dwall.exe --check-config` shows its output but gives the shell no exit
code. In scripts, use a waiting invocation:

```bat
cmd /c dwall.exe --check-config || echo configuration is broken
```

```powershell
$check = Start-Process dwall.exe -ArgumentList --check-config -Wait -PassThru
if ($check.ExitCode -ne 0) { Write-Error "configuration is broken" }
```

Task Scheduler waits by itself and records the exit code as the task's "Last Run
Result", so a scheduled `--once` needs no wrapper.

## Auto-start

```console
> dwall.exe --install-autostart
auto-start registered: "C:\tools\dwall\dwall.exe"
```

This writes one value (name `Dwall`, quoted command line) under
`HKCU\Software\Microsoft\Windows\CurrentVersion\Run`: per-user, no administrator
rights, and the same entry the settings window's "start at login" switch manages.
There is only one, so installing from a second copy replaces the first and says so:

```console
> dwall.exe --install-autostart
replacing the existing auto-start entry: "C:\Program Files\Dwall Settings\dwall.exe"
auto-start registered: "C:\tools\dwall\dwall.exe"
```

Check what is registered with
`reg query "HKCU\Software\Microsoft\Windows\CurrentVersion\Run" /v Dwall`. Entries
written before the command line was quoted are still recognised, so an older entry
does not look "off"; installing once rewrites it in the quoted form.

Starting this way is silent: release builds are GUI-subsystem executables, and the
settings window launches the daemon with `CREATE_NO_WINDOW` as well. A **debug** build
(`cargo run -p dwall`, `bun run dev`) is a console application, which would open a
console window at every logon, so `--install-autostart` warns when asked to register
one and the release archives are checked to be GUI-subsystem before packaging.

For a scheduled task instead (a delay, a specific user, restart on failure), point
the action at `dwall.exe` directly rather than using both mechanisms.

## Themes

A theme is a directory holding a `theme.toml` manifest (`schema = 1`) plus its
images and optional thumbnails; the format is documented in
[custom-themes.md](./custom-themes.md). Put themes in `themes_directory`, or your
own in `customized_themes_directory`, and point either key elsewhere if you prefer.
The daemon picks new directories up on its next cycle.

## Logs

| | |
| --- | --- |
| File | `%LOCALAPPDATA%\com.thep0y.dwall\log\dwall.log` |
| Rotation | Each start moves the previous file to `log\backup\dwall_<unix-seconds>.log`. |
| Verbosity | `DWALL_LOG=debug` or `DWALL_LOG=dwall=debug` in the daemon's environment. |

Release builds compile `debug!`/`trace!` out, so there `DWALL_LOG` can only raise the
level up to `info`; use a debug build for verbose statements, or the nightly daemon,
which keeps them.

## Living next to the settings app

- Both binaries share one configuration file; there is no IPC. The window polls the
  daemon's state, and a config edit is picked up through the file's modification time.
- **One daemon per session**: it holds the mutex `Local\com.thep0y.dwall.daemon` for
  its lifetime, so a second copy — the installer's, a portable one, a duplicate
  auto-start entry — exits with `dwall is already running; this instance exits`
  instead of fighting over the desktop. Only builds that have the lock honour it: a
  copy old enough to predate it holds none, so update the installed version before
  relying on the guard.
- The settings window detects the daemon by that mutex and stops it by executable
  name, so it can see and stop a daemon it did not start.

## Updating and uninstalling

Updating means replacing `dwall.exe` and restarting the daemon; the in-app updater
belongs to the settings app. The `Run` entry stores the path, so keeping the
location is enough.

```console
> dwall.exe --uninstall-autostart
```

Then delete the folder, and — only if no settings app uses them — the per-user data
directories `%APPDATA%\dwall` (configuration, downloaded and customised themes,
`random_selection.json`) and `%LOCALAPPDATA%\com.thep0y.dwall` (`log\`, thumbnail
cache). Themes can be large, so check before deleting.

## Package managers

winget and Scoop manifests are templates under `.packaging/`, rendered from a
release's checksum, so a submission is a copy rather than four hand-edited files:

```console
> ./.scripts/package-daemon.ps1 -Version 0.2.5
> ./.scripts/render-package-manifests.ps1 -Version 0.2.5 -Validate
```

Copy the result from `dist-manifests/winget/` into
`manifests/t/thep0y/Dwall/<version>/` in
[microsoft/winget-pkgs](https://github.com/microsoft/winget-pkgs), or
`dist-manifests/scoop/dwall.json` into a Scoop bucket. Neither package is published
yet, so `winget install thep0y.Dwall` only works once the manifest is merged — the
release archive works today and is what it installs (as a portable package exposing
`dwall` on `PATH`).

## Troubleshooting

| Symptom | Check |
| --- | --- |
| `--check-config` exits 3 | The parse error names the offending line; paths are validated too. |
| Wallpaper never changes | The log's first lines: a missing theme directory, an empty monitor map and random mode with no themes all warn there. |
| `--once` exits 5 | A daemon already runs; stop it from the settings window or Task Manager. |
| Nothing on the console | Read the log file — and remember the shell caveat above. |
| Wallpaper stops after an edit | A broken config is not fatal: the daemon keeps the previous configuration and retries every 5 s, logging each attempt. |
