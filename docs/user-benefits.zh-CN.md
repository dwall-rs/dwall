# 哪款工具更适合

功能事实见 [shared-comparison-content.zh-CN.md](./shared-comparison-content.zh-CN.md)；
正面对比见 [DWALL 对比 Auto Dark Mode](./dwall-vs-auto-dark-mode.zh-CN.md) 与
[DWALL 对比 WinDynamicDesktop](./dwall-vs-windynamicdesktop.zh-CN.md)。本页只从成本与
收益的角度看 DWALL：它给你什么，又要求你付出什么。

## DWALL 给你什么

- **跟随太阳而非时钟的壁纸。** 应用的是太阳高度角与方位角最贴近当前位置的那一帧，
  因此无论季节或纬度，过渡都落在主题作者设定的位置。
- **低到可以忽略的常驻开销。** 一个单线程守护进程，约 4 MB 私有内存，没有通知区域
  图标；设置窗口关闭时既无 webview 也无 HTTP 栈。
- **用同一套计算切换系统浅色/深色主题**，任务栏与应用主题因此随壁纸变化，而不是在固定
  时刻变化。
- **每显示器主题**、镜像壁纸主题与帧的锁屏，以及可选的每日随机主题（从你选定的主题池中
  抽取）。
- **可预期的部署。** 一份 `config.toml`，一个无需管理员权限的 NSIS 安装程序，一个供
  无界面机器使用的 0.35 MB 守护进程压缩包（0.6 MB 可执行文件），以及供脚本与计划任务
  使用的 CLI（`--check-config`、`--once`、`--install-autostart`）。
- **无遥测，无账号。** 守护进程完全不发网络请求；主题与缩略图下载在设置窗口中进行，
  并会遵循镜像模板或 SOCKS5 代理，以应对无法访问 GitHub 的网络。

## 代价是什么

- **仅限 x64 Windows 10/11。** 不支持 ARM64、32 位 Windows 和 Windows 7。
- **六种界面语言**（英语、简体中文、繁体中文、香港中文、日语、韩语），而两个替代方案
  分别为约 55 种和 31 种。
- **没有脚本，也没有自动化钩子。** 如果你需要“游戏启动时切换”“遵循环境光传感器”或
  “随后运行这个脚本”，那是 Auto Dark Mode 的地盘。
- **它更改的是系统主题，而不是其余个性化设置。** 主题色、鼠标光标和 `.theme` 文件都
  不受影响。
- **主题库较小。** 以下载形式提供十五套太阳对齐套装（每套 5 到 24 帧），你也可以用
  [Dwall Maker](https://github.com/dwall-rs/maker) 添加自己的主题；拥有数百个现成主题的
  社区库属于 WinDynamicDesktop，而不是这里。
- **没有 Microsoft Store 页面。** 更新来自应用内更新器，或在这些清单被合并后来自
  winget 与 Scoop，也可以自己替换守护进程压缩包。
- **每个会话只能有一个守护进程。** 启动第二个副本会被拒绝，而不是并行运行；这是有意
  为之，但也意味着便携副本无法遮蔽已安装的副本。

## 如果你的优先项是……

| 优先项 | 更合适 | 原因 |
| --- | --- | --- |
| 以尽可能低的后台开销获得太阳精确的壁纸过渡 | DWALL | 帧的选择即太阳角度匹配；约 4 MB 守护进程，无托盘。 |
| 充分发挥 Windows 深色模式的潜力（主题色、光标、任务栏、条件、脚本） | [Auto Dark Mode](./dwall-vs-auto-dark-mode.zh-CN.md) | 这些正是它的功能集，而不是副作用。 |
| 在任何 Windows 版本或架构上使用大型现成主题库与 `.ddw` 文件 | [WinDynamicDesktop](./dwall-vs-windynamicdesktop.zh-CN.md) | 社区库、ARM64/x86 支持、商店安装、脚本能力。 |
| 依据配置文件进行无界面或受管部署 | DWALL | 便携式守护进程压缩包、CLI，守护进程不需要 WebView2。 |
| 除了壁纸什么都不碰 | [WinDynamicDesktop](./dwall-vs-windynamicdesktop.zh-CN.md) | 它已完全不切换系统主题。 |

DWALL 可以与任一替代方案并存 —— 它们可以并行安装，且触及的设置不同 —— 但两者都会设置
壁纸，所以这方面请只选其一。
