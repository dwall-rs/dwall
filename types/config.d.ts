type GithubMirrorTemplate = string;

interface Socks5 {
  host: string;
  port: number;
}

type Network = GithubMirrorTemplate | Socks5;

interface Config {
  network?: Network;
  selected_theme_id?: string;
  interval: number;
  image_format: string;
  themes_directory: string;
  customized_themes_directory: string;
  position_source: PositionSource;
  auto_detect_color_scheme: boolean;
  lock_screen_wallpaper_enabled: boolean;
  monitor_specific_wallpapers: string | Record<string, string>;
  title_bar_color_follows_windows_theme: boolean;
}

interface PositionSourceAutomatic {
  type: "AUTOMATIC";
  update_on_each_calculation?: boolean;
  cache_minutes?: number;
}

interface PositionSourceManual {
  type: "MANUAL";
  latitude?: number;
  longitude?: number;
  altitude?: number;
}

type PositionSource = PositionSourceAutomatic | PositionSourceManual;
