# DWALL 对比 Auto Dark Mode

两者都是免费、开源的 Windows 应用，都会运行按用户的常驻引擎、切换系统浅色/深色主题，
也都能设置每显示器壁纸。区别在于由什么驱动切换，以及它们触及 Windows 的多少部分。

## 一览

|            | DWALL                                                  | Auto Dark Mode                                                                      |
| ---------- | ------------------------------------------------------ | ----------------------------------------------------------------------------------- |
| 主要职责 | 依据太阳位置选择壁纸帧 | 按计划切换 Windows 深色/浅色主题 |
| 切换信号 | 本地计算的太阳高度角/方位角（折射、海拔、极地纬度），或手动坐标 | 固定时间、来自 Windows 位置的日出日落时间、夜间模式状态、环境光传感器、热键、脚本 |
| 壁纸 | 主题 = 绑定到太阳位置的帧，持续匹配；每显示器；可选锁屏 | 每个主题每台显示器一张图片，纯色或幻灯片；未有锁屏功能的文档 |
| 还会更改 | 应用与系统（任务栏）两个浅色/深色主题值，以及锁屏壁纸 | 应用/系统主题、主题色、鼠标光标、`.theme` 文件、任务栏/标题栏、灰度 |
| 可扩展性 | 有文档的主题格式（`theme.toml`、Dwall Maker）与自定义主题目录；配置文件变更后守护进程会重新读取；不含脚本或条件模块 | 自定义脚本、条件（游戏、电池、夜间模式）、热键 |
| 技术栈 | Rust 守护进程（无异步运行时）+ Tauri/SolidJS GUI | .NET 9：WinUI 3 GUI、基于 WinForms 的引擎、控制台 CLI 外壳 |
| 占用 | 守护进程私有工作集约 2 MB（私有字节 3–8 MB），0.63 MB 可执行文件；无托盘图标 | 项目未有文档说明；托盘图标与命名管道命令服务器 |
| CPU 架构 | x64 | x64、ARM64 |
| 语言 | 6 | 31 |
| 分发 | 安装程序 + 便携式守护进程压缩包；无商店页面 | GitHub、Microsoft Store（免费）、winget `ArminOsaj.AutoDarkMode`；Chocolatey/Scoop 条目为非官方 |
| 系统要求 | Windows 10/11 x64；GUI 需要 WebView2 | Windows 10 22H2 或更高版本、Windows 11 |
| CLI | `dwall.exe --check-config`、`--once`、`--install-autostart`、`--uninstall-autostart`、`--config`、`--help`、`--version` | `AutoDarkModeShell.exe`，通过引擎的命名管道发送命令 |
| 许可证 | AGPL-3.0-only | GPL-3.0 |

## 选择 DWALL，如果

- 你希望壁纸本身跟随太阳，而不是在两种状态间翻转主题。
- 常驻开销应只有几 MB，且没有托盘图标。
- 你以无界面方式部署：便携式守护进程、一个配置文件，不需要 GUI 和 WebView2。

## 选择 Auto Dark Mode，如果

- 你的目标是 Windows 个性化：主题色、光标、任务栏、`.theme` 文件。
- 你需要太阳之外的调度 —— 夜间模式、环境光、电池或游戏条件、热键、自定义脚本。
- 你需要 ARM64、Microsoft Store 安装，或 DWALL 所提供的六种语言之外的语言。

## 各用一句话概括

Auto Dark Mode 覆盖 Windows 的更多部分，且可脚本化；DWALL 有意收窄到太阳驱动的壁纸
过渡，从而让常驻进程保持小巧。DWALL 同样会切换系统主题，但只依据太阳位置；如果你希望
按自己的计划切换 —— 固定时刻、夜灯、环境光传感器 —— 请使用 Auto Dark Mode 或 Windows
自带的计划功能。

## 资料来源

上文关于 Auto Dark Mode 的事实来自该项目自己的资料 —— 仓库与发布元数据
（github.com/AutoDarkMode/Windows-Auto-Night-Mode）、其 `docs/backend-flow.md`、
`PRIVACY.md`、关于已知限制与故障排查的 wiki 页面，以及其 Microsoft Store 页面。
DWALL 的数字是从本仓库的 release 构建实测的；方法见
[technical-comparison.zh-CN.md](./technical-comparison.zh-CN.md)。
