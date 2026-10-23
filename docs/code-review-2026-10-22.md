# Dwall 代码审查（2026-10-22）

审查范围：整个仓库（守护进程、设置后端、SolidJS 前端、日志与时间库、构建与发布链路）。
本报告只列**当前仍然存在**的问题；已修复项在第 0.3 节列出，避免与上一轮重复。

## 0. 方法与实测基线

四路并行评审（守护进程核心 / `dwall-settings` 后端 / 前端 / 测试覆盖），随后逐条交叉复核关键结论
（含 `solid-js` 资源访问器、`zip` 解压、镜像路由）。审查过程**未修改任何源文件**。

本轮实测（工作树 = HEAD `a7070d7`，`git status` 干净）：

| 项 | 结果 |
| --- | --- |
| `cargo clippy --all-targets -- -D warnings` | exit 0，0 警告 |
| `cargo clippy --release --all-targets -- -D warnings` | exit 0，0 警告 |
| `cargo test` | 480 passed / 0 failed（dwall lib 242 + bin 8 + 集成 13 + settings 126 + logging 8 + time 83） |
| `bun run check` | 干净，482 文件，exit 0 |
| `target/release/dwall.exe`（按发布特性重编） | 649,216 B（634 KiB） |
| `target/release/dwall-settings.exe` | 4,248,576 B（4.05 MiB） |
| `dist/` | 394,291 B：JS 247,052（gzip 77.3 KB）/ CSS 115,856（gzip 17.9 KB）/ SVG 30,781 / html 602 |
| `target/` 本机累计 | 约 82.7 GB（仅本机构建产物，非发布体积） |

### 0.3 上一轮（2026-10-17）已修复、本轮不再列为问题的项

守护进程失败退避 `daemon.rs:120`；`patch` 空配置守卫 `settings.store.ts:101`；设置错误面板与重试
`settings.store.ts:83`；`every_configured_monitor_failed` `theme_engine.rs:435`；显示器 provider 共享
`daemon.rs:169`；`has_configuration_changed` 节流 `monitor.rs:563`；大小写折叠的 `is_inside`
`filesystem.rs:205`；`cache_minutes` 上界 `config.rs:141`；日志备份上限 5 `logging/src/lib.rs:326`；
时区偏移 60 s 缓存 `time/src/offset.rs:44`；`get_last_daemon_error` 只读尾部 `daemon.rs:116`；
`asin` clamp `solar.rs:403`；`SimpleRng` 零种子 `theme.rs:521`。

## 1. 结论摘要

整体工程质量很高：纯函数核心（太阳 / 时间 / 配色）测试密度大、注释解释「为什么」、内存与体积确实被
当作一等约束。当前风险集中在三类：(a) 前端对 `createResource` 错误态无防护且全应用无 `ErrorBoundary`；
(b) 设置后端若干阻塞 / 无界 / 未校验路径；(c) 少数会静默改变用户数据的行为。

| # | 级别 | 位置 | 问题 |
| --- | --- | --- | --- |
| F1 | 高 | `src/stage/SunPathPanel.tsx:57`、`src/stage/FixedStage.tsx:52`、`src/layout/EngineStatus.tsx:42` | `createResource` 错误态访问会 `throw`，且无 `ErrorBoundary`；位置权限被拒 / 主题文件损坏时整棵 UI 树崩溃 |
| F2 | 高 | `crates/dwall-settings/src/network/mirror.rs:46` | 稳定版更新检查先打 `api.github.com`，失败即回落 `github.com`——配置的镜像在最常见的更新检查里完全失效 |
| F3 | 高 | `crates/dwall-settings/src/theme/applier.rs:31` | `stop_daemon()` 在 async 上下文直接调用，阻塞 tokio worker 最多 5 s |
| F4 | 高 | `crates/dwall-settings/src/commands/theme.rs:84` | 一个自定义主题缺 `images/` 目录会让整个自定义主题列表失败 |
| F5 | 高 | `src/store/settings.store.ts:222` | 「应用主题 / 保存随机池」用草稿配置整体写盘并标记为已保存，连带提交用户未保存的其它设置 |
| F6 | 高 | `src/library/LibraryColumn.tsx:17` + `src/models/themes.ts:18` | 搜索每次按键都重建全部 `ThemeTile` 并重新发起每张缩略图的 IPC |
| F7 | 高 | `src/store/download.store.ts:60` | 进度监听在 `try` 之外 await，注册失败则状态永久停在 `downloading`，所有下载按钮失效 |
| F8 | 中高 | `crates/dwall-settings/src/commands/catalog.rs:208`、`.../theme.rs:17` | 读路径未校验 renderer 传入的 `theme_id` / 目录，可读任意目录；被 asset scope `**/*` 放大 |
| F9 | 中 | `crates/dwall-settings/src/thumbnail/store.rs:29` | 缩略图互斥锁表无界，仅手动清缓存才释放 |
| F10 | 中 | `crates/dwall-settings/src/network/client.rs:94` | 下载无字节上限、无校验和，唯一磁盘护栏是 8 GiB 解压上限 |
| F11 | 中 | `crates/dwall-settings/src/filesystem.rs:250` | 目录移动非原子：copy 中途失败留下半成品目标目录，重试必撞 `DestinationExists` |
| F12 | 中 | `crates/dwall-settings/src/thumbnail/cleanup.rs:88`、`.../storage.rs:54` | `while let Ok(Some(..))` 让目录遍历在首个 I/O 错误处静默截断 |
| F13 | 中 | `crates/dwall-settings/src/process.rs:129` | 进程匹配 fail-open：无法描述的进程被当作自家守护进程，配合 PID 复用可能误杀 |
| F14 | 中 | `src/store/settings.store.ts:253` | 主题目录移动失败只 `log.error`，无 toast，用户的目录选择被静默丢弃 |
| F15 | 中 | 全局 | 键盘可操作性：隐藏元素仍可聚焦、点击 `<div>` 无键盘支持、全局禁用选择与右键 |
| F16 | 低-中 | 全局 | `prefers-reduced-motion` 缺失（三个无限动画） |

