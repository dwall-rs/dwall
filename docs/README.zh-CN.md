# 文档

| 文档 | 回答的问题 |
| --- | --- |
| [../README.zh-CN.md](../README.zh-CN.md) | DWALL 是什么，如何安装与使用。 |
| [headless.md](./headless.md) | 不依赖设置窗口运行 `dwall.exe`：配置、命令、自启动、日志。 |
| [custom-themes.md](./custom-themes.md) | 主题布局、清单，以及 Dwall 实现了哪些部分。 |
| [shared-comparison-content.zh-CN.md](./shared-comparison-content.zh-CN.md) | DWALL、WinDynamicDesktop 与 Auto Dark Mode 的已核实规格与功能对照表。 |
| [dwall-vs-auto-dark-mode.zh-CN.md](./dwall-vs-auto-dark-mode.zh-CN.md) | DWALL 对比 Auto Dark Mode。 |
| [dwall-vs-windynamicdesktop.zh-CN.md](./dwall-vs-windynamicdesktop.zh-CN.md) | DWALL 对比 WinDynamicDesktop。 |
| [technical-comparison.zh-CN.md](./technical-comparison.zh-CN.md) | 进程模型、依赖、文件格式、分发与更新路径。 |
| [user-benefits.zh-CN.md](./user-benefits.zh-CN.md) | 哪款工具适合哪类用户，以及 DWALL 要求你付出什么。 |

比较文档集中的中文译本以 `*.zh-CN.md` 的形式与各文件并列存放。

## 本目录的写作规则

- **先核实，再落笔。** 关于其他项目的每条陈述都必须来自该项目自己的仓库、文档、发布
  元数据或商店页面，并附上链接。不要评价其内部实现、性能或代码质量。
- **测量 DWALL。** DWALL 的数字来自本仓库的 release 构建
  （`cargo build -p dwall --release`、任务管理器私有字节、文件大小）—— 绝不凭直觉。
  说明数字是如何得到的。
- **不下结论，不做营销。** 说明每个工具做什么、各自更适合什么场景，包括 DWALL 落后
  的地方。
- **一个事实只写一处。** 共享规格放在
  [shared-comparison-content.zh-CN.md](./shared-comparison-content.zh-CN.md)；其他文档
  链接过去，而不是重复它。
