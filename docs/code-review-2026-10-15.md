# Dwall 代码审查与可执行修复清单（2026-10-15）

> 本文件是仓库全量审查的**工作清单**，不是发布文档。每条含：位置（`path:line`）、证据、影响、修复方案、验收方式。
> 完成一条就把 `- [ ]` 改为 `- [x]`，并在行尾补 `→ 提交 <sha>`。

## 0. 审查来源与验证基线

**来源**：5 路并行深度评审（守护进程核心 / Tauri 设置后端 / SolidJS 前端 / logging+time+Windows 平台层 / 构建·依赖·体积·CI）+ 交叉复核。

**已实测的基线（只读命令）**

| 项 | 值 | 结论 |
| --- | --- | --- |
| `cargo fmt --all -- --check` | exit 0 | 通过 |
| `cargo clippy -p dwall -p logging -p time --all-features --all-targets -- -D warnings` | exit 0 / 0 警告 | 通过 |
| `cargo clippy -p dwall-settings --all-targets -- -D warnings`（`DWALL_SKIP_BUILD=1`） | exit 0 | 通过 |
| `cargo test -p dwall --all-features` | 96 + 7 通过 | 通过（但覆盖面窄，见 P2-36/P2-51） |
| `tsgo --noEmit` / `biome check src`（375 文件） | exit 0 | 通过 |
| `bun run test` | **exit 1**（缺 jsdom、0 测试文件） | 见 P2-35 |
| `target/release/dwall.exe` | 600,576 B (586.5 KB) | 体积卖点成立 |
| `target/release/dwall-settings.exe` | 5,346,304 B (5.10 MB) | 合理 |
| NSIS 安装包 0.2.5 | 2.02 MB | 合理 |
| `dist/` | 360,866 B（JS 210,007 / CSS 119,476 / SVG 30,781） | 其中约 16 KB 为死 CSS |
| 运行中 daemon（用户既有进程） | WorkingSet 35.0 MB / Private 7.42 MB / 3 线程 | 与 AGENTS "a few MB" 相符 |
| 依赖 | daemon 直接 9、依赖树 90；settings 直接 24 | 精简；但 `windows` 双版本（P1-13） |

**状态图例**：`P1` 必修（功能/崩溃/数据损失/隐私） · `P2` 应修（可靠性/性能/一致性） · `P3` 加固与清理。

---

## 批次总览

| 批次 | 内容 | 条目 |
| --- | --- | --- |
| 1 | 守护进程核心：内存泄漏、阻塞、可观测性、随机模式 | P1-14, P1-15, P1-3, P1-4, P1-2, P2-38, P2-39, P2-40, P2-41, P2-42 |
| 2 | 设置后端：进程生命周期、自启、panic、路径校验 | P1-1, P1-10, P1-11, P1-16, P2-27(路径) |
| 3 | 前端：控件失效、动效、崩溃、错误可见性 | P1-6, P1-7, P1-8, P1-9 |
| 4 | 下载与缩略图缓存 | P1-5, P2-44, P2-45, P2-46, P2-47, P2-48, P2-49 |
| 5 | 其余高价值 P2（time/logging/构建/CI） | P2-43, P2-50, P2-51, P2-52, P2-28~P2-34 |
| 6 | P3 批量清理 | P3-* |

---

## 进度日志（本地工作树，未提交）

| 批次 | 状态 | 已完成内容与验证 |
| --- | --- | --- |
| 1 守护进程核心 | ✅ | P1-14（`CoTaskMemFree` guard）、P1-15（定位路径改用带 `maximumAge/timeout` 重载）、P1-3（读失败退避重试 + `ConfigWriter` 原子替换 + `main` 记录致命错误）、P1-4（逐显示器失败 `warn!` + 全失败汇总 `warn!`）、P1-2（`DailyRandomThemeSelector::restore` + `random_state::date_key` 复用当日持久化选择）、P2-38（release 默认 `Info`，同步把每周期 INFO 降级为 DEBUG/改为"真变化时才 INFO"）、P2-39、P2-40、P2-41、P2-42、P3-7。验证：`cargo fmt --check` / `clippy --all-features --all-targets -D warnings` / `cargo test -p dwall --all-features` = **99 passed**。 |
| 2 设置后端 | ✅ | P1-1（新增 `daemon::should_run`，`applier` 与 `setup` 共用；随机模式不再停引擎）、P1-10（`Run` 值加引号，`is_enabled` 兼容旧值）、P1-11（`daemon::daemon_path()` 取代 5 处 `unwrap`）、P1-16（`validate_theme_id` / `validate_directory`，`open_url` 限 http(s)、`open_dir` 限已存在目录）、P2-14（引擎/定位/显示器等命令改 `async` + `spawn_blocking`）、P2-13/P3-30（改读 `DWALL_LOG_DIR` + 纯文本取最后一条 ERROR）、P3-21 部分（拒绝目标位于源目录内）、额外：`setup_app` 读配置失败回退默认值而不是拒绝启动。验证：`DWALL_SKIP_BUILD=1 cargo clippy -p dwall-settings --all-targets -- -D warnings` = 0 警告。 |
| 3 前端 | ✅ | P1-6（`Switch` 提取 `onClick`/`onKeyDown` 并用新的 `src/utils/call-event-handler.ts` 组合调用，顺带补齐 `disabled` 与 Space/Enter 键盘操作）、P1-7（`styles/index.css` 的 `@theme` 补 5 个 `--animate-*` 令牌 + keyframes；已用 `bun run build:vite` 验证产物 CSS 真的产出 `animate-*` 与 `@keyframes`）、P1-8（`localStorage` 值先经 `in LANGUAGES` 校验）、P1-9（挂载 Toaster + 各 store 失败弹窗 + 新增 `common.operationFailed` 的 6 语言键）。验证：`tsgo --noEmit` / `biome check src`（376 文件）/ `bun run build:vite` 全部通过（CSS +1.2 KB）。 |
| 4 下载与缩略图 | ✅ | P1-5（`Response::chunk()` 流式落盘 + 每块查取消 + `connect_timeout(15s)`/`read_timeout(60s)`；解压改 `File` 源 + `spawn_blocking` + 8 GiB/16384 条目上限；进度按 1% 节流）、P2-44（`DownloadTaskGuard` Drop 兜底，`DownloadTaskManager` 改 `std::sync::Mutex` 同步方法）、P2-45（仅 `206` 才续写，否则截断重下）、P2-49（`<theme>.staging` 目录 + 成功后 `rename` 原子替换，失败保留下载前的可用主题）、P2-46 部分（按 key 单飞 + 魔数校验）、P2-47（清缓存真的删盘）、P2-48 部分（前端 `pending.close()` + GitHub API 超时）。验证：settings clippy `-D warnings` 0 警告；`tsgo` / `biome` 0 错误。 |
| 5 其余 P2 | ✅ | P2-43（time 比较契约/饱和/`now()` 不 panic/日历夹取，+4 单测）、P2-50（logging：`log` re-export、per-target 级别真正生效、ANSI 仅终端、写失败不 panic、`dwall=debug` 解析，+3 单测）、P2-51（设置 crate 14 个单测）、P2-52（删 shell 插件、收窄 capability、显式 tokio/reqwest 特性）、P2-28/29/35（CI 门禁：只读 check、声明 biome/jsdom、vitest 可用并接入 CI、触发路径、automerge 条件）、P2-30（结论为有意保留 + 隐私已修）、P2-31（移除 Google Fonts 外链 + 收紧 CSP）、P2-32（去 `requireLiteralLeadingDot`，其余附理由待产品决策）、P2-34（license 对齐 AGPL）、P2-36 部分（守护进程新增选择器/路径/还原单测）、P2-37（README 中英 + AGENTS 修正）；追加 P1-12（遥测默认关闭）、P1-13（windows 全栈统一到 0.62：`windows`/`windows-core`/`windows-future` 各只剩一个版本，跟随 Tauri 的 tao 一并升级；`cargo tree` 实测无重复）、P1-15b（锁屏 WinRT 有界等待，并抽出 `async_operation` 供 geolocation 复用）、P3-3/4/5/19/24/32。验证：`cargo fmt --check`、`cargo clippy`（CI 形式与 `--all-features --all-targets` 两种）均 0 警告、`cargo test` 工作区 **94+7+14+3+80** 全通过、`tsgo`/`biome`（376 文件）/`vitest`（3）/`vite build` 全通过。 |
| 6 P3 清理 | ⏳ | 已完成 P3-3/4/5/7/19/21/24/30/32/49（P3-49 的 `.gitattributes` 部分；其余见该行）；其余 P3 待办：日志轮转上限、`S_FALSE`/COM 释放顺序、`SAFETY` 注释、友好名缓存失败污染、`position_index`、NSIS 语言、`hooks.nsi` 卸载残留、隐藏窗口兜底、`is_enabled` 兼容、缩略图 LRU、`.bak`/`rust-version`/`icns`/`gen/schemas`、死代码与 Tailwind 重复、`user-select`、a11y、空态、`auto_detect_color_scheme` 写回等。另：`cargo clippy -r` 与生产特性组合发现的 release-only 警告已修（`fix(logging)`），并已加入 CI 门禁 |

