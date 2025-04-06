<p align="center">
<img height="64" width="64" src="./src-tauri/icons/128x128.png" />
<br/>
<a href="https://github.com/dwall-rs/dwall/releases/latest"><img src="https://img.shields.io/github/downloads/dwall-rs/dwall/total.svg?style=flat-square" alt="GitHub releases"></a>
</p>

# DWALL

DWALL 是一款轻量级应用程序，旨在模拟 macOS 壁纸切换行为，通过基于太阳高度角和方位角动态更改 Windows 系统上的桌面背景。体验随着一天中太阳自然移动而无缝过渡的壁纸变化。

## 动机

面对现有的解决方案如 [WinDynamicDesktop](https://github.com/t1m0thyj/WinDynamicDesktop) 和 [AutoDarkMode](https://github.com/AutoDarkMode/Windows-Auto-Night-Mode)，您可能会疑惑为何需要另一个工具。虽然这些应用程序提供类似的功能，但它们作为独立项目运行，没有整合计划。此外，它们的资源消耗可能高于此类任务的理想水平。

认识到这一差距，DWALL 被开发为一种高效的替代方案。它可以根据太阳位置自动切换明暗主题并更改壁纸，同时在运行期间保持最小的内存使用量。对于寻求资源占用更少选项的用户，DWALL 提供了一个极具吸引力的选择。

## 特性

- **基于太阳的调度：** 自动根据太阳路径调整壁纸。
- **低资源占用：** 针对性能进行优化，对系统资源的影响最小。
- **无缝集成：** 轻松融入您的工作流程，没有侵入性通知或设置。

## 用前须知

本项目仍然处于开发阶段，可能存在一些问题。如果您在使用过程中遇到任何问题，请随时在 GitHub 页面上提出问题。

## 使用步骤

1. 在 [releases](https://github.com/dwall-rs/dwall/releases/latest) 中下载最新的 DWALL 可执行文件。
2. 运行 DWALL 可执行文件。
3. 允许 DWALL 访问您的位置信息，或者在设置页面手动设置您的位置。
4. 在侧边菜单中点击你想使用的壁纸，如果未下载，需要先点击“下载”按钮下载这套壁纸，如果下载完成则可以点击“应用”按钮使用这套壁纸。
5. 如果你使用多个显示器，可以在显示器选择器中选择你想单独配置的显示器，然后重复第 4 步的操作。

## 截图

以下是展示 DWALL 运行效果的两张截图：

![主页](images/home.avif)

![设置](images/settings.avif)

## 运行日志

DWALL 的日志等级默认为`warning`，您可以通过设置环境变量`DWALL_LOG`来调整日志等级，如`DWALL_LOG=info`，需要注意的是，release 版本不会输出`info`以下的日志。

## 常见问题

### 1. 为什么设置和守护进程完全隔离？

最初的版本中，设置和守护进程是在同一个进程中运行的，同时支持通过任务栏托盘管理，但这会使得进程的内存占用较大（相比于其他同类型程序仍然要小很多），这不是我想要的结果，所以我设置和守护进程完全隔离，这样可以减少守护进程的内存占用，同时也使得进程的管理更加简单。

当然，这也带来了进程间通信的问题，目前的版本，设置进程（图形化程序）和守护进程没有实现进程间通信，二者交流的唯一途径是配置文件，这就会导致当守护进程异常退出时，设置进程无法及时获取守护进程的状态，这是一个需要解决的问题。

### 2. 壁纸下载失败。

壁纸资源保存在 github 中，但一些国家和地区因为网络管制无法正常访问 github，如果不设置 Github 镜像模板，就会导致壁纸下载失败。你需要通过搜索引擎自行搜索可用的 Github 镜像站或加速站。假设你搜索到了一个可用的 github 加速站`https://ghproxy.cc`，可在设置页面中如下配置 Github 镜像模板：

```
https://ghproxy.cc/https://github.com/<owner>/<repo>/releases/download/<version>/<asset>
```

如果你搜索到到的是一个 github 镜镜站`https://kkgithub.com/`，则需要如下配置 Github 镜像模板：

```
https://kkgithub.com/<owner>/<repo>/releases/download/<version>/<asset>
```

然后就可以正常下载壁纸了。

### 3. 为什么不支持自定义壁纸？

因为壁纸需要与太阳位置相关联，而大部分用户没有相关的天文学知识，无法将壁纸与太阳位置完美匹配，因此，我们提供一些已经与太阳位置完美关联的壁纸，用户可以根据自己的喜好选择。

### 4. 我能设计与太阳位置完美匹配的成套壁纸，如何添加到 DWALL 中？

你可以将制作好的壁纸上传到网盘当中，创建一个 issue 并说明设计思路，我们会在合适的时机将你的壁纸添加到 DWALL 中。

### 5. 缩略图为什么加载失败？

缩略图使用的是 Github 直链，部分地区对 Github 有网络管制，导致无法加载，你可以通过设置 Github 镜像模板来解决这个问题。

### 6. 为什么不使用托盘管理后台进程？

与第 1 个问题相同，托盘进程本身要比后台进程的内存占用还要大，考虑到这一点，我决定不使用托盘管理后台进程。

---

我们欢迎社区的贡献，以帮助改进 DWALL。如果您遇到任何问题或对新功能有建议，请随时在我们的 GitHub 页面上提出问题或提交拉取请求。
