interface Config {
  github_mirror_template?: string;
  selected_theme_id?: string;
  interval: number;
  image_format: string;
  themes_directory: string;
  coordinate_source: CoordinateSource;
  auto_detect_color_mode: boolean;
  lock_screen_wallpaper_enabled: boolean;
  monitor_specific_wallpapers: string | Record<string, string>;
}

interface CoordinateSourceAutomatic {
  type: "AUTOMATIC";
}

interface CoordinateSourceManual {
  type: "MANUAL";
  latitude?: number;
  longitude?: number;
}

type CoordinateSource = CoordinateSourceAutomatic | CoordinateSourceManual;