## 2. 功能实现

> **修复状态（2026-10-23 核对 `main`）**：以下条目大多在 review 之后已修复，条目后标注了对应提交。
> #756（设置后端）、#758（守护进程与日志）已合并；#757（前端）仍在评审中，其提交标注为「见 #757」。
> 没有标注的条目尚未核对，或仍未处理。

### 2.1 守护进程（`crates/dwall`）

- **锁屏主题不确定**（已修复 `f51e77c`）：`theme_engine.rs:308` 的 `get_first_configured_theme` 对 `Individual` 用
  `map.values().next()`。`HashMap` 迭代顺序随机，多显示器各配不同主题时锁屏每天可能换一个。建议按
  显示器顺序取第一个成功应用的主题。
- **日志默认级别分支实际不可达**（已修复 `21ab81c`）：`logging/src/lib.rs:263` 的 `default_log_level()` 在 debug 下返回
  `Trace`，但 `logging/build.rs:4` 对所有未启用 `max-level-info` 的构建（含 debug）注入
  `DWALL_LOG=debug`，`option_env!` 永远命中。建议 build.rs 仅在 `not(debug_assertions)` 时注入，
  或删掉 `Trace` 分支并同步注释。
- **原子写临时文件命名**（已修复 `563f913`）：`config.rs:643` 用 `path.with_extension("toml.tmp")`，`--config foo.conf`
  会生成 `foo.toml.tmp`。同一目录跑两个不同配置实例会互相覆盖临时文件。建议用
  `with_file_name(format!("{name}.tmp"))` 并加 PID。
- **死代码 / 陈旧注释**（已修复 `2ba1b11`；其中 `with_themes_directory` 仍有调用者，未删）：`with_themes_directory`（`config.rs:379`）无调用者；
  `ThemeError::DefaultThemeMissing`（`theme.rs:25`）从不构造；缓存槽 `SolarPosition`
  （`utils.rs:125`）只读不写；`RawConfig` 注释写「will be removed in 0.3.0」而当前已是
  `0.3.0-alpha.6`（`config.rs:455`）。
- **理论溢出**（已修复 `01cb477`，实际位置是 `geolocation.rs:32`）：`geolocation.rs:122` 的 `(duration.as_secs() as i64) * 10_000_000` 无 `checked_mul`。
  建议 `saturating_mul`。
- **显示器友好名定长缓冲**（已修复 `5b68565`）：`monitor.rs:414` 固定 512 字节，超长名会 `ERROR_INSUFFICIENT_BUFFER`
  并回落；可先查长度。
- **配置变更检测精度**（未处理，低风险）：靠 mtime 比较（`config.rs:689`）。NTFS 为 100 ns，没问题；FAT/exFAT 或网络盘
  为 2 s 粒度，可能漏检同一秒内的两次写入。低风险。

### 2.2 设置后端（`crates/dwall-settings`）

- **镜像对稳定版更新检查失效（F2）**（已修复 `b960a51`）：`network/mirror.rs:46` 对 `releases/latest/download/<asset>`
  先请求 `https://api.github.com/.../releases/latest`，且用临时 `reqwest::Client`（无连接复用、不走
  SOCKS5）；失败则返回原始 `github.com` URL。稳定版清单 `update.rs:26` 恰好走这条分支。建议：`latest`
  路由直接套模板、复用共享 `HttpClient`、配置镜像时绝不回落 `github.com`。
- **同步命令在主线程做 N+1 文件 IO**：`commands/catalog.rs:72`、`:199`。非 async 的
  `#[tauri::command]` 在主线程执行；`get_theme_catalog` 对 15 个目录逐个做 `is_dir()` + 未缓存的
  `solar.json` / `theme.toml` 读取解析，自定义主题再各做一次 `read_dir`。建议改 async +
  `spawn_blocking` 或复用路径键缓存。
