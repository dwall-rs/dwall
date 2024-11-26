use std::{
    fs,
    path::{Path, PathBuf},
    sync::{Arc, LazyLock},
    time::Duration,
};

use time::{
    macros::{offset, time},
    OffsetDateTime,
};
use tokio::{
    sync::{mpsc, Mutex},
    time::sleep,
};
use windows::Win32::UI::WindowsAndMessaging::{
    SystemParametersInfoW, SPIF_SENDWININICHANGE, SPIF_UPDATEINIFILE, SPI_GETDESKWALLPAPER,
    SPI_SETDESKWALLPAPER, SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS,
};

use crate::{
    color_mode::{determine_color_mode, set_color_mode},
    error::{DwallError, DwallResult},
    geo::get_geo_postion,
    lazy::APP_CONFIG_DIR,
    solar::{calculate_angle_difference, SolarAngle, SunPosition},
};

pub static THEMES_DIR: LazyLock<PathBuf> = LazyLock::new(|| APP_CONFIG_DIR.join("themes"));

#[derive(Debug, thiserror::Error)]
pub enum ThemeError {
    #[error("theme is not exists")]
    NotExists,
    #[error("missing solar config file")]
    MissingSolarConfigFile,
    #[error("the number of images does not match `solar.json`")]
    ImageCount,
}

#[tauri::command]
pub fn check_theme_exists(id: String) -> DwallResult<()> {
    let theme_dir = THEMES_DIR.join(id);

    if !theme_dir.exists() {
        return Err(ThemeError::NotExists.into());
    }

    let solar_angles = read_theme_solar_config(&theme_dir)?;
    let mut indices: Vec<u8> = solar_angles.iter().map(|angle| angle.index).collect();
    indices.sort_unstable();
    indices.dedup();

    if !check_images_in_directory(theme_dir, &indices, "jpg") {
        return Err(ThemeError::ImageCount.into());
    }

    Ok(())
}

fn read_theme_solar_config(theme_dir: &Path) -> DwallResult<Vec<SolarAngle>> {
    let solar_config_file = theme_dir.join("solar.json");
    if !solar_config_file.exists() || !solar_config_file.is_file() {
        return Err(ThemeError::MissingSolarConfigFile.into());
    }

    let solar_config = fs::read_to_string(&solar_config_file)?;
    let solar_angles: Vec<SolarAngle> = serde_json::from_str(&solar_config)?;

    Ok(solar_angles)
}

fn check_images_in_directory<P: AsRef<Path>>(directory: P, indices: &[u8], format: &str) -> bool {
    let path = directory.as_ref().join(format);
    if !path.is_dir() {
        println!("The provided path is not a directory.");
        return false;
    }

    for i in indices {
        let file_name = format!("{}.{}", i + 1, format);
        let full_path = path.join(file_name);

        if !full_path.exists() || !full_path.is_file() {
            println!(
                "File {} does not exist or is not a regular file.",
                full_path.display()
            );
            return false;
        }
    }

    true
}

pub type CloseTaskSender = Arc<Mutex<Option<mpsc::Sender<()>>>>;

#[tauri::command]
pub async fn close_last_theme_task(sender: tauri::State<'_, CloseTaskSender>) -> DwallResult<()> {
    if let Some(tx) = sender.lock().await.take() {
        tx.send(()).await.unwrap();
    }

    Ok(())
}

#[tauri::command]
pub async fn apply_theme(
    id: String,
    sender: tauri::State<'_, CloseTaskSender>,
    format: String,
) -> DwallResult<()> {
    let (tx, mut rx) = mpsc::channel::<()>(1);

    tauri::async_runtime::spawn(async move {
        loop {
            tokio::select! {
            _ = sleep(Duration::from_secs(1)) => {
                    let postion = get_geo_postion()?;

                    let theme_dir = THEMES_DIR.join(&id);
                    let solar_angles = read_theme_solar_config(&theme_dir)?;

                    let now = OffsetDateTime::now_utc().to_offset(offset!(+8));
                    let date_time = now.replace_time(time!(19:00));


                    let sun_position = SunPosition::new(
                                                postion.latitude,
                                                postion.longitude,
                                                date_time,
                                                8);
                    let altitude = sun_position.altitude();
                    let azimuth = sun_position.azimuth();
                    println!(
                        "Calculated solar angles - Elevation: {:.1}°, Azimuth: {:.1}°",
                        altitude, azimuth
                    );

                    let index = find_closest_image(&solar_angles, altitude, azimuth);
                    set_wallpaper(theme_dir.join(&format).join(format!("{}.jpg", index.unwrap()+1)))?;
                    println!("{:?}", index);
                    let color_mode = determine_color_mode(altitude);
                    println!("color mode: {:?}", color_mode);
                    set_color_mode(color_mode)?;
                },
                _ = rx.recv() => {
                    println!("收到退出信号，正在关闭");
                    break;
                }
            }
        }
        Ok::<(), DwallError>(())
    });

    let sender = sender.clone();
    let mut sender = sender.lock().await;
    *sender = Some(tx);

    Ok(())
}

fn get_current_wallpaper() -> DwallResult<PathBuf> {
    let mut buffer = vec![0u16; 1024];

    unsafe {
        SystemParametersInfoW(
            SPI_GETDESKWALLPAPER,
            buffer.len() as u32,
            Some(buffer.as_mut_ptr() as *mut _),
            SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0),
        )?;

        let path_str = String::from_utf16_lossy(&buffer)
            .trim_matches('\0')
            .to_string();

        Ok(PathBuf::from(path_str))
    }
}

fn set_wallpaper(image_path: PathBuf) -> DwallResult<()> {
    let current_wallpaper = get_current_wallpaper()?;

    if current_wallpaper == image_path {
        return Ok(());
    }

    let wide_path: Vec<u16> = image_path
        .to_string_lossy()
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();

    unsafe {
        SystemParametersInfoW(
            SPI_SETDESKWALLPAPER,
            0,
            Some(wide_path.as_ptr() as *mut _),
            SPIF_UPDATEINIFILE | SPIF_SENDWININICHANGE,
        )?;
    }

    Ok(())
}

fn find_closest_image(configs: &[SolarAngle], altitude: f64, azimuth: f64) -> Option<u8> {
    configs
        .iter()
        .map(|config| {
            let difference = calculate_angle_difference(config, altitude, azimuth);
            (config.index, difference)
        })
        .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
        .map(|(index, _)| index)
}
