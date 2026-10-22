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

### One commit per change

**Split the work by feature.** A commit carries one functional change — a feature, a bug
fix, a test batch, a cleanup. Never fold unrelated changes into it, even when they were made
in the same pass: a reviewer has to be able to accept or reject each one on its own.

- **A change that spans several areas is several commits.** When a feature or a fix needs
  work in more than one place — a shared type and the callers that use it, a component and
  the store it reads — commit each part separately, and say in every one of them why that
  part exists. "Part of <feature>" is not a reason; name the job the part does.
- **Ask after every commit.** Never push on your own initiative: after each commit, ask the
  maintainer whether to push it or leave it local, and wait for the answer. Pushing,
  force-pushing and updating a PR all need that instruction.
- **Re-check before every amend or rebase**: run `git log --oneline '@{u}..HEAD`
  immediately beforehand, because the maintainer can push while you work. If the target
  commit reached the remote in the meantime, add a new commit instead of rewriting it and
  say what happened.
- Never rewrite a commit that is already pushed without an explicit instruction.

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
- **One-line subject, short body.** Add a body (at most a few lines, no trailers) whenever
  the commit is one part of a larger change, or when the *why* is not inferable from the
  subject. Never add tool or `Co-authored-by` attribution lines.

Examples:

```text
perf(dwall): bound solar cache and share angle lists
fix(stage): mark today's theme on its own collage tile
fix(settings): identify the daemon by mutex and executable name
```

### Branching and pull requests

`main` takes no direct change. **Nothing is committed or pushed to it** — every change
starts on its own branch and reaches `main` through a pull request whose `pr-check` is
green.

Protection is not on yet: `gh api repos/dwall-rs/dwall/branches/main/protection` answers
404. The repository is public, so protection is available on the free plan; until it is
switched on, this rule is held by the people and agents working in the repository rather
than by the platform — which is all the more reason to follow it, and to switch protection
on, requiring `pr-check`, as soon as someone gets to it.

```bash
git fetch origin
git switch -c fix/nsis-space-in-name origin/main
# ... work, commit ...
git push -u origin fix/nsis-space-in-name
gh pr create --base main --fill
```

- **Branch names describe the change, not the person**: `<type>/<slug>`, using the commit
  types (`feat/sun-path`, `fix/nsis-space-in-name`, `docs/english-docs`,
  `ci/upx-optional`). One branch per functional change — a branch that accumulates three
  unrelated fixes cannot be reviewed, and cannot be reverted without dragging the others
  along.
- **Branch from `main`.** A change based on `dev` cannot be reviewed or reverted on its
  own, and it drags whatever else `dev` carries into the pull request.
- **The pull request is the unit of review.** Its description says what changes and why, in
  the same terms as the commit body, so that the merged history stays readable on its own.
- **Title and description are English only**, for the same reason the commit messages are:
  the change record is read by people, tools and agents that did not take part in the
  conversation that produced it. Discuss in any language in the comments — that is
  conversation, not record.
- **Keep the branch current by rebasing on `main`**, not by merging `main` into it, so the
  history stays linear and the diff stays reviewable.
- **No direct pushes, no force-pushes to `main`, no `--no-verify`.** An urgent one-line fix
  is still a pull request; with `pr-check` green it merges in under a minute.
- **Delete the branch once it is merged.** Nothing removes it for you yet.
- **Agents follow this too.** Create the branch *before* the first commit rather than
  committing on `main` and moving the commits afterwards: a rebased-away commit still shows
  up in the reflog and in whoever is watching the terminal, and it means part of the work
  was written under different assumptions than the part that is reviewed.

The one commit that reaches `main` without a pull request is the release bump that
`publish.yml` fast-forwards in (`update-main-version` merges `release` with `--ff-only`) —
after the release is public, and only when `main` has not moved. It changes nothing but the
version and the lock file, and it is tolerated because no protection exists to bypass it;
the day protection exists, either list `github-actions[bot]` as an exception or make
`publish.yml` open a pull request and tag after that merges. A rule the repository's own
automation ignores is not a rule.