- **路径穿越（F8）**（已修复 `6c9c4de`，并由 `8810696` 收窄 asset scope）：`commands/catalog.rs:208` 的 `get_theme_wallpapers` / `match_wallpaper` 直接
  `join(theme_id)`；`commands/theme.rs:17` 的 `validate_theme_cmd` 接受任意 `themes_directory`。
  写路径 `commands/download.rs:18` 已调用 `validate_theme_id`，读路径却漏了。配合
  `tauri.conf.json:16` 的 `assetProtocol.scope = ["**/*"]` 影响被放大。建议读路径一律校验并收窄 scope。
- **缩略图锁表无界（F9）**（已修复 `12f8e7d`）：`thumbnail/store.rs:29`，条目只在手动清缓存时移除。建议 Drop 守卫删除
  key，或改有界 LRU。
- **下载无界且无校验（F10）**（已修复 `60d6ebf`，`max_bytes` 更早已有；SHA-256 未做，发布资产没有可比对的校验和）：`network/client.rs:94` 的 `download_file` 无 `max_bytes`、无
  `Content-Length` 校验、无 SHA-256。
- **目录移动非原子（F11）**（已修复 `677beb7`）：`filesystem.rs:250`，失败时目标目录残留，重试撞 `DestinationExists`。
  建议先复制到临时同级目录再 rename。
- **遍历静默截断（F12）**（已修复 `96b9258`）：`thumbnail/cleanup.rs:88`、`:120`、`:164`、`:188`、
  `thumbnail/storage.rs:54`。建议 `match` 错误后 `warn!` + `break`。
- **进程匹配 fail-open（F13）**（已修复 `8baa60e`）：`process.rs:129`、`:242`。未知会话 / 无法取镜像名时判为自家进程。
- **取消无法阻止解压、staging 泄漏**（已修复 `b12d071`）：`theme/downloader.rs:96`、`:228`。future 被 drop 时
  `<id>.staging` / `.zip.tmp` 残留。建议阶段间复查取消标志并用 Drop 守卫清理。
- **归档加固**（已修复 `87b0eda`、`395bc02`）：`archive.rs:74` 报错信息硬编码 8 GiB 而非 `max_bytes`；`archive.rs:100` 的
  `enclosed_name()` 不拒绝 `:`（NTFS ADS）与 `NUL`/`CON` 等设备名。zip-slip 本身已正确处理。
- **主题 id 校验缺口**（已修复 `ecc666b`、`76656ec`）：`filesystem.rs:42` 未拒绝 Windows 保留名与结尾点 / 空格；
  `theme/download_url.rs:8` 未做 URL 编码（空格转 `.`），`#` / `?` / `%` 会静默改变请求。
- **日志敏感信息**（已修复 `8b2cd1e`）：`commands/system.rs:145` 在 `info!` 记录完整 `download_url`（私有镜像可能内嵌
  token），建议只记 host。
- **被吞掉的错误**（已修复 `0c727b0`）：`commands/catalog.rs:67`、`:109`、`theme/status.rs:60`、
  `network/mirror.rs:57`、`commands/random.rs:26`。建议至少 `debug!` / `warn!`。

### 2.3 前端（`src/`）

- **F1（高）资源错误态击穿整棵树**（主要部分更早合并，剩余缺口 `83a0b5c`，见 #757）：Solid 的资源访问器在错误态会 `throw`
  （`solid-js/dist/solid.js:328`：`if (err !== undefined && !pr) throw err;`），而全应用无
  `ErrorBoundary`。最现实的触发路径是用户拒绝定位权限：`commands/solar.rs:45` 用 `?` 直接抛错，
  `src/hooks/useSolarPosition.ts:16` 的资源随即进入错误态，`FixedStage.tsx:82`、
  `EngineStatus.tsx:42`、`SunPathPanel.tsx:57` 的无保护访问即抛错。注意：主题未安装**不是**错误
  （`catalog.rs:188` 返回 `Ok(vec![])`），损坏的 `theme.toml` / `solar.json` 才是。修复：统一
  `res.error ? fallback : res()`，并加 `<ErrorBoundary>` 与重试 UI（`ThemeThumbnail.tsx:56` 是范例）。
- **F5（高）应用操作提交未保存设置**（已修复 `c7f0fb7`）：`settings.store.ts:222` 从草稿 `{ ...config, wallpaper_mode }`
  写盘并 `s.saved = snapshot(next)`。用户改了表单没保存、却在舞台点了「应用主题」时，这些改动被静默写入。
  建议只补丁 `wallpaper_mode`。
