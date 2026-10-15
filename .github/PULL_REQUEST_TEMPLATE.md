## What this is

A hardening pass over the whole repository, driven by a five-way code review (daemon core, Tauri settings backend, SolidJS frontend, `logging`/`time`/Windows platform layer, build-dependency-size-CI). Every change is traceable to an item in **[`docs/code-review-2026-10-15.md`](docs/code-review-2026-10-15.md)**, which carries a `path:line`, the evidence, the impact, the fix and an acceptance criterion per item, plus a per-batch progress log and an appendix of findings that were checked and deliberately left alone.

**80 files, +3 282 / −1 244 across 18 commits** (this file's own wording update included; GitHub's diff stat is authoritative if a later commit drifts it). No public API change in `dwall`; `src/ipc/**` remains the only importer of `@tauri-apps/api`.

| Area | Files | +/− |
| --- | --- | --- |
| `crates/dwall-settings` | 29 | +1 126 / −360 |
| `crates/dwall` (daemon) | 15 | +640 / −117 |
| `src` (frontend) | 19 | +214 / −73 |
| `crates/logging` | 2 | +141 / −31 |
| `crates/time` | 3 | +89 / −14 |
| root / manifests | 8 | +394 / −647 |
| `.github` (CI, this template) | 3 | +178 / −2 |
| `docs` | 1 | +500 / −0 |

New: `.gitattributes`, `docs/code-review-2026-10-15.md`, `.github/PULL_REQUEST_TEMPLATE.md`, `crates/dwall/src/platform/windows/async_operation.rs`, `src/utils/call-event-handler{,test}.ts`. Removed: `src/utils/fonts.ts`.

## The failures this fixes

Each of these was reproduced or read out of the source during the review; the bold part is the user-visible symptom.

### Daemon (`crates/dwall`)

- **The wallpaper quietly stops changing.** Four independent causes, all fixed: `IDesktopWallpaper::GetWallpaper` fills an `[out] LPWSTR` that the caller must release with `CoTaskMemFree`, but `windows-rs` hands back a bare `PWSTR` with no `Drop`/`Free` — the daemon leaked one block per monitor per cycle (its only unbounded allocation); an unreadable/invalid config propagated out of `run()`, so the process **exited with no trace** (release has no console and the message only went to stderr); per-monitor apply failures were swallowed by `is_ok()` with no log and no counter; and every WinRT call waited with `windows-future`'s `INFINITE` timeout (the lock-screen file/uri calls and the geolocation reads), so a wedged shell, an unreachable network share or an unresponsive location service hung the single-threaded daemon forever. All of them now wait through one bounded helper (`platform/windows/async_operation.rs`): 30 s for the lock screen and the geoposition read, 10 s for the access-status request, which has no platform-side budget at all. The daemon calls neither `get()`/`join()` nor `IntoFuture`: an executor is exactly what a synchronous single-threaded daemon does not have, and awaiting would park the same thread on a waker for the same latency.
- **A restarted daemon re-rolled the day's random theme,** because the daily draw lived only in memory. It is now restored from `random_selection.json` when still valid, and placed first in a fresh shuffle so it does not repeat.
- **A hot-plugged display got no wallpaper for up to five minutes,** and the log claimed it was reapplying: `has_configuration_changed()` compared against the 5-minute monitor cache without refreshing it, and `QueryDisplayConfig` had no retry for `ERROR_INSUFFICIENT_BUFFER` (which *is* the hot-plug case).
- `verify_theme_change()` slept *after* re-reading, so it always compared the pre-change value; `debug!(path = %uri.DisplayUri()?)` made a log statement able to fail the function in non-`log-max-level-info` builds; `ConfigWriter` now replaces the file atomically (tmp + rename) so a concurrent read cannot see a torn document; coordinates and the full config dropped from `INFO` to `DEBUG` (they are personal data and the log directory is one click away in the UI); release default log level is `INFO` and the per-cycle chatter dropped to `DEBUG`, so the release log carries events instead of noise.

### Settings backend (`crates/dwall-settings`)

