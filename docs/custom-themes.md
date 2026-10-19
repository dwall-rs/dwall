# Custom themes

A theme is one directory. Which subdirectory holds the images depends on which root the
theme sits under, and Dwall decides that by looking at both roots in order:

```
# Themes you write go here:
<customized_themes_directory>/
  <id>/                        # directory name == theme.id == the id Dwall uses
    theme.toml                 # metadata, appearance, solar frames  (schema = 1)
    images/<n>.<ext>           # n = frame index + 1; ext from the theme's image_format (jpeg → jpg)
    thumbnails/<n>.<ext>       # optional; previews only

# The curated themes the app installs go here, with a format-named image directory:
<themes_directory>/
  <id>/
    theme.toml
    jpg/1.jpg                  # <image_format>/<n>.<ext>, not images/ (jpeg → jpg, png → png)
```

A curated set copied into place by hand — the usual way to deploy themes on a headless
install — therefore belongs under `themes_directory`; put it under
`customized_themes_directory` and Dwall will look for `images/` and find nothing. If the
same id exists under both roots, the one under `themes_directory` wins.

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
  referenced frame image exists in the directory the theme's root implies (`images/`
  for a custom theme, `<image_format>/` for a curated one).
- **Matching** (`dwall::WallpaperSelector`): the frame whose declared position is the
  smallest **angle** away from the current one, measured on the sky rather than in the
  altitude/azimuth plane — so a degree of azimuth counts for `cos(altitude)` of a degree
  of altitude. Near the zenith the azimuth barely matters (it swings fastest exactly
  where it means least); at the horizon the two count the same. Declare frames where the
  sun actually goes for the latitudes you care about, and spread them by altitude.
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
