# 共享参考：DWALL、WinDynamicDesktop、Auto Dark Mode

三个项目的已核实事实集中于此。比较文档链接到这里，而不是重复它们。

关于其他项目的每条陈述都来自该项目自己的仓库、文档、发布元数据或商店页面；DWALL 的数字
是从本仓库的 release 构建实测的（`bun run build:daemon`；大小取自文件，内存取自运行中
进程：任务管理器显示的私有工作集，以及提交的私有字节）。此处没有任何内容是根据项目的
语言或架构估算或推断的。

## 功能

| | DWALL | WinDynamicDesktop | Auto Dark Mode |
| --- | --- | --- | --- |
| 用途 | 依据太阳位置选择壁纸帧 | macOS Mojave 动态桌面的 Windows 移植 | 按计划切换 Windows 深色/浅色主题 |
| 触发变化的依据 | 本地计算的太阳高度角与方位角 | 按位置计算的日出/日落时间，或固定的 24 小时计划 | 固定时间、日出日落时间、夜间模式、环境光、热键、脚本 |
| 系统浅色/深色主题 | 应用与系统（任务栏）两个值都依据太阳位置切换；配置中可关闭 | 不切换（v4 中已移除；由社区脚本实现） | 切换 —— 其核心功能 |
| 锁屏 | 是，可选 | 是（`lockScreenTheme`） | 未有锁屏功能的文档 |
| 每显示器壁纸 | 是 —— 每台显示器可指定主题；未指定条目的显示器不会得到壁纸 | 是（按显示器配置 `activeThemes`） | 是（每个主题可给每台显示器不同图片） |
| 随机 / 每日变化 | 每天从主题池中随机选一个主题 | 主题内打乱 | 壁纸幻灯片推进 |
| 虚拟桌面 | 所有虚拟桌面共用一套壁纸：Windows 会把桌面的壁纸套用到每个虚拟桌面 | 已有文档说明的限制：所有桌面共用一套图片 | 已有文档说明的限制：单显示器配置下 |
| 可扩展性 | 有文档的主题格式（`theme.toml`、[Dwall Maker](https://github.com/dwall-rs/maker)）与自定义主题目录；配置文件变更后守护进程会重新读取，因此无需重启即可被外部驱动；不含脚本引擎 | PowerShell 脚本、CLI 参数 | 自定义脚本、热键、条件 |
| CLI | `dwall.exe --check-config`、`--once`、`--install-autostart`、`--uninstall-autostart`、`--config`、`--help`、`--version` | `WinDynamicDesktop.exe /theme:<N>=<id>`、`/theme:<auto\|dark\|light>` | `AutoDarkModeShell.exe`，通过引擎的命名管道 |

## 架构与技术栈

| | DWALL | WinDynamicDesktop | Auto Dark Mode |
| --- | --- | --- | --- |
| 进程 | Rust 守护进程（常驻）+ Tauri/SolidJS 设置窗口（按需） | 单个 WinForms 进程：托盘、对话框与调度器合一 | 带托盘与命名管道服务器的 .NET 引擎（每会话）+ WinUI 3 窗口 + 控制台 CLI |
| 引擎 / GUI 分离 | 是；无 IPC，配置文件即接口 | 否 | 是，通过命名管道 |
| 语言 / 框架 | Rust；SolidJS + Tauri 2 | C# / .NET 10，WinForms | C# / .NET 9，WinUI 3 + WinForms |
| 后台运行时开销 | 单线程，无异步运行时，未链接 GUI 工具包，无网络代码 | .NET 运行时 + WinForms | .NET 运行时 + WinForms |
| 自启动 | `HKCU\...\Run` 值，或你自行创建的计划任务 | 由安装程序提供 | 任务计划程序的登录任务 |
| 配置 | `config.toml` | `settings.json` | 注册表 + 自己的设置存储 |
| 许可证 | AGPL-3.0-only | MPL-2.0 | GPL-3.0 |
| CPU 架构 | x64 | x86、x64、ARM64 | x64、ARM64 |
| 界面语言 | 6 种（en、zh-CN、zh-HK、zh-TW、ja、ko） | 提供约 55 种供翻译；以编译后的 `.mo` 文件发布 | 31 |
| 最低系统要求 | Windows 10/11（x64） | Windows 7+（面向 Windows 11 开发）；商店版需 Windows 10 19041 或更高 | Windows 10 22H2 或 Windows 11 |

## 占用（DWALL 为实测，其他为官方公布）

| | DWALL | WinDynamicDesktop | Auto Dark Mode |
| --- | --- | --- | --- |
| 常驻私有内存 | 私有工作集约 2 MB —— 即任务管理器「内存」列显示的值 —— 私有字节在 3–8 MB 之间，取决于是否走过 WinRT 壁纸与锁屏路径（方法见 [technical-comparison.md](./technical-comparison.md)） | 未有文档说明 | 未有文档说明 |
| 安装程序 / 压缩包 | 2.0 MB NSIS 安装程序（0.2.5）；0.35 MB 守护进程压缩包 | 每种架构 62–72 MB（自包含） | 未有文档说明 |
| 主可执行文件 | 0.63 MB 守护进程，4.1 MB 设置程序 | 单个约 70 MB 的可执行文件 | 未有文档说明 |
| 通知区域图标 | 无，出于设计 | 有（应用的操作入口） | 有 |

两个竞品都没有公布 CPU 数据；本文档不臆造任何数字。

## 分发与更新

| | DWALL | WinDynamicDesktop | Auto Dark Mode |
| --- | --- | --- | --- |
| GitHub releases | 安装程序 + 带 `.sha256` 的守护进程压缩包 | 每种架构的安装包与便携版，附校验和 | 每种架构的安装程序与便携版，附 `.sha256` |
| 微软商店 | 无 | 有，免费 | 有，免费 |
| winget | 清单已备好，未提交 | `t1m0thyj.WinDynamicDesktop` | `ArminOsaj.AutoDarkMode` |
| Scoop / Chocolatey | 清单已备好，未提交 | Scoop Extras、Chocolatey | 非官方条目，项目已如此标注 |
| 更新途径 | 应用内更新器（原子替换按用户安装） | 每周检查并通知，打开发布页面；商店版通过商店更新 | 内置更新器；项目说明其不支持系统级安装 |

## 网络与隐私

| | DWALL | WinDynamicDesktop | Auto Dark Mode |
| --- | --- | --- | --- |
| 遥测 | 代码中无（`TELEMETRY_ENABLED = false`） | 无文档说明，源码中也不存在 | 无；项目声明设置保持离线 |
| 位置 | Windows 地理定位器，或你输入的坐标 | 本地输入的地点名称通过 LocationIQ API 地理编码；商店版使用 Windows 位置 API | 为计算日出日落时间使用 Windows 位置服务，或使用固定时间 |
| 其他网络请求 | 从 GitHub 下载主题与缩略图，支持镜像模板与 SOCKS5；守护进程自身不发任何请求 | 从 GitHub/GitLab/Bitbucket 下载内置的 macOS 主题；每周检查更新 | 向 GitHub 检查更新 |

## 主题

| | DWALL | WinDynamicDesktop | Auto Dark Mode |
| --- | --- | --- | --- |
| 格式 | 包含 `theme.toml`（`schema = 1`）、图片、可选缩略图的目录；规格由 [Dwall Maker](https://github.com/dwall-rs/maker) 维护 | 包含 `theme.json` 与编号图片的 ZIP/`.ddw`；已有文档 | 每个主题使用普通图片文件；`.theme` 文件仅用于解析其他设置 |
| 每个主题的帧数 | 15 套主题目录共 5–24 帧，每个受监控的太阳位置一帧 | 昼夜两张，外加可选的日出/日落列表 | 每台显示器每个主题一张图片 |
| 来源 | 来自 `dwall-rs/dwall-assets` 的精选套装 | 内置的 macOS 主题（约 19 个），外加提供免费与付费主题的社区站点 | 你自己的图片，或 `.theme` 文件 |
| 自制主题 | 支持（复制到 `customized_themes_directory`） | 支持，有文档化的格式与主题制作工具 | 不是主题包格式；直接选择壁纸 |

## 使用场景

| 场景 | 更合适 | 原因 |
| --- | --- | --- |
| 每个主题通过 5–24 帧跟随太阳的壁纸 | DWALL | 帧的选择即太阳角度匹配 |
| Windows 深色模式外加主题色、光标、任务栏 | Auto Dark Mode | DWALL 只切换应用/系统主题值 |
| 拥有大型社区库的 macOS 风格动态壁纸 | WinDynamicDesktop | 社区目录（427 个免费社区主题）与 `.ddw` 导入 |
| 脚本化自动化与条件（游戏、电池、环境光） | Auto Dark Mode | 脚本引擎与条件模块 |
| 尽可能小的常驻进程，无托盘图标 | DWALL | 私有工作集约 2 MB（私有字节 3–8 MB），无通知区域图标 |
| 依据配置文件进行无界面或受管部署 | DWALL | 便携式守护进程、CLI，守护进程不需要 WebView2 |
| ARM64 设备 | WinDynamicDesktop、Auto Dark Mode | DWALL 仅提供 x64 |
| 六种语言之外的非英语界面 | WinDynamicDesktop 或 Auto Dark Mode | 提供约 55 种 / 实装 31 种 |