- **Random-wallpaper users had the product silently disabled:** saving the configuration while the engine mode was `random` wrote `run_at_startup = false` and stopped the daemon. "Start it" is now derived from the mode in one place.
- **A failed download wiped the theme the user already had:** the destination directory was deleted before extracting. Downloads now stream into a `.staging` directory and atomically replace the installed theme only on success, guarded by a `Drop` type that releases the task slot and cleans up even on cancellation; the archive is extracted from a `File` via `spawn_blocking` (no more `fs::read` of the whole archive, or of every thumbnail, into memory), bounded to 8 GiB / 16 384 entries; `Range` resume is only attempted for `206 Partial Content` (a `200` used to append to a partial file and produce a corrupt archive); a 15 s connect and 60 s idle-read timeout replace "wait forever" on a stalled mirror.
- **A 200 OK HTML error page became a cached "thumbnail"**, because the body was never checked. Bodies are now validated against image magic bytes, and concurrent requests for the same key collapse into a single download.
- **The renderer was trusted with filesystem paths and URLs.** `validate_theme_id()` rejects traversal/separators/reserved characters before a theme id is used as a directory name (`download_dir`/`clear_download_cache` recursively delete that directory); `validate_directory()` requires an absolute path without `..`; `open_dir`/`open_url` accept only an existing directory and `http(s)`; `move_directory` refuses a destination inside the source.
- **An autostart entry with a space in the path was parsed as `C:\Program` plus arguments,** and the "is it registered?" comparison never matched; three `unwrap`s could panic on a read-only registry. All fixed, plus eight commands that blocked the webview thread now use `spawn_blocking`/`async`.
- A failed daemon start now surfaces the last `ERROR` line from `dwall.log` instead of leaving the UI silent; the applier no longer refuses to start when `config.toml` is missing, and reads `DWALL_LOG_DIR` like the rest of the crate.
- **Editing the SOCKS5 proxy needed an app restart** — the `reqwest::Client` was built once at startup. It now sits behind an `RwLock` and is rebuilt when the config is written, so a changed proxy applies immediately.
- The startup `POST` to a third-party endpoint (no consent, no proxy, no timeout) is disabled behind `TELEMETRY_ENABLED = false`, which is what `docs/shared-comparison-content.md` already claimed.

### Frontend (`src`)

- **Every switch in the settings window was dead** (auto-detect colour scheme, lock screen, run at startup, start minimised…): `Switch` spread `{...others}` *after* its own `onClick`/`onKeyDown`, so the fallthrough attribute overwrote the handler the component had just defined. `splitProps` now keeps them, toggling respects `disabled`, Space/Enter work, `aria-disabled` is set, and both the plain and `[data, handler]` Solid handler forms go through a shared `callEventHandler` helper (with unit tests).
- **Five animation utilities were referenced but never defined,** so Tailwind v4 emitted nothing for them (no rise-in, no recording dot, no breathing ring). The `--animate-*` tokens and keyframes are declared, and the generated CSS was inspected to confirm they exist.
- **Failed actions looked like success:** every store caught its IPC error and dropped it. They now raise a toast through a `Toaster` that `AppShell` actually mounts, with a new `common.operationFailed` string in all six locales. A persisted locale that is not a supported language (blank window) is now validated, and `update.store` closes the pending-update handle it left open.

### `logging` / `time`

- `logging`: the global max level was set from the logger's own level, so a per-target level above it could never take effect; the macros expanded to `::log::`, which silently loses every record if the caller resolves a different `log` version; the debug formatter wrote ANSI escapes into redirected output and used `eprintln!`, which panics when stderr is closed; `DWALL_LOG=dwall=debug` fell back to the default because only a bare level was parsed; release lines now carry `at file:line`; the log file is truncated after rotation (it was opened with `create(true).write(true)` and no `truncate`, so a file that survived the rename would be overwritten from offset 0 and keep the previous run's tail).
- `time`: `days_to_ymd` narrowed its input to `u32`, so a bogus timestamp was truncated and the internal `+719468` could overflow in debug builds; `OffsetDateTime::eq` compared `utc().ok()` values, making two unrepresentable times compare equal while `partial_cmp` returned `None`; `assume_utc()`/`timestamp()` saturated instead of casting a negative `i64` to a huge `u64`; and `UtcDateTime::now()` panicked for a clock before the epoch — every log record reads the current time, so that panic takes the daemon down.
- Also in the daemon's hot paths: `WallpaperSelector::find_closest_entry` used `partial_cmp().unwrap()`, and TOML permits `altitude = nan`, so one theme file could abort the daemon; frame indices used `index + 1` on a `u8`, which wraps to `0.jpg` in release; `GeolocationAccessStatus` (a struct with constants) had an `unreachable!()` arm that is reachable.

