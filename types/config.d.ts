interface Config {
  github_mirror_template?: string;
  selected_theme_id?: string;
  interval: number;
  image_format: string;
  themes_directory: string;
  position_source: PositionSource;
  auto_detect_color_scheme: boolean;
  lock_screen_wallpaper_enabled: boolean;
  monitor_specific_wallpapers: string | Record<string, string>;
}

interface PositionSourceAutomatic {
  type: "AUTOMATIC";
}

interface PositionSourceManual {
  type: "MANUAL";
  latitude?: number;
  longitude?: number;
  altitude?: number;
}

type PositionSource = PositionSourceAutomatic | PositionSourceManual;