---

## 批次 1 · 守护进程核心

### - [x] P1-14 `IDesktopWallpaper::GetWallpaper` 返回的 `PWSTR` 从不释放（每显示器每周期泄漏一块 COM 内存）

- 位置：`crates/dwall/src/platform/windows/wallpaper.rs:228-242`
- 证据：
  ```rust
  let current_wallpaper_ptr = match unsafe { self.desktop_wallpaper.GetWallpaper(monitor_path) } { ... };
  let current_wallpaper_path = unsafe { current_wallpaper_ptr.to_hstring() };  // 只拷贝，原指针丢失
  ```
- 复核结论（已逐层核对 vendored 源码，**确认无任何自动释放**）：
  - `windows-0.61.3` 生成签名 `pub unsafe fn GetWallpaper<P0>(...) -> Result<PWSTR>`，函数体仅 `.map(|| result__)`；
  - `windows-strings-0.4.2/src/pwstr.rs` 既无 `impl Free for PWSTR` 也无 `CoTaskMemFree`，`to_hstring()`（`pwstr.rs:70`）只做拷贝；
  - `windows-core-0.61.2` 仅在 `array.rs:104` 对 WinRT `Array<T>` 调 `CoTaskMemFree`。
  - `[out] LPWSTR` 的 COM 约定 = 被调用方 `CoTaskMemAlloc` / 调用方 `CoTaskMemFree`。
- 影响：调用链 `run_engine_loop → process_theme_cycle → process_single → set_monitor_wallpaper → set_wallpaper → has_wallpaper` 每周期都走（即使壁纸未变也先查询）。默认 15 s、最小 1 s ⇒ 2 屏约 1.1 万次/天，**守护进程唯一的无界内存增长**，直接违背 "memory is a feature"。
- 修复：取到指针后 `windows::Win32::System::Com::CoTaskMemFree(Some(ptr.0 as *const core::ffi::c_void))`（用 `scopeguard` 式 guard 保证异常路径也释放）；顺带把"当前壁纸路径"缓存进 `DisplayMonitor` 或 `WallpaperSetter`，避免每周期查询。
- 验收：连续运行观察 `dwall.exe` 私有字节不再单调上升；`cargo clippy -D warnings` 通过。

### - [x] P1-15 WinRT 异步调用全是 INFINITE 阻塞等待，单线程守护进程可被永久挂死

> 本轮已修：周期路径（`GetGeopositionAsync`）改用平台自带的 `maximumAge/timeout` 重载（1 min 缓存 / 30 s 上限）。
> 未修（拆为 P1-15b）：锁屏路径 `StorageFile::GetFileFromPathAsync(...).get()` 与 `LockScreen::SetImageFileAsync(...).get()`（`wallpaper.rs:152`、`179`）没有超时重载；它们只在锁屏图**真正变化**时执行，不在每个周期内，故风险低于定位路径，但仍是潜在永久阻塞，需线程隔离 + 有界等待。

- 位置：`crates/dwall/src/platform/windows/geolocation.rs:54-60`、`:152`、`:179`、`crates/dwall/src/platform/windows/wallpaper.rs:152`、`:179`
- 证据：
  ```rust
  let geoposition = handle_windows_error("Getting geoposition asynchronously", || {
      geolocator.GetGeopositionAsync()      // 用的是无参重载（存在 maximumAge/timeout 重载）
  })?.get()
  ```
- 复核结论：`windows-future-0.2.1/src/get.rs:22` 的 `IAsyncOperation::get()` 依赖 `Waiter`（`CreateEventW` + `SetCompleted` + `WaitForSingleObject(handle, INFINITE)`，`waiter.rs`），**无超时**。
- 影响：该路径在周期循环内、daemon 单线程同步；定位服务不返回（服务禁用 / 无 GPS / 驱动异常）即永久卡死且不恢复，此后不再换壁纸；锁屏路径的 `GetFileFromPathAsync().get()` / `SetImageFileAsync().get()` 同理。
- 修复：改用 `GetGeopositionAsync(max_age, timeout)`；无法设超时的调用改为独立线程 + `mpsc::recv_timeout`，或 `SetCompleted` + 有界等待；至少把"可能永久阻塞"写进注释。
- 验收：真机禁用"定位服务"后进程仍能继续周期循环（不卡死）。

### - [x] P1-3 配置读取/校验失败 ⇒ 守护进程静默退出，日志零痕迹

- 位置：`crates/dwall/src/daemon.rs:51`、`:200`；`crates/dwall/src/config.rs:497-503`、`:524`；`crates/dwall/src/bin/daemon.rs:6`
- 证据：
  ```rust
  let mut config = ConfigReader::read_from_path(self.config_watcher.config_path())?;  // 解析/校验失败 ⇒ run() Err ⇒ 进程退出
  fs::write(config_path, toml_string.as_bytes())?;                                   // 非原子写：截断→写入
  ```
- 影响：①写入非原子，GUI 保存与 daemon 轮询存在截断读窗口；空文件会被解析为"默认配置"，并把 `random_state` 清空（`daemon.rs:57-60`）；②致命错误只写 stderr，release 下 `windows_subsystem="windows"` 无控制台 ⇒ **日志里完全看不到死因**，用户只看到"壁纸不动"。
- 修复：读/校验失败保留上一份配置 + `error!` + 退避重试，绝不向上冒泡终止；`has_changed()` 的 IO 错误降级为"本次不判定"；`ConfigWriter::write_to_path` 改为 tmp + `fs::rename` 原子替换；`main` 捕获 Err 并写日志（`daemon.rs`/`main.rs` 两侧）。
- 验收：手动写入非法 `config.toml`（如 `latitude = 200`）后进程存活并持续记录错误。

### - [x] P1-4 单显示器应用失败被完全吞掉（失败原因无日志、不计失败阈值）

- 位置：`crates/dwall/src/theme_engine.rs:142-147`、`crates/dwall/src/daemon.rs:210-215`
- 证据：
  ```rust
  if self.process_single(monitor_id, theme_id, solar_position).is_ok() { success_count += 1; }
  Err(_) => { *consecutive_failure_count += 1; ... }   // 真实错误从未记录
  ```
- 影响：主题被卸载 / ID 拼错 / 图片缺失（最可能的三种失败）既无日志也不计入 `MAX_CONSECUTIVE_FAILURE_THRESHOLD`；叠加 release 下 `debug!` 被编译掉，"为什么不换壁纸"无法诊断。
- 修复：`match` 后 `warn!(error = %e, monitor_id, theme_id, "Failed to apply wallpaper")`；失败数回传计入连续失败；`daemon.rs:213` 的 kv 从 message 里拆出来。
- 验收：故意把主题目录改名后日志出现明确 warn。

### - [x] P1-2 当日随机主题不持久：任何重启都会重抽

- 位置：`crates/dwall/src/daemon.rs:88-111`、`crates/dwall/src/random_state.rs:43`
- 证据：
  ```rust
  let needs_switch = self.random_state.as_ref().map(|s| s.selector.needs_switch(&today)).unwrap_or(true);  // 启动时恒 true
  ```
  `random_state::read()` 只被 `crates/dwall-settings/src/commands/random.rs:29` 使用，daemon 只写不读。
- 影响：叠加 P1-1（apply 会重启）、开机自启、崩溃重启 ⇒ 同一天可反复重抽并覆盖 `random_selection.json`，且 `current_index` 归零破坏"用尽前不重复"。
- 修复：`DaemonApplication::new()` 或进入 Random 分支时先 `read()`，`date == today` 且主题仍存在 ⇒ 直接采用并 `mark_applied`。
- 验收：随机模式下重启 daemon，`random_selection.json` 的 theme 不变。

