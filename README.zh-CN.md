<p align="center">
<img height="64" width="64" src="./crates/dwall-settings/icons/128x128.png" />
<br/>
<a href="https://github.com/dwall-rs/dwall/releases/latest"><img src="https://img.shields.io/github/downloads/dwall-rs/dwall/total.svg?style=flat-square" alt="GitHub releases"/></a>
<img src="https://img.shields.io/badge/Windows%2010-%230079d5.svg?style=for-the-badge&logo=Windows%2010&logoColor=white)" alt="Windows 10" height="20" />
<img src="https://img.shields.io/badge/Windows%2011-%230079d5.svg?style=for-the-badge&logo=Windows%2011&logoColor=white)" alt="Windows 11" height="20" />
<br/>
<a href='./README.md'>English</a> | 简体中文
</p>

# DWALL

一款 Windows 动态壁纸应用，跟随的是**太阳位置**而非时钟：它在本地计算太阳高度角与方位角，应用与之匹配的主题帧，并让系统浅色/深色主题与锁屏随之切换（主题切换默认开启，可关闭）。常驻部分是一个持有 3–8 MB 私有内存的单线程 Rust 守护进程（可执行文件本身 0.63 MB）；设置窗口是独立进程，只在打开时运行。

## 与其他方案对比

| | DWALL | [WinDynamicDesktop](https://github.com/t1m0thyj/WinDynamicDesktop) | [Auto Dark Mode](https://github.com/AutoDarkMode/Windows-Auto-Night-Mode) |
| --- | --- | --- | --- |
| 帧由何决定 | 太阳高度角/方位角匹配 | 昼夜与日出/日落时间 | ——（主题切换） |
| 系统浅色/深色主题 | 依据太阳位置切换 | 不切换 | 切换 —— 其核心功能 |
| 锁屏 | 是 | 是 | 未有文档说明 |
| 每显示器主题 | 是 | 是 | 是 |
| 脚本 / 自动化 | 无 | PowerShell 脚本、CLI | 脚本、条件、热键 |
| 常驻开销 | 3–8 MB 守护进程，无托盘图标 | 未有文档说明；约 70 MB 应用带托盘图标 | 未有文档说明；托盘图标 |
| 架构 / 语言 | x64 / 6 | x86、x64、ARM64 / 提供约 55 种 | x64、ARM64 / 31 |
| 分发 | GitHub releases | GitHub、Microsoft Store、winget、Scoop、Chocolatey | GitHub、Microsoft Store、winget |
| 许可证 | AGPL-3.0-only | MPL-2.0 | GPL-3.0 |

上表每条事实均来自其他项目自身的资料；完整对照表，以及它们更合适的情形，见 [docs/](./docs/README.zh-CN.md)。

## 特性

- **基于太阳的调度** —— 帧与太阳位置匹配；15 套主题目录中每套使用 5 到 24 帧。
- **同步系统主题** —— 浅色/深色主题依据同一计算切换（默认开启，开关在设置页）。
- **锁屏** —— 可选地镜像壁纸的主题与帧。
- **每显示器主题** —— 每台显示器可用不同主题，显示器变化时重新应用。
- **每日随机主题** —— 可选地每天从你选定的主题池中抽取一个主题。
- **可调刷新频率** —— 每 1 秒到 1 小时评估一次（默认 15 秒）。
- **自定义主题** —— 一个包含 `theme.toml` 与图片的目录；见 [docs/custom-themes.md](./docs/custom-themes.md)。
- **无界面模式** —— 同一个可执行文件可依据配置文件运行，并提供 CLI；见 [docs/headless.md](./docs/headless.md)。
- **受限网络** —— 主题下载支持镜像模板与 SOCKS5。
- **六种语言，无遥测** —— 英语、简体中文、繁体中文、香港中文、日语、韩语。

## 系统要求

64 位 Windows 10 或 11。设置窗口需要 WebView2 运行时（当前 Windows 已自带，安装程序也会处理）；守护进程既不需要 WebView2，也不需要管理员权限。不支持精简版 Windows 镜像。

## 安装

1. 从 [latest release](https://github.com/dwall-rs/dwall/releases/latest) 下载安装程序（`Dwall.Settings_<version>_x64-setup.exe`）并运行 —— 它按用户安装，无需管理员权限。
2. 只要守护进程，或用于受管 / 无界面机器：请改用 `Dwall_<version>_x64-daemon.zip`，并按 [docs/headless.md](./docs/headless.md) 操作。

winget 与 Scoop 清单已备好，但尚未提交到这些仓库。

## 使用

1. 运行 DWALL 并允许位置访问，或在设置页面手动设置坐标。
2. 在侧边栏中选择一个主题。尚未下载的主题会显示**下载**按钮，且下载完成前无法选中；下载完成后用**应用**即可使用。
3. 使用多台显示器时，在显示器选择器中选中相应显示器，并对每台显示器重复第 2 步。
4. 可选地启用锁屏壁纸。
5. DWALL 会持续在后台运行 —— 没有托盘图标；设置窗口可从开始菜单再次启动。

## 截图

![主页](images/home.avif)

![设置](images/settings.avif)

## 运行日志

等级为 `info`；设置 `DWALL_LOG`（可直接写等级，如 `debug`，或写 `dwall=debug`）可更改。release 构建会把 `debug`/`trace` 语句编译掉，因此其中只可能出现 `info`、`warn` 和 `error`。守护进程写入 `%LOCALAPPDATA%\com.thep0y.dwall\log\dwall.log`，每次启动时把上一份文件轮转到 `backup\`；设置窗口可为你打开该目录。

## 常见问题

**为什么设置窗口和守护进程是两个独立进程？**
它们最初是一个带托盘图标的单进程，而目标是把后台开销控制在几 MB，于是 GUI 工具包、webview 和 HTTP 客户端都被移入一个只在打开时存在的窗口。代价是没有 IPC：二者共享 `config.toml`，窗口靠轮询获取守护进程状态，而不是被通知。同样出于这个原因，这里没有托盘图标 —— 托盘进程的开销比它要控制的守护进程还大。

**主题下载失败。** 壁纸集托管在 GitHub 上，部分网络无法访问。在设置页面中设置镜像模板，例如用前缀式代理 `https://ghproxy.cc/https://github.com/<owner>/<repo>/releases/download/<version>/<asset>`，或用替换主机名的镜像 `https://kkgithub.com/<owner>/<repo>/...`。缩略图来自同一主机，需要同样的设置。

**可以使用自己的壁纸吗？** 可以 —— 自定义主题是一个包含 `theme.toml` 清单（schema 1）及其图片的目录。见 [docs/custom-themes.md](./docs/custom-themes.md)。精选套装仍是最省事的选择，因为它们每一帧都已与太阳轨迹对齐。

**可以贡献一套壁纸吗？** 提交一个 issue 说明设计思路（以及图片的获取地址）；合适的套装会被加入精选目录。

## 参与贡献与许可证

欢迎提交 issue 和 pull request。DWALL 以 [AGPL-3.0-only](./LICENSE) 许可证发布；源代码位于 `crates/`（Rust）和 `src/`（前端），布局与约定见 [AGENTS.md](./AGENTS.md)。