- **F6（高）搜索重建全部 tile**（已修复 `6705c10`）：`store/catalog.selectors.ts:24` → `models/themes.ts:18` 每次调用
  都产生新对象数组，`<For>` 按引用协调，导致每次按键全部重挂载并重新 `mirrorUrl` +
  `getOrSaveCachedThumbnails`。建议让 `themeList` 在目录不变时保持引用稳定。
- **F7（高）下载监听注册失败即卡死**（已修复 `2647070`）：`store/download.store.ts:60` 在 `try` 之前 await；
  `cancelDownload`（`:98`）无 catch，调用方均 `void`。
- **状态管理（中）**（已修复 `90e9ef3`；其中 F14 由 `c40821c` 修复）：轮询失败每 5 s 弹一次 toast（`store/engine.store.ts:45` + `App.tsx:20`）；
  并发刷新可写回陈旧状态；`toggleApply`（`store/fixed.store.ts:75`）与 autostart 切换
  （`components/settings/EngineSection.tsx:60`）的 rejection 未处理；目录选择失败静默（F14）；
  catalog 重载可覆盖并发安装标记（`store/catalog.store.ts:33`）。
- **Solid 细节（低-中）**（已修复 `226583e`）：`scope/MonitorCard.tsx:20`、`library/ThemeTile.tsx:21` 解构 props
  （违反 AGENTS，目前被父组件重建掩盖）；`hooks/useMediaQuery.ts:9` 的 `query` 非响应式；
  `scene/WallpaperImage.tsx:30` 用裸表达式 `props.path;` 建依赖。
- **架构（中）**（未处理）：`store/fixed.store.ts:75` 与 `store/random.store.ts:51` 在 JS 里手工拼
  `monitor_specific_wallpapers` 的 union 与 `"all"` 伪显示器——按 AGENTS「业务逻辑在 Rust」，
  建议下沉为窄命令。
- **死代码（低）**（部分修复 `7b9e456`；`useScrollEdges` 并非死代码，7 个 IPC 包装各自对应已注册命令，均保留）：`hooks/useScrollEdges.ts` 从未导入；`lib/layout.ts:3` 的 `MIN_WINDOW` /
  `MQ_MIN` / `MQ_SHORT` 未用；`models/monitors.ts:6` 的 `ALL_MONITOR_ID` 未用；
  `store/ui.store.ts:73` 的 `TITLEBAR_BORDER_KEY` 未导出使用；无调用者的 IPC 包装
  `requestLocationPermission`、`openPrivacyLocationSettings`、`openConfigDir`、`validateTheme`、
  `getAppliedThemeID`、`clearThumbnailCache`、`getCustomizedThemes`；`utils/logger.ts:130` 的
  `logger.hot` 整个节流 API 未用。

## 3. UI / UX 与可访问性（按实际用户影响重新分级）

壁纸是视觉产物，为「屏幕阅读器 + 全盲用户」做完整 WCAG 改造的投入产出比很低；本项目也已提供
headless 文本路径（`docs/headless.md`：`--check-config` / `--once` / `--config`）。因此本节按
**实际受影响的用户**分级，而不是按无障碍规范条目。

### 3.1 保留（与残障无关，或服务大量正常用户）

1. **隐藏但可聚焦（中）**：`src/layout/GlobalBar.tsx:15` 仅用 `opacity-0 pointer-events-none` 隐藏
   Fixed / Random 切换与提交槽；`src/views/MainView.tsx:44` 的抽屉关闭时仅 `translate-x-full`。
   键盘用户会 Tab 到不可见控件。建议 `inert` / `visibility:hidden`。
2. **点击 `<div>` 无键盘支持（中）**：`src/scope/UnifiedCard.tsx:11`、
   `src/scope/MonitorCard.tsx:29`、`src/library/ThemeTile.tsx:47`、
   `src/stage/WallpaperStrip.tsx:31`。**用键盘无法选中主题 / 显示器**，是功能缺失而非无障碍。
   建议改 `<button type="button">`。
3. **全局禁用选择与右键（中）**：`src/index.css:12` 对 `*` 设 `user-select:none`，
   `src/index.tsx:9` 生产环境全局 `preventDefault` 右键。任何人都无法复制主题路径 / 错误信息，
   也无法在坐标 / 镜像输入框右键粘贴。建议对 `input, textarea, [data-selectable]` 恢复选择，
   并仅在非输入区禁用右键。
4. **焦点管理（低）**：抽屉开关不转移焦点；关闭后焦点落回 `<body>`。
5. **空态 / 错误态 / 重试（中，与无障碍无关）**：`store/catalog.store.ts:33` 的 `status` 无任何组件
   读取，目录加载失败只 toast，库区留白无重试；搜索无「无匹配」态；`scene/WallpaperImage.tsx:58`
   把 `onError` 当成功（显示破图，`fallback` 永不生效）。

### 3.2 顺手做（成本极低，服务非全盲的残障用户）

