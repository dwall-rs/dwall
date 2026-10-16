# Custom themes

A theme is one directory:

```
<themes_directory>/            # or <customized_themes_directory> for your own
  <id>/                        # directory name == theme.id == the id Dwall uses
    theme.toml                 # metadata, appearance, solar frames  (schema = 1)
    images/<n>.<ext>           # n = frame index + 1; ext from the theme's image_format (jpeg → jpg)
    thumbnails/<n>.<ext>       # optional; previews only
```

Wallpapers are chosen by matching the current solar position against the frames
declared in `theme.toml` (altitude in `[-90, 90]`, azimuth in `[0, 360)`), so a theme
defines the sun positions its images correspond to. Drop the directory into
`customized_themes_directory` and it appears in the theme library; nothing else is
needed — no import step, no restart.

The manifest format is owned by the **Dwall Maker** project, which also generates
these directories: [github.com/dwall-rs/maker](https://github.com/dwall-rs/maker) →
`docs/theme-format.md` (Chinese). That document is authoritative for `theme.toml`;
this page covers only what Dwall itself implements and what is still missing.

## Implementation notes

Not needed to *use* a theme — kept here because the daemon is the component that
reads them.

- **Manifest** (`crates/dwall/src/theme_manifest.rs`): parses `theme.toml`
  (`schema = 1`) and exposes frames as `SolarAngle`s plus the theme's `image_format`.
- **Daemon** (`crates/dwall/src/theme.rs`, `theme_engine.rs`): reads frames from
  `theme.toml` when present, otherwise the legacy `solar.json`; image paths use the
  theme's format for custom themes and the global config's for legacy ones. Frame
  data is LRU-cached per theme directory.
- **Validation** (`dwall::ThemeValidator`): schema, angle ranges, and that every
  referenced `images/<index + 1>.<ext>` exists.
- **Matching** (`dwall::WallpaperSelector`): the frame with the smallest solar-angle
  distance to the current position.
- **Settings** (`crates/dwall-settings`): lists custom themes (id = directory name,
  metadata from `[theme]`), merges them into the catalog with `source = "custom"`
  and local thumbnails, and resolves wallpapers through `theme.toml`.
- **Frontend**: custom themes appear in the theme library, where thumbnails load
  through the asset protocol directly (no mirror or cache).

### Not implemented yet

- `.dwz` import (theme-format §7) — needs the build-time shared `SALT`.
- `[appearance]` light/dark image selection (§5.4).
- Thumbnail generation when `thumbnails/` is missing: the library falls back to the
  full-size images, which the asset protocol serves.