## Gotchas

- Windows-only. There is no cross-platform fallback; platform code lives under
  `crates/dwall/src/platform/windows/`.
- The release profile is tuned for size and low memory: `panic = "abort"`,
  `lto = true`, `codegen-units = 1`, `opt-level = "z"`, `strip = true`,
  `incremental = false`. Change it deliberately, not incidentally.
- The version source of truth is `package.json`; the release workflow syncs it
  into `Cargo.toml`.
- The update check has two paths, both in `check_for_updates_cmd`: a stable build
  reads `releases/latest/download/latest.json`, which GitHub keeps free of
  pre-releases; a pre-release build lists the releases through `api.github.com`
  and compares them with `semver` (a stable release beats a pre-release, and
  nothing older is ever offered), then reads that release's `latest.json`. Every
  release — pre-release included — therefore has to carry a `latest.json` asset,
  and the GitHub API rejects requests without a `User-Agent`, which `HttpClient`
  now always sends.
- `tauri-action` writes every updater manifest with the
  `releases/latest/download/<asset>` route, which only resolves while that release is the
  newest *stable* one. A pre-release's manifest therefore has to be re-pointed at its own
  tag before the download (`update::retag_manifest_url`), or the install asks the newest
  stable release for an asset it does not have and fails with 404 — while the check, which
  only reads the version, still looks fine.
- Only the newest pre-release is kept: `publish.yml` drops the older ones as soon as a new
  pre-release is published (the tags stay, so the source is still reproducible), because
  the updater never offers an older one and a stale installer left on the releases page is
  what people install by mistake. Stable releases are all kept — the download counts show
  older ones are still in use. The winget and Scoop manifests are rendered for the stable
  channel only, so a submitted manifest can never point at a dropped pre-release.
- Rust caches are keyed by `dtolnay/rust-toolchain`'s `cachekey`, which is a hash of the
  resolved compiler, and the `restore-keys` prefixes keep it: cargo's fingerprints include
  the compiler, so artifacts from another rustc are unusable — restoring them only costs a
  ~600 MB download and a full rebuild. A stable update changes that key without touching
  any file the path filters watch, which is why `warm-cache.yml` also runs weekly: it is
  the only job that saves a *debug* cache, and publish/nightly only ever save the release
  one. Every Rust cache also lists `target/<profile>/.fingerprint` next to `deps` and
  `build`; without it cargo cannot tell a restored artifact is fresh and rebuilds anyway.
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
- The standalone daemon archive is produced by `.scripts/package-daemon.ps1`
  (`Dwall_<version>_x64-daemon.zip` plus a `.sha256`), attached to releases by
  `publish.yml` and to the nightly release by `nightly.yml`. It ships `dwall.exe`
  together with `docs/headless.md` and `docs/custom-themes.md`, so keep those
  documents accurate: they are the only documentation a headless install gets.
- The winget and Scoop manifests are templates in `.packaging/`, filled from a
  release's checksum by `.scripts/render-package-manifests.ps1` (rendered in
  `publish.yml` too, so a stale template fails the release). They are submitted to
  those package repositories by hand; run the script with `-Validate` before
  submitting.
- The daemon must stay windowless when it is auto-started: release builds are
  GUI-subsystem executables (`windows_subsystem` is set for
  `not(debug_assertions)`), while a debug build is a console application that the
  `Run` key would give a console window at sign-in. `--install-autostart` warns for
  a console build, `package-daemon.ps1` refuses to package one, and
  `DaemonLauncher` always passes `CREATE_NO_WINDOW`.
- Comparison documents follow the writing rules in `docs/README.md`: cite the other
  project's own repository, documentation or listing, measure DWALL's numbers from a
  release build and say how, prefer no verdict over a favourable one. Never restate an
  unsourced figure or a superlative.
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
- Don't give the daemon theme management. It reads `themes_directory` and
  `customized_themes_directory` and nothing else; downloading, mirrors, unpacking
  and thumbnails stay in `dwall-settings`, and a headless install deploys its
  themes by hand.
