# Custom themes

Custom themes use the **Dwall Maker** format. The authoritative spec is Dwall
Maker's [`docs/theme-format.md`](../../dwall-maker/docs/theme-format.md)
(`theme.toml`, `schema = 1`).

## Layout (on disk)

```
<customized_themes_directory>/
  <id>/                      # directory name == theme.id == the theme id Dwall uses
    theme.toml               # single manifest: metadata + appearance + solar frames
    images/<n>.<ext>         # n = index + 1; ext from theme.image_format (jpeg→jpg)
    thumbnails/<n>.<ext>     # optional; previews only
```

## What Dwall implements

- **Manifest** (`crates/dwall/src/theme_manifest.rs`): parses `theme.toml`
  (`schema = 1`), and exposes the frames as `SolarAngle`s plus the theme's
  `image_format`.
- **Daemon** (`crates/dwall/src/theme.rs`, `theme_engine.rs`): `read_solar_angles`
  reads frames from `theme.toml` when present, else the legacy `solar.json`;
  image paths use the **theme's** image format (custom) or the global config's
  (legacy). Frame data is LRU-cached per theme directory.
- **Validation** (`dwall::ThemeValidator`): schema, angle ranges
  (`altitude ∈ [-90, 90]`, `azimuth ∈ [0, 360)`), and that every referenced
  `images/<index + 1>.<ext>` exists.
- **Matching**: the frame with the smallest solar angle distance to the current
  position (`dwall::WallpaperSelector`).
- **Settings** (`crates/dwall-settings`):
  - `get_customized_themes_cmd` lists installed custom themes (id = directory
    name, metadata from `[theme]`, thumbnails or images).
  - `get_theme_catalog` merges them into the catalog with `source = "custom"`
    and local thumbnail paths.
  - `get_theme_wallpapers` / `match_wallpaper` resolve custom themes via
    `theme.toml`.
- **Frontend**: custom themes appear in the theme library; `ThemeThumbnail`
  loads their local thumbnails directly through the asset protocol (no mirror /
  cache).

## Not yet implemented

- `.dwz` import (theme-format §7) — needs the build-time shared `SALT`.
- `[appearance]` light/dark image selection (§5.4).
- Automatic thumbnail generation when `thumbnails/` is missing (the library
  falls back to the full-size images, which the asset protocol still serves).

## Verification

- Rust: `cargo test -p dwall --lib` (manifest parsing/schema/range tests).
- Frontend: `bun run check` + `bun run build:vite`.