### Build, dependencies, CI

- **The `windows` crate was in the binary twice** (`crates/time` asked for `0`, which resolved to the newer line while everything else sat on 0.61), compiling and linking two copies of the generated bindings. The workspace now pins the line Tauri itself resolves (`0.62`) and `windows-future` moved to `0.3`; `cargo tree` shows exactly one version of every windows-family crate, with tao (via tauri-runtime-wry) unified onto 0.62.2 as well. Two API renames came with it: `windows-future`'s `get()` → `join()` (neither is used any more, see the WinRT waits below) and `windows-core`'s `Error::from_win32()` → `from_thread()`.
- **`tauri-plugin-shell` was registered and granted `shell:default` but never called.** Removed (dependency, `init()`, capability), which also removed the only provider of tokio's `time` feature — so `tokio` and `reqwest` now declare the features they actually use. To remove the plugin in the release artifacts, see the size table below.
- **The CI checks were not checks:** `bun run check` ran `biome check --write`, so it rewrote files and reported success; `bun run test` failed outright (`Cannot find dependency 'jsdom'`, no test files, watch mode in CI); the trigger paths missed `index.html`/`tsconfig.node.json`/`.cargo/**`; and auto-merge used `!failure()`, so a PR whose checks were **skipped** by path filtering still merged. All fixed: the writing form moved to `bun run fix`, `@biomejs/biome` and `jsdom` are lockfile devDependencies, `vitest --run` runs in CI with a real test, and auto-merge now requires the change detection to have run and each check to be `success` or `skipped`.
- **Release-only lints were invisible.** `cfg(debug_assertions)` hides whole blocks from the debug pass, and the shipped configuration (`--features build-script,log-max-level-info`) compiles `debug!`/`trace!` away. CI now runs clippy three ways (debug, `-r`, and release + shipped features) and the warm-cache job populates the release cache so this does not recompile every dependency per PR.
- **Line endings had no policy,** so the Windows runners (`core.autocrlf=true`) checked out CRLF while biome expects LF — `biome check` reported **every** file as misformatted, which the old `--write` form had been hiding by rewriting them. `.gitattributes` now forces LF checkouts (binary assets excluded, NSIS scripts kept at CRLF), which as a side effect removes the recurring phantom-modified files and `LF will be replaced by CRLF` noise.

## Behaviour changes to be aware of

| Change | Note |
| --- | --- |
| Telemetry POST disabled | If usage data is wanted, it must return as an explicit, user-visible opt-in persisted in the config and routed through `HttpClient` so the proxy applies. |
| Google Fonts runtime `<link>` removed; CSP tightened | Latin copy falls back to the system UI font (`--font-*` still lists `"Inter", system-ui, …`, so machines with Inter installed are unchanged). CJK families were already unreachable in mainland China and fell back to system fonts. Bundling subset fonts instead is a product decision (it costs size). |
| `assetProtocol` no longer matches dot-prefixed paths (`requireLiteralLeadingDot` back to its safe default) | No functional loss. |
| Release log level is `info`; lines include `at file:line`; the log file is truncated after rotation | `DWALL_LOG` accepts a bare level or `dwall=debug` / `dwall=info,tauri=warn`. `debug`/`trace` are still compiled out of release. |
| Proxy/mirror changes apply immediately | Previously required restarting the window. |
| A failed theme download no longer deletes the installed theme | It leaves the previous theme in place and the partial download in `.staging`. |

## Verification

Run on Windows (x86_64-pc-windows-msvc), Rust stable, Bun 1.4.2.

```
cargo fmt --all -- --check                                              0
cargo clippy -- -D warnings                                            0
cargo clippy -r -- -D warnings                                         0
cargo clippy -r -p dwall --features build-script,log-max-level-info \
             --all-targets -- -D warnings                              0
DWALL_SKIP_BUILD=1 cargo clippy -r -p dwall-settings \
             --features log-max-level-info --all-targets -- -D warnings 0
cargo clippy -r --all-features --all-targets -- -D warnings              0
cargo test                                                             0
```

