<!--
  Keep the sections that apply and delete the rest, including these comments.
  The pull request title becomes the squash commit message, so use the
  repository's Conventional Commit style: `fix(settings): ...`, `feat(dwall): ...`.
-->

## What and why

<!-- Two or three sentences: what changed and which problem it solves. Link the issue if there is one. -->

## Verification

<!--
  Paste the commands you ran with their result. Do not claim a result you did
  not observe, and say what you did not verify.
-->

```sh
cargo fmt --all -- --check
cargo clippy -- -D warnings
cargo clippy -r -- -D warnings
cargo clippy -r -p dwall --features build-script,log-max-level-info --all-targets -- -D warnings
cargo test
bun run check && bun run test    # frontend changed?
bun run build:vite               # frontend changed?
```

`pr-check.yml` runs the gates above, so a red CI usually means one of them.

Not verified:

## Conventions

<!-- Delete what the change does not touch. -->

- [ ] Rust: feature modules only — no `domain/`/`infrastructure/`/`services/`/`app/` directories.
- [ ] Daemon (`crates/dwall`): no new background thread, unbounded cache or per-cycle clone — memory is a feature.
- [ ] Frontend: `@tauri-apps/api` imported only under `src/ipc/`, and new UI strings added to all six locales under `src/i18n/`.
- [ ] Generated `src/ipc/bindings/**` committed when Rust types changed (`bun run typegen`).
- [ ] `dwall` public API changes come with the matching `dwall-settings` and `tests/trait_mocks.rs` updates.
- [ ] Docs updated when behaviour changed — `docs/headless.md` and `docs/custom-themes.md` ship inside the daemon archive.
- [ ] Comparison docs cite the other project's own sources and measure DWALL's numbers (rules in `docs/README.md`).

## Notes for the reviewer

<!--
  Trade-offs you made deliberately, measurements and how they were taken,
  anything surprising in the diff, and follow-ups left out of scope.
-->

## Screenshots

<!-- UI changes only; otherwise delete this section. -->