### - [x] P2-38 生产 release 里 INFO 日志一条都不落盘

- 位置：`crates/logging/src/lib.rs:228-234`、`crates/logging/build.rs:4-6`、`crates/dwall/src/bin/daemon.rs:17`
- 证据：
  ```rust
  if cfg!(debug_assertions) { LevelFilter::Trace } else { LevelFilter::Warn }
  ```
  `max-level-info` 构建时 build.rs **不**注入 `DWALL_LOG`（实证：`target/*/build/logging-*/output` 为空；未开该 feature 的目录含 `cargo:rustc-env=DWALL_LOG=debug`），运行时无 env ⇒ 默认 `Warn`，`log::info!` 在宏层被 `max_level()` 拦掉。
- 影响：release 日志里没有 "Theme cycle completed" 一类主诊断 ⇒ 与 P1-3/P1-4 叠加成"零可观测"。
- 修复：release 默认级别改为 `Info`（`else if cfg!(feature = "max-level-info") { Info }`），或在两个 `main` 里显式 `.with_level(LevelFilter::Info)`；同步修正 README/AGENTS 的描述。

### - [x] P2-39 `has_configuration_changed` 判定变化后不刷新缓存 ⇒ 热插拔最长 5 分钟失效

- 位置：`crates/dwall/src/platform/windows/monitor.rs:522-537`、`:471-473`
- 证据：
  ```rust
  let current_monitors = fetch_system_monitors()?;
  let cache = self.cache.borrow();          // 只读比对，既不更新也不失效
  ```
- 影响：随后 `process_theme_cycle → get_monitors()` 命中 5 分钟缓存 ⇒ 新显示器拿不到壁纸、已拔显示器仍被设置，日志却打印 "Monitor configuration changed, reapplying wallpapers"（误导）。
- 修复：在判定"已变化"的分支里 `self.cache.borrow_mut().update(current_monitors);`（或让 `reload_if_monitors_changed` 先 `refresh_monitors()`）。

### - [x] P2-40 `QueryDisplayConfig` 未处理 `ERROR_INSUFFICIENT_BUFFER`

- 位置：`crates/dwall/src/platform/windows/monitor.rs:148-165`
- 影响：两次调用间显示配置变化（**正是要检测热插拔的场景**）必然返回该错误，而 `reload_if_monitors_changed` 又 `.unwrap_or(false)` 吞掉 ⇒ 热插拔更易漏检。
- 修复：对该错误做有界重试（重新取尺寸，≤3 次）。

### - [x] P2-41 `verify_theme_change` 先重读后 sleep，验证的是陈旧值（并白阻塞 100 ms）

- 位置：`crates/dwall/src/platform/windows/color_scheme.rs:94-108`
- 证据：
  ```rust
  let current_color_scheme = self.get_current_scheme()?;   // ← sleep 之前读的
  if !verify_theme_change(&scheme, &current_color_scheme)? { warn!(...) }
  ...
  std::thread::sleep(Duration::from_millis(100)); Ok(current == expected)
  ```
- 影响：校验等价于不存在，`warn!` 永远只判断"切换前是否已是目标值"；100 ms 真实阻塞单线程 daemon。
- 修复：`sleep; let current = self.get_current_scheme()?; Ok(current == *expected)`。

### - [x] P2-42 `debug!(... ?...)` 让日志语句改变业务行为

- 位置：`crates/dwall/src/platform/windows/wallpaper.rs:134`、`:292`
- 证据：`debug!(path = %uri.DisplayUri()?, "Target lock screen image");`
- 影响：未开 `max-level-info` 的构建里（`build:daemon-stage`/`build:daemon-debug`）`?` 会真正求值 ⇒ 锁屏设置在真正写之前返回 Err；release 被常量折叠 ⇒ debug/release 行为不一致（Heisenbug）。
- 修复：先求值再记录（`let display = uri.DisplayUri().ok(); debug!(path = ?display, ...)`）。

**批次 1 验证**：`cargo fmt --all -- --check`、`cargo clippy -p dwall -p logging -p time --all-features --all-targets -- -D warnings`、`cargo test -p dwall --all-features`。

---

## 批次 2 · 设置后端（进程生命周期 / 自启 / panic / 路径校验）

### - [x] P1-1 随机模式保存候选池会"杀掉引擎且不重启"

- 位置：`crates/dwall-settings/src/theme/applier.rs:18-38`
- 证据：
  ```rust
  terminate_process(pid)?;                                   // 先无条件杀
  writer.write_to_path(&config_path, &config)?;
  if config.monitor_specific_wallpapers().is_none_or(|w| w.is_empty()) { return Ok(()); }  // Random ⇒ None ⇒ 直接返回
  ```
  `Config::monitor_specific_wallpapers()` 对 `WallpaperMode::Random` 恒返回 `None`（`crates/dwall/src/config.rs:360-367`）。
- 影响：随机模式保存后守护进程被杀且永不重启（`src/store/random.store.ts:57-63` 还把状态标成 `applied: true`），而 `src/stage/RandomCommitRow.tsx:1` 注释写明应当"restarts the engine"。用户感知：保存后墙纸不再随太阳切换。
- 修复：抽出 `fn needs_daemon(config) -> bool`（Random ⇒ true；Fixed ⇒ 映射非空），`setup.rs` 与 `applier.rs` 共用同一判定。
- 验收：随机模式保存后 `get_engine_status` 仍为 running。

### - [x] P1-10 自启注册表值未加引号（未加引号路径劫持）

- 位置：`crates/dwall-settings/src/autostart.rs:26-40`、`:50-86`
- 证据：`registry_key.set(Self::APP_NAME, REG_SZ, path_bytes)?;`（裸路径）、`Vec::from_str(exe_path_str)`
- 影响：NSIS 默认 currentUser 安装到 `%LOCALAPPDATA%\Dwall Settings\`（含空格）。Windows 会逐段尝试候选，通常仍能启动，但属经典的**未加引号路径劫持**（在 `%LOCALAPPDATA%` 放 `Dwall.exe` 即可在开机时执行）；`is_enabled()` 用同样未加引号的字符串自比，掩盖差异。
- 修复：写入 `"\"{path}\""`；`is_enabled()` 比较时去除两侧引号以兼容旧值。

### - [x] P1-11 `DAEMON_EXE_PATH.get().unwrap()` 遍布设置端 ⇒ 缺 `dwall.exe` 时整个设置程序 abort

- 位置：`crates/dwall-settings/src/theme/applier.rs:19,53`、`src/commands/system.rs:47`、`src/autostart.rs:22`、`src/theme/status.rs:15`、`src/main.rs:108`
- 影响：`src/setup.rs:31-39` 解析失败只记 `error!` 继续执行 ⇒ 之后任何"应用/停止/自启/查看状态"都会 panic；`panic="abort"` ⇒ 进程直接消失，无任何提示。
- 修复：统一改为返回 `DwallSettingsError::Daemon(...)`，由前端展示；`setup.rs` 解析失败时把状态置为不可用而非静默继续。

### - [x] P1-16 `open_url` / `open_dir` / `theme_id` 无校验：任意程序执行 + 任意目录递归删除

- 位置：`crates/dwall-settings/src/commands/window.rs:19-28`、`src/theme/paths.rs:12-14`、`src/theme/downloader.rs:79-85`、`src/commands/config.rs:14-20`
- 证据：
  ```rust
  pub async fn open_url(url: String) -> ... { open::that(url)?; }   // 无 scheme 白名单
  config.themes_directory().join(theme_id)                         // join 绝对路径会整体替换 base
  if dir.exists() { fs::remove_dir_all(dir).await?; }              // theme_id 完全来自前端
  ```
  `open` 5.4.4 在 Windows 走 `powershell.exe -Command`（`Test-Path` → `Invoke-Item`，否则 `Start-Process`）。
- 影响：`open_url("calc.exe")` 会执行本地程序；`theme_id = "C:\\Users\\x"`/`".."` 会对任意目录 `remove_dir_all`。当前前端只传常量与选择器结果，实际利用前提是渲染层被注入，但命令已注册且无 ACL 限制。
- 修复：`open_url` 只允许 `http/https`；`open_dir` 改枚举 token（config/log/themes）在 Rust 侧解析；`theme_id` 白名单 `[A-Za-z0-9 ._-]`；`themes_directory` 规范化后必须在允许根内；`remove_dir_all` 前再断言。

### - [x] P2-17 网络/代理配置只在启动时读取一次，UI 改动需重启才生效

> 本轮：`HttpClient` 内部改为 `Arc<RwLock<Arc<reqwest::Client>>>`（`Clone` 共享同一句柄），新增 `reconfigure(network)`；`ThemeDownloader`/`ThumbnailCache` 暴露同名方法；`write_config_file` 与 `apply_theme_cmd` 在写盘成功后立即把新的 network 配置推给两个客户端（失败只 `warn!`，不影响保存）。现在改 SOCKS5 地址立即生效，不再需要重开设置窗口。

- 位置：`crates/dwall-settings/src/setup.rs:41-49`
- 影响：`HttpClient`（含 SOCKS5）构造一次后由 `ThemeDownloader`/`ThumbnailCache` 长期持有；镜像模板每次用新 config 生效，但**代理不生效**，同一面板里两个网络设置行为不一致。
- 修复：把 client 放进 `RwLock`/`ArcSwap` 并在写配置后重建；或保存网络配置后提示并 `relaunch()`。

### - [x] P2-14 同步命令在主线程做秒级阻塞调用

> 已改造为 `async` + `spawn_blocking`：`get_engine_status` / `start_engine` / `stop_engine` / `kill_daemon_cmd` / `request_location_permission` / `get_solar_position` / `get_solar_path` / `get_monitors_cmd`。
> 未改造（只读本地小文件，量级可接受）：`get_theme_catalog` / `get_theme_wallpapers` / `match_wallpaper` / `get_app_info` / `check_auto_start`。

- 位置：`crates/dwall-settings/src/commands/solar.rs:14-23,30-34`、`src/commands/engine.rs:15-35`、`src/commands/monitor.rs:10-13`、`src/commands/catalog.rs:53-73,155-175`
- 证据：Tauri 2 非 async 命令在主线程执行（`tauri-macros/src/lib.rs`：async 选项用于"Runs the command on the async runtime instead of blocking the main thread"）；`get_current_position()` 内部同步等待 WinRT，首次还会弹定位授权。
- 影响：`useSolarPosition` 每 60 s、`get_engine_status` 每 5 s（`src/App.tsx:18`）触发；期间窗口不重绘、不处理 IPC。
- 修复：改 `async fn` 或 `#[tauri::command(async)]` + `tauri::async_runtime::spawn_blocking`；引擎状态轮询加节流（≥15 s）并只在窗口可见时轮询。

