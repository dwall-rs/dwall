# AGENTS.md

Guidance for AI coding agents working in this repository.

## Project

**Dwall** — a Windows dynamic-wallpaper app driven by solar position. It ships
two binaries:

- **`dwall`** — the daemon. Always-on, synchronous, single-threaded, and
  deliberately tiny (a few MB of private memory). This low memory footprint is
  a headline feature of the product.
- **`dwall-settings`** — the Tauri 2 GUI backend, paired with a SolidJS
  frontend. Only runs while the settings window is open.

Windows-only (`x86_64-pc-windows-msvc`). The two binaries communicate only
through the config file and process lifecycle; the daemon must never depend on
the GUI.

## Layout

```
crates/dwall/            daemon (bin) + shared library
crates/dwall-settings/   Tauri GUI backend
crates/logging/          custom logger + log macros
crates/time/             hand-rolled time types (no chrono/time dependency)
src/                     SolidJS frontend (Vite + Tailwind v4 + TypeScript)
```

Both Rust crates are organized **by feature, not by layer**. There are no
`domain/`, `infrastructure/`, `services/`, or `app/` directories — do not
reintroduce them.

## Commands

### Rust (run from repo root)

```sh
cargo fmt --all -- --check
cargo clippy -- -D warnings
cargo clippy -r -- -D warnings
cargo test
```

- Lint in **both** profiles. `cfg(debug_assertions)` hides whole blocks from the
  debug pass, and the shipped feature set
  (`cargo clippy -r -p dwall --features build-script,log-max-level-info --all-targets`)
  compiles `debug!`/`trace!` away, which can leave their arguments' bindings
  unused. CI runs all three.
- `cargo test -p dwall --all-features` runs the daemon tests (unit + mockall
  integration + insta snapshots). `--all-features` includes the `ts` feature, so
  it also **regenerates `src/ipc/bindings/**`**: commit the regenerated bindings
  (or run `bun run typegen`) instead of leaving them modified.
- `cargo test` at the root also runs the settings crate; the crate has only a
  binary target, so the per-crate form is `cargo test -p dwall-settings --bins`.
- Building `dwall-settings` requires `target/release/dwall.exe` to exist,
  because it is referenced as a bundle resource in
  `crates/dwall-settings/tauri.conf.json`. CI creates an empty dummy file;
  locally run `bun run build:daemon` (or `cargo build -p dwall --release
--features build-script`) first.
- Set `DWALL_SKIP_BUILD=1` to skip `tauri_build` when you only need to
  type-check the settings crate.

### Frontend (run from repo root)

```sh
bun i
bun run check        # tsgo --noEmit && biome check src   (read-only)
bun run fix          # biome check --write src            (rewrites files)
bun run test         # vitest --run
bun run build:vite   # production frontend build
```

### Full dev / release

```sh
bun run dev     # build daemon (debug) + tauri dev
bun run build   # release: vite build + daemon release + tauri bundle
```

## Rust conventions

- **Feature-module layout.** A file/module is named after the capability it
  provides (`config.rs`, `solar.rs`, `color_scheme.rs`, `theme.rs`,
  `daemon.rs`). SRP is enforced by module cohesion and visibility
  (`pub(crate)`), not by physical layers.
- **Abstractions are earned.** Add a trait only when there is a real second
  implementation or a cross-crate boundary. The provider traits
  (`PositionProvider`, `MonitorProvider`, `WallpaperProvider`,
  `ColorSchemeProvider`) are part of the public API consumed by
  `dwall-settings`, so keep their signatures stable unless you update both
  crates and `crates/dwall/tests/trait_mocks.rs`.
- **Memory is a feature.** In the daemon, avoid:
  - unbounded global caches (bound them with an LRU / capacity),
  - background threads and synchronization that single-threaded code doesn't
    need,
  - per-cycle heap clones (share via `Rc`/references instead).
    Any new resident allocation or dependency needs justification.
- **Windows bindings.** `dwall` depends on the `windows` crate, declared once in
  `[workspace.dependencies]` and pinned to the line Tauri itself resolves
  (currently `0.62`), with an explicit feature list — only enable features that
  are used. WinRT async operations are waited on through
  `platform/windows/async_operation.rs`: never `windows-future`'s `get()`/`join()`
  (infinite wait) and never `IntoFuture` (it needs an executor that this
  synchronous, single-threaded daemon deliberately does not have).
- **Errors.** `thiserror`. Use `DwallResult` / `DwallError` in `dwall` and
  `DwallSettingsResult` / `DwallSettingsError` in `dwall-settings`. Preserve
  `#[from]` conversions rather than adding ad-hoc error types.
- **Logging.** Use the `logging` crate macros with tracing-style key/value
  args: `info!(theme_id = %id, "message")`, `warn!`, `error!`, `debug!`,
  `trace!`. `debug!`/`trace!` are compiled out when the `max-level-info`
  feature is enabled (release builds). Do not use `println!` for diagnostics.
- **`time` is intentional.** `crates/time` is a hand-rolled replacement for
  `chrono`/`time` — it has no calendar/timezone dependency (only `thiserror`,
  plus `windows`/`libc` for the local UTC offset). Don't swap it out.