6. **`prefers-reduced-motion`（低-中）**：`src/styles/index.css:30` 的 `bdot` / `breathe` /
   `pulse-ring` 三个无限动画，用于引擎状态点、随机脏标记、保存按钮、匹配徽章。这是给前庭功能障碍 /
   动效敏感用户的，与视力无关。一条全局 media query 即可。
7. **低视力对比度与焦点可见性（低）**：低视力是随年龄增长最常见的视觉障碍，看壁纸、调设置正是他们会做的
   事。当前 token 体系整体对比度尚可，建议抽查焦点环与次要文字。

### 3.3 可选 / 建议不做（纯读屏支持）

8. 控件可访问名称：`SettingsRow` 的标签是 `<div>`（`components/ui/field/FieldTitle/FieldTitle.tsx:9`），
   `FieldLabel` 无 `for`（`.../FieldLabel.tsx:10`），开关 / 坐标输入均无 `aria-label`
   （`scope/MonitorCard.tsx:60`、`scope/UnifiedCard.tsx:27`、`components/settings/AppearanceSection.tsx:39`、
   `.../EngineSection.tsx:58`、`.../PathsSection.tsx:168`）。主要服务读屏；语音控制（Windows 语音访问 /
   Dragon）用户也受益，但那属于「有明确需求再做」。
9. SVG `role="img"`、太阳读数 hover-only、进度条 `aria-label`、Toaster 英文 live region 文案。
10. 图标按钮无名称（`EngineSection.tsx:91`、`PathsSection.tsx:143`）——若做 8，一并补。
11. 死交互：`PathsSection.tsx:129` 的「查看模板列表 ↗」是无 `onClick` 的下划线 `span`。

**约定建议**：要么补齐 3.1 + 3.2 并在 `AGENTS.md` 写明「GUI 不做读屏级无障碍，文本路径以 CLI /
配置文件为准」，要么现在就停下并明确 3.3 不做，避免每轮审查重复争论。当前代码是「一半投资」状态
（已有 `role="switch"`、`aria-label`、`Button` 的 dev 警告、`sr-only`，但标签关联、`inert`、
焦点管理又缺）。

### 3.4 i18n

六个语言键集完全一致（151 键，占位符一致），无硬编码文案。问题：

- **语言自动检测只做精确匹配**：`src/i18n/index.ts:50` 的 `browserLang in LANGUAGES` 对
  `ja` / `ko` / `zh` / `en` 系统一律回落 `en-US`，且忽略 `navigator.languages`。
  建议 `Intl.Locale` 归一化并遍历。
- **`settings.subtitle` 未翻译**：`en-US.ts:111` 的 `"preferences"` 在五种非英文里仍是原文，
  渲染于 `src/views/SettingsView.tsx:27`。
- **死键**：`commit.unsaved` / `commit.saved` / `commit.savedAt`（`en-US.ts:101`）。
- **`localStorage` 无保护**：`src/i18n/index.ts:47`、`:68` 无 `try/catch`，抛错会导致白屏
  （`src/store/ui.store.ts:28` 是正确范例）。

## 4. 日志

> **修复状态（2026-10-23 核对 `main`）**：以下条目均已处理，条目后标注了对应提交。其中 `parse_log_level`
> 是按决策收口的——选择把限制写进文档，而不是改解析让 `enabled` 退化成前缀匹配、丢掉其他 target。

- **正面**：分级宏统一、key/value 风格、`debug!` / `trace!` 在发布特性下编译掉、写文件用
  `Mutex<File>` 且失败不 panic、按启动轮转且上限 5、`get_last_daemon_error` 只读尾部 64 KB、
  生产构建把错误镜像回 UI（`logging/src/lib.rs:204`）。
- **默认级别不可达**（已修复 `21ab81c`）：见 2.1。
- **运行时无大小轮转**（已修复 `64b38f8`）：轮转只在进程启动时发生；长期不重启的守护进程日志持续增长（发布版 Info 增速可控，
  自建 `build:daemon-debug` 为 Debug，每 15 s 一条周期日志）。
- **`parse_log_level` 丢弃后续 target 指令**（`logging/src/lib.rs:291`），与 `Logger` 支持 target
  的能力不匹配；当前调用方未用到。（已修复 `3ffe699`：文档化该限制，而不是改解析）
- **敏感 URL**（已修复 `8b2cd1e`）：见 2.2。
- **等级用词**（已修复 `988a9c6`）：`thumbnail/cleanup.rs:148` 等把单个删不掉的缓存文件记为 `error!`，宜降为 `warn!`。

## 5. 内存与体积

> **修复状态（2026-10-23 核对 `main`）**：本节的可执行项已处理；只剩安装包重建标为未处理，那是发布前的
> 动作，不是代码缺陷。CSS 审计的结论见该条自身。