**批次 2 验证**：`DWALL_SKIP_BUILD=1 cargo clippy -p dwall-settings --all-targets -- -D warnings`。

---

## 批次 3 · 前端

### - [x] P1-6 `Switch` 传入 `onClick` 会整体替换内部处理器 ⇒ 监视器开关完全失效

- 位置：`src/components/ui/switch/Switch.tsx:50-51`；调用点 `src/scope/MonitorCard.tsx:60-65`、`src/scope/UnifiedCard.tsx:27-32`
- 证据：
  ```tsx
  onClick={handleClick}
  {...others}          // splitProps 未提取 onClick ⇒ others.onClick 覆盖 handleClick
  ```
  编译产物 `dist/assets/index-*.js`：`e.$$click=a, z(e, M({...},n),!1,!0)`。
- 影响：点击只执行 `stopPropagation()`，`onCheckedChange` 永不触发，外层卡片点击也被吞 ⇒ 逐显示器"应用"不可用。
- 修复：把 `onClick` 纳入 `splitProps` 并显式合并；调用方改用 `onPointerDown` 挡冒泡。

### - [x] P1-7 全站自定义动画类不存在 ⇒ 所有动效静默失效

- 位置：`src/layout/EngineStatus.tsx:33`、`src/views/SettingsView.tsx:44,60`、`src/views/Body.tsx:13,21`、`src/library/ThemeTile.tsx:42`、`src/stage/WallpaperStrip.tsx:35`、`src/stage/Preview.tsx:33`、`src/stage/PoolCollage.tsx:40`、`src/stage/WriteNote.tsx:39`、`src/stage/RandomCommitRow.tsx:27`、`src/components/settings/SettingsGroup.tsx:16`
- 证据：`src/` 内无任何 `--animate-rise|bdot|breathe|fade-up|pulse-ring` 令牌或同名 `@keyframes`；产物 CSS 中 `.animate-*` 只有 `spin`/`pulse`。
- 修复：在 `src/styles/index.css` 的 `@theme` 补 5 个 `--animate-*` 与对应 `@keyframes`。

### - [x] P1-8 未校验的 `localStorage.locale` 会让应用白屏

- 位置：`src/i18n/index.ts:47-48`、`:59-61`
- 证据：`const savedLocale = localStorage.getItem("locale") as Locale; if (savedLocale) return savedLocale;` + `i18n.flatten(dictionaries[locale()])`
- 影响：`flatten(undefined)` 抛 `TypeError`，发生在模块加载期 ⇒ 启动白屏且无法在 UI 内恢复。
- 修复：`if (savedLocale && savedLocale in LANGUAGES) return savedLocale as Locale;` 并加 `?? dictionaries["en-US"]` 兜底。

### - [x] P1-9 所有 IPC 动作失败对用户完全不可见

> 方案：在 `AppShell` 挂载既有的 `<Toaster position="bottom-right" />`，并在 engine / catalog / settings（`save` 与 `applyWallpaperMode`）的失败路径调用 `toast.error(t("common.operationFailed"), { description })`；新增 i18n 键 `common.operationFailed`（6 个语言文件）。`random.store` 不再重复弹窗（`applyWallpaperMode` 已负责）。

- 位置：`src/store/engine.store.ts:26-28,44,69`、`src/store/random.store.ts:64-66`、`src/store/settings.store.ts:148-155`、`src/store/catalog.store.ts:36-38`、`src/views/SettingsView.tsx:30-36`
- 证据：`random.store.ts:64-66` 是空 `catch`；`settings.store.ts:179-182` 抛出的错误被调用方 `void` 吞成未处理 rejection；生产构建下 `logger` 的 debug/info/warn 是 no-op，release 无 devtools；已有 Toaster **从未挂载**。
- 修复：各 store 增 `error` 字段；挂载 Toaster（`role="status"`/`aria-live`）；失败时展示可重试提示。

**批次 3 验证**：`bunx tsgo --noEmit`、`bunx biome check src`、`bun run build:vite`。

---

## 批次 4 · 下载与缩略图缓存

### - [x] P1-5 下载/解压两次全量驻留内存 + 取消无效 + 无读超时

- 位置：`crates/dwall-settings/src/network/client.rs:64-92`、`src/archive.rs:12,21,23`、`src/theme/downloader.rs:113-120`
- 证据：
  ```rust
  let bytes = response.bytes().await?;                        // 整个 zip 进内存
  if let Some(cancel) = cancel_flag && cancel.load(...) { }   // 取消在读完 body 之后才检查
  let archive = fs::read(zip_path).await ...                  // 解压前再整包读一次
  ```
- 影响：峰值 ≈ 2× 包体；"取消下载"传输期间无效；只有 `connect_timeout(120s)`，无总/读超时 ⇒ 服务器卡住则永久挂起；进度回调只在完成时 `(total,total)`。
- 修复：`bytes_stream()` 分块写盘 + 每块查取消标志 + `.timeout()`；解压改 `by_index` 流式 + `spawn_blocking`，累计写出字节/条目数设上限。

### - [x] P2-44 下载任务注册在错误分支泄漏 ⇒ 该主题永久"正在处理"

- 位置：`crates/dwall-settings/src/theme/downloader.rs:43,52,60-62,66-70`、`src/network/tasks.rs:31-49`
- 影响：`prepare_directory` / `finalize_download` / `extract_zip` / `remove_file` 四处 `?` 提前返回都不 `remove_task` ⇒ 任一步失败后该主题永远报 "already being processed"。
- 修复：用 Drop guard 保证任何返回路径都 `remove_task`。

### - [x] P2-45 Range 续传不校验 `206`/`Content-Range` ⇒ `.tmp` 永久损坏

