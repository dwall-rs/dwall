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
crates/time/             hand-rolled, zero-dependency time types
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
cargo test
```

- `cargo test -p dwall --all-features` runs the daemon tests (unit + mockall
  integration + insta snapshots).
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
bun run check        # tsgo --noEmit && biome check --write src
bun run test         # vitest
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
- **Windows bindings.** `dwall` depends on the `windows` crate (version pinned
  to match Tauri, currently `0.61`) with an explicit feature list. Only enable
  the features actually used; trim unused ones.
- **Errors.** `thiserror`. Use `DwallResult` / `DwallError` in `dwall` and
  `DwallSettingsResult` / `DwallSettingsError` in `dwall-settings`. Preserve
  `#[from]` conversions rather than adding ad-hoc error types.
- **Logging.** Use the `logging` crate macros with tracing-style key/value
  args: `info!(theme_id = %id, "message")`, `warn!`, `error!`, `debug!`,
  `trace!`. `debug!`/`trace!` are compiled out when the `max-level-info`
  feature is enabled (release builds). Do not use `println!` for diagnostics.
- **`time` is intentional.** `crates/time` is a hand-rolled, zero-dependency
  replacement for `chrono`/`time`. Don't swap it out.
- **Snapshots.** `insta` snapshots live in a `snapshots/` directory next to the
  source file and are named after the module path (e.g.
  `src/snapshots/dwall__solar__tests__....snap`). Update them with
  `cargo insta review` / `cargo insta accept`.
- **Formatting.** `rustfmt` defaults, edition 2024, 4-space indent.

## Frontend conventions

- **SolidJS**, not React. Fine-grained reactivity (`createSignal`,
  `createStore`, `<Show>`, `<For>`) — no virtual DOM, no hooks.
- **Tailwind v4** for styling; keep class names with the component.
- **TypeScript** is checked with `tsgo --noEmit`; **Biome** formats and lints
  (2-space indent, double quotes). Run `bun run check`.
- **Path aliases**: `@/` and `~/` both resolve to `src/`.
- **i18n** uses `@solid-primitives/i18n`. When adding UI strings, update every
  locale file under `src/i18n/` (`en-US`, `zh-CN`, `zh-HK`, `zh-TW`, `ja-JP`,
  `ko-KR`).

## Commits

Use Conventional Commits in English, imperative mood. The release changelog is
generated from these prefixes, so use them consistently:

`feat:`, `fix:`, `perf:`, `refactor:`, `chore:`, `style:`, `test:`, `ci:`.

Example: `perf(dwall): bound solar cache and share angle lists`.

## Gotchas

- Windows-only. There is no cross-platform fallback; platform code lives under
  `crates/dwall/src/platform/windows/`.
- The release profile is tuned for size and low memory: `panic = "abort"`,
  `lto = true`, `codegen-units = 1`, `opt-level = "z"`, `strip = true`,
  `incremental = false`. Change it deliberately, not incidentally.
- The version source of truth is `package.json`; the release workflow syncs it
  into `Cargo.toml`.
- The config file is `%APPDATA%\dwall\config.toml` and is shared by both
  binaries through `dwall::config` / `dwall::config::ConfigReader`.
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
