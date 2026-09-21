# Custom themes — design (not yet implemented)

Status: **design only**. Custom-theme support (list + thumbnail generation +
display) is implemented together with the custom-theme UI, so no half-finished
dead code lands first.

## Goal

Let users add their own themes under the configured
`customized_themes_directory`, list them in the settings UI, preview them, and
apply them — with thumbnails generated automatically when missing.

## Directory layout (on disk)

```
<customized_themes_directory>/
  <theme-id>/
    metadata.toml            # image_format, theme_name, author, version
    solar.json               # [{ index, altitude, azimuth }, ...]
    images/                  # full-size wallpapers: <n>.<jpeg|png>
      1.jpg
      2.jpg
    thumbnails/              # small previews: <n>.avif | <n>.webp
      1.avif
      2.webp
```

- Wallpaper image index is `n` starting at **1** (matches `solar.json`'s
  `index` + 1 and the existing daemon logic).
- Custom thumbnails accept **only `avif` and `webp`**. Other extensions are
  ignored.

## Thumbnail rules

1. Look for `thumbnails/<n>.avif`, then `thumbnails/<n>.webp`.
2. If neither exists, generate `thumbnails/<n>.webp` from `images/<n>.<fmt>`.
3. If the source image is missing or generation fails, return `None` and the UI
   shows a neutral placeholder.

Generation parameters (initial):

- Longest edge ≈ **320 px**, preserve aspect ratio.
- WebP quality ≈ **70**.
- Idempotent: skip if a thumbnail already exists.
- Serialize per `(theme, index)` to avoid duplicate work when several tiles
  request the same thumbnail concurrently.

## Where the work lives

Thumbnail decode/encode is **business logic → Rust**, never the frontend:

- The webview cannot encode WebP without first loading the full-size image into
  memory (the exact lag/memory problem we removed). Rust can stream, downscale
  and cache.
- The frontend only triggers generation and loads the resulting path via the
  asset protocol (`convertFileSrc`, see `src/ipc/asset.ts`).

Dependencies (added to `dwall-settings` only — the GUI is not size-constrained;
the daemon is untouched):

- WebP encode/decode: `image` crate (webp feature) or `webp` crate.
- AVIF is decode-only for reading existing thumbnails; no AVIF encoding needed.

## Rust commands (`dwall-settings`)

```rust
/// Custom themes found under the customized themes directory.
#[tauri::command]
fn list_custom_themes() -> DwallSettingsResult<Vec<CustomTheme>>;

/// Wallpaper solar angles of a custom theme (from its solar.json).
#[tauri::command]
fn get_custom_theme_wallpapers(theme_id: String) -> DwallSettingsResult<Vec<SolarAngle>>;

/// Ensure a thumbnail exists for (theme_id, index) and return its path.
/// Generates a webp from the wallpaper when no avif/webp thumbnail is present.
#[tauri::command]
fn ensure_custom_thumbnail(theme_id: String, index: u8) -> DwallSettingsResult<Option<String>>;
```

`CustomTheme` (extends the existing `CustomizedTheme`):

```ts
type CustomTheme = {
  id: string;
  directory: string;
  imageFormat: "jpeg" | "png";
  themeName: string;
  author: string;
  version: number;
  wallpaperCount: number;   // from solar.json / images
  thumbnailCount: number;   // existing avif/webp thumbnails
};
```

Changes to existing code:

- `get_customized_themes_cmd` currently scans `thumbnails/*.avif` and **skips the
  whole theme** when `images.len() != thumbnails.len()`. Replace that with:
  accept `avif` **and** `webp`, and report missing thumbnails instead of
  dropping the theme. (Or fold this command into `list_custom_themes`.)
- Reuse `dwall::theme::read_solar_angles` for `get_custom_theme_wallpapers`.

## Frontend

- `ThemeThumbnail` (`src/scene/ThemeThumbnail.tsx`) gains a source kind:
  - **catalog theme** → remote thumbnail URL (mirrored + cached), as today.
  - **custom theme** → `ensureCustomThumbnail(themeId, index)` → asset URL.
- The library lists catalog themes **and** custom themes (visually grouped /
  badged as "custom").
- Preview / wallpaper strip use the same thumbnails (never full-size images).
- The settings "paths" section already exposes `customized_themes_directory`;
  add a "选择目录" action and a refresh trigger.

## Edge cases

- Missing `metadata.toml` / `solar.json` → theme is invalid; show it as broken
  rather than silently dropping it (helps users fix their theme).
- Thumbnail present but stale (source image changed) → out of scope for v1;
  a "regenerate thumbnails" action can come later.
- Very large source images → decode with a size hint / downscale-on-load to cap
  memory.
- `solar.json` index not contiguous with image filenames → validate and surface
  a clear error (the daemon already errors on mismatch).

## Verification

- Rust: unit test thumbnail generation (source jpeg/png → webp at expected
  size); test avif/webp lookup precedence; test missing-source returns `None`.
- Frontend: `bun run check` + `bun run build:vite`; manual check that a custom
  theme without thumbnails renders after generation and that no full-size image
  is loaded.