- 位置：`crates/dwall-settings/src/network/client.rs:56-67`、`src/theme/downloader.rs:102-110,144-150`
- 影响：服务器忽略 Range 返回 200 时整包追加到 `.tmp` 尾部 ⇒ 坏包；失败路径只在 `len()==0` 时删除 ⇒ 该主题再也下载不成功（无 UI 清理入口）。
- 修复：校验 `PARTIAL_CONTENT` + `Content-Range` 起始 offset，否则截断重写；失败路径删除 `.tmp`；用真实 `Content-Length` 计算进度。

### - [x] P2-46 缩略图无内容校验 + 磁盘键不含 URL + 无 single-flight

> 本轮已修：①按 key 的 `tokio::sync::Mutex` 单飞（并发同 key 只下载一次）；②下载后与命中缓存前都做魔数校验（JPEG/PNG/GIF/BMP/WebP/ISO-BMFF），HTML 限流页会被删除并重新下载；③`clear_thumbnail_cache` 现在真的删盘。
> 未做：磁盘缓存键仍是 `(theme, serial, ext)`，不含 URL hash —— 改布局会让既有缓存全部失效，且魔数校验已覆盖实际故障模式，留待后续按需处理。

- 位置：`crates/dwall-settings/src/thumbnail/cache.rs:118-160`、`src/thumbnail/store.rs:34-49`、`src/theme/paths.rs:34-36`
- 影响：只判 `size > 0` ⇒ 镜像返回的 200 HTML 限流页被当图片缓存 30 天（且 `clear_thumbnail_cache_cmd` 前端零调用，用户无法自救）；磁盘键 `(theme,serial,ext)` 不含 URL；`OnceCell` 只 `get/set` 从不 `get_or_init` ⇒ 并发同 key 双份下载并互相截断同一文件。
- 修复：写前校验魔数/解码，失败即删；磁盘键纳入 URL/内容 hash；用 `cell.get_or_init(...)` 单飞；UI 接上清理入口。

### - [x] P2-47 `clear_thumbnail_cache_cmd` 不删磁盘文件（与命令语义不符）

- 位置：`crates/dwall-settings/src/thumbnail/cache.rs:193-213`、`src/thumbnail/cleanup.rs:30-61`
- 影响：只清内存表；磁盘上最多 100 MB 缩略图保留（仅有 30 天/100 MB 规则），用户以为已清理。
- 修复：直接 `remove_dir_all(DWALL_CACHE_DIR/thumbnails)` 后重建，或改名"清理过期缓存"并在 UI 说明。

### - [x] P2-48 更新检查：资源句柄泄漏 + API 不可达时静默回落 github 直连

> 本轮已修：①前端 `update.store` 在重新检查与安装失败时 `pending.close()`，释放 Rust 侧注册的资源（`Update extends Resource`）；②`fetch_latest_release_tag` 加 5 s 连接 / 10 s 总超时（校验更新时用户本来就在等）。

- 位置：`crates/dwall-settings/src/commands/system.rs:121-126`、`src/network/mirror.rs:101-118`
- 影响：`resources_table().add(update)` 的 rid 只增不减（前端构造 JS `Update` 后不 dispose）；镜像解析依赖未认证 `api.github.com`（无超时/无代理/每次新建 Client）⇒ 被墙场景必然失败并回退 github 直链，恰是最需要镜像的场景。
- 修复：检查完成后 `resources_table().close(rid)`；避免依赖 API（显式 tag/缓存）；加超时与代理；失败返回明确错误。

### - [x] P2-49 Range 之外的下载可靠性：重新下载前先删除已安装主题

- 位置：`crates/dwall-settings/src/theme/downloader.rs:52,79-85`
- 影响：任何下载/解压失败都让用户丢掉原本可用的主题；`handle_download_error` 不恢复目录。
- 修复：解压到 `theme_dir.tmp` → 校验 → `remove_dir_all(theme_dir)` + `rename` 原子替换；失败清理临时目录并保留原主题。

**批次 4 验证**：`DWALL_SKIP_BUILD=1 cargo clippy -p dwall-settings --all-targets -- -D warnings`。

---

## 批次 5 · 其余高价值 P2

### - [x] P2-43 `OffsetDateTime` 的比较契约与 i64 回绕

- 位置：`crates/time/src/lib.rs:604-618`、`:404-408`、`:439-441`、`:156-159`、`crates/time/src/math.rs:67-72`
- 证据：`self.utc().ok() == other.utc().ok()`（两者都失败时判**相等**，而 `partial_cmp` 为 `None`）；`assume_utc()`/`timestamp()` 对可能为负的 i64 直接 `as u64` ⇒ 静默回绕。
- 修复：让 UTC 秒成为唯一内部表示（`utc()` 不可失败），或在负值处饱和/返回 `Err` 并让 `eq`/`partial_cmp` 语义一致；`from_timestamp` 加上界校验（253402300799）并返回 `Result`。

### - [x] P2-50 `logging` 的 per-target 级别、`log` re-export、ANSI 与写失败

> 本轮：①`pub use log;` + 宏改 `$crate::log::`（版本分裂不再静默丢失日志）；②`set_max_level` 取全局与各 target 的最大值，`with_target_level` 调高才真正生效（含单测）；③debug 输出的 ANSI 仅在 stderr 是终端时启用，重定向不再写裸转义；④`eprintln!` 改为 `let _ = writeln!(stderr, …)`，写失败不再 panic；⑤release 行格式追加 `at file:line`；⑥`DWALL_LOG`/`RUST_LOG` 支持 `dwall=debug`、`dwall=info,tauri=warn`（含单测）；⑦修正时间戳注释。

- 位置：`crates/logging/src/lib.rs:104-107`、`:66-73`、`:152`、`:171-190`、`crates/logging/src/macros.rs:11,20`
- 问题：①`set_max_level(self.level)` 是全局上限，`with_target_level` 想调**高**永远无效（且全仓 0 调用点）；②宏用 `::log::`，版本分裂即静默无输出（应 `#[doc(hidden)] pub use log;` + `$crate::log::`）；③debug 路径 `eprintln!` 写失败会 panic，release `Stderr` 分支在 GUI 子系统下静默丢日志；④ANSI 无 `is_terminal()` 判断（重定向/未开 VT 的 conhost 下乱码）；⑤`RUST_LOG=dwall=debug` 解析失败静默回落；⑥时间戳注释与实际格式不符。

### - [x] P2-51 设置 crate 零单元测试

> 本轮新增 14 个单测：`network/mirror.rs`（`resolve_raw`/`apply_template` 模板替换与缺占位符回落）、`filesystem.rs`（`validate_theme_id` 的穿越/分隔符/长度、`validate_directory` 绝对路径与 `..`、目标位于源目录内）、`thumbnail/cache.rs`（`looks_like_image` 六种格式 + HTML/空响应、`get_url_extension`）。运行：`cargo test -p dwall-settings --bins`（该 crate 只有 bin target）。
> 仍缺：`cleanup.rs` 的容量/过期计算、`process.rs` 的 `is_same_process`（后者受 `cfg(debug_assertions)` 影响，需重构为可注入参数）。

- 位置：`crates/dwall-settings/src/**`（无 `#[test]`）
- 应补：`MirrorUrlResolver::{resolve,resolve_raw}`、`get_url_extension`（`src/thumbnail/cache.rs:164-178`）、缓存容量/过期计算（`src/thumbnail/cleanup.rs`）、`is_same_process`（`src/process.rs:45-80`）。

### - [x] P2-52 capability/插件最小权限

> 本轮：删除 `tauri-plugin-shell`（依赖 + `init()` + capability 里的 `shell:default`，全仓无调用）；capability 去掉未被使用的 `core:window:allow-start-dragging/minimize/close`，`dialog:default` 收窄为 `dialog:allow-open`；`reqwest` 显式声明 `native-tls`、`tokio` 显式声明 `rt-multi-thread/fs/sync/time/io-util`（此前依赖未使用的 shell 插件间接提供 `time` 特性）。已用 `cargo clean -p dwall-settings` + 真实 `tauri_build` 重跑验证 capability 合法。

- 位置：`crates/dwall-settings/capabilities/default.json:8-19`、`src/main.rs:50-52`、`Cargo.toml:39`
- 问题：`shell:default`（含 `allow-open`）与 `core:window:allow-start-dragging/minimize/close` 全无人使用；`dialog:default` 附带 `allow-message/save`。
- 修复：删除 shell 插件依赖/注册与对应权限；`dialog` 按需仅列 `allow-open`。

### - [x] P2-28 / P2-29 / P2-35 CI 门禁

