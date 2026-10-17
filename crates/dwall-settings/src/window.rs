//! Window creation and titlebar color mode.

use std::error::Error;
use std::ffi;

use dwall::ColorScheme;
use tauri::{WebviewUrl, WebviewWindowBuilder};
use windows::{
    Wdk::System::SystemServices::RtlGetVersion,
    Win32::{
        Foundation::{HWND, STATUS_SUCCESS},
        Graphics::Dwm::{
            DWMWA_CAPTION_COLOR, DWMWA_COLOR_DEFAULT, DWMWA_USE_IMMERSIVE_DARK_MODE,
            DwmSetWindowAttribute,
        },
        System::SystemInformation::OSVERSIONINFOW,
    },
};

use crate::error::DwallSettingsResult;

/// Creates a window with the given parameters
pub fn build_window(
    app: &tauri::AppHandle,
    label: &str,
    title: &str,
) -> Result<(), Box<dyn Error>> {
    let window_builder = WebviewWindowBuilder::new(app, label, WebviewUrl::default())
        .title(title)
        // Resizable on purpose, even though the layout is designed for maximized use:
        // Windows sizes a maximized window against the monitor's *work area* only when
        // the window has a sizing frame. Without `WS_THICKFRAME` (what `resizable(false)`
        // removes) it maximizes against the whole monitor, so with the taskbar docked
        // left or right the window grows by the taskbar width and pushes its content off
        // screen. Measured on a 200% DPI display with the taskbar on the left: client
        // area 1536x937 (work area 1488x960) without the frame, 1488x937 with it. The
        // inner minimum below keeps the layout's smallest tier intact.
        .resizable(true)
        .maximizable(true)
        .transparent(true)
        .visible(cfg!(debug_assertions))
        .min_inner_size(1024., 640.)
        .maximized(true);

    match window_builder.build() {
        Ok(_) => Ok(()),
        Err(build_error) => Err(build_error.into()),
    }
}

/// Applies the window's title bar appearance.
///
/// Windows 11 (build 22000+) can paint the caption with an arbitrary colour, which is what
/// makes the title bar match Dwall's own background; `follow_windows_theme` hands the
/// caption back to Windows instead. Windows 10 has no caption-colour attribute at all —
/// only the immersive dark-mode switch — so there the caption follows the light/dark side
/// of the app while its colour stays the system's.
pub fn set_color_mode(
    window_handle: HWND,
    color_mode: ColorScheme,
    follow_windows_theme: bool,
) -> DwallSettingsResult<()> {
    if is_windows_11_or_newer() {
        set_windows_11_caption_color(window_handle, &color_mode, follow_windows_theme)
    } else if follow_windows_theme {
        // The caption colour was never touched for this window, so there is nothing to
        // hand back to the system.
        Ok(())
    } else {
        set_legacy_dark_mode(window_handle, &color_mode)
    }
}

/// Caption colours of the two appearances: the window's own background, so the title bar
/// and the window read as one surface. Taken from the frontend's `--background` token
/// (`src/index.css`): light `oklch(1 0 0)` and dark `oklch(0.145 0 0)`, in sRGB.
const LIGHT_CAPTION_COLOR: u32 = color_ref(0xFF_FF_FF);
const DARK_CAPTION_COLOR: u32 = color_ref(0x0A_0A_0A);

/// Converts an `0xRRGGBB` colour to the `COLORREF` (`0x00BBGGRR`) DWM expects.
const fn color_ref(rgb: u32) -> u32 {
    ((rgb & 0xFF) << 16) | (rgb & 0xFF_00) | ((rgb >> 16) & 0xFF)
}

fn set_windows_11_caption_color(
    window_handle: HWND,
    color_mode: &ColorScheme,
    follow_windows_theme: bool,
) -> DwallSettingsResult<()> {
    let caption_color = if follow_windows_theme {
        // `DWMWA_COLOR_DEFAULT` tells DWM to use the colour the Windows theme asks for.
        DWMWA_COLOR_DEFAULT
    } else {
        match color_mode {
            ColorScheme::Dark => DARK_CAPTION_COLOR,
            ColorScheme::Light => LIGHT_CAPTION_COLOR,
        }
    };

    unsafe {
        DwmSetWindowAttribute(
            window_handle,
            DWMWA_CAPTION_COLOR,
            &caption_color as *const u32 as *const ffi::c_void,
            std::mem::size_of::<u32>() as u32,
        )
        .map_err(Into::into)
    }
}

fn set_legacy_dark_mode(window_handle: HWND, color_mode: &ColorScheme) -> DwallSettingsResult<()> {
    let dark_mode_value: u32 = match color_mode {
        ColorScheme::Dark => 1,
        ColorScheme::Light => 0,
    };

    unsafe {
        DwmSetWindowAttribute(
            window_handle,
            DWMWA_USE_IMMERSIVE_DARK_MODE,
            &dark_mode_value as *const _ as *const ffi::c_void,
            std::mem::size_of::<u32>() as u32,
        )
        .map_err(Into::into)
    }
}

fn is_windows_11_or_newer() -> bool {
    let mut os_version_info = OSVERSIONINFOW {
        dwOSVersionInfoSize: std::mem::size_of::<OSVERSIONINFOW>() as u32,
        ..Default::default()
    };

    let version_check_status = unsafe { RtlGetVersion(&mut os_version_info) };

    if version_check_status != STATUS_SUCCESS {
        return false;
    }

    os_version_info.dwBuildNumber >= 22000
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_colour_is_reordered_into_a_colorref() {
        assert_eq!(color_ref(0x11_22_33), 0x0033_2211);
    }

    #[test]
    fn the_caption_colours_are_the_window_background() {
        // Neutral values, so writing them either way looks the same: this pins that the
        // literals are the sRGB form of `--background` and not a leftover of the old palette.
        assert_eq!(LIGHT_CAPTION_COLOR, 0x00FF_FFFF);
        assert_eq!(DARK_CAPTION_COLOR, 0x000A_0A0A);
    }
}
