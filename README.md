# DWALL

DWALL is a lightweight application designed to emulate the macOS wallpaper switching behavior by dynamically changing the desktop background on Windows systems based on the solar altitude and azimuth angles. Experience a seamless transition of wallpapers that mimic the natural movement of the sun throughout the day.

## Motivation

With existing solutions like [WinDynamicDesktop](https://github.com/t1m0thyj/WinDynamicDesktop) and [AutoDarkMode](https://github.com/AutoDarkMode/Windows-Auto-Night-Mode), you might wonder why another tool is necessary. While these applications offer similar features, they operate as separate projects without plans for consolidation. Additionally, their resource consumption can be higher than ideal for such tasks.

Recognizing this gap, DWALL was developed to provide an efficient alternative. It automatically toggles between light and dark themes and changes wallpapers according to the position of the sun, all while maintaining minimal memory usage during operation. For users seeking a less resource-intensive option, DWALL offers a compelling choice.

## Features

- **Sun-Based Scheduling:** Automatically adjusts wallpapers in sync with the sun's path.
- **Low Resource Footprint:** Optimized for performance with minimal impact on system resources.
- **Seamless Integration:** Easily integrates into your workflow without intrusive notifications or settings.

## Before You Begin

This project is still in the development stage and may have some issues. If you encounter any problems during use, please feel free to raise an issue on the GitHub page.

## Usage Steps

1. Download the [latest DWALL executable](https://github.com/dwall-rs/dwall/releases/latest).
2. Run the DWALL executable.
3. Allow DWALL to access your location information, or manually set your location in the settings page.
4. In the side menu, click on the wallpaper you want to use. If it's not downloaded, you'll need to click the "Download" button to download this set of wallpapers. Once downloaded, you can click the "Apply" button to use this set of wallpapers.
5. If you use multiple monitors, you can select the monitor you want to configure separately in the monitor selector, then repeat step 4.

## Screenshots

Below are two screenshots showcasing DWALL in action:

![home](images/home.avif)

![settings](images/settings.avif)

## Running Logs

DWALL's log level is set to `warning` by default. You can adjust the log level by setting the environment variable `DWALL_LOG`, for example, `DWALL_LOG=info`. Please note that the release version will not output logs below the `info` level.

---

We welcome contributions from the community to help improve DWALL. If you encounter any issues or have suggestions for new features, feel free to open an issue or submit a pull request on our GitHub page.