- 位置：`.github/workflows/pr-check.yml:88`、`:120-129`、`:144-157`；`package.json:21`
- 问题：①`biome check --write` 在 CI 里自动改码再返回成功（且 `biome` 未声明在 package.json，依赖 `biomejs/setup-biome` 或本机全局安装）；②`target/debug/{deps,build}` 缓存实测约 39 GB（上限 10 GB）必然失效；③触发路径漏 `index.html`/`.cargo/**`/`tsconfig.node.json`；④`automerge` 在检查 job 被路径过滤跳过时 `!failure()` 仍成立；⑤CI 不跑 `bun run test`（而该命令本身已坏）。
- 本轮：①`check` 改为只读（`tsgo --noEmit && biome check src`），新增 `fix` 脚本负责 `--write`，并把 `@biomejs/biome` 与 `jsdom` 正式写入 devDependencies（bun.lockb 已同步）；②`test` 改为 `vitest --run` 并有真实测试，CI 的 frontend-check 增加 "Run unit tests" 步骤；③触发路径与 filters 补 `index.html`、`tsconfig.node.json`、`.cargo/**`；④`auto-merge` 条件改为「detect-changes 成功，且两个检查要么 success 要么 skipped」，不再用 `!failure()`。
- 未改（附理由）：`target/debug/{deps,build}` 缓存路径 —— 本机 39 GB 是多种 feature/多次构建累积的结果，CI 冷构建的 debug target 通常仍在 10 GB 上限内；删掉 `deps` 会让缓存失去意义。彻底解决建议改用带自动裁剪的 `Swatinem/rust-cache`（属供应链决策）。

### - [x] P2-30 release/nightly 构建的日志级别不一致

- 位置：`.github/workflows/nightly.yml:312`、`crates/dwall-settings/tauri.debug.conf.json`、`package.json:10-11`
- 问题：nightly 用 `build:daemon-debug`（无 `max-level-info`）⇒ build.rs 注入 `DWALL_LOG=debug`，nightly 默认 debug 级落盘（含完整配置/坐标），与 README"默认 warning"矛盾。
- 结论：nightly 保留 verbose 是有意的（便于排障）；真正的危害是**坐标等个人信息默认入日志**，已由 P3-24 修复（坐标与完整配置降到 debug，info 只留非敏感摘要）。README 已同步说明 release 默认 `info`、debug/trace 被编译掉。

### 批次 5 追加（本轮一并完成）

- [x] **P1-12 未同意遥测**：`tracker.rs` 的 `POST https://app.thepoy.cc/api/track/dwall` 每次启动无条件发送、不走用户代理、无超时、无同意，且与 `docs/shared-comparison-content.md:128` 的 "No data transmission" 矛盾。现改为显式 `const TELEMETRY_ENABLED: bool = false`（附文档说明：若恢复必须做成设置项 opt-in 并走 `HttpClient` 以尊重代理）。
- [x] **P1-13 `windows` crate 双版本**：`crates/time` 的 `version = "0"` 解析到 0.62.2，与 dwall/settings 的 0.61.3 并存（`windows`/`windows-core`/`windows-future` 各两份，生成的绑定被编译并链接两次）。最终采用**跟随 Tauri 升到 0.62**：`windows = { version = "0.62" }` 提到 `[workspace.dependencies]`、三处 `workspace = true`，并把 `windows-future` 一并升到 `0.3`（与 `windows` 0.62 的 `windows-core` 0.62 配套）。实测 `cargo tree --workspace` 中每个 windows 家族 crate 只剩一个版本：`windows 0.62.2` / `windows-core 0.62.2` / `windows-future 0.3.2` / `windows-result 0.4.1` / `windows-strings 0.5.1`，且 tao（tauri-runtime-wry）也统一到 0.62.2。迁移中修掉两处 API 变更：`windows-future` 0.3 用 `join()` 取代 `get()`（本项目改用自带有界等待，不再调用二者），`windows-core` 0.62 把 `Error::from_win32()` 改名为 `from_thread()`（`monitor.rs` 改为用函数**返回**的错误码构造：`Error::from_hresult(WIN32_ERROR(result).to_hresult())`，比读 `GetLastError` 更准确）。
- [x] **P1-15b 锁屏 WinRT 无界等待**：`StorageFile::GetFileFromPathAsync(…).get()` 与 `SetImageFileAsync(…).get()` 走 `windows-future` 的 `INFINITE` 等待，broker 卡住即永久挂死守护进程（锁屏开关默认开启）。新增 `wait_bounded`/`wait_action_bounded`：轮询 `Status()` + 30 s 上限（轮询而非完成回调，因为 `StorageFile` 不是 `Send`，无法跨线程投递结果）。为此显式依赖已在依赖树中的 `windows-future = "0.2"`（WinRT 异步类型不在 `windows` 里）。
- [x] **P3-3 时间/索引溢出**：`days_to_ymd` 接收 `u64` 并把 days 夹到 9999-12-31（`ymd()` 不再 u32 截断），`end_of_day` 用 `saturating_add`；`wallpaper_image_path`/主题校验的 `index + 1` 改为 `u16::from(index) + 1`（含单测）。
- [x] **P3-4 NaN panic**：`WallpaperSelector::find_closest_entry` 由 `partial_cmp().unwrap()` 改为 `total_cmp`（TOML 允许 `altitude = nan`，此前会让守护进程 abort），含 NaN 单测。
- [x] **P3-5 `unreachable!()`**：`GeolocationAccessStatus` 是 `struct(i32)+常量`，通配分支可达；改为 `warn!(status = other.0)` + 返回 `Unspecified` 错误。
- [x] **P3-19 `UtcDateTime::now()` panic**：改为 `unwrap_or_default()`（时钟回拨到 1970 前时饱和到 epoch 而不是 panic——日志每条记录都会取当前时间，panic 等于带崩守护进程）。
- [x] **P3-24 个人信息入日志**：`geolocation.rs` 的经纬度/海拔与 `config.rs` 的完整 `Config`（含坐标与代理）从 `info!` 降到 `debug!`，`info!` 只保留 interval/格式/开关等非敏感摘要。
- [x] **P3-32 依赖特性"蹭"用**：见 P2-52。

### - [x] P2-31 Google Fonts 运行时外链

> 本轮：移除运行时注入的 Google Fonts `<link>`（删除 `src/utils/fonts.ts` 与 i18n 中的调用），CSP 去掉 `https://fonts.googleapis.com` 与 `fonts.gstatic.com`。
> **视觉影响需知**：拉丁文案从 Inter 回落到系统 UI 字体（`styles/i18n.css` 的 `--font-*` 仍是 `"Inter", system-ui, …`，装了 Inter 的机器不变）；CJK 用户此前的 webfont 在中国大陆本就取不到、实际已走系统字体，因此对目标用户群无变化。若确定要统一字体，应改为随包内置子集字体（会增加体积，与"小体积"卖点冲突，属产品决策）。

- 位置：`src/utils/fonts.ts:55-58`、`crates/dwall-settings/tauri.conf.json:13`
- 影响：中国大陆不可达（正是镜像功能的受众）、离线首屏回退、泄露语言/IP；`FONT_MAP`/`i18n.css` 还维护 13 种语言但只发布 6 种。
- 修复：字体改为随包内置（或移除 Google Fonts 依赖，仅用系统字体栈），同步收紧 CSP 到 `'self'`。

### - [x] P2-32 assetProtocol scope 过宽 + 窗口约束

> 本轮：去掉 `requireLiteralLeadingDot: false`（默认 true 后 `**/*` 不再匹配点开头路径，`.ssh` 之类目录不可读），功能无损失。
> 未改（附理由，属产品决策）：①scope 无法收窄到具体目录 —— `themes_directory`/`customized_themes_directory` 允许用户任选盘符，`**/*` 是「可配置目录」的直接后果，要收窄得先把壁纸/缩略图改成自定义协议或命令流式读取；②`transparent + maximized + resizable(false) + min_inner_size(1024,640)` 在 1366×768@125% 下会溢出工作区且无法调整，但改 `resizable` 可能破坏固定布局，需设计确认。

- 位置：`crates/dwall-settings/tauri.conf.json:14-20`、`src/window.rs:27-34`
- 问题：`allow: ["**/*"]` + `requireLiteralLeadingDot: false` ⇒ 渲染层可读任意文件；`transparent(true)` + `maximized(true)` + `resizable(false)` + `min_inner_size(1024,640)` ⇒ 高 DPI 小屏（1366×768@125%）内容被裁切且无法调整。
- 修复：scope 收窄到实际目录；评估 `resizable`/`min_inner_size` 与透明窗口的取舍。

