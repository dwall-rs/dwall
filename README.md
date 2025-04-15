<p align="center">
<img height="64" width="64" src="./src-tauri/icons/128x128.png" />
<br/>
<a href="https://github.com/dwall-rs/dwall/releases/latest"><img src="https://img.shields.io/github/downloads/dwall-rs/dwall/total.svg?style=flat-square" alt="GitHub releases"/></a>
<img src="https://img.shields.io/badge/Windows%2010-%230079d5.svg?style=for-the-badge&logo=Windows%2010&logoColor=white)" alt="Windows 10" height="20" />
<img src="https://img.shields.io/badge/Windows%2011-%230079d5.svg?style=for-the-badge&logo=Windows%2011&logoColor=white)" alt="Windows 11" height="20" />
</p>

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

> [!WARNING]  
> This program may not run on lightweight operating systems. If you are using a lightweight version of Windows and encounter errors, please do not submit an issue. Try running the program after installing the official Windows operating system.

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

## Frequently Asked Questions

### 1. Why are the settings and daemon processes completely isolated?

In the initial version, the settings and daemon processes ran in the same process, with taskbar tray management support. However, this resulted in higher memory usage (though still much less than other similar programs), which wasn't the desired outcome. Therefore, the settings and daemon processes were completely isolated to reduce the daemon's memory footprint and simplify process management.

Of course, this also introduced inter-process communication challenges. In the current version, the settings process (graphical program) and daemon process don't implement inter-process communication. Their only means of communication is through configuration files, which means that when the daemon process abnormally exits, the settings process cannot promptly obtain the daemon's status. This is an issue that needs to be addressed.

### 2. Wallpaper download failure.

Wallpaper resources are stored on GitHub, but some countries and regions cannot access GitHub normally due to network restrictions. If you don't set a GitHub mirror template, wallpaper downloads will fail. You need to search for available GitHub mirror or acceleration sites using a search engine. For example, if you find an available GitHub acceleration site like `https://ghproxy.cc`, you can configure the GitHub mirror template in the settings page as follows:

```
https://ghproxy.cc/https://github.com/<owner>/<repo>/releases/download/<version>/<asset>
```

If you find a GitHub mirror site like `https://kkgithub.com/`, you would need to configure the GitHub mirror template as follows:

```
https://kkgithub.com/<owner>/<repo>/releases/download/<version>/<asset>
```

Then you can download wallpapers normally.

### 3. Why doesn't support custom wallpapers?

The wallpaper transitions need to correlate with the sun's position. Since most users lack specialized astronomical knowledge to perfectly align custom wallpapers with solar movements, we provide pre-configured wallpaper sets that are scientifically aligned with sun positions. Users can choose from these curated collections.

### 4. I can design sun-position-aligned wallpaper sets. How to contribute them to DWALL?

You may upload your designed wallpaper sets to cloud storage, then create a GitHub issue describing your design concept. We'll consider incorporating qualified submissions into DWALL during future updates.

### 5. Why do thumbnails fail to load?

Thumbnails use GitHub direct links. Some regions have network restrictions that prevent access to GitHub. You can resolve this by setting up a GitHub mirror template.

### 6. Why not use system tray to manage the background process?

As mentioned in Question 1, the system tray process itself consumes more memory than the background process. Given this consideration, I decided not to use system tray management for the background process.

---

We welcome contributions from the community to help improve DWALL. If you encounter any issues or have suggestions for new features, feel free to open an issue or submit a pull request on our GitHub page.
