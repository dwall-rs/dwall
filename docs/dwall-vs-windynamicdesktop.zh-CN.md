# DWALL 对比 WinDynamicDesktop

两者都把 macOS 风格的动态壁纸带到 Windows：一个位置、日出与日落，以及一张随一天推移而
变化的壁纸。区别在于图片如何选择、应用还会更改什么，以及保持运行需要多少开销。

## 一览

|            | DWALL                                                       | WinDynamicDesktop                                                                     |
| ---------- | ----------------------------------------------------------- | ------------------------------------------------------------------------------------- |
| 图片选择 | 太阳高度角/方位角与当前位置最接近的那一帧（每套主题 5–24 帧，共提供 15 套主题） | 昼夜图片，外加可选的日出/日落列表，按日出/日落时间或固定的 24 小时计划轮换 |
| 系统浅色/深色主题 | 由太阳驱动切换 | 不切换 —— v4 中已移除，现由社区脚本实现 |
| 位置 | Windows 地理定位器，或你输入的坐标；使用手动坐标时可离线工作 | 地点名称通过 LocationIQ API 地理编码，因此输入地点需要联网 |
| 主题来源 | 精选的太阳对齐套装（`dwall-rs/dwall-assets`），加上你自己的目录 | 内置约 19 个 macOS 主题、一个社区站点（427 个免费社区主题，部分付费）、`.ddw` 导入与主题制作工具 |
| 主题格式 | `theme.toml` + 每帧的图片（[Dwall Maker](https://github.com/dwall-rs/maker) 规格） | ZIP/`.ddw` 中的 `theme.json` + 编号图片 |
| 锁屏 | 是 | 是 |
| 每显示器主题 | 是 | 是 |
| 自动化 | 配置文件变更后守护进程会重新读取，脚本改写它即可生效、无需重启；另有 CLI（`--once`、`--check-config`、`--install-autostart`）供脚本与计划任务调用；应用内不含脚本引擎 | PowerShell 脚本、CLI 参数（`/theme:<N>=<id>`） |
| 架构 | 极小的守护进程（常驻）+ 设置窗口（按需），配置文件是唯一接口 | 单个 WinForms 进程，同时承载托盘、对话框与调度器 |
| 常驻开销 | 守护进程私有工作集约 2 MB（私有字节 3–8 MB）；0.35 MB 压缩包（0.63 MB 可执行文件）；无托盘图标 | 项目未有文档说明；自包含的约 70 MB 可执行文件；托盘图标 |
| CPU 架构 | x64 | x86、x64、ARM64 |
| 最低系统要求 | Windows 10/11 | Windows 7+（面向 Windows 11 开发） |
| 语言 | 6 | 提供约 55 种供翻译 |
| 分发 | GitHub releases（安装程序 + 守护进程压缩包） | GitHub releases、Microsoft Store（免费）、winget、Scoop、Chocolatey |
| 许可证 | AGPL-3.0-only | MPL-2.0 |

## 选择 DWALL，如果

- 你希望帧由太阳的实际位置决定，而不是由昼夜计划决定。
- 常驻进程应保持在几 MB，且没有通知区域图标。
- 你处于离线或受限网络环境：位置可手动输入，主题下载支持镜像模板与 SOCKS5。
- 你依据配置文件进行无界面部署。

## 选择 WinDynamicDesktop，如果

- 你想要最大的现成主题库，以及来自别处的 `.ddw` 文件。
- 你需要 Windows 7、32 位 Windows 或 ARM64。
- 你想要 Microsoft Store 安装，或对运行中的应用进行脚本与 CLI 控制。
- 你不需要由它替你切换系统浅色/深色主题。

## 各用一句话概括

WinDynamicDesktop 拥有更大的生态、更广的平台支持和脚本语言；DWALL 逐帧把图片与太阳
位置匹配，用同一套计算切换应用与任务栏两个浅色/深色主题值，其守护进程是 0.63 MB，而
WinDynamicDesktop 可执行文件为 62–72 MB —— 后者未公布内存占用，因此这里不做内存比较。

## 资料来源

关于 WinDynamicDesktop 的事实来自该项目自己的资料 —— 仓库与发布元数据、`README.md`、
其 wiki（常见问题、自定义主题、已知问题）、`src/JsonConfig.cs`、安装脚本，以及其
Microsoft Store 页面。DWALL 的数字是从本仓库的 release 构建实测的；方法见
[technical-comparison.zh-CN.md](./technical-comparison.zh-CN.md)，完整对照表见
[shared-comparison-content.zh-CN.md](./shared-comparison-content.zh-CN.md)。
