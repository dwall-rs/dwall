# DWALL

DWALL 是一款轻量级应用程序，旨在模拟 macOS 壁纸切换行为，通过基于太阳高度角和方位角动态更改 Windows 系统上的桌面背景。体验随着一天中太阳自然移动而无缝过渡的壁纸变化。

## 动机

面对现有的解决方案如 [WinDynamicDesktop](https://github.com/t1m0thyj/WinDynamicDesktop) 和 [AutoDarkMode](https://github.com/AutoDarkMode/Windows-Auto-Night-Mode)，您可能会疑惑为何需要另一个工具。虽然这些应用程序提供类似的功能，但它们作为独立项目运行，没有整合计划。此外，它们的资源消耗可能高于此类任务的理想水平。

认识到这一差距，DWALL 被开发为一种高效的替代方案。它可以根据太阳位置自动切换明暗主题并更改壁纸，同时在运行期间保持最小的内存使用量。对于寻求资源占用更少选项的用户，DWALL 提供了一个极具吸引力的选择。

## 特性

- **基于太阳的调度：** 自动根据太阳路径调整壁纸。
- **低资源占用：** 针对性能进行优化，对系统资源的影响最小。
- **无缝集成：** 轻松融入您的工作流程，没有侵入性通知或设置。

## 截图

以下是展示 DWALL 运行效果的两张截图：

![主页](images/home.avif)

![设置](images/settings.avif)

## 运行日志

DWALL 的日志等级默认为`warning`，您可以通过设置环境变量`DWALL_LOG`来调整日志等级，如`DWALL_LOG=info`，需要注意的是，release 版本不会输出`info`以下的日志。

---

我们欢迎社区的贡献，以帮助改进 DWALL。如果您遇到任何问题或对新功能有建议，请随时在我们的 GitHub 页面上提出问题或提交拉取请求。