- **内存：优秀。** 守护进程单线程；缓存是定长结构体 6 槽（`utils.rs:79`），无全局无界容器；主题数据
  用容量 32 的 LRU（`theme.rs:77`）；显示器友好名缓存按已连接显示器收敛；无后台线程。上一轮实测静息态
  Private 约 2.8–3.0 MB、运行 13.7 h 后约 7.5 MB，与设计目标一致（本轮未重新运行守护进程，以免改动
  用户壁纸）。
- **体积：优秀。** `dwall.exe` 649,216 B，`dwall-settings.exe` 4.05 MiB，前端 `dist` 394 KB
  （gzip 后 JS 77 KB + CSS 18 KB）。发布 profile（`lto` / `opt-level="z"` / `panic=abort` /
  `strip`）生效。NSIS 安装包约 1.8 MB。
- **可优化（低）**（已处理 `a0b9d07`，见 #764）：审计结论是 CSS 体积**不**来自手写选择器。114,177 B 中
  Tailwind 生成的 utility 占 94,882 B（83.1%）、preflight 4,035 B、`@property` 4,057 B，手写 CSS 合计约
  6.9 KB；删掉 9 个不可能被选中的语言方案后（-3,318 B）已无手写规则可删。要再压缩只能收窄 `@source`
  或删除未被引用的 `components/ui` 目录。另：review 的 115.9 KB 测于 `a7070d7`，在 `main` 上早已不成立。
- **构建目录**（未处理，本机动作）：`target/` 本机累计约 82.7 GB（debug 60 GB + release 17 GB），可 `cargo clean`。
- **依赖（低）**（已修复 `203203e`）：`package.json` 中 `solid-icons` 与 `jsdom` 无任何引用（图标库是
  `lucide-solid`；测试未用 jsdom 环境），建议移除。
- **打包残留**（未处理，发布前重建）：`target/release/bundle/nsis/` 仍是 `0.3.0-alpha.2` 的安装包，而版本已
  `0.3.0-alpha.6`（非代码问题，发布前重建）。

## 6. 构建 / CI / 打包

> **修复状态（2026-10-23 核对 `main`）**：以下条目已全部修复，条目后标注了对应提交。

- **CI 质量高**：第三方 action 全部 pin 到 SHA；`pr-check.yml` 有 `concurrency`、绑定新鲜度检查、
  路径过滤；`bun run check` + `bun run test` + `cargo test` 都跑。
- **`publish.yml` 缺 `concurrency`**（已修复 `1b56c32`）：两次发布可并发，互相强推 release / tag。
- **`delete-nightly-release-and-tag` 无 `needs:`**（`publish.yml:432`，已修复 `ce9d905`）：`if: success()` 一开始即
  成立，会在构建 / 上传进行中删除 `nightly` release 与 tag，与 `nightly.yml` 竞争。
- **`update-main-version` 失败不影响完成通知**（已修复 `4d0b9d3`）：通知只看 `publish-release.result`，`--ff-only`
  合并失败时 release 已发布但版本号未回到 `main`，汇总仍报成功。
- **`package_version: ${{ env.VERSION }}`**（`publish.yml:67`，已修复 `8f3128b`）：作业级 outputs 求值时机存在取空风险，
  建议改为步骤 `id` 输出。
- **安装器数据破坏（中）**（已修复 `3682969`：卸载不再删除 `$APPDATA\dwall`，升级只清缩略图缓存）：`crates/dwall-settings/windows/hooks.nsi:55` 卸载时
  `RMDir /r "$APPDATA\dwall"` 会无提示删除用户所有自定义 / 下载主题与配置；`:39` 每次安装（含升级）
  都清 `$LOCALAPPDATA\dwall` 与 `com.thep0y.dwall`（日志与缩略图缓存）。建议卸载保留用户数据或明确
  询问，升级只清缓存不清日志。
- **能力过宽**（已修复 `a1496a2`；asset scope 另由 `8810696` 收窄）：`crates/dwall-settings/capabilities/default.json:14` 授予 `process:default`（含
  `exit`）而实际只用 `relaunch`；配合 `tauri.conf.json:16` 的 `**/*` asset scope，建议最小化。
- **两个 tokio 运行时（低）**（已修复 `a1be3d3`）：`crates/dwall-settings/src/main.rs:34` 用 `#[tokio::main]`，而 Tauri
  默认另起运行时；可用 `tauri::async_runtime::set` 统一以省一份运行时内存。

## 7. Rust 还需要补的测试用例

当前 480 个测试、2 个 insta 快照，密度已经很高，主要缺口在「安全路径 / 日历校验 / 不可 panic 契约 /
真实文件系统成功路径」。

### 7.1 高风险

**`crates/time`（静默日期错误代价最大）**

- `month_days_handle_leap_years_and_centuries` — `month.rs`：`Month::February.days_in_month` 对
  2024 / 2023 / 2000 / 1900 / 2100 → 29 / 28 / 29 / 28 / 28；各月 30 / 31 边界。
