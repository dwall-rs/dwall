# Documentation

| Document | Answers |
| --- | --- |
| [../README.md](../README.md) | What DWALL is, how to install and use it. |
| [headless.md](./headless.md) | Running `dwall.exe` without the settings window: configuration, commands, auto-start, logs. |
| [custom-themes.md](./custom-themes.md) | Theme layout, the manifest, and what Dwall implements. |
| [shared-comparison-content.md](./shared-comparison-content.md) | Verified specifications and the feature matrix for DWALL, WinDynamicDesktop and Auto Dark Mode. |
| [dwall-vs-auto-dark-mode.md](./dwall-vs-auto-dark-mode.md) | DWALL against Auto Dark Mode. |
| [dwall-vs-windynamicdesktop.md](./dwall-vs-windynamicdesktop.md) | DWALL against WinDynamicDesktop. |
| [technical-comparison.md](./technical-comparison.md) | Process model, dependencies, file formats, distribution and update paths. |
| [user-benefits.md](./user-benefits.md) | Which tool fits which user, and what DWALL asks in return. |

Chinese translations of the comparison set sit next to each file as `*.zh-CN.md`.

## Writing rules for this directory

- **Verify, then write.** Every claim about another project must come from that
  project's own repository, documentation, release metadata or store listing, and be
  linked. Do not characterise its internals, performance or code quality.
- **Measure DWALL.** Numbers for DWALL come from this repository's release build
  (`cargo build -p dwall --release`, Task Manager private bytes, file sizes) — never
  from intuition. Say how a number was obtained.
- **No verdicts, no marketing.** State what each tool does and where each is the
  better fit, including where DWALL is behind.
- **One fact, one place.** Shared specifications live in
  [shared-comparison-content.md](./shared-comparison-content.md); the other documents
  link to it instead of repeating it.
