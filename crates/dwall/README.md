# dwall

The daemon and shared library for [Dwall](https://github.com/dwall-rs/dwall), a
Windows dynamic-wallpaper app driven by the sun's position.

The binary is also the headless control surface (`--check-config`, `--once`,
`--install-autostart`, `--config`); see [docs/headless.md](https://github.com/dwall-rs/dwall/blob/main/docs/headless.md).
The settings window lives in the separate `dwall-settings` crate and is not a
dependency of this one.