- `month_try_from_rejects_zero_and_thirteen` — `month.rs`：`try_from(0)` / `try_from(13)` →
  `InvalidMonth`；`try_from(1|12)` → Ok；`December.as_u8() == 12`。
- `utc_new_rejects_every_invalid_component` — `lib.rs`：1969 年、day 0 / 32、2023-02-29、2024-02-30、
  hour 24、minute 60、second 60 → 精确 `Error` 变体；2024-02-29 为 Ok。
- `utc_rfc3339_is_zero_padded_and_carries_millis` — `lib.rs`：
  `from_timestamp_millis(1_700_000_123_456)` 的 `to_rfc3339` / `millisecond` / `timestamp_millis` 往返。
- `utc_differences_and_ranges_behave_at_the_boundaries` — `lib.rs`：`diff_seconds` / `diff_days` /
  `duration_since` 反向、`is_between_inclusive` / `exclusive` 端点。
- `days_to_ymd_clamps_beyond_max_and_round_trips_epoch` — `math.rs`：`days_to_ymd(0)` → 1970-01-01，
  `MAX_DAYS` → 9999-12-31，`u64::MAX` → 9999-12-31（不 panic）。

**`crates/dwall-settings/src/archive.rs`（安全）**

- `archive_entry_with_parent_components_is_refused`：构造含 `../../escape.txt` 的 zip →
  `extract_with_limit` 返回含 `escapes the extraction directory` 的错误，且目标目录外无文件。
- `archive_symlink_entries_are_skipped`：含 symlink 条目 + 普通文件 → Ok，普通文件在，symlink 未创建。
- `archive_at_the_entry_limit_is_accepted`：恰好 `MAX_ARCHIVE_ENTRIES` 个条目 → Ok；+1 → Err。
- `remove_file_on_a_directory_is_an_error`：目录路径 → Err（只吞 `NotFound`）。

**`crates/dwall/src/lazy.rs`（唯一不变量是「绝不 panic」）**

- `lazy_ensure_dir_creates_and_is_idempotent`：需先把 `ensure_dir` / `temp_base` 改为
  `pub(crate)` 且接收基目录参数，断言目录创建、重复调用无错、目标是文件时不 panic。

**`crates/dwall/src/config.rs`**

- `config_validate_rejects_out_of_range_manual_coordinates`：手动 (90.1, 0) / (0, 180.1) / (-90.1, 0) /
  (0, -180.1) → Err；±90 / ±180 → Ok。
- `image_format_rejects_an_unknown_string`：`"webp"`、`"JPEG"` → Err；`Png.as_str() == "png"`、
  `Jpeg.as_str() == "jpg"`。
- `monitor_specific_wallpapers_all_variant_helpers`：`All` / `Individual(空)` / `Individual(有)` 的
  `is_all` / `is_empty` / `get`。
- `wallpaper_mode_rejects_an_unknown_tag`：`mode = "shuffle"` → Err。

**`crates/dwall/src/solar.rs`**

- `position_new_rejects_nan_and_infinity`：`(NaN, 0, 0)`、`(0, INF, 0)` → Err。
- `solar_position_at_the_south_pole_and_longitude_dateline`：南极 / ±180° 在至日，altitude ∈
  [-90, 90]、azimuth ∈ [0, 360)，南北极高度互为相反数。
- `equator_sun_rises_and_sets_once_per_day`：赤道采样一整天，阈值穿越恰好一升一落。

### 7.2 中风险

- **`bin/daemon.rs`**：`version_string_names_the_profile_and_package`、
  `print_summary_lists_the_documented_keys`（需把 `print_summary` 改为写 `impl Write`）、
  `parse_args` 的 `--config` 与命令顺序组合、`check_config` 退出码映射、autostart 安装 / 卸载。
- **Windows 模块（当前 0 测试）**：`registry_delete_of_a_missing_value_succeeds`
  （`HKCU\Software` 可测）、`registry_open_of_a_missing_key_errors`；
  `WallpaperSetter::has_wallpaper` / 重试 / 锁屏路径（可用 trait mock）；
  `ColorSchemeScheduler` 值相等时不写注册表的分支。
- **`update.rs` / `system.rs`**：`update_select_release_ignores_equal_and_older_versions`
  （含 `0.3.0+build`）、`Release::version` 对 `v1.2.3` / `1.2.3` / `nightly` / `v1.2`、
  `prerelease_endpoint_reports_invalid_json`（复用 `network/client.rs` 的 `TestServer`）。
- **`filesystem.rs`**：`filesystem_move_directory_moves_a_tree_and_refuses_an_existing_destination`、
  `find_files_in_dir_is_flat_and_case_insensitive`（`1.JPG` / `2.jpg` / `notes.txt` / `sub/3.jpg`）。