### - [x] P2-33 / P2-34 / P2-36 / P2-37 一致性与文档

> P2-34 ✅ `package.json` 的 `license` 由 `MIT` 改为 `AGPL-3.0`，与 `LICENSE` 及 workspace `Cargo.toml` 一致。
> P2-37 ✅ README（中英）修正：运行日志等级与文件位置（默认 `info`、`DWALL_LOG` 支持的写法、release 编译掉 debug/trace、日志实际在 `%LOCALAPPDATA%\com.thep0y.dwall\log` 且有 backup 轮转）；FAQ#3 由"不支持自定义壁纸"改为自定义主题的正确说明（指向 `docs/custom-themes.md`）；FAQ#1 关于"设置端无法及时获知守护进程状态"的过时描述改为轮询 + 读日志尾部。`AGENTS.md` 修正：`crates/time` 不是"零依赖"、`--all-features` 会重写 `src/ipc/bindings/**`（需提交/跑 typegen）、设置 crate 的单测命令需 `--bins`、`check`/`fix` 脚本语义。
> P2-36（部分）新增守护进程单测：`WallpaperSelector` 最近匹配/方位角环绕/NaN 不 panic/空表、`wallpaper_image_path` 的 1-based 与 `u8::MAX` 加宽、`DailyRandomThemeSelector::restore`；引擎级（含 `ThemeEngine` 泛型化）仍待办。
> P2-33 未做：NSIS 语言列表（SimpChinese/English → 需加 Japanese/Korean/TradChinese）必须跑一次真实 bundle 才能验证语言名合法，属发布链路改动，留待有 NSIS 环境的批次。

- `crates/dwall-settings/tauri.conf.json:38`：NSIS 仅 SimpChinese/English，而 UI 支持 6 语言。
- `package.json:25`（MIT）vs `Cargo.toml:15` + `LICENSE`（AGPL-3.0）冲突。
- 测试缺口（daemon 侧）：`DaemonApplication`/`ThemeEngine`/`ConfigWatcher`/随机模式切换与持久化/`WallpaperSelector` 全无测试；`crates/dwall/tests/trait_mocks.rs` 只验证 mockall 自身。
- README 漂移：FAQ#3 称"不支持自定义壁纸"（实际支持）、日志等级描述与实现不符、`docs/shared-comparison-content.md:128` 的 "No data transmission" 与 `tracker.rs` 行为矛盾。
- 生成物漂移：跑 `cargo test -p dwall --features ts --lib export_bindings` 会改写 `src/ipc/bindings/WallpaperMode.ts`（Rust 文档注释已改英文、提交的绑定仍是中文）⇒ 绑定未同步，CI 无校验。

---

## 批次 6 · P3 / 加固清单（择要，可批量处理）