- **Snapshots.** `insta` snapshots live in a `snapshots/` directory next to the
  source file and are named after the module path (e.g.
  `src/snapshots/dwall__solar__tests__....snap`). Update them with
  `cargo insta review` / `cargo insta accept`.
- **Formatting.** `rustfmt` defaults, edition 2024, 4-space indent.

## Frontend conventions

- **Rust is the single source of truth.** All backend access goes through
  `src/ipc/` — the only place that imports `@tauri-apps/api`. Types are
  generated from Rust with `ts-rs` into `src/ipc/bindings/`; never edit those
  files by hand. Regenerate with `bun run typegen` (requires the `ts` /
  `typegen` cargo features). Business logic (solar position, wallpaper
  matching, config migration, mirror URLs) lives in Rust, not the frontend.
- **SolidJS**, not React. Fine-grained reactivity (`createSignal`,
  `createStore`, `<Show>`, `<For>`) — no virtual DOM, no hooks.
- **Tailwind v4** for styling; keep class names with the component.
- **TypeScript** is checked with `tsgo --noEmit`; **Biome** formats and lints
  (2-space indent, double quotes). Run `bun run check`.
- **Path aliases**: `@/` and `~/` both resolve to `src/`.
- **i18n** uses `@solid-primitives/i18n`. When adding UI strings, update every
  locale file under `src/i18n/` (`en-US`, `zh-CN`, `zh-HK`, `zh-TW`, `ja-JP`,
  `ko-KR`).
- **Feature-first structure.** Organize UI by capability (`stage/`, `scope/`,
  `library/`, `scene/`, `views/`, `layout/`, `components/settings/`); shared
  infrastructure lives in `store/`, `hooks/`, `lib/`, `utils/`, `types/`.
  `models/` holds only pure UI types and pure mapping helpers — no store, IPC,
  or i18n access. Adapters that read stores belong in `store/` (e.g.
  `store/catalog.selectors.ts`).

## Commits

### One commit per conversation

**Create at most one commit per conversation.** Do not split a session's work
across several commits, however unrelated the parts look: stack them in the working
tree and commit once. Several commits per conversation make the remote merge
painful, and a review of the branch cannot follow them.

- If the change set looks like it wants splitting, **ask** before committing; the
  answer is a single commit unless the maintainer says otherwise.
- If you already created several, offer to squash them before the conversation
  ends (with `git reset --soft <base>` plus one commit, or `git rebase -i`). Never
  rewrite commits that are already pushed without an explicit instruction.
- Do not push. Commit locally and leave pushing, force-pushing and PR updates to
  the maintainer.

### Message format

```text
<type>(<scope>): <subject>
```

Use Conventional Commits in English, imperative mood, lowercase, no trailing
period. The release changelog is generated from these prefixes, so use them
consistently:

`feat:`, `fix:`, `perf:`, `refactor:`, `chore:`, `style:`, `test:`, `ci:`.

- **Scope** names the area that changed — a crate (`dwall`, `settings`,
  `logging`, `time`, `deps`) or a frontend feature (`layout`, `stage`, `library`,
  `scene`, `style`, `settings`, `lib`) — matching the existing history.
- **Subject only, no body.** History here is one line per commit; add a short body
  (at most a few lines, no trailers) only when the *why* is not inferable from the
  subject. Never add tool or `Co-authored-by` attribution lines.

Examples:

```text
perf(dwall): bound solar cache and share angle lists
fix(stage): mark today's theme on its own collage tile
fix(settings): identify the daemon by mutex and executable name
```

## Gotchas

- Windows-only. There is no cross-platform fallback; platform code lives under
  `crates/dwall/src/platform/windows/`.
- The release profile is tuned for size and low memory: `panic = "abort"`,
  `lto = true`, `codegen-units = 1`, `opt-level = "z"`, `strip = true`,
  `incremental = false`. Change it deliberately, not incidentally.
- The version source of truth is `package.json`; the release workflow syncs it
  into `Cargo.toml`.
- The config file is `%APPDATA%\dwall\config.toml` and is shared by both
  binaries through `dwall::config` / `dwall::config::ConfigReader`. The daemon
  writes a commented default file when it is missing.
- `dwall.exe` is also the headless control surface (`--check-config`, `--once`,
  `--install-autostart`, `--config`); see `docs/headless.md`. Its exit codes are
  part of that interface.
- The daemon claims a per-session named mutex
  (`Local\com.thep0y.dwall.daemon`) for its whole lifetime, so only one runs per
  session. Never identify a daemon by its install path: the settings app checks
  the mutex (`dwall::daemon_is_running`) and stops it by executable name, because
  a portable or headless copy lives wherever the user put it.
- `crates/dwall/src/lazy.rs` must not panic: it runs from a GUI-subsystem process
  built with `panic = "abort"`, where a failure is invisible. Log and fall back.
- `crates/dwall-settings/build.rs` reads `DWALL_SKIP_BUILD`; CI's Rust job
  builds only the daemon and mocks `target/release/dwall.exe` before running
  clippy/tests.

## Don't

- Don't add `domain`/`infrastructure`/`services`/`app` layer directories.
- Don't add unbounded caches or background threads to the daemon.
- Don't introduce new runtime dependencies casually — small binary and low
  memory are the point.
- Don't break the `dwall` public API without updating `dwall-settings` and
  `crates/dwall/tests/trait_mocks.rs`.
