# Running the daemon on its own (headless)

`dwall.exe` does not need the settings window. It reads a TOML file, applies the
wallpaper that matches the current solar position, and can register itself to start
at sign-in. That makes it usable on a machine where the GUI adds nothing — a kiosk,
a locked-down or LTSC/Server image, a VM template — and for setups whose
configuration is managed by hand, by a dotfiles repository or by MDM/GPO.

Unlike the settings app, the daemon has no WebView2 runtime requirement: it links
only the Windows APIs it uses.

## Getting the binary

Download `Dwall_<version>_x64-daemon.zip` from the
[releases page](https://github.com/dwall-rs/dwall/releases) and unpack it wherever
you like: the archive holds the daemon, the licence, this document, the theme
format and a checksum list, and all of the daemon's state is per-user rather than
per-directory. A nightly archive (`Dwall_nightly_x64-daemon.zip`) is published with
each nightly build and keeps that build's debug logging.

Each archive is published next to a `.sha256` file, so a download can be verified
before it is unpacked:

```powershell
Get-FileHash .\Dwall_0.2.5_x64-daemon.zip -Algorithm SHA256   # compare with the .sha256 asset
```

Other ways to get the same binary:

- copy `dwall.exe` out of an existing installation (usually
  `C:\Program Files\Dwall Settings\dwall.exe` — the installer ships it as a
  resource of the settings app);
- build the shipped configuration:

```sh
bun run build:daemon      # cargo build -p dwall --release --features build-script,log-max-level-info
```

The result is `target/release/dwall.exe` (about 600 KB).

## Configuration

| | |
| --- | --- |
| Default path | `%APPDATA%\dwall\config.toml` |
| Override | `dwall.exe --config <PATH>` (also `--config=<PATH>`) |
| Validate | `dwall.exe --check-config` |

On the first run the daemon **writes a commented default file** to that path, so
there is always something to edit. Every key is optional; a missing key falls back
to the documented default. The file includes the paths for:

- wallpapers and wallpaper mode (`fixed` per monitor, or `random` with an optional
  pool and a daily draw),
- `themes_directory` (default `%APPDATA%\dwall\themes`) and
  `customized_themes_directory` (default `%APPDATA%\dwall\customize`),
- the update interval, image format, colour-scheme detection and lock-screen
  wallpaper switches,
- the position source: automatic (Windows Geolocator) or manual coordinates.

Validate an edit before restarting the daemon:

```console
> dwall.exe --check-config
configuration OK: C:\Users\you\AppData\Roaming\dwall\config.toml
  interval: 600s
  image format: Jpeg
  auto-detect colour scheme: true
  themes directory: C:\Users\you\AppData\Roaming\dwall\themes (ok)
  customized themes directory: C:\Users\you\AppData\Roaming\dwall\customize (ok)
```

`--config` also moves the random-mode state file (`random_selection.json`) next to
the chosen configuration file, so several independent configurations can coexist.
Note that the settings window always uses the default path: a daemon started with
`--config` somewhere else is invisible to it.

## Commands

```console
> dwall.exe --help
```

| Command | Meaning |
| --- | --- |
| *(none)* | Run the daemon in the foreground. |
| `--config <PATH>` | Use a configuration file other than the default. |
| `--check-config` | Validate the configuration and print a summary. Applies nothing. |
| `--once` | Apply exactly one update cycle and exit. |
| `--install-autostart` | Register this executable to start at sign-in. |
| `--uninstall-autostart` | Remove that registration. |
| `-h`, `--help` / `-V`, `--version` | Usage / version. |

| Exit code | Meaning |
| --- | --- |
| 0 | Success. Also returned when a daemon is already running. |
| 1 | The daemon stopped because of a fatal error. |
| 2 | The command line was not understood. |
| 3 | The configuration is invalid or unreadable. |
| 4 | Auto-start registration failed. |
| 5 | `--once` refused because a daemon is already running. |

`--once` is meant for scripts, scheduled tasks and smoke tests; because two daemons
would fight over the same desktop, it refuses to run while another instance holds
the session's daemon lock.

### Invoking it from a shell

`dwall.exe` is built as a **GUI-subsystem** executable, so that auto-starting it at
sign-in never flashes a console window. `cmd` and PowerShell therefore return to
the prompt as soon as the process starts: a bare `dwall.exe --check-config` shows
its output (the daemon attaches to the parent console) but does **not** give the
shell an exit code to test. In scripts, use an invocation that waits:

```bat
:: cmd.exe / batch file
cmd /c dwall.exe --check-config || echo configuration is broken
```

```powershell
# PowerShell
$check = Start-Process dwall.exe -ArgumentList --check-config -Wait -PassThru
if ($check.ExitCode -ne 0) { Write-Error "configuration is broken" }
```

Task Scheduler waits for the process by itself, and records the exit code as the
task's "Last Run Result", so a scheduled `--once` needs no wrapper.

## Auto-start

```console
> dwall.exe --install-autostart
auto-start registered: "C:\tools\dwall\dwall.exe"

> dwall.exe --uninstall-autostart
auto-start entry removed
```

This adds one value to
`HKCU\Software\Microsoft\Windows\CurrentVersion\Run` (value name `Dwall`,
quoted command line). It is per-user, needs no administrator rights, and is the
same entry the settings window's "start at login" switch manages — there is only
ever one, so installing from a second copy replaces the first and says so:

```console
> dwall.exe --install-autostart
replacing the existing auto-start entry: "C:\Program Files\Dwall Settings\dwall.exe"
auto-start registered: "C:\tools\dwall\dwall.exe"
```

To see what is registered right now:

```console
> reg query "HKCU\Software\Microsoft\Windows\CurrentVersion\Run" /v Dwall
```

Values written before the command line was quoted are accepted when checking
whether auto-start is enabled, so an older entry does not look "off". Install once
from the copy you want to keep and it is rewritten in the quoted form.

If you prefer a scheduled task (for example to add a delay, run only when a
particular user is logged on, or restart it on failure), point the action at
`dwall.exe` directly and do not also use the `Run` key.

## Themes

A theme is a directory containing a `theme.toml` manifest (schema 1) plus its
images and thumbnails; the format is documented in
[custom-themes.md](./custom-themes.md). Put themes in `themes_directory` (wallpaper
packs) or `customized_themes_directory` (your own), or point those keys somewhere
else.

Downloading themes, resolving GitHub mirrors and generating thumbnails live in the
settings app — the daemon only reads theme directories. Headless installations
therefore fetch a theme pack themselves (for example a release asset) and unpack it
into `themes_directory`. Nothing else is required; the daemon picks the directory up
on its next cycle.

## Logs

| | |
| --- | --- |
| File | `%LOCALAPPDATA%\com.thep0y.dwall\log\dwall.log` |
| Rotation | The previous file is moved to `log\backup\dwall_<unix-seconds>.log` at every start. |
| Verbosity | `DWALL_LOG=debug` (or `DWALL_LOG=dwall=debug`) in the daemon's environment. |

Release builds compile `debug!`/`trace!` out, so `DWALL_LOG` can only raise the
level up to `info` there. Run a debug build if you need the verbose statements.

Because the daemon is a GUI-subsystem executable, it has no console of its own: CLI
commands attach to the console you typed them in, and everything else lands in the
log file.

## Living next to the settings app

- The daemon and the settings window share one configuration file, so both show the
  same theme and monitor configuration. There is no IPC: the window polls the
  daemon's state and an edit is picked up through the file's modification time.
- **One daemon per session.** The daemon claims a named mutex
  (`Local\com.thep0y.dwall.daemon`) for its whole lifetime, so starting a second
  copy — the installer's, a portable one, or a second auto-start entry — exits
  immediately with `dwall is already running; this instance exits` rather than
  fighting over the desktop. Only builds that have the lock honour it: a copy old
  enough to predate it holds none, so update the installed version before relying
  on the guard.
- The settings window detects the daemon by that mutex and stops it by executable
  name, so it can see and stop a daemon it did not start.
- Installed side by side, the two copies are still *one* product: whichever runs
  first wins, and the settings window keeps managing it.

## Updating and uninstalling

Updating a headless copy means replacing `dwall.exe` (and restarting the daemon);
the updater lives in the settings app. The `Run` entry stores the path, so as long
as the location does not change there is nothing else to do.

To remove a headless installation:

```console
> dwall.exe --uninstall-autostart
```

then delete the folder, and — if no settings app is using them — the per-user data
directories:

- `%APPDATA%\dwall` — `config.toml`, downloaded themes, customised themes,
  `random_selection.json`
- `%LOCALAPPDATA%\com.thep0y.dwall` — `log\` (and `backup\`) and the thumbnail
  cache

## Troubleshooting

| Symptom | Check |
| --- | --- |
| `--check-config` exits 3 | The TOML is invalid or a path failed validation; the parse error names the line. |
| The daemon starts but the wallpaper never changes | The log's first lines: a missing theme directory, an empty `monitor_specific_wallpapers`, or random mode with an empty pool all warn there. |
| `--once` exits 5 | A daemon is already running; stop it first (the settings window's "stop engine", or Task Manager). |
| Nothing appears on the console | The release binary attaches to the parent console when there is one; otherwise read the log file. When invoking from `cmd`/PowerShell, remember that they do not wait for a GUI-subsystem process — see [Invoking it from a shell](#invoking-it-from-a-shell). |
| Wallpapers stop after an edit | A broken config is not fatal: the daemon keeps the previous configuration and retries every 5 s, logging each attempt. |