| ID | 位置 | 问题 |
| --- | --- | --- |
| P3-1 | `crates/dwall/src/color_scheme.rs:345-361` + `utils.rs:165-169` | `ThresholdConfig`/`DaylightState` 缓存 24h 且配置重载不清 ⇒ 改经纬度后仍用旧位置；`DaylightState` 跨午夜沿用前一天 |
| P3-2 | `crates/dwall/src/solar.rs:191` | `cache_minutes` 未校验 ⇒ `Instant + Duration` / 乘法溢出 panic（abort） |
| P3-3 | `crates/dwall/src/theme.rs:143` | ~~帧索引 255 触发 `u8 + 1` 溢出（release 静默取 `0.jpg`）~~ **已修（批次 5）**：`u16::from(index) + 1` + 单测；`crates/time` 的 `ymd()`/`days_to_ymd`/`end_of_day` 同步做夹取与饱和 |
| P3-4 | `crates/dwall/src/wallpaper.rs:44-53` | ~~`partial_cmp().unwrap()`：TOML 允许 `nan`，daemon 从不调用 `ThemeValidator` ⇒ panic~~ **已修（批次 5）**：改 `total_cmp` + NaN 单测 |
| P3-5 | `crates/dwall/src/theme.rs:462-472` | ~~池成员变化后 `select_next` 返回已移除主题~~ **已修（批次 1/5）**：`restore` 只接受仍在候选集内的主题；`select_next` 的成员变化问题仍在（见下方 P3-5b） |
| P3-5b | `crates/dwall/src/theme.rs:462-472` | `select_next` 只在索引越界时重建乱序表，池**成员**变化后可能返回已移除主题 → 待办 |
| P3-6 | `crates/dwall/src/daemon.rs:200-223` | 配置检测与墙纸周期共用 `sleep(interval)` ⇒ 最长 1 h 才生效 |
| P3-7 | `crates/dwall/src/config.rs:532-569` | ~~首次 `has_changed()` 恒 true ⇒ 启动必多一轮重建 + 误导日志~~ **已修（批次 1）**：`run()` 启动时先建立 mtime 基线 |
| P3-8 | `crates/dwall/src/lazy.rs:10-87` | `LazyLock` 内 `panic!`；且目录日志发生在 `Logger::init()` 之前必然丢弃 |
| P3-9 | `crates/dwall/src/theme.rs:214-231` | 主题解析缓存无 mtime 校验 ⇒ 原地更新后永久用旧数据 |
| P3-10 | `crates/dwall/src/platform/windows/geolocation.rs:93-118` | 每周期调用会触发系统权限 UI 的 `RequestAccessAsync`，应改 `GetGeolocationStatus` |
| P3-11 | `crates/dwall/src/platform/windows/geolocation.rs:116` | `_ => unreachable!()` 匹配 `struct(i32)` 常量类型 ⇒ 未映射值即 abort |
| P3-12 | `crates/dwall/src/platform/windows/monitor.rs:540-575` | 友好名缓存无上限，且把一次性失败的 `"Display N"` 永久缓存 |
| P3-13 | `crates/dwall/src/platform/windows/monitor.rs:377-406` | 固定 512 B 友好名缓冲 + `take()` 静默截断 |
| P3-14 | `crates/dwall/src/platform/windows/monitor.rs:582,84,105-108` | `position_index` 是易变遍历序号却导出 API；`adapterId` 只取 `LowPart` |
| P3-15 | `crates/dwall/src/platform/windows/wallpaper.rs:47-50,307-315` | `S_FALSE` 时漏 `CoUninitialize`；`Drop` 里先 `CoUninitialize` 再释放接口（顺序反） |
| P3-16 | platform 全层 24 处 | `unsafe` 无一条 `// SAFETY:`；`time/src/month.rs:43` 的 `# Safety` 写给了安全函数 |
| P3-17 | `crates/dwall/src/platform/windows/color_scheme.rs:40-71` | 只读 `AppsUseLightTheme` 判"已设置" ⇒ `SystemUsesLightTheme` 漂移时永不修复 |
| P3-18 | `crates/dwall/src/platform/windows/registry.rs:85-118,50-63,106` | 只支持 HKCU、暴露裸指针两段式 API、无 `ERROR_MORE_DATA` 重试、打印入参指针 |
| P3-19 | `crates/time/src/lib.rs:69-71` | ~~`now()` 用 `expect` 而非已有的 `Error::SystemTime`（该变体是死代码）~~ **已修（批次 5）**：饱和到 epoch 不再 panic |
| P3-20 | `crates/time/src/lib.rs:299-319` | `is_before/is_after/is_between_*` 秒截断，与派生 `Ord` 不一致 |
| P3-21 | `crates/time/src/lib.rs:629-639`、`color_scheme.rs:216-222` | `Date::new` 不校验；`with_hms` 用当前偏移解释整日 ⇒ DST 日偏 1 h |
| P3-22 | `crates/logging/src/lib.rs:94-102` | 日志无轮转、`backup/` 每次启动新增且从不清理；启动失败 `?` 终止守护进程 |
| P3-23 | `crates/logging/src/lib.rs:126-128` + `time/src/offset/windows.rs:6-17` | 每条记录一次 `GetTimeZoneInformation`；`TIME_ZONE_ID_INVALID` 静默按 UTC |
| P3-24 | `crates/dwall/src/platform/windows/geolocation.rs:84-89` + `config.rs:504` | ~~经纬度/完整配置以 INFO 明文写入日志（隐私）~~ **已修（批次 5）**：降到 `debug!`，`info!` 只保留非敏感摘要 |
| P3-25 | `crates/dwall-settings/src/filesystem.rs:81-146` | 迁移无进度/不可中断；失败留半个目标目录（**目标位于源目录内已修，批次 2**） |
| P3-26 | `crates/dwall-settings/windows/hooks.nsi:42-43,55-57` | 升级删缩略图缓存与日志（壁纸包在 `%APPDATA%` 不受影响）；卸载残留 `com.thep0y.dwall` |
| P3-27 | `crates/dwall-settings/src/commands/window.rs` + `src/App.tsx:14` + `src/main.rs:53-63` | release 隐藏窗口无兜底；单实例回调不 `show()/unminimize()`；插件注册顺序违反其 README |
| P3-28 | `crates/dwall-settings/src/theme/paths.rs:12-14` vs `src/commands/catalog.rs:138-142` | 主题目录两套真相，同名自定义主题被静默遮蔽 |
| P3-29 | `crates/dwall-settings/src/thumbnail/storage.rs:21-25` + `cleanup.rs:109-116` | "touch 保活"只改 atime，淘汰看 mtime ⇒ 保活无效，非 LRU |
| P3-30 | `crates/dwall-settings/src/daemon.rs:54-79` | ~~`get_last_daemon_error` 读错目录 + 按 JSON 解析纯文本日志 ⇒ 永远 `None`~~ **已修（批次 2）**：改读 `DWALL_LOG_DIR` 并按纯文本取最后一条 ERROR |
| P3-31 | `crates/dwall-settings/src/network/mirror.rs:31`、`src/tracker.rs:10`、`src/commands/theme.rs:104`、`src/commands/config.rs:15` | kv 日志约定违背；`debug!(config = ?config)` 打印全量配置 |
| P3-32 | `crates/dwall-settings/Cargo.toml:24,27` | tokio `time` 特性只由未使用的 shell 插件提供；`reqwest` TLS 靠 updater 的 native-tls 间接开启 |
| P3-33 | `crates/logging/src/macros.rs.bak` | 118,492 B 遗留文件（tracing 的 macros.rs），被 `.gitignore *.bak` 挡住未入库 |
| P3-34 | `Cargo.toml`（workspace）+ `crates/time/src/math.rs:13` | 未声明 `rust-version`（`u16::is_multiple_of` 需 1.87；edition 2024 只要求 1.85） |
| P3-35 | `crates/dwall-settings/tauri.conf.json:32`、`capabilities/default.json:2` | `icons/icon.icns` 不存在；capability 引用被 gitignore 的 `../gen/schemas/...` |
| P3-36 | `src/components/ui/**`、`src/styles/toast.css` | 7 个 UI 模块（约 2394 行）零引用；747 行 toast.css（≈16 KB）随包发布但 Toaster 未挂载；Tailwind 被 `@import` 3 次导致 preflight 重复 |
| P3-37 | `src/index.css:12-18`、`src/index.tsx:9` | 全局 `user-select: none` + 屏蔽右键菜单 ⇒ 路径/错误信息/版本号无法复制 |
| P3-38 | `src/components/ui/label/Label.tsx:11-19`、`field/FieldLabel`、`input/Input.tsx:12-15`、`biome.json:26-35` | label 未与控件关联；Switch 无键盘；4 个可点击 div 无 role；biome 恰好关闭了对应 a11y 规则 |
| P3-39 | `src/App.tsx:18`、`src/store/engine.store.ts:31-72` | 每 5 s 全进程快照轮询；启停无 busy 态、乐观更新 |
| P3-40 | `src/components/settings/PathsSection.tsx:126-132`、`src/layout/FixedCommit.tsx:22-25`、`src/library/LibraryColumn.tsx:53-58`、`src/components/settings/AboutSection.tsx:21,33` | 装饰性"链接"（假可供性）；FixedCommit 主题名与 `isApplied` 回退链不一致；无空态/加载态；错误态显示 "…" |
| P3-41 | `src/ipc/config.ts:7-8`、`src/ipc/system.ts:9` | 类型契约不符（声明 `Config`/`void`，实际 `()`/`Option<u32>`） |
| P3-42 | `src/store/settings.store.ts:57-59`、`src/components/settings/ThemeSegmented.tsx:26` | `auto_detect_color_scheme` 只读不写 ⇒ 外观偏好下次启动被覆盖 |
| P3-43 | `src/components/settings/EngineSection.tsx:53-54` | `okShow={flash() === "startup"}` 永不触发（`ok("startup")` 从未调用） |
| P3-44 | `src/stage/SunPathPanel.tsx:184-191` | 太阳标记点命中区仅 7 px（远低于 24 px 目标） |
| P3-45 | `src/library/ThemeTile.tsx:19` + `src/hooks/useMediaQuery.ts:9-15` | 每个磁贴各自一个 `matchMedia` 监听 |
| P3-46 | `src/hooks/useScrollEdges.ts:44`、`src/components/ui/tooltip/TooltipContent/useTooltipContent.ts:138` | 只 observe `firstElementChild`；rAF 未取消 |
| P3-47 | `crates/dwall-settings/src/thumbnail/error.rs`、`src/thumbnail/store.rs:30-31` | `ThumbnailError` 无人构造（死代码）；内存缓存只增不减 |
| P3-48 | `crates/dwall-settings/src/theme/download_url.rs:8-13` vs `commands/catalog.rs:58` | 主题包名 `Big.Sur.zip` vs 缩略图目录 `BigSur`（需一次联网核实） |
| P3-49 | `.github/workflows/*.yml`、`.gitignore`、`README.md` | action 用浮动 tag；无 `cargo audit`/`deny.toml`/`rust-toolchain.toml`/`SECURITY.md`；`.scripts/mirror-latest-json.js` 无引用；`bunfig.toml` 硬编码 npmmirror；仅二进制 `bun.lockb`；`publish.yml:101-111` 只 sed `Cargo.toml` 不同步 `Cargo.lock`。`.gitattributes` **已补**（`* text=auto eol=lf` + 二进制资产 + NSIS 保持 CRLF）：此前 Windows runner 的 `core.autocrlf=true` 让 checkout 出 CRLF，而旧的 `biome check --write` 会把它们改回去并返回成功，问题因此一直隐藏；改成只读门禁后立刻暴露（`src/App.tsx` 等未改动文件也报格式错误）。已在全新 worktree 中验证 `tsgo`/`biome`/`vitest` 全绿 |

---

## 附：非缺陷结论（避免重复排查）

- 太阳位置数学经独立复算正确（Beijing 2026-06-21 日出 04:47 / 日落 19:47）；`crates/time` 的 `assume_utc()` 是**真换算**（`local - offset`），与标准 `time` crate 语义不同但此处用法正确——**无时区 bug**。
- 守护进程侧缓存全部有界：主题角度 LRU=32（`crates/dwall/src/theme.rs:73`）、显示器 300 s（`monitor.rs:412`）、typed cache 固定 4 槽 TTL；命中返回 `Rc` 不克隆角度表。
- 平台层幂等：相同墙纸/锁屏图跳过、颜色方案先比后写，避免每周期闪烁与全系统广播。
- zip 解压经 `ZipFile::enclosed_name` / `safe_prepare_path` 清洗，**无 zip-slip**（真正的缺口是流式化与尺寸上限）。
- `LOG_MAX_LEVEL_INFO` 的编译期剔除是零开销且保留类型检查（优于 `cfg!`），kv 宏参数只求值一次。
- 只有 `src/ipc/**` 引入 Tauri；`models/` 保持纯类型；业务逻辑全在 Rust；`thiserror` + `#[from]` 统一；配置迁移有 20+ 单测 + insta 快照。
- i18n：6 个语言 key 与占位符完全一致，且用 `dict: RawDictionary` 让缺键成为编译错误。
- 前端 positioner 生命周期干净（监听器/ResizeObserver/rAF/定时器均有清理）；Tabs 实现符合 `DESIGN.md`。
- capability 仅 main 窗口 + `updater:deny-check`；无 `dangerous_*`；无硬编码密钥；`dist`/`target`/`node_modules`/`.qwen` 未入库。
- 未验证项：`GetGeopositionAsync()` 无参重载是否带平台默认超时；STA 与原生等待是否死锁（需真机复现）；`GetWallpaper` 是否可能返回静态字符串（现有证据下可能性极低）；发布侧远端资产命名。
