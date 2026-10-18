# 技术比较

三个项目由什么构成，以及由此带来的结果。功能层面的事实见
[shared-comparison-content.zh-CN.md](./shared-comparison-content.zh-CN.md)；本文档讨论
结构与机制。这里不发布任何对比基准测试，因为三个项目都没有发布，而本仓库也不会臆造。

## 进程模型

**DWALL** —— 两个从不直接通信的可执行文件：

```
dwall.exe        守护进程：读取 config.toml，计算太阳位置，设置壁纸、
                 浅色/深色主题与锁屏。单线程、无异步运行时、未链接 GUI
                 工具包、无网络代码。持有每会话命名互斥体。
dwall-settings   Tauri 2 + SolidJS 窗口，打开时启动：主题目录与下载、
                 更新器、缩略图。
        └──────── config.toml（唯一接口；窗口轮询状态并读取日志尾部） ────────┘
```

这样拆分是为了让常驻进程保持小巧：窗口的工具包、webview 和 HTTP 栈只在窗口打开时占用
内存。守护进程持有互斥体 `Local\com.thep0y.dwall.daemon`，窗口也据此检测并停止它。

**WinDynamicDesktop** —— 一个 WinForms 进程，同时是托盘图标、对话框与调度器；设置存放
在 `settings.json`。

**Auto Dark Mode** —— 三个可执行文件：带托盘图标与命名管道服务器的每会话 .NET 引擎、
按需打开的 WinUI 3 窗口，以及通过管道发送命令的控制台外壳；自启动由任务计划程序的登录
任务完成。

## 塑造行为的依赖

| | DWALL 守护进程 | DWALL 设置程序 | WinDynamicDesktop | Auto Dark Mode |
| --- | --- | --- | --- | --- |
| 运行时 | 除操作系统外无（Rust，静态链接） | WebView2 | .NET 10，自包含（约 70 MB） | .NET 9 |
| Windows API | 壁纸 COM、注册表、WinRT 地理定位、Win32 控制台 | —— | Win32/WinForms、通过 HTTPS 访问 LocationIQ | Win32/WinUI、Windows 位置服务 |
| 网络 | 无 | 主题/缩略图下载、更新器 | 主题下载、地理编码、更新检查 | 更新检查 |

DWALL 守护进程有意不链接异步运行时：WinRT 操作通过一个有界 50 ms 轮询辅助而不是执行器
来等待，因此卡住的平台调用不会让进程永远挂起。

## 文件格式

| | DWALL | WinDynamicDesktop | Auto Dark Mode |
| --- | --- | --- | --- |
| 配置 | `config.toml`（`%APPDATA%\dwall\config.toml`），首次运行时写入带注释的默认配置 | `settings.json` | 注册表与自己的存储 |
| 主题 | 包含 `theme.toml`（`schema = 1`）、`images/<n>.<ext>`、可选 `thumbnails/` 的目录；权威规格见 [Dwall Maker](https://github.com/dwall-rs/maker) 仓库 | 含 `theme.json` 与编号图片的 ZIP/`.ddw` | 每个主题的图片文件；`.theme` 文件用于解析其他个性化设置 |
| 随机状态 | `random_selection.json`，与所选配置文件同目录 | `settings.json` 中的 shuffle 设置 | —— |
| 日志 | `%LOCALAPPDATA%\com.thep0y.dwall\log\dwall.log`，每次启动轮转到 `backup\` | 调试日志开关 | 自己的日志文件 |

由于配置文件就是接口，它也是公开约定：`--config` 让多份互不相干的配置并存，
`--check-config` 则在不应用任何更改的情况下校验其中一份。

## 分发与更新

| | DWALL | WinDynamicDesktop | Auto Dark Mode |
| --- | --- | --- | --- |
| 安装程序 | NSIS，按用户安装，2.0 MB（0.2.5），无需管理员权限 | Inno Setup，按用户或按机器安装，62–72 MB | 安装程序，按用户安装 |
| 便携 / 无界面 | `Dwall_<version>_x64-daemon.zip`（0.35 MB），完全不需要 GUI 即可使用 | `*_Portable.exe`（同一个托盘应用） | 便携 ZIP（同一个引擎） |
| 更新 | 应用内更新器原子替换按用户安装；`latest.json` 指向 NSIS 包 | 每周检查并通知，打开发布页面；商店版通过商店更新 | 应用内更新器，项目说明其不覆盖系统级安装 |
| 商店 | 未上架 | 免费 | 免费 |

守护进程压缩包是一等产物，而不是精简副本：它与安装程序打包的是同一个可执行文件，发布时
附带校验和，因此无需安装任何东西就能准备一台无界面机器。

## DWALL 的数字是如何得到的

- **可执行文件与压缩包大小** —— release 构建
  （`bun run build:daemon`、`cargo build -p dwall --release --features build-script,log-max-level-info`）
  以及生成的压缩包的文件大小。
- **常驻内存** —— 操作系统报告的运行中守护进程的私有字节，由 release 构建配合
  `.scripts/measure-daemon-memory.ps1` 实测。这个数字取决于**进程做过什么**，而不是
  运行了多久：第一次真正更换壁纸之前约 3 MB 私有字节、约 270 个句柄；一旦走过 WinRT
  的壁纸与锁屏路径，就稳定在约 7.5 MB、约 430 个句柄。一台连续运行十三小时、期间更换
  过十次壁纸的机器实测 7.5 MB / 427 个句柄——与五分钟内更换两次的运行处于同一水平，
  说明这部分开销是一次性的，而不是每次更换都增加。工作集更大（23–42 MB），因为它计入
  守护进程并不拥有的共享系统 DLL；对于“这要花费我多少”，私有字节才是有意义的数字。
  单次采样两个方向都会误导：COM/WinRT 机制启动时进程会短暂持有十几个线程，之后回落到
  4–7 个。
- **CPU** —— 不予给出：守护进程每个间隔（默认 15 秒）计算一次，其间休眠，因此任何数字
  都完全取决于采样方式。

另外两个项目的每个数字都来自它们自己公布的材料，而“未有文档说明”就是字面意思。