`cargo test` — **198 Rust tests, all green**: `dwall` 94 + integration 7, `dwall-settings` 14 (new: this crate previously had none), `logging` 3 (new), `time` 80 (4 new).

Real release builds, no warnings:

| Binary | Size | vs. before |
| --- | --- | --- |
| `target/release/dwall.exe` | 606 720 B | 600 576 B → **+6 144 B (+1.0 %)** |
| `target/release/dwall-settings.exe` | 4 131 328 B | 5 346 304 B → **−1 214 976 B (−22.7 %)** |

The settings binary shrinks by nearly a quarter because the unused shell plugin and its dependency tree are gone and only one `windows` crate is linked; the daemon grows ~1 % from the new error paths and timeouts.

Frontend: `bun run check` (tsgo + biome over 376 files), `bun run test` (3 tests) and `bun run build:vite` all pass; `dist/` is 363.9 KB. The full frontend job (`tsgo --noEmit`, `biome check src`, `vitest --run`) was **also verified inside a fresh Windows worktree with `core.autocrlf=true`**, i.e. the exact condition that used to fail, and all three are green.

Every commit was additionally compiled in isolation (`cargo check --workspace --all-targets` in a detached worktree) so the history is bisectable; two ordering mistakes found that way were fixed by reordering the commits (the dependency commit must precede the daemon's WinRT work and follow the settings `daemon_path`).

**Not verified:** the GUI and the daemon were never *run* against a live desktop — the app changes the user's wallpaper, lock screen and colour scheme, so runtime behaviour (wallpaper transitions, lock-screen application, theme download against a real mirror) still needs a manual pass. Everything above is compile/lint/test-level plus the fresh-checkout CI simulation.

## Deliberately deferred (documented in the checklist, not silently dropped)

- NSIS installer languages (add ja/ko/zh-Hant): needs a real `tauri build` run to validate the language names.
- Narrowing the `assetProtocol` scope below `**/*`: the theme directories are user-configurable across drives, so this needs a custom protocol or streaming commands first.
- `resizable(false)` + `min_inner_size(1024, 640)` on a maximised transparent window clips on 1366×768 @125 %: changing it may break the fixed layout, so it needs a design call.
- Engine-level tests (`ThemeEngine` is still constructed concretely, so `cargo test` cannot reach the cycle; genericising the providers is the prerequisite).
- The remaining P3 backlog (log rotation cap, `SAFETY` comments on the 24 `unsafe` blocks, COM release ordering, friendly-name cache pollution, thumbnail LRU keyed on atime, dead UI modules, `user-select`, a11y labels, empty states, `cargo audit`/`deny.toml`, `rust-toolchain.toml`, `package.json` versioning of the updater, …) — listed with status in the checklist.

## Reviewing this

The checklist is the index: `docs/code-review-2026-10-15.md` maps every item to `path:line`, and each commit body names the item ids it closes. Suggested reading order:

1. `crates/dwall-settings/src/daemon.rs` + `filesystem.rs` — the "engine silently stopped" and path-validation fixes.
2. `crates/dwall-settings/src/theme/downloader.rs` + `network/client.rs` — staging swap, bounds, resume rule, client reconfiguration.
3. `crates/dwall/src/platform/windows/async_operation.rs` + `wallpaper.rs` + `geolocation.rs` — one bounded WinRT wait for every call site (polling `Status()`; no `get()`/`join()`/`IntoFuture`) and the COM string guard.
4. `crates/dwall/src/daemon.rs` — retry loop, mtime baseline, persisted random selection.
5. `crates/logging/src/lib.rs` — level filtering, ANSI gating, file handling.
6. `.github/workflows/pr-check.yml` — the gate changes (these are what will police every later PR).

Test plan for the reviewer: `cargo test`, the clippy matrix above, `bun run check && bun run test && bun run build:vite`, then a manual pass over the settings window (toggle switches, save with random mode, download a theme with a mirror configured, change the proxy without restarting, trigger a failed download and confirm the installed theme survives).