- **`downloader.rs`**：`download_with_retry` 的断点续传算术、`handle_download_error` 保留非空分片、
  `stage_archive` 解压失败删除 zip。
- **`mirror.rs` / `client.rs`**：`resolve` 对 `None` / 非 GitHub URL / Socks5 原样返回、
  `resolve_raw` 缺占位符 / 非 `/raw/` 的行为、`reconfigure` 失败保留旧 client、SOCKS5 分支。
- **`theme_manifest.rs` / `theme.rs`**：`hours` 模式与 `[[images]]`、`exists` / `read` 缺失与非法
  文件、index 255 / 256 边界、`get_theme_directory_path` 优先自定义目录、`is_theme_directory`、
  损坏 `solar.json`、`SimpleRng::new(0)` 与 `shuffle` 是排列。
- **`color_scheme.rs`**：`WhiteNight` 半带端点、`detect` 振幅恰为 4.0 → `Normal`、空高度回落
  `Normal`。
- **`schedule.rs` / `theme_engine.rs`**：`next_change_with_step` 步长 0 强制为 1、变化恰在 horizon
  上；`scheduled_wait` clamp。
- **`utils.rs`**：`cache_huge_ttl_does_not_panic`（`checked_add` 回退路径）。
- **`time/src/offset/windows.rs`**：注入时钟跨越 DST 的偏移刷新。
- **`logging`**：`parse_log_level` 取首个指令、`with_target` 的顺序依赖、`with_target_level`、
  `enabled` 前缀匹配、`get_log_level` 的 `DWALL_LOG` 优先于 `RUST_LOG`、`restore_color`、
  `__fmt_key` 在两种 profile 下的形态。
- **`daemon.rs`（dwall）**：`random_pool`、`apply_wallpaper_mode`、`handle_random_mode`
  （池为空 / 选中主题被删）；`failure_backoff(0 / 1 / 17 / u32::MAX)`。
- **设置命令包装**：`should_run`、`local_thumbnails`、`match_wallpaper`（同一图片索引复用多个条目）、
  `download_url::build` 的空格与特殊字符、`thumbnail/storage` 的目录大小求和、
  `process::image_name_matches` 大小写。

### 7.3 测试基础设施建议

1. **引入 `proptest`**（当前 0 处）给纯数值核心加往返性质：`days_to_ymd(days_since_epoch(y,m,d)) ==
   (y,m,d)`（1970..=9999）、时间戳往返、`Offset::from_str(o.to_string())`、任意 lat/lon/时间戳下
   太阳高度 / 方位恒在范围。
2. **时钟注入**：复用 `time/src/offset.rs:72` 的 `get_at(now, read)` 接缝，为 `Date::with_hms`、
   日志、守护循环提供可测时钟。
3. **可捕获输出**：`print_summary` 改为写 `impl Write`；日志格式写入可注入 writer，断言不打印坐标。
4. **`lazy` 接缝**：`ensure_dir` / `temp_base` 参数化基目录。
5. **扩展 insta**：给 `ImageFormat` / `PositionSource` / `WallpaperMode` / `Network` /
   `ThemeManifest` / `RandomSelection` 加 serde 快照——它们决定 `src/ipc/bindings/**` 的形状。
6. **共享 HTTP fixture**：把 `network/client.rs:219` 的 `TestServer` 提升为 `#[cfg(test)]` 公共模块，
   支持多响应脚本，供 mirror / update / downloader 复用。
7. **RAII `ScratchDir`**：合并 `archive.rs` / `downloader.rs` / `filesystem.rs` / `cleanup.rs` 四处
   重复的 `scratch(name)`，避免并行测试碰撞与 `%TEMP%` 泄漏。
8. **`max-level-info` 门控测试**：断言发布特性下 `debug!` / `trace!` 参数不求值。
9. **CI**：补 `cargo test -p dwall --all-features`（会重生成 bindings，需一并提交）与
   `cargo test -p dwall-settings --bins`。

## 8. 修复批次计划

1. **用户可见的崩溃与静默数据问题**：F1（资源错误防护 + `ErrorBoundary`）、F4（自定义主题列表容错）、
   F5（应用不再提交无关设置）、F7（下载监听入 `try`）、F14（目录移动失败提示）。
2. **网络与阻塞**：F2（镜像 `latest` 路由 + 复用 client + 不回落）、F3（`spawn_blocking(stop_daemon)`）、
   F6（`themeList` 引用稳定）。
3. **安全与资源边界**：F8、F9、F10、F11、F12、F13。
4. **可访问性与打磨**：3.1 全部、3.2、空态与重试、i18n 归一化；3.3 按约定决定做或不做的明确化。
5. **发布链路**：`publish.yml` 加 `concurrency`、nightly 删除加 `needs`、修正 `env.VERSION` 输出、
   安装器卸载保留用户数据、重建 alpha.6 安装包。
6. **测试**：按 7.1 → 7.2 顺序补齐。
