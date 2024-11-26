# DWALL

By calculating the solar elevation angle and azimuth angle to simulate the wallpaper switching logic of macOS, implement a smooth dynamic wallpaper switching feature on the Windows system.

### Purpose

Why create another similar software when there is already [WinDynamicDesktop](https://github.com/t1m0thyj/WinDynamicDesktop) and [AutoDarkMode](https://github.com/AutoDarkMode/Windows-Auto-Night-Mode)?

These two pieces of software have similar functionalities, and ideally, they should have been combined into one application, but the projects do not have such intentions.

Moreover, as background-running processes, the memory usage of these two applications far exceeds what is necessary for their functions.

Taking into account these two reasons, I developed dwall, which can automatically switch between light and dark modes and change wallpapers according to the sun's angle, while maintaining very low memory consumption when running in the background. This project can serve as an alternative to the aforementioned two applications.
