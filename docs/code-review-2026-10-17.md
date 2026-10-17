# Dwall 代码审查（2026-10-17）

> 本文是**审查报告**，不是待办清单的替身。前一份清单
> [code-review-2026-10-15.md](./code-review-2026-10-15.md) 的批次 6（P3）当时标为 ⏳ 未完成，
> 本文第 7 节逐条复核了那批遗留项，第 1–6 节是新发现与各维度专项结论，第 8 节是 Rust 侧还缺的测试用例。

## 0. 审查方法与实测基线

范围：`crates/dwall`、`crates/dwall-settings`、`crates/logging`、`crates/time`、`src/**`（SolidJS）、
`Cargo.toml`/`package.json`、`.github/workflows/**`、`.scripts/**`、`.packaging/**`、`docs/**`。
四路并行评审（守护进程核心 / Tauri 设置后端 / SolidJS 前端 / 构建·体积·CI）+ 交叉复核。

**本轮实测（只读命令，工作树 = HEAD + 未提交的版本号提升到 `0.3.0-alpha.2`）**

| 项 | 结果 |
| --- | --- |
| `cargo fmt --all -- --check` | exit 0 |
| `cargo clippy -r -p dwall --features build-script,log-max-level-info --all-targets` | exit 0，0 警告 |
| `cargo test -p dwall -p logging -p time` | **196 passed / 0 failed**（dwall lib 101 + bin 5 + 集成 7 + logging 3 + time 80） |
| `target/release/dwall.exe`（当前源码，FileVersion `0.3.0-alpha.2`） | **632,832 B（618.0 KiB）** |
| `target/debug/dwall.exe` | 2,741,248 B → release 小 **4.33×** |
| `target/release/dwall.exe`（0.2.5，前一份清单实测） | 600,576 B（586.5 KiB） → **+5.4%** |
| `target/release/dwall-settings.exe`（0.3.0-alpha.2） | 4,234,240 B（4.04 MiB）；debug 21,730,816 B |
| NSIS 安装包 `Dwall Settings_0.3.0-alpha.2_x64-setup.exe` | 1,806,833 B（0.2.5 = 1,756,558、0.1.33 = 2,070,883） |
| `dist/`（2026-10-17 12:53 构建） | 384,534 B：JS 231,251 / CSS 121,900 / SVG 30,781 / html 602 |
| `dist/`（0.2.5 基线） | 360,866 B → JS **+21 KB（+10%）**、CSS +2.4 KB |
| `dist-daemon/Dwall_0.2.5_x64-daemon.zip` | 359,401 B（351 KiB）+ `.sha256`（校验通过；内含 5 条 SHA256SUM 全部重算 OK，无 BOM） |
| `dist-manifests/` | 3,373 B，**内嵌的 sha 与磁盘上的 zip 不一致**（见 6.3） |
| `target/release` / `target/debug` | 17.93 GB / **61.51 GB**（本机累计 79.4 GB，含 2,361 个 rlib = 5.0 GB） |
| 依赖（`cargo tree`） | daemon 正常图 **39** 个 crate-version（含 build-deps 52）；`Cargo.lock` 514 个包；**daemon 图无重复版本**，GUI 图有 23 组重复 |
| 运行中的 `dwall.exe`（已安装 0.3.0-alpha.4） | **1 线程 / 244 句柄 / Private 2.48 MB / 私有工作集 1.38 MB / 工作集 21.25 MB** |
| 上者 75 秒（约 5 个周期）后复测 | 244 句柄 / 2.477 MB **完全不变**，PeakWorkingSet 21.61 MB |

**图例**：✅ 本轮已逐行独立复核 · ○ 专项评审结论（关键项已抽样复核） · ⚠ 未验证 / 需真机确认。
严重度：**P1** 必修（崩溃 / 数据损失 / 功能不可用）· **P2** 应修 · **P3** 加固与清理。

---

## 1. 最严重的问题（按影响排序）

| ID | 位置 | 问题（失败模式） | 修法 |
| --- | --- | --- | --- |
| **D1** ✅ | [daemon.rs:374-380](../crates/dwall/src/daemon.rs#L374-L380) + [geolocation.rs:69-71](../crates/dwall/src/platform/windows/geolocation.rs#L69-L71) | **连续 3 次周期失败即 `std::process::exit(1)`，而"定位被拒绝/定位服务关闭"是持续性错误。** 默认配置是 `PositionSource::Automatic`（[config.rs:81-88](../crates/dwall/src/config.rs#L81-L88)），因此**未授权定位的机器上守护进程启动约 45 秒后永久退出**，开机自启后不会自行恢复（只有用户在设置里点"启动引擎"）。用默认 `interval`=15 s 计，3 次失败 = 45 s；`interval` 允许到 1 s，此时 **3 秒**即退出。 | 持续性错误不应终止进程：改为指数退避 + 上限后继续运行（保留失败计数用于日志），或只在"配置/环境不可用"时退出并让设置端负责重启。 |
| **D2** ✅ | [settings.store.ts:67-69](../src/store/settings.store.ts#L67-L69) + [SettingsView.tsx:30-85](../src/views/SettingsView.tsx#L30-L85) | **`patch`/`patchPosition` 在 `config === null` 时把 config 写成 `{}`，随后整个设置页抛 TypeError 崩溃。** 只有标题栏被 `<Show when={status === "ready"}>` 包住，四个 section 无条件渲染（`SettingsView.tsx:82-85`）；任意 Switch 在配置加载完成前（或**加载失败后永久**）被点击即触发。`config` 变成 `{}` 后 [EngineSection.tsx:114](../src/components/settings/EngineSection.tsx#L114)、`:123` 的 `config?.position_source.type` 会解引用第二跳 → `TypeError`；全仓**没有任何 `ErrorBoundary`**（grep 0 处）。 | `patch`/`patchPosition` 在 `!prev` 时直接 return；并给 section 加 `status === "ready"` 门（或 ErrorBoundary）。 |
| **D3** ✅ | [settings.store.ts:58-64](../src/store/settings.store.ts#L58-L64) | **配置读取失败对用户完全不可见。** `error` 字段被写入但**从未被任何代码读取**（全仓 grep 只有 `status === "ready"` 一处消费）。用户看到的是一个"所有开关都是关的、主题目录为空"的正常界面，`config` 永远 `null` → D2 变成可反复触发的稳定缺陷。 | 展示错误 + 重试按钮（catalog.store 已有 toast 先例）。 |
| **S1** ✅ | [commands/download.rs:11-25](../crates/dwall-settings/src/commands/download.rs#L11-L25) → [downloader.rs:68-124](../crates/dwall-settings/src/theme/downloader.rs#L68-L124) | **下载主题时对 `themes_directory` 不做校验**（`write_config_file`/`apply_theme_cmd` 都校验了）：渲染层给的 `config.themes_directory` 直接用于 `remove_dir_all(staging)` → 解压 → `remove_dir_all(theme_dir)` → `rename`。校验不一致是事实（唯一防线 `validate_directory` 本身也只是一条"绝对路径 + 无 `..`"的弱规则）。 | 两处路径同样调用 `validate_directory`；更根本的是把可写根目录收窄为白名单。 |
| **S2** ✅ | [downloader.rs:96-127](../crates/dwall-settings/src/theme/downloader.rs#L96-L127) + [archive.rs:64](../crates/dwall-settings/src/archive.rs#L64) | **解压成功 ≠ 主题有效，却直接覆盖已安装主题。** `extract_unwrapped_root_dir` 只在"所有条目共享同一顶层目录"时才剥壳（过滤 `.DS_Store`/`Thumbs.db`/`__MACOSX`）；发行包多一个顶层 `README.md` 就会得到 `staging/<Theme>/solar.json`，交换后旧主题被删、新主题多一层嵌套 → 主题从库里消失、守护进程每周期 warn，而命令返回 `Ok`，UI 报成功。解压后没有任何 `ThemeValidator::validate`。 | 交换前断言 `staging` 根下存在 `solar.json`/`theme.toml` 并通过 `ThemeValidator`；失败则保留旧主题。 |
| **S3** ○ | [thumbnail/store.rs:27-45](../crates/dwall-settings/src/thumbnail/store.rs#L27-L45) vs [cache.rs:158-167](../crates/dwall-settings/src/thumbnail/cache.rs#L158-L167) | **缩略图锁的 key 含 URL，落盘文件名不含**：前端传的是镜像改写后的 URL，改一次镜像模板就换一次 key → 同一张图的两个并发请求走两把不同的锁、写同一个文件（单飞失效、可能写出截断图片）；且缓存无法失效（30 天内只按 mtime 命中）。 | URL 参与文件名校验（或哈希进文件名），或让两处 key 一致。 |
| **S4** ○ | [archive.rs:49-62](../crates/dwall-settings/src/archive.rs#L49-L62) | **zip 炸弹上限只是"声明值"检查**：只累加中央目录的 `entry.size()`，deflate 解码与 `io::copy` 都没有实际写出量上限 → 声明 1 KiB、实际 1 MiB 的解压炸弹可以写满磁盘（16,383 个条目 × 高压缩比）。 | 用计数 writer 限制"实际写出字节"，或按条目数 × 最大压缩比估算。 |
| **S5** ○ | [filesystem.rs:152-177](../crates/dwall-settings/src/filesystem.rs#L152-L177) | **"移动到自身内部"的判断大小写敏感**：`destination.starts_with(source)` 按组件比较但对大小写敏感（Windows 不敏感），`D:\Themes` → `d:\themes\new` 通过检查 → `copy_dir` 递归进自己新建的目录 → 最后 `remove_dir_all(source)` 把目标一起删掉（数据丢失）。 | 比较前 `canonicalize`（或至少 `to_lowercase`）两侧路径。 |
| **D4** ✅ | [theme_engine.rs:60-64](../crates/dwall/src/theme_engine.rs#L60-L64)、[:86-122](../crates/dwall/src/theme_engine.rs#L86-L122) | **"墙纸应用失败"从不计入失败阈值**：`process_monitors` 把每个显示器的错误降级为 `warn!` 后返回 `Ok(success_count)`，`process_theme_cycle` 永远 `Ok(())`，`run_once` 永远 `Ok(true)`。前一份清单 P1-4 声称"失败数回传计入连续失败"，**实际未实现**。副作用：阈值唯一现实触发源就是 D1 的定位错误。 | `process_theme_cycle` 在 `success_count == 0 && !monitors.is_empty()` 时返回 Err（或让 `run_once` 返回真实结果）。 |
| **D5** ✅ | [wallpaper.rs:110-132](../crates/dwall/src/platform/windows/wallpaper.rs#L110-L132) + [monitor.rs:504-560](../crates/dwall/src/platform/windows/monitor.rs#L504-L560) | **热插拔后新显示器最长 5 分钟拿不到墙纸。** `WallpaperSetter` 自己持有一个 `DisplayMonitorProvider`（[wallpaper.rs:89](../crates/dwall/src/platform/windows/wallpaper.rs#L89)，与引擎的那个是**两个独立缓存**）；engine 的 provider 被 `reload_if_monitors_changed` 刷新后，setter 的缓存仍是旧的 → `monitors.get(monitor_id)` 为 `None` → 直接 `return Err(MonitorNotFound)`，**不会进入 refresh+retry 分支**（只有 `SetWallpaper` 错误才重试）。P2-39 只修了两个缓存中的一个。 | 复用同一个 provider（或让 `MonitorNotFound` 也走 refresh+retry）。 |
| **D6** ✅ | [monitor.rs:544-560](../crates/dwall/src/platform/windows/monitor.rs#L544-L560)、[theme_engine.rs:384](../crates/dwall/src/daemon.rs#L384) | **每 15 秒做一次完整的 SetupAPI/`QueryDisplayConfig` 枚举。** `has_configuration_changed()` 无条件 `fetch_system_monitors()`，而 `reload_if_monitors_changed()` 在每次循环都调用；那个"减少 90% API 调用"的 5 分钟缓存只覆盖 `get_monitors()`。默认 15 s → **5,760 次/天**，每次还有多个堆分配（`vec![Default::default(); path_count]`）。 | 变化检测也走缓存 + 监听 `WM_DISPLAYCHANGE`/`WM_DEVICECHANGE` 触发。 |

---

## 2. 功能实现

### 2.1 守护进程（`crates/dwall`）

除 D1/D4/D5/D6 外：

| 严重度 | 发现 |
| --- | --- |
| P2 ✅ | **`--config` 指向的配置文件损坏后，守护进程不保留上一份可用配置。** [daemon.rs:95-109](../crates/dwall/src/daemon.rs#L95-L109) 读失败即 `sleep(5s); continue`——此时上一轮的 `theme_engine` 已在循环体结束时析构，进程进入"没有任何引擎"的状态，直到用户改回文件。而 [docs/headless.md:192](./headless.md#L192) 明确写着"守护进程保留上一份配置"，**文档与实现相反**。 |
| P2 ✅ | **配置变更最长要等一个 `interval` 才生效。** [daemon.rs:341-387](../crates/dwall/src/daemon.rs#L341-L387)：`has_changed()` 只在循环顶部检查，随后跑一个周期再 `sleep(interval)`。设置窗口的"保存"（[config.rs:17-46](../crates/dwall-settings/src/commands/config.rs#L17-L46)）不重启守护进程，只写文件——**本机真实配置里 `interval = 600`**（`%APPDATA%\dwall\config.toml`），即改一个非"应用"类设置后最长 **10 分钟**才生效。只有 `apply_theme_cmd` 会重启守护进程。 |
| P2 ✅ | **锁屏主题在"每显示器不同主题"时不确定。** [theme_engine.rs:277-283](../crates/dwall/src/theme_engine.rs#L277-L283) 取 `HashMap::values().next()`：`HashMap` 的随机种子每进程不同，因此锁屏图可能每次重启都换一个主题。 |
| P2 ✅ | **`cache_minutes` 无校验 → 配置可触发 abort。** [solar.rs:191](../crates/dwall/src/solar.rs#L191) 的 `60 * cache_minutes`（u64 乘法溢出）与 [utils.rs:137](../crates/dwall/src/utils.rs#L137) 的 `Instant::now() + ttl`（加法溢出 panic）。手改 `config.toml` 写一个极大值即可让守护进程消失（`panic = "abort"` + GUI 子系统 = 无痕迹）。[Config::validate](../crates/dwall/src/config.rs#L284-L305) 只校验 `interval` 与经纬度。 |
| P2 ✅ | **`unreachable!("Unregistered cache type")`**（[utils.rs:88-96](../crates/dwall/src/utils.rs#L88-L96)）：类型缓存用 `TypeId` 派发，`Cacheable` 是 `pub(crate)` 且可被手工实现——一旦有人实现却没加进 `define_cache!` 宏表，守护进程直接 abort。同时 `CacheValue::SolarPosition` 槽位**从未被读写**（死代码）。 |
| P3 ✅ | **`sin_alt.asin()` 未 clamp**（[solar.rs:374-380](../crates/dwall/src/solar.rs#L374-L380)）：浮点误差使 `|sin_alt| > 1` 时返回 NaN，NaN 会一路进入 Schmitt 触发器（`altitude > switch_point` 恒 false → 强制 Dark）。极罕见，但 `clamp(-1.0, 1.0)` 是一行成本。 |
| P3 ✅ | **主题解析缓存无 mtime 校验**（[theme.rs:217-234](../crates/dwall/src/theme.rs#L217-L234)）：键是 `canonicalize()` 后的路径，就地替换 `solar.json`/`theme.toml` 后仍用旧数据（P3-9，仍成立）。 |
| P3 ✅ | **`select_next` 只在索引越界时重建乱序表**（[theme.rs:460-478](../crates/dwall/src/theme.rs#L460-L478)）：候选池成员在轮次中途变化（删主题/改 pool）后仍可能返回已被移除的主题（P3-5b，仍成立）。 |
| P3 ✅ | **死代码**：`ThemeError::DefaultThemeMissing`/`MonitorWallpaperConfigurationMissing`/`ThemeManifestMissing` 三个变体各只出现一次（定义处）；`MonitorSpecificWallpapers::is_all`（[config.rs:118](../crates/dwall/src/config.rs#L118)）、`Config::with_themes_directory`（[config.rs:323](../crates/dwall/src/config.rs#L323)）、`lib.rs` 的 `GeographicPosition` 别名、`DwallError::NulError` 全仓无调用；[config.rs:42-49](../crates/dwall/src/config.rs#L42-L49) 是注释掉的 `impl`。 |
| P3 ✅ | **迁移脚手架过期**：[config.rs:400-404](../crates/dwall/src/config.rs#L400-L404) 自述 `github_mirror_template` "0.3.0 移除"，而当前版本已是 `0.3.0-alpha.2`。 |
| P3 ✅ | **`readme = "README.md"` 指向不存在的文件**（[crates/dwall/Cargo.toml:5](../crates/dwall/Cargo.toml#L5)）：`crates/dwall/README.md` 不存在，`exclude = ["CHANGELOG.md"]` 同样不存在 → `cargo package`/publish 会失败。 |
| P3 ✅ | **配置写盘不做 fsync**，且临时名用 `path.with_extension("toml.tmp")`（[config.rs:585-596](../crates/dwall/src/config.rs#L585-L596)）：`--config my.conf` 会生成 `my.toml.tmp`（功能无碍，语义易混）。 |
| P3 ✅ | **配置里的目录未校验**：相对路径/空路径都能通过（`Config::validate` 不看路径），而 `Run` 键启动时的工作目录不可预期。 |
| P3 ✅ | **`error!(error = ?win_error)` 的错误码构造不准**：[monitor.rs:121](../crates/dwall/src/platform/windows/monitor.rs#L121) 把 `DisplayConfigGetDeviceInfo` 返回的 Win32 码用 `HRESULT::from_nt` 包装，得到的"HRESULT"是无意义值（日志里的原始 `result` 仍可读，所以只是可读性问题）。 |
| P3 ✅ | **`MONITOR_FRIENDLY_NAMES_CACHE` 无上限且缓存失败值**（[monitor.rs:563-566](../crates/dwall/src/platform/windows/monitor.rs#L563-L566)、[:586-601](../crates/dwall/src/platform/windows/monitor.rs#L586-L601)）：一次失败得到的 `"Display N"` 会被永久缓存；表本身在进程生命周期内只增不减（严格意义上违反 AGENTS 的"无界全局缓存"禁令）。 |
| P3 ✅ | **`adapterId` 只取 `LowPart`**（[monitor.rs:86-89](../crates/dwall/src/platform/windows/monitor.rs#L86-L89)），查询目标名时又硬编码 `HighPart: 0`——LUID 高位非零的机器上目标名查询会失败（P3-14）。 |
| P3 ○ | **COM 配对不完整**：[wallpaper.rs:71-76](../crates/dwall/src/platform/windows/wallpaper.rs#L71-L76) 在 `CoInitialize` 返回 `S_FALSE` 时不做 `CoUninitialize`（按 COM 约定每次成功调用——含 `S_FALSE`——都应配对）；`Drop`（[:350-359](../crates/dwall/src/platform/windows/wallpaper.rs#L350-L359)）先 `CoUninitialize` 再由字段析构释放接口，顺序相反（P3-15）。 |
| P3 ○ | **`get_current_scheme` 只读 `AppsUseLightTheme`**（[color_scheme.rs:38-64](../crates/dwall/src/platform/windows/color_scheme.rs#L38-L64)）：`SystemUsesLightTheme` 被用户手改后永远不修复，因为 `set_color_scheme` 的"已是目标值"提前返回只比较 Apps 值（P3-17，功能性问题）。 |
| P3 ○ | **地理定位每周期调 `RequestAccessAsync`**（[geolocation.rs:128-137](../crates/dwall/src/platform/windows/geolocation.rs#L128-L137)）：该 API 可能弹系统授权 UI，应改用 `GetGeolocationStatus`（P3-10）。 |
| P3 ○ | **注册表 API 暴露裸指针两段式接口、无 `ERROR_MORE_DATA` 重试**（[registry.rs:85-118](../crates/dwall/src/platform/windows/registry.rs#L85-L118)），且 `debug!(pointer = ?data)` 打印入参指针（P3-18）。 |
| P3 ○ | **`unsafe` 块并非都有 `// SAFETY:`**（platform 层约 24 处；`time/src/month.rs:41-46` 的 `# Safety` 写给了一个安全函数）。前一轮只补了部分。 |

### 2.2 设置后端（`crates/dwall-settings`）

除 S1/S2/S3/S4/S5 外（○ 为专项评审结论）：

| 严重度 | 发现 |
| --- | --- |
| P2 ○ | **守护进程身份：互斥体是会话级的，进程扫描是全机的。** [daemon.rs:47-49](../crates/dwall-settings/src/daemon.rs#L47-L49) 用 `Local\` 互斥体（正确的会话语义），而 [process.rs:57-86](../crates/dwall-settings/src/process.rs#L57-L86) 遍历所有进程按 `dwall.exe` 名字匹配、**不过滤会话**：多用户/RDP/快速切换时 `stop_engine`/`kill_daemon_cmd`/`ThemeApplier::apply` 会杀掉**别的会话**的守护进程（而 `get_engine_status` 说"没在运行"）。另外 PID 来自快照、`OpenProcess` 时不再校验映像名 → PID 复用 TOCTOU 可杀无关进程。 |
| P2 ○ | **停止→启动没有等待。** [applier.rs:26-54](../crates/dwall-settings/src/theme/applier.rs#L26-L54)：`TerminateProcess` 返回即写配置、启动新进程、`sleep(100ms)`、查互斥体。旧进程尚未释放互斥体时新进程会 `ERROR_ALREADY_EXISTS` 直接退出 → "应用主题"间歇性失败，报的还是误导性的 `Daemon process failed to start`。 |
| P2 ○ | **下载失败/解压失败会漏下 `.zip`。** [downloader.rs:96-106](../crates/dwall-settings/src/theme/downloader.rs#L96-L106)：`remove_file(zip_path)` 在 `extract_zip` 之后，解压失败即提前返回 → `themes/<id>.zip` 永远留在盘上（可能上百 MB），而没有任何清理路径会碰它。 |
| P2 ○ | **取消下载会被当成可重试失败。** [client.rs:140](../crates/dwall-settings/src/network/client.rs#L140) 返回 `Other("Download cancelled")`，[downloader.rs:176-178](../crates/dwall-settings/src/theme/downloader.rs#L176-L178) 对任何 `Err` 都重试 → 取消后还会再发 2 次请求；错误类型里没有 `Cancelled` 变体，UI 只能靠字符串匹配区分"取消"和"失败"。 |
| P2 ○ | **镜像对稳定版通道基本无效。** [mirror.rs:46-69](../crates/dwall-settings/src/network/mirror.rs#L46-L69)：改写 `releases/latest/download/...` 前必须先用**新建的** reqwest client（不走配置里的代理）请求 `api.github.com` 取 tag；在被墙的地区这一步必然失败（连接 5 s / 总 10 s）然后回落到 Discord 直连——恰恰是最需要镜像的场景。 |
| P2 ○ | **续传没有资源身份校验，全链路没有校验和。** [client.rs:102-132](../crates/dwall-settings/src/network/client.rs#L102-L132) 只发 `Range: bytes=N-`，无 `If-Range`/ETag；[downloader.rs:197-205](../crates/dwall-settings/src/theme/downloader.rs#L197-L205) 又刻意保留非空断点 → 资源被替换后会把新响应追加到旧字节后面；`themes` 资产没有 `.sha256` 可校验。 |
| P2 ○ | **没有任何并发上限。** [tasks.rs](../crates/dwall-settings/src/network/tasks.rs) 只管取消标志；库视图里每个可见缩略图都会发一个请求，[cache.rs:101-113](../crates/dwall-settings/src/thumbnail/cache.rs#L101-L113) 还在每次未命中时以 5% 概率 `tokio::spawn` 一次全目录清理（无去重），会与在途下载和 `clear_thumbnail_cache` 互相踩。 |
| P2 ○ | **两个 tokio 运行时。** [main.rs:35-36](../crates/dwall-settings/src/main.rs#L35-L36) 用 `#[tokio::main]` 但从不 `tauri::async_runtime::set(...)`，Tauri 于是自建一个（`tauri-2.12.4/src/async_runtime.rs:243-250`）→ `tokio::spawn`（[setup.rs:66](../crates/dwall-settings/src/setup.rs#L66)、[:76](../crates/dwall-settings/src/setup.rs#L76)）与 8 个命令用的 `spawn_blocking` 跑在两个线程池上，worker 线程数翻倍。 |
| P2 ○ | **在错误的地方做阻塞 IO。** [commands/catalog.rs:60-80](../crates/dwall-settings/src/commands/catalog.rs#L60-L80) 是**同步命令**（Tauri 在主线程执行），而它对**每个**目录条目都 `ConfigReader::read_from_path` 一遍（15+ 次读盘 + TOML 解析）；[settings/daemon.rs:87-107](../crates/dwall-settings/src/daemon.rs#L87-L107) 把整份 `dwall.log` 读进内存再 `from_utf8_lossy`（第二份拷贝），且在 async 路径里同步调用。 |
| P2 ○ | **保存配置在"杀掉守护进程之后"。** [applier.rs:26-37](../crates/dwall-settings/src/theme/applier.rs#L26-L37)：先 `terminate_process` 再 `write_to_path`；写盘校验失败就留下一个已停的引擎，用户只看到"保存失败"。反过来写更安全。 |
| P2 ○ | **缩略图的"读取续命"是无效的。** [storage.rs:21-25](../crates/dwall-settings/src/thumbnail/storage.rs#L21-L25) 只打开文件"更新访问时间"，而淘汰/过期都看 `metadata.modified()`（[cleanup.rs:109-117](../crates/dwall-settings/src/thumbnail/cleanup.rs#L109-L117)、[:193-214](../crates/dwall-settings/src/thumbnail/cleanup.rs#L193-L214)），且 Windows 默认不更新 atime → 每天看的缩略图仍会在写入 30 天后被删，注释与实现相反。 |
| P3 ○ | **`.zip`→`.tmp` 改名纯属多余**（[downloader.rs:188-195](../crates/dwall-settings/src/theme/downloader.rs#L188-L195)）、**`kill_daemon_cmd` 与 `stop_engine` 完全重复**且其 TS 包装无调用者、**`thumbnail/error.rs` 的 `ThumbnailError` 从未被构造**（连带 `error.rs` 的 `#[from]` 与 `thumbnail/mod.rs` 的导出都是死的）、**`tracker.rs` 已是 no-op 但 `setup.rs:66` 仍 spawn 一个空任务**。 |
| P3 ○ | **`update.rs:99-107` 用 GitHub 的 `prerelease` 标志而不是 semver 的 `pre`** 来判定通道：被误标为 stable 的 `v0.4.0-alpha.1` 会击败真正的 stable。`RELEASES_API_MAX_BYTES = 4 MiB`（[update.rs:36](../crates/dwall-settings/src/update.rs#L36)）对 30 条带完整 changelog 的响应也可能不够。 |
| P3 ○ | **capability 过宽**（[capabilities/default.json](../crates/dwall-settings/capabilities/default.json)）：`core:default` 已含 `core:window:default`（又显式列了一次），并额外授予未使用的 tray/menu/image；`process:default` 含 `allow-exit`（只用到 `relaunch`）。`hooks.nsi:42-43` 每次安装都删缓存与日志，`:56-57` 卸载时删掉 `%APPDATA%\dwall`（**含用户全部已下载主题**）且不询问。 |

### 2.3 前端（`src/**`）

除 D2/D3 外（○ 为专项评审结论）：

| 严重度 | 发现 |
| --- | --- |
| P1 ○ | **引擎状态轮询会变成"每 5 秒一个错误 toast"。** [App.tsx:19](../src/App.tsx#L19) 每 5 s 调 `refreshEngine`，[engine.store.ts:12-17](../src/store/engine.store.ts#L12-L17)/[:32-38](../src/store/engine.store.ts#L32-L38) 在失败时 toast，`get_engine_status` 的 Rust 侧返回 `Result`（可失败）→ 无节流、无去重的持续错误流。 |
| P2 ○ | **`listen()` 在 `try` 之外**（[download.store.ts:60](../src/store/download.store.ts#L60)）：监听注册失败会让状态永远停在 `"downloading"`，所有下载按钮永久禁用（[:51](../src/store/download.store.ts#L51) 的并发保护也无法清除），并且 rejection 逃逸成未处理拒绝。 |
| P2 ○ | **"应用此主题"会静默提交所有未保存的编辑。** [settings.store.ts:173-196](../src/store/settings.store.ts#L173-L196) 把带 `wallpaper_mode` 的**草稿**整个交给 `apply_theme_cmd`，Rust 会把整份配置写盘（[applier.rs:35-37](../crates/dwall-settings/src/theme/applier.rs#L35-L37)），随后 store 标记为已保存。于是"只想应用主题"等于"提交所有脏字段"，且任一脏字段非法会让整个应用失败。 |
| P2 ○ | **主题库每次目录变化整体重建。** [catalog.store.ts:58-64](../src/store/catalog.store.ts#L58-L64) 改 `installed` → [catalog.selectors.ts:24](../src/store/catalog.selectors.ts#L24) → [models/themes.ts:11-19](../src/models/themes.ts#L11-L19) 每次 `map` 都产生**新对象** → [LibraryColumn.tsx:55](../src/library/LibraryColumn.tsx#L55) 的 `<For>` 认为全部是新条目 → 每个 `ThemeTile` 与其 `ThemeThumbnail` 资源被销毁重建，重新走 `mirrorUrl` + `getOrSaveCachedThumbnails` 两次 IPC，入场动画与 `animation-delay` 重播。任何一次下载完成都会触发。 |
| P2 ○ | **表单控件没有可访问名称**：[SettingsRow.tsx:32](../src/components/settings/SettingsRow.tsx#L32) 的标签渲染成 `<div>`，[Label.tsx:31-32](../src/components/ui/label/Label.tsx#L31-L32) 的 `<label>` 没有 `for`，[Switch.tsx:46-55](../src/components/ui/switch/Switch.tsx#L46-L55) 是 `<span role="switch" tabIndex={0}>` 且无 `aria-label` → 读屏软件会念出 8 个无名开关与无名数值输入框。 |
| P2 ○ | **抽屉在屏幕外仍可聚焦**：[MainView.tsx:44-52](../src/views/MainView.tsx#L44-L52) 只用 `translate-x-full` 隐藏抽屉，没有 `inert`/`aria-hidden`/`hidden` → <1280px 时 Tab 会走到视野外的搜索框与下载按钮。 |
| P2 ○ | **全站无法选中/复制文本**：[index.css:12-18](../src/index.css#L12-L18) 的 `user-select: none` 作用于 `*`（覆盖 `<input>` 的默认值），[index.tsx:8-10](../src/index.tsx#L8-L10) 在生产构建里屏蔽右键菜单 → 主题目录路径（[PathsSection.tsx:63-65](../src/components/settings/PathsSection.tsx#L63-L65)，还是截断显示的）无法复制，镜像模板/SOCKS 主机无法用鼠标选取编辑。 |
| P2 ○ | **目录迁移失败被吞掉**：[settings.store.ts:203-215](../src/store/settings.store.ts#L203-L215) 只 `log.error`，而 [logger.ts:119-123](../src/utils/logger.ts#L119-L123) 的 error 在生产构建里**直接输出到 console、其余级别是 no-op**——但 release 无 devtools，等于没有记录。 |
| P3 ○ | 其他：`PoolCollage` 的"今日主题"只在挂载时取一次（新 pool 保存后过期）；`FixedStage` 的 `matchedEntry` 在切换主题时保留旧值（会短暂标错"匹配"并预览错帧）；自启开关无 try/catch；`App.tsx:15` 的 `showWindow` 是浮空 promise（生产窗口默认隐藏，见 [window.rs:41](../crates/dwall-settings/src/window.rs#L41)）；4 个可点击 `<div>` 无 role/键盘；`SunPathPanel` 的标记点命中区约 7 px；`RandomStats` 在 0/0 时显示 "All" 与红色 `0` 自相矛盾；两个视图同时挂载（`Body.tsx:10-25`）；`lib/layout.ts:3,7,9` 的常量无人引用，因此文档里"窗口过矮时隐藏太阳弧"的行为并不存在；`ipc/config.ts:7` 声明返回 `Config` 而 Rust 返回 `()`、`ipc/system.ts:9` 声明 `void` 而 Rust 返回 `Option<u32>`；`index.html:7` 的 favicon 指向不存在的 `logo.svg`；`tsconfig.json:26` include 了不存在的 `types/`；11 个文件混用 `@/` 与 `~/`；`solid-icons`、`@tauri-apps/plugin-shell`（未在 Cargo.toml/capability 里）、`jsdom`（无 vitest 配置，装了也没用）是未使用的依赖。 |

### 2.4 文档与实现不一致

- [docs/headless.md:192](./headless.md#L192)："坏配置不是致命的：守护进程保留上一份配置并每 5 秒重试" → 实际**不保留配置**（引擎已被析构，见 2.1）。
- [docs/custom-themes.md:6-10](./custom-themes.md#L6-L10) 画的是 `<themes_directory>/<id>/images/<n>.<ext>`，但 [theme.rs:140-152](../crates/dwall/src/theme.rs#L140-L152) 只在**自定义**主题（`customized_themes_directory`）用 `images/`，`themes_directory` 下的主题用的是 `<image_format>/`（如 `jpeg/1.jpg`）。两个根目录的布局**不一样**，而这份文档只写了 `images/`——它同时是 headless 压缩包里唯一的主题格式说明，而手工部署主题正是 headless 场景。
- [.scripts/package-daemon.ps1:90](../.scripts/package-daemon.ps1#L90) 的 README 硬编码"about 600 KB"，当前二进制已是 618 KB（脚本自己也会打印真实大小）。
- README "单线程 Rust 守护进程约 4 MB"：实测**私有工作集 1.38 MB / Private 2.48 MB / 工作集 21.25 MB**，"≈4 MB"偏保守；但同一句话里的 "4 MB" 也可能被读成"体积"，而真实二进制是 618 KiB——建议明确写成"常驻内存"。

---

## 3. UI 设计

前端整体是**完成度较高的一套自研设计系统**（Tailwind v4 `@theme` 令牌、自带 `animate-rise/fade-up/bdot/breathe/pulse-ring` 关键帧、自研 positioner/tabs/tooltip/select/toast），设计语言一致性没问题。问题集中在"可达性"和"状态与错误可见性"两类，而非视觉风格：

1. **可达性（P2）**：表单标签与控件未关联（2.3 表）；抽屉在屏外可聚焦；4 处可点击 `<div>` 无 role/键盘路径；而 [biome.json:26-36](../biome.json#L26-L36) 恰好把这四条 a11y 规则关掉了，所以没有门禁会拦住它们。
2. **错误可见性（P1/P2）**：配置读取失败无任何提示（D3）；下载状态的异常路径会把 UI 卡死在"下载中"；自启/目录迁移失败静默。
3. **文本不可选（P2）**：全局 `user-select: none` + 屏蔽右键，对"要复制路径/URL 去排查问题"的场景是硬伤。
4. **体积/一致性（P3）**：`src/styles/toast.css`（747 行，vendored sonner）**完全不可能匹配**——组件发出的是 `data-slot="toaster"/"toast"`，而样式表只选 `[data-sonner-toaster]`/`[data-sonner-toast]`/`.sonner-*`。我在**已构建的 CSS** 里实测：`[data-sonner-toast]` 60 处、`[data-sonner-toaster]` 24 处、`.sonner-` 19 处，而 `data-slot=toast` **0 处** → 这 103 条规则是 121,900 B CSS 里的纯死重（专项评审测得 13,823 B / 11.3%）。同理 `src/components/ui/` 下 badge/button-group/toggle/toggle-group/slider/spinner/TooltipGroup/TooltipArrow 等约 2400 行零引用。
5. **一处需要澄清的 CSP 疑点（已排除）**：设置端专项评审怀疑 `style-src-elem 'self'` 会挡住 JSX 里的内联样式。**实测不成立**：全仓 `style=` 全部是 Solid 对象语法（`style={{...}}`），Solid 走 `node.style.setProperty()`（CSSOM），CSP 的 `style-src-attr` 不拦 CSSOM 属性赋值。无需修改 CSP。
6. 视觉细节：角度一律 3 位小数（`formatAngle`）在 104 px 磁贴里以 8.5 px 显示 `-12.345° · 200.123°`；`SunPathPanel` 标记点 7 px 命中区远低于 24 px 目标。

---

## 4. 日志

**做得好**：kv 宏零开销（`max-level-info` 编译期剔除，参数只求值一次）；release 默认 `info` 且逐周期 DEBUG 已降级，正常运行时只记录"墙纸真的换了"这一条 INFO（[wallpaper.rs:308-316](../crates/dwall/src/platform/windows/wallpaper.rs#L308-L316)）；平台层幂等（相同墙纸/锁屏/颜色方案跳过）避免了日志与系统广播刷屏；ANSI 只在 stderr 是终端时启用；`dwall=debug` 这类写法能解析（有单测）。

**问题**：

| 严重度 | 发现 |
| --- | --- |
| P2 ✅ | **`backup/` 无上限地增长。** [logging/lib.rs:84-119](../crates/logging/src/lib.rs#L84-L119) 每次进程启动都把上一个日志改名进 `log/backup/<stem>_<unix秒>.log`，从不清理：设置端每次"应用主题"都会重启守护进程（[applier.rs:44](../crates/dwall-settings/src/theme/applier.rs#L44)），定时任务里每分钟一次 `--once` 就是 1440 个文件/天。同一秒内的两次启动还会互相覆盖（`fs::rename` 覆盖同名）。 |
| P2 ✅ | **每条日志记录一次 `GetTimeZoneInformation`**（[logging/lib.rs:157](../crates/logging/src/lib.rs#L157) → [offset/windows.rs:6-17](../crates/time/src/offset/windows.rs#L6-L17)）：无缓存，`TIME_ZONE_ID_INVALID` 静默按标准时区处理（P3-23）。 |
| P2 ✅ | **`debug!(config = ?config)` 打印完整配置**（含经纬度与代理，[config.rs:513](../crates/dwall/src/config.rs#L513)），与上一行 `info!` 的注释"故意不打印整个结构体：它带有用户的精确坐标与网络/代理设置"**自相矛盾**。debug 构建与 nightly 会真实落盘（[crates/dwall/build.rs:8-10](../crates/dwall/build.rs#L8-L10) 注入 `DWALL_LOG=debug`）。 |
| P2 ✅ | **`enabled()` 按 target 前缀过滤**（[logging/lib.rs:142-150](../crates/logging/src/lib.rs#L142-L150)）：只接受 `dwall*`，因此 tauri/wry/reqwest/hyper 的日志**全部丢弃**。这是有意的，但没有任何文档说明；排查 WebView/网络问题时日志里一条线索都没有。 |
| P2 ✅ | **`has_configuration_changed` 的错误被静默吞掉**（[theme_engine.rs:67-71](../crates/dwall/src/theme_engine.rs#L67-L71) 的 `unwrap_or(false)`）：显示器枚举持续失败时**零日志**。这是守护进程里唯一一处"错误完全不留痕"。 |
| P3 ○ | 设置端：**没有一个 Tauri 命令记录自己的错误**（错误序列化后返回前端，UI 不显示就等于没有记录）；下载最终失败只有 `warn!` 且拿 URL 的语句是 `debug!`（release 被编译掉）→ 用户报障时无法知道哪个 URL 失败了；缩略图失败无日志；`daemon.rs:93,97`、`commands/catalog.rs:87,100`、`commands/theme.rs:92,96` 等处用 `.ok()`/`unwrap_or_default()` 吞掉 IO 错误。 |
| P3 ✅ | **`std::process::exit(1)` 会跳过 logger 的 flush**（[bin/daemon.rs:374-380](../crates/dwall/src/bin/daemon.rs#L374-L380) 在 `error!` 之后立即退出）；release 下文件写入是无缓冲的 `writeln!`，所以通常已经落盘，但这仍是一条"最后一条日志可能丢失"的路径。 |
| P3 ○ | **日志级别/风格违规**：`network/tasks.rs:65,67` 把 `{task_id}` 插进消息文本（而非 `task_id = %task_id`）；`commands/theme.rs:115` 位置参数；`network/client.rs:50` 内联 host/port。 |

---

## 5. 内存占用

**结论：守护进程的内存纪律是本仓库最扎实的部分，本轮未发现新的无界增长。**

- 实测（已安装 0.3.0-alpha.4，运行中）：**1 线程、244 句柄、Private 2.48 MB、私有工作集 1.38 MB、工作集 21.25 MB**；75 秒后（约 5 个周期）句柄与 Private **一个字节都没变** → P1-14 的 `CoTaskMemFree` guard（[wallpaper.rs:50-60](../crates/dwall/src/platform/windows/wallpaper.rs#L50-L60)）确实生效，曾经的"每显示器每周期泄漏一块 COM 内存"已闭环。
- 缓存全部有界：主题角度 LRU = 32（[theme.rs:73](../crates/dwall/src/theme.rs#L73)）；显示器信息 300 s（[monitor.rs:434](../crates/dwall/src/platform/windows/monitor.rs#L434)）；typed cache 固定 4 槽 TTL（[utils.rs:115-122](../crates/dwall/src/utils.rs#L115-L122)）；命中返回 `Rc`，不克隆角度表。
- 无后台线程（全仓只有 `std::thread::sleep`）；不长期持有 WinRT 对象（`Geolocator` 每次调用新建并析构，只长期持有 `IDesktopWallpaper` 与命名互斥体句柄）。
- 剩余的三处严格意义上的违规：`MONITOR_FRIENDLY_NAMES_CACHE` 无上限（2.1）；瞬时分配方面每周期有 2 次显示器 `HashMap` 克隆 + 1 次 `canonicalize` + 每个显示器几条 `PathBuf`/`format!`（D6 的枚举频率放大了这些）；typed cache 的 `SolarPosition` 槽位从未使用（死槽）。
- 设置端才是内存与线程的大头（Tauri + WebView2 + 两个 tokio 运行时），但只在窗口打开时存在，符合产品定位。

---

## 6. 体积与发布链路

### 6.1 产物

| 产物 | 体积 | 对比 |
| --- | --- | --- |
| `target/release/dwall.exe`（当前源码，含图标资源） | **632,832 B = 618.0 KiB** | 0.2.5 = 600,576 B → **+32 KB / +5.4%**（windows 0.62 升版 + 随机模式 + 有界异步等待等） |
| `target/debug/dwall.exe` | 2,741,248 B | release 小 4.33× |
| `target/release/dwall-settings.exe` | 4,234,240 B（4.04 MiB） | debug 21.7 MB |
| NSIS 安装包 | 1,806,833 B（1.72 MiB） | 0.2.5 = 1.68 MiB、0.1.33 = 1.97 MiB |
| `Dwall_0.2.5_x64-daemon.zip` | 359,401 B = 351 KiB | 约为二进制的 57% |
| `dist/` 前端产物 | 384,534 B | JS **231,251 B（+21 KB / +10% vs 0.2.5）**、CSS 121,900 B、SVG 30,781 B、html 602 B |
| `target/release` + `target/debug` | 17.93 GB + **61.51 GB** | 本机累计 79.4 GB（`codegen-units=1` + fat LTO 的代价是构建时间/磁盘，不是运行期） |

`opt-level = "z"` 的取舍**没有问题**：每个周期做的都是 Win32/WinRT 调用（`QueryDisplayConfig` + SetupAPI 枚举、注册表读、COM Get/SetWallpaper），唯一的计算是对 ≤N 个 24 B 的 `SolarAngle` 做一次线性扫描，`-Oz` 的百分之几完全被 15 秒睡眠淹没。

### 6.2 可回收的体积 / 依赖

1. **CSS 死重 13.8 KB（11.3%）**：删掉 [src/styles/toast.css](../src/styles/toast.css) 及其 `@import`（[index.css:4](../src/styles/index.css#L4)），该文件的选择器在组件上不可能命中（见 3.4）。JS 侧还有约 2400 行零引用的 UI 组件。
2. **GUI 二进制里有 23 组重复依赖**，其中 `zip 4.6.1`（经 `tauri-plugin-updater`）与 `zip 8.6.0`（直接依赖，[crates/dwall-settings/Cargo.toml:31](../crates/dwall-settings/Cargo.toml#L31)）意味着**同一个 4.2 MB 可执行文件里链接了两套 zip + flate2 + miniz_oxide**；还有 `toml 0.8/1.1`、`windows-sys 0.45/0.60/0.61`、`indexmap 1.9/2.14`、`base64 0.22/0.23` 等。daemon 图**完全没有重复版本**（workspace 的 `windows` 固定版本策略有效）。
3. **`windows` 特性表**（[crates/dwall/Cargo.toml:39-54](../crates/dwall/Cargo.toml#L39-L54)）：`Win32_Security` 看起来"repo 内无人引用"，**但它必需**——`CreateMutexW` 的签名引用 `Win32::Security::SECURITY_ATTRIBUTES`，而 `CreateMutexW` 本身带 `#[cfg(feature = "Win32_Security")]`（我在 vendored 的 `windows-0.62.2/src/Windows/Win32/System/Threading/mod.rs` 里核对了 29 处同类门控）；删掉它 = `single_instance.rs:69` 编译失败。`Storage_Streams`（= `["Storage"]`）值得试删；设置端的 `Win32_System_ProcessStatus`、`Win32_Globalization` 与冗余的 `Win32_System_Registry` 也值得试删——**但这三者都要"删掉再编译"来确认**，仅靠 grep 得出的"未使用"结论在 `windows` crate 上不可靠（本次审查中已出现过一次这样的误判）。由于 release 是 fat LTO，删特性主要省编译时间与依赖面，**不会明显减小 618 KB 的二进制**，不要把它当体积优化来宣传。
4. 前端未使用依赖 `solid-icons`、`@tauri-apps/plugin-shell`、`jsdom`（[package.json](../package.json)）——不影响发布产物。
5. 想再压 daemon 体积，唯一有量级的候选是用 `SHGetKnownFolderPath`（`Win32_UI_Shell` 已启用）替掉 `dirs`（连带 `dirs-sys`/`option-ext`/`windows-sys`/`windows-link` 四个 crate）；但 daemon 只有 39 个 crate-version，收益有限。

### 6.3 发布链路（**本次审查里最严重的一类问题**）

| 严重度 | 发现 |
| --- | --- |
| **P1** ○ | **发布线与 `main` 已脱节，`publish.yml` 的最后一步必然失败，却仍然报告成功。** 实测：`main` = `968e558`（version 0.1.33，**没有任何 tag 可从它 describe 到**）、`dev` = `0b51764`（describe 到 `v0.2.5`）、`main..dev` = **369** 个提交、merge-base 在 5 月 → `git merge-base --is-ancestor dev main` 退出码 1。而 [publish.yml:401-407](../.github/workflows/publish.yml#L401-L407) 的 `update-main-version` 做的是 `git checkout main; git merge release --ff-only`——当发布是从 `dev` 切出来的时候**不可能 fast-forward**，于是版本提升永远进不了 main；[publish.yml:455-483](../.github/workflows/publish.yml#L455-L483) 的 `notify-completion` 只看 `needs.publish-release.result`，于是打出"✅ Release published successfully!"。旁证：`origin/release` 仍存在且停留在 0.1.34（`cleanup-release-branch` 是 `if: success()`，从未跑成）；changelog 的基线是 `git describe --tags --exclude=nightly refs/heads/main`（[:119](../.github/workflows/publish.yml#L119)），取不到 tag 时回落到 main 的**根提交**（[:124-129](../.github/workflows/publish.yml#L124-L129)）——正是当前状态，因此那里生成的 changelog 会横跨整个历史。 |
| P2 ○ | **`publish.yml` 没有 `concurrency:`**（`pr-check.yml:3-5` 有）：两次手动触发会争抢 `git push origin release --force`（[:114](../.github/workflows/publish.yml#L114)）与 tag，`build-tauri` 又按 `ref: release` 取代码（[:238](../.github/workflows/publish.yml#L238)）→ A 可能打包出与传给 `package-daemon.ps1 -Version`（[:312](../.github/workflows/publish.yml#L312)）不一致的版本；且 `package_version` 是 `${{ env.VERSION }}` 形式的 job output（[:67](../.github/workflows/publish.yml#L67)、[:106](../.github/workflows/publish.yml#L106)），没有空值校验（为空会产出 `Dwall__x64-daemon.zip`）。 |
| P2 ○ | **`delete-nightly-release-and-tag` 没有 `needs:`**（[:432-453](../.github/workflows/publish.yml#L432-L453)）：`if: success()` 一开始就成立，会在构建进行中删除 `nightly` release/tag，与正在上传资产的 `nightly.yml` 竞争。 |
| P2 ○ | **清单哈希来源不可信、且压缩不可复现**：`render-package-manifests.ps1:46-54` 直接信任 `dist-daemon/<archive>.sha256`，而 `Compress-Archive`（[package-daemon.ps1:123](../.scripts/package-daemon.ps1#L123)）会把条目 mtime 写进压缩包。本工作树里就有实证：磁盘上的 zip 是 `6277cbe4…`（与它自己的 `.sha256` 一致、5 条内部哈希全部通过），而 `dist-manifests/**` 里内嵌的是同一文件名的 `F191A558…`（清单渲染于 10/16 12:48，zip 于 13:40 重新打过）→ 按 `docs/headless.md:168-174` 的顺序操作，提交上去的 winget/Scoop 清单指向的是**不存在的字节**。 |
| P2 ○ | **发布路径上的第三方 action 全部用浮动 tag，且持有签名私钥**（`TAURI_SIGNING_PRIVATE_KEY`，[:300-301](../.github/workflows/publish.yml#L300-L301)）：`tauri-apps/tauri-action@v0.5.23`、`actions/checkout@v4`、`actions/cache@v5`、`oven-sh/setup-bun@v1`（pr-check 里是 `@v2`）、`biomejs/setup-biome@v2`、`pascalgn/automerge-action@v0.16.4`；`.github/dependabot.yml` 只有 `bun`/`cargo` 两个 ecosystem，**没有 `github-actions`**，所以永远不会有人来升级它们。 |
| P3 ○ | `publish.yml` 顶层没有 `permissions:`（各带注释的 job 已收窄，未注释的 `pre-checks`/`notify-completion` 继承仓库默认）；发布链路自己不跑 fmt/clippy/test，完全依赖 PR 门禁；`.scripts/*.ps1` 的 `-Validate` 在 CI 里从不传（[:328](../.github/workflows/publish.yml#L328)）；`package-daemon.ps1:109,120` 用 `Set-Content -Encoding utf8`，在 Windows PowerShell 5.1 下会写 BOM（`sha256sum -c` 会拒绝第一行；CI 的 pwsh 7 下实测无 BOM）；`update.rs:26-30` 硬编码更新端点（忽略 `Cargo.toml` 的 `repository`），`?per_page=30` 限制了预发布版本能回溯的范围；nightly 与 stable 共用 `identifier: com.thep0y.dwall`/productName（`tauri.debug.conf.json` 只覆盖 `beforeBuildCommand`）→ 安装 nightly 会顶掉 stable，且没有回到 nightly 的路径。 |
| P3 ✅ | **本机的 `dist/`、`target/release/*.exe`、NSIS 包都是 10/17 12:53 从"未提交的工作树"构建的**（HEAD 提交于 10:43，而工作树的 `Cargo.toml` 仍是 0.2.5 + 未提交的版本提升）→ **没有任何提交能复现这些产物**；`dist-daemon/*.zip` 更旧（10/16 13:40），且其中打包的 `docs/headless.md` 与仓库当前版本**内容不同**（8,605 B vs 8,621 B）→ 单机 zip 已经过期。已安装的 `E:\Program Files\Dwall Settings\dwall.exe` 是第三个不同的构建（`0.3.0-alpha.4`，629,760 B）。 |

---

## 7. 前一份清单（P3 批次 6）复核

前一份文档把批次 6 标为 ⏳ 且未逐条打勾。本轮逐条核对：

| ID | 状态 | 说明 |
| --- | --- | --- |
| P3-1 阈值/白昼状态缓存 24h 且配置重载不清 | **仍存在** | [color_scheme.rs:343-361](../crates/dwall/src/color_scheme.rs#L343-L361)：缓存只有 `TypeId` 键与固定 TTL，改经纬度或跨午夜后仍用旧值（`DaylightState` 是按天变化的量）。 |
| P3-2 `cache_minutes` 溢出 panic | **仍存在** | 见 2.1（配置可触发 abort）。 |
| P3-3 时间/索引溢出 | 已修 | `u16::from(index)+1`、`days_to_ymd` 夹取、`end_of_day` 饱和，均有单测。 |
| P3-4 NaN panic | 已修 | `total_cmp` + NaN 单测。 |
| P3-5 / P3-5b 池成员变化 | **半修** | `restore` 已校验；`select_next` 仍可能返回被移除的主题。 |
| P3-6 配置检测与周期共用 `sleep` | **仍存在** | 见 2.1，本机 `interval=600` 是真实复现。 |
| P3-7 首次 `has_changed` 恒 true | 已修 | `run()` 启动时先建基线。 |
| P3-8 `LazyLock` 内 panic / 初始化前日志丢失 | **半修** | 不再 panic；但 `init_logger()` 里先求值 `DWALL_LOG_DIR` → `ensure_dir` 的日志仍丢在 `Logger::init()` 之前。 |
| P3-9 主题缓存无 mtime | **仍存在** | 见 2.1。 |
| P3-10 每周期 `RequestAccessAsync` | **仍存在** | 见 2.1。 |
| P3-11 `unreachable!` 匹配 WinRT 常量 | 已修 | 改为 `warn!` + `Unspecified`。 |
| P3-12 友好名缓存无上限 | **仍存在** | 见 2.1。 |
| P3-13 固定 512 B 友好名缓冲 | **仍存在** | [monitor.rs:393-429](../crates/dwall/src/platform/windows/monitor.rs#L393-L429)：超长名字会失败并回落到 `"Display N"`。 |
| P3-14 `position_index`/`adapterId` | **仍存在** | 见 2.1。 |
| P3-15 `S_FALSE`/COM 释放顺序 | **仍存在** | 见 2.1。 |
| P3-16 `unsafe` 缺 `SAFETY` 注释 | **半修** | 新代码有，platform 层其余仍缺。 |
| P3-17 只读 `AppsUseLightTheme` | **仍存在** | 见 2.1（功能性）。 |
| P3-18 注册表 API | **仍存在** | 见 2.1。 |
| P3-19 `now()` panic | 已修 | 饱和到 epoch。 |
| P3-20 比较方法秒截断 | **仍存在** | [time/lib.rs:305-325](../crates/time/src/lib.rs#L305-L325) 用 `as_secs()`，而派生 `Ord` 用完整 `Duration`。 |
| P3-21 `Date::new` 不校验 / DST 日偏 1h | **仍存在** | [time/lib.rs:659-669](../crates/time/src/lib.rs#L659-L669)：`with_hms` 用**当前**偏移解释一整天的每个小时（`DaylightState::detect` 会调用 24 次），DST 切换日会偏 1 小时。 |
| P3-22 日志轮转无上限 | **仍存在** | 见第 4 节，已量化。 |
| P3-23 每条记录一次 `GetTimeZoneInformation` | **仍存在** | 见第 4 节。 |
| P3-24 个人信息入日志 | **半修** | `info!` 已清理；`debug!(config = ?config)` 仍打印全量配置。 |
| P3-25 迁移无进度/不可中断 | 仍存在（设置端） | `move_directory` 无进度、失败留半个目标目录。 |
| P3-26 NSIS 升级删缓存/日志、卸载残留 | **仍存在** | `hooks.nsi:42-43,56-57`。 |
| P3-27 隐藏窗口无兜底 | **仍存在** | 生产窗口 `.visible(false)`，只靠前端 `showWindow`。 |
| P3-28 主题目录两套真相 | 仍存在 | `theme/paths.rs` 与 `commands/catalog.rs` 各有一套解析。 |
| P3-29 缩略图 touch 保活无效 | **仍存在** | 见 2.2。 |
| P3-30 `get_last_daemon_error` 读错目录 | 已修 | 改读 `DWALL_LOG_DIR`、按文本取最后 ERROR（守护进程侧确认为纯文本行格式）。 |
| P3-31 kv 日志约定违背 | **半修** | 守护进程侧已改；设置端仍有 3 处。 |
| P3-32 依赖特性"蹭"用 | 已修 | `reqwest` 显式 `native-tls`、`tokio` 显式特性列表。 |
| P3-33 `logging/src/macros.rs.bak` | **仍存在** | 118,492 B 仍在工作树（被 `.gitignore` 挡住）。 |
| P3-34 未声明 `rust-version` | **仍存在** | 代码用到 `u16::is_multiple_of`（1.87）、`as_chunks`、let-chains、edition 2024。 |
| P3-35 `icon.icns` 不存在 | **仍存在** | `tauri.conf.json:32` 仍列着它；实测 `crates/dwall-settings/icons/` 下无 `.icns`（NSIS 目标下未触发，但换打包目标即失败）。 |
| P3-36 死 UI 模块 / 死 CSS | **仍存在** | 本轮实测确认（第 3、6 节）。 |
| P3-37 `user-select: none` | **仍存在** | 见第 3 节。 |
| P3-38 label 未关联 / 无键盘 | **仍存在** | 见第 3 节。 |
| P3-39 5 秒轮询 / 无 busy 态 | **仍存在** | 见 2.3。 |
| P3-40~P3-48 | **仍存在** | 空态/加载态、假链接、IPC 类型不符、`okShow` 永不触发、7 px 命中区、每磁贴一个 `matchMedia`、`useScrollEdges` 只 observe 首个子元素、`ThumbnailError` 死代码、`Big.Sur.zip` vs `BigSur` 两套命名。 |
| P3-49 CI/仓库卫生 | **半修** | cachekey/fingerprint/.gitattributes 已修；**仍存在**：`publish.yml:105,112` 只 sed/`git add` `Cargo.toml` 与 `package.json`，**不同步 `Cargo.lock`**（每次发布后工作树必脏）；无 `cargo audit`/`deny.toml`/`rust-toolchain.toml`；`.scripts/mirror-latest-json.js` 无引用；`bunfig.toml` 硬编码 npmmirror。 |

CI 门禁本身（[pr-check.yml:99-179](../.github/workflows/pr-check.yml#L99-L179)）主体是健全的：前端只读 check + `bun run test` + `build:vite`，Rust 侧跑三种 clippy 组合（含 shipped features）与 `cargo test`；`restore-keys` 保留了 toolchain cachekey，`target/*/.fingerprint` 已列入（AGENTS 里那两条 gotcha 已被上一个提交修掉）；`auto-merge` 的条件也改成了"检测成功 且 两个检查非失败"。仍有四个门禁缺口：

1. **没有任何工作流校验 `src/ipc/bindings/**` 与 Rust 类型同步**：全仓没有 `--all-features`、也没有 `bun run typegen`（grep 过所有 workflow）→ 绑定可以陈旧地发布，而 `tsgo --noEmit` 会照着陈旧文件一路绿灯（AGENTS 专门警告过这个陷阱）。
2. **shipped feature 只对 `-p dwall` 做了 clippy**（[pr-check.yml:176](../.github/workflows/pr-check.yml#L176)、`warm-cache.yml:120`）：`dwall-settings` 从未在 `log-max-level-info` 下 lint，而那正是 `debug!`/`trace!` 被编译掉、可能留下未使用绑定的配置（发布用的就是 `tauri build --features log-max-level-info`）。
3. **`.scripts/**`、`.packaging/**`、`docs/**` 不在 PR 触发路径里**（[pr-check.yml:11-25](../.github/workflows/pr-check.yml#L11-L25)、`:48-65`）：只改打包脚本或清单模板的 PR **不会触发任何工作流**，问题会在发布中途才暴露。
4. Rust job 没有 `timeout-minutes`（`publish.yml:234` 有 60）；没有 `actionlint`/PSScriptAnalyzer，因此 `.scripts/package-daemon.ps1` 的语法错误也要等到发布时才炸。

---

## 8. Rust 侧还需要补的测试用例

现状：全仓 **221 个** `#[test]`（dwall lib 101、bin 5、集成 7、settings 25、logging 3、time 80），本轮 `cargo test` 全绿。但覆盖面高度集中在**纯函数**（太阳数学、配置序列化、颜色方案判定、路径拼接）——**守护进程的编排层、平台层、设置端的下载/缓存/进程管理几乎为零**。

### 8.1 先要做的三处小重构（否则测不到）

1. **把 `ThemeEngine` 泛型化。** 现在 [theme_engine.rs:22-52](../crates/dwall/src/theme_engine.rs#L22-L52) 接收的是具体类型（`WallpaperSetter`/`DisplayMonitorProvider`/`ColorSchemeScheduler`/`GeographicPositionProvider<Positioner>`），所以 [tests/trait_mocks.rs](../crates/dwall/tests/trait_mocks.rs) 里那 4 个 mock 只能验证 mockall 自身（7 个测试全是自证）。`build_engine`（[daemon.rs:28-42](../crates/dwall/src/daemon.rs#L28-L42)）是唯一构造点，改成 `ThemeEngine<W, M, C, P>` 或 `&dyn` 聚合体即可打通整条周期逻辑。
2. **把 `std::process::exit(1)` 改成返回错误。** [daemon.rs:374-380](../crates/dwall/src/daemon.rs#L374-L380) 目前让"3 次失败"这条最重要的策略无法测试（也会连带杀掉测试进程）。
3. **把硬编码的 `DWALL_CONFIG_DIR` 变成参数。** `ThemeStatusProvider`（[theme/status.rs:19](../crates/dwall-settings/src/theme/status.rs#L19)）、`get_theme_catalog` 等直接读全局目录，同时这也是 2.2 里"每个条目读一次配置"的根因。

### 8.2 `crates/dwall`（守护进程）

P0（对应第 1、2 节的缺陷，写出来即回归网）：

| 目标文件 | 建议用例 |
| --- | --- |
| `daemon.rs` | `three_consecutive_cycle_failures_end_the_loop`（重构后）、`a_cycle_where_every_monitor_fails_is_a_failure`（D4；现在会失败，正好钉住要修的行为）、`random_mode_same_day_reload_keeps_todays_theme`、`random_mode_with_an_empty_pool_applies_nothing_and_does_not_fail`、`restore_rejects_a_removed_theme_and_redraws`、`the_random_state_file_lives_next_to_the_config_file`、`a_broken_config_keeps_the_last_good_configuration`（2.1；现在会失败） |
| `theme_engine.rs`（泛型化后） | `every_configured_monitor_is_attempted_even_when_one_fails`、`a_failing_monitor_does_not_stop_the_others`、`lock_screen_is_skipped_when_no_monitor_succeeded`、`lock_screen_theme_is_deterministic_for_a_multi_monitor_map`（D5 之外的 `get_first_configured_theme`；现在会失败）、`a_missing_image_is_reported_per_monitor`、`a_custom_theme_uses_its_manifest_image_format`、`monitors_without_a_configured_theme_are_skipped` |
| `config.rs` | `default_config_toml_round_trips`（`write_default` → `read_from_path` 相等；现在只测了 JSON 往返）、`write_default_contains_the_documented_header`、`write_atomically_keeps_the_old_file_when_the_rename_fails`、`write_atomically_leaves_no_temp_file_on_failure`、`validate_rejects_an_out_of_range_cache_minutes`（先加校验）、`validate_rejects_a_relative_themes_directory`、`config_watcher_reports_appeared_disappeared_and_modified`、`the_startup_baseline_is_not_a_change`、`forwards_compatibility_still_migrates_the_legacy_mirror_key`（已有） |
| `theme.rs` | `the_solar_cache_evicts_the_oldest_entry_at_capacity`、`a_cache_hit_moves_the_entry_to_the_back`、`the_solar_cache_never_exceeds_its_capacity`、`cached_theme_data_is_invalidated_when_solar_json_changes`（P3-9；现在会失败）、`select_next_skips_a_theme_removed_after_the_round_started`（P3-5b；现在会失败）、`a_manifest_theme_wins_over_solar_json`、`is_theme_directory_accepts_both_layouts`、`validator_rejects_altitude_out_of_range`、`validator_rejects_azimuth_360`、`validator_reports_a_missing_frame_image`、`validator_accepts_a_legacy_solar_json_theme`（`ThemeValidator` 目前 0 测试） |
| `color_scheme.rs` | `daylight_state_is_recomputed_on_a_new_day`（P3-1；现在会失败）、`the_threshold_cache_is_dropped_when_the_position_changes`、`threshold_at_the_23_5_45_66_5_boundaries`、`midnight_sun_svalbard_june` / `polar_night_longyearbyen_december` / `white_night_iceland_march`（真实坐标 + 日期，断言 `DaylightState` 变体）、`refraction_is_clamped_at_the_horizon`、`a_nan_altitude_does_not_flip_the_scheme` |
| `solar.rs` | `altitude_matches_a_reference_ephemeris_within_a_tolerance`（用 NOAA 计算器的金值，多城市 × 多时刻，容差 0.02°；现在只有区间断言与 insta 快照）、`azimuth_is_south_at_local_noon_in_the_northern_hemisphere`、`altitude_is_clamped_when_sin_alt_exceeds_one`、`the_position_cache_ttl_is_honoured`、`update_on_each_calculation_bypasses_the_cache` |
| `platform/windows/async_operation.rs` | `poll_returns_when_the_status_leaves_started`、`poll_times_out_on_a_wedged_operation`、`a_failed_status_is_surfaced_by_get_results`（给 `poll` 注入闭包即可，不需要真 WinRT） |
| `platform/windows/geolocation.rs` | `time_span_converts_seconds_to_100ns_ticks`（纯函数，现在 0 测试）、`an_altitude_of_zero_logs_the_barometer_hint`（需把纯逻辑从 COM 调用里拆出来） |
| `platform/windows/monitor.rs` | `the_monitor_cache_expires_after_its_ttl`、`an_unchanged_monitor_set_is_not_a_change`、`a_swapped_monitor_is_detected`、`a_resolution_change_is_not_detected`（钉住现状/缺陷）、`a_failed_friendly_name_is_not_cached_forever`（P3-12；现在会失败）、`the_friendly_name_cache_stays_bounded` |
| `bin/daemon.rs`（集成测试） | 用 `env!("CARGO_BIN_EXE_dwall")` 起进程：`help_exits_zero_and_prints_the_usage`、`an_unknown_flag_exits_two`、`a_broken_config_exits_three`、`check_config_on_a_missing_file_reports_defaults_and_exits_zero`、`once_refuses_with_five_while_a_daemon_runs`。**退出码是 `docs/headless.md` 记录在案的对外接口，现在零测试。** |
| `random_state.rs` / `lazy.rs` / `utils.rs` | `date_key_pads_single_digit_months_and_days`、`random_selection_round_trips`、`corrupt_selection_json_is_an_error`、`the_cache_rejects_an_unregistered_type_instead_of_aborting`（把 `unreachable!` 改掉之后）、`an_entry_expires_after_its_ttl` |

### 8.3 `crates/time`

`new_rejects_years_above_9999`（当前 `new(65535,…)` 能通过但 `year()` 回读成 9999）、`eq_is_reflexive_for_a_representable_instant`（当前 epoch 之前的值 `a != a`，违反 `Eq` 自反律——建议改成 i64 内部表示后钉住）、`from_timestamp_millis_round_trips`、`timestamp_and_offset_are_inverse`、`add_days_reports_overflow`、`with_hms_uses_the_offset_of_that_date_not_today`（DST，需先给 `with_hms` 传入 offset）、`is_before_is_consistent_with_ord_within_one_second`（P3-20；现在会失败）。

### 8.4 `crates/logging`

`the_previous_log_is_moved_into_backup_on_start`、`a_second_start_does_not_lose_the_first_log`、`the_backup_directory_does_not_grow_without_bound`（P3-22；现在会失败）、`enabled_filters_by_target_prefix`、`a_write_failure_does_not_panic`、`the_timestamp_is_local_with_an_offset`、`rotation_is_skipped_when_no_previous_file_exists`（临时目录即可测，现在 3 个测试全是 `parse_log_level` 与 `effective_max_level`）。

### 8.5 `crates/dwall-settings`

现有 25 个（`filesystem` 5 / `update` 8 / `network::mirror` 7 / `thumbnail::cache` 3 / `window` 2）。**零测试**：`archive`、`network::{client,tasks,progress}`、`daemon`、`process`、`theme::{downloader,paths,download_url,applier,status,validator}`、`thumbnail::{store,cleanup,storage}`、`setup`、`error` 与全部 `commands/*`。

优先级建议：

1. `archive.rs`：`parent_dir_entries_are_rejected`、`absolute_entry_names_stay_inside_the_target`、`an_archive_over_the_entry_limit_is_refused`、`a_stray_top_level_file_keeps_the_root_directory`（S2）、`a_non_zip_payload_is_refused`、`a_declared_size_lie_does_not_bound_the_written_bytes`（S4，断言**实际**上限）。
2. `theme/downloader.rs`：`a_cancelled_download_is_not_retried`（2.2 表）、`a_failed_extraction_leaves_no_zip`、`an_archive_without_solar_json_does_not_replace_an_installed_theme`（S2）、`a_failed_swap_keeps_the_previous_theme`。
3. `thumbnail/cache.rs`：`a_cached_file_is_not_reused_for_a_different_url`（S3）、`different_urls_for_one_thumbnail_take_different_locks`、`a_non_image_body_is_removed_and_reported`。
4. `network/client.rs`（用 `std::net::TcpListener` 起一个极简 HTTP 夹具，不引入 hyper/axum）：`resume_only_happens_on_206`、`a_200_answer_truncates_the_partial_file`、`the_cancel_flag_stops_the_transfer`、`get_text_rejects_a_body_over_the_limit`、`progress_is_zero_when_the_length_is_unknown`。
5. `filesystem.rs`：`a_destination_inside_the_source_is_refused_case_insensitively`（S5；现在会失败）、`an_existing_destination_is_refused`、`copy_dir_recurses_into_nested_directories`、`find_files_in_dir_matches_the_extension_case_insensitively`。
6. `network/{tasks,progress}.rs`：`a_duplicate_task_is_refused`、`the_guard_removes_the_task_on_drop`、`cancelling_an_unknown_task_is_a_noop`；`one_emit_per_percent`、`no_emit_when_the_total_is_unknown`、`the_final_chunk_always_emits`。
7. `thumbnail/{cleanup,store}.rs`：`files_older_than_thirty_days_are_removed`、`the_newest_files_are_kept_to_reach_the_size_limit`、`empty_theme_dirs_are_removed`、`a_load_moves_a_file_out_of_the_eviction_set`（P3-29；现在会失败）、`the_same_key_returns_the_same_lock`。
8. `daemon.rs` / `process.rs` / `setup.rs`：`should_run_matrix_for_random_and_fixed_modes`、`the_last_error_line_is_returned`、`a_log_without_errors_returns_none`、`a_missing_log_returns_none`、`the_exe_name_stops_at_the_first_nul`、`resolve_daemon_path_requires_an_existing_sibling`、`a_directory_named_dwall.exe_is_rejected`。
9. `theme/{paths,download_url,status}.rs` 与 `update.rs`：`staging_dir_does_not_collide_with_an_installed_theme`、`spaces_become_dots`、`the_asset_is_pinned_to_the_themes_tag`、`all_monitors_agreeing_returns_that_theme`、`a_mis_flagged_prerelease_does_not_beat_a_newer_stable`。

前端侧（顺带）：**完全没有 vitest 配置**（`vite.config.ts` 无 `test` 块，`jsdom` 装了但无用），因此 store/组件/hook 一个都测不了，现在 4 个测试全是纯函数。要测 store 必须先加 `vitest.config.ts`（`environment: "jsdom"` + `resolve.conditions: ["development","browser"]`），否则 Solid 会解析到 server 构建。

---

## 9. 建议的修复批次

| 批次 | 内容 | 理由 |
| --- | --- | --- |
| 0 | **发布链路**：修 `main`/`dev`/tag 的关系、让 `update-main-version` 的目标分支与发布分支一致、断言 ff-only 结果、让 `notify-completion` 反映它的结果、给 `publish.yml` 加 `concurrency`、把 `Cargo.lock` 纳入版本提升提交、修 `delete-nightly-release-and-tag` 的 `needs` | 现在"版本提升 + 清理"这条链已经静默失败了多个版本（main 停在 0.1.33、`origin/release` 停在 0.1.34），比任何代码缺陷都更该先处理 |
| 1 | D1（失败不再退出/退避）、D2+D3（null config + 错误可见） | 用户可感知的"应用不工作/界面崩"，且都是小改动 |
| 2 | S1 + S2（下载前的路径校验 + 交换前校验主题） | 数据损失面最大，改动最小 |
| 3 | D5 + D6 + D4（两个显示器缓存合一、变化检测走缓存、失败计入阈值） | 热插拔与"新显示器没墙纸"的直接原因 |
| 4 | S3/S4/S5 + `.zip` 泄漏 + 取消不重试 | 资源与可靠性集群 |
| 5 | 日志：轮转上限、`GetTimeZoneInformation` 缓存、去掉 `debug!(?config)`、补 `has_configuration_changed` 的 warn | 可观测性，成本低 |
| 6 | 配置生效延迟（D6 的 `interval` 问题）+ 设置端写盘顺序 + 设置端会话过滤/等待退出 | 交互与生命周期正确性 |
| 7 | 第 8 节测试（先 8.1 的三处重构） | 让以上修复有回归网 |
| 8 | CI 补门禁（bindings 同步、settings shipped-feature clippy、`.scripts`/`.packaging`/`docs` 触发路径、action 固定版本 + dependabot `github-actions`） | 防止同类问题再次漏进来 |
| 9 | P3 清理（死代码、`macros.rs.bak`、`rust-version`、`icon.icns`、死 CSS、capability 收窄、`docs/custom-themes.md` 布局说明、`.zip`/清单哈希来源） | 机械、可批量 |

## 10. 本轮未验证 / 需真机确认

- **`RequestAccessAsync` 在"定位服务被系统关闭"时返回 `Denied` 还是 `Unspecified`**：两者都走 D1 的退出路径，但日志文案不同；需真机（本机已安装的守护进程在运行，我没有动它）。
- **`GetGeopositionAsync(maxAge, timeout)` 在服务被禁用时的实际返回延迟**：决定 D1 的 45 秒是否准确。
- **`find_daemon_process` 跨会话行为**：需要第二个用户会话/RDP 才能复现。
- **`Storage_Streams` / 设置端的 `Win32_System_ProcessStatus`、`Win32_Globalization`、冗余的 `Win32_System_Registry` 是否可删**：需改 manifest 后跑一次 release 构建确认（我没有改动任何被跟踪文件）。`Win32_Security` 已定论：**必需**（`CreateMutexW` 带该 feature 门控）。
- **发布链路的状态**：`main`/`dev`/tag 的关系、`origin/release` 的残留、`dist-manifests` 里的旧哈希都是通过 `git`/文件读取实测的；**`publish.yml` 失败的具体时序（是 merge 步骤失败还是 push 被拒）没有实跑验证**（需要触发一次 release）。
- **`target/` 的 79 GB 与 GitHub 的 10 GB 缓存预算**：本机是多次 feature/多次构建累积的结果；CI 冷构建是否真的装得下 `target/release/deps`（本机 10.66 GB）需要 `gh cache list` 实测，不要凭假设。
- **本机 `/target` 与 `dist/` 是 10/17 12:53 从"未提交的工作树"构建的**（cargo 认为与当前源一致，但我没有重新链接）；体积数字对当前源码有效，不同 `--features` 组合的体积差异未单独测量。
- **daemon 二进制的逐 crate 体积占比没有测量**（需要 `cargo bloat` + 重建）；第 6.2 节的排序只是按依赖权重的推测（已标注）。
- 前端专项评审里的**运行时行为**结论（toast 流、抽屉聚焦、Solid store 语义）我逐项复核了关键链条（D2/D3 与死 CSS 已实测），其余为源码级推断。
- 内存采样基于**已安装的 0.3.0-alpha.4**（不是 `target/release/dwall.exe`），36 分钟 / 144 个周期只能界定而不能证伪更慢的泄漏。

---

## 11. 修复记录：批次 1 / 2 / 3（2026-10-17 本轮已实现）

只做了第 9 节的**批次 1（D1、D2+D3）、批次 2（S1、S2）与批次 3（D5、D6、D4）**，其余批次未动。

### D1 ✅ 守护进程不再因持续失败而退出

- [daemon.rs](../crates/dwall/src/daemon.rs)：删掉 `MAX_CONSECUTIVE_FAILURE_THRESHOLD` 与 `std::process::exit(1)`，新增 `CycleFailureTracker` + `failure_backoff()`——每次连续失败退避翻倍（5 s 起、300 s 封顶），成功即归零；失败计数用 `u32` + `saturating_add`，配置重载会重建 tracker（用户改配置是最可能的修复手段，立即重试）。日志改为 `error!(error, consecutive_failures, retry_in_seconds, "Theme update cycle failed, retrying")`。
- 行为变化：定位被拒/服务关闭不再让守护进程消失；墙纸不更新的原因仍会每轮写入日志（设置端读最后一条 ERROR 的逻辑不受影响）。
- 测试（7 个）：`the_backoff_starts_at_the_base_delay`、`the_backoff_doubles_per_consecutive_failure`、`the_backoff_stops_growing_at_the_ceiling`、`a_successful_cycle_resets_the_failure_run`、`three_consecutive_failures_no_longer_end_the_run`（钉住"没有终止态"）、`a_failure_run_that_lasts_for_days_neither_panics_nor_overflows`、`the_capped_backoff_is_never_shorter_than_the_base_delay`。
- **未做（批次 3 的 D4）**：`process_monitors` 的逐显示器失败仍未回传给失败计数——现在它不再影响生死，只是"全失败时是否记一条 error"的问题。

### D2 + D3 ✅ 设置页不再因缺配置崩溃、加载失败可见

- [settings.store.ts](../src/store/settings.store.ts)：`patch`/`setPositionType`/`patchPosition`/`patchSocks5` 在 `config` 为空时直接返回（不再写 `{}`）；`load()` 现在是幂等的重试入口（先清 `error`/`saveError`，失败写 `status: "error"`），并导出给视图。
- [SettingsView.tsx](../src/views/SettingsView.tsx)：三个依赖配置的 section 只在 `status === "ready"` 时渲染；`error` 状态显示错误详情 + "重试"按钮；加载中显示 `common.loading`；`AboutSection`（日志目录/更新检查）始终可见，因为排查坏配置正需要它。
- i18n：新增 `settings.loadFailed`、`settings.retry`，6 个语言文件同步（en-US/zh-CN/zh-HK/zh-TW/ja-JP/ko-KR）。
- **顺带修掉一个新发现的缺陷**：`load()` 曾把 `config` 与 `saved` 存成**同一个对象**，而 Solid 的 `setStore` 是就地合并，因此刚打开窗口时 `isConfigDirty` 恒为 false——头部的"保存配置/放弃"按钮永远不会启用（内联保存图标走的是直接调用 `save()`，所以此前没被发现）。现在 `saved` 用深拷贝快照（`snapshot()`，`load`/`save`/`applyWallpaperMode`/`discard` 一致），`patchPosition` 也只在 MANUAL 变体上生效（此前会把坐标塞进 AUTOMATIC 变体）。
- 测试：新增 [settings.store.test.ts](../src/store/settings.store.test.ts)（13 个）——空配置下所有写入口都是 no-op、`save` 不落盘、加载成功/失败/重试恢复、草稿合并、`patchPosition` 只改目标字段、`setPositionType` 的字段播种、`isConfigDirty` + `discard`、`save` 成功/失败、`applyWallpaperMode` 失败时快照不前移。
- 前置条件：新增 [vitest.config.ts](../vitest.config.ts)（`resolve.conditions: ["development","browser"]`，否则 vitest 会把 `solid-js/store` 解析到 SSR 构建，测到的是另一套实现）。

### S1 ✅ 下载命令校验两个主题根目录

- [filesystem.rs](../crates/dwall-settings/src/filesystem.rs)：新增 `validate_theme_roots(&Config)`，把两条 `validate_directory` 收成唯一入口；[commands/download.rs](../crates/dwall-settings/src/commands/download.rs)、[commands/config.rs](../crates/dwall-settings/src/commands/config.rs)、[theme/applier.rs](../crates/dwall-settings/src/theme/applier.rs) 全部改用它（下载路径此前是唯一漏检的）。
- 测试（4 个）：绝对路径通过；`themes_directory` 相对路径拒绝；`customized_themes_directory` 相对路径拒绝；任一根含 `..` 拒绝。

### S2 ✅ 交换前校验"解压出来的确实是一个主题"

- [theme/downloader.rs](../crates/dwall-settings/src/theme/downloader.rs)：新增 `verify_staged_theme()`，在 `remove_dir_all(theme_dir)` + `rename` **之前**用守护进程自己的校验器检查 staging 目录（`is_customized = false`，与守护进程的读取规则一致）；失败则清理 staging、`error!` 记录并返回错误，**保留已安装的主题**。
- 为避免"校验器和守护进程的规则漂移"，直接复用 `dwall::ThemeValidator`；但它读的是按路径缓存的解析结果，而 `<id>.staging` 会被同一主题的每次下载复用——所以 [theme.rs](../crates/dwall/src/theme.rs) 新增了 `ThemeValidator::validate_uncached()`（两个入口共用同一份校验体），下载路径用它。**这一点有专门的回归测试**：`reusing_a_staging_directory_revalidates_its_new_contents`——我实测把它改回缓存版 `validate` 时该测试**确实失败**（第二次下载会沿用第一次的帧表），改回 `validate_uncached` 才通过。
- 测试（7 个）：合法 legacy 主题通过；**多一个顶层文件导致嵌套的归档被拒绝**（S2 的原始故障模式）；空目录拒绝；缺帧图片拒绝；清单主题按自身 `image_format` 校验；无可用名字的 staging 拒绝；同路径复用 staging 时按新内容重新校验。
- 兼容性实测：本机 `%APPDATA%\dwall\themes` 里的真实主题是 `solar.json` + `jpg/1..8.jpg`（10 帧含重复索引、8 张图全在），全部通过新校验 ✓。唯一的行为变化：**缺图的主题包会被整体拒绝**（此前会装上、并在太阳走到那一帧时报 warn）；这是"绝不删掉可用主题"的代价，且错误信息会指出缺失文件。

### D5 ✅ 显示器缓存合一，热插拔立即生效

- 根因是两个 `DisplayMonitorProvider`：引擎一个（[theme_engine.rs](../crates/dwall/src/theme_engine.rs)）、`WallpaperSetter` 自己又一个（原 [wallpaper.rs:89](../crates/dwall/src/platform/windows/wallpaper.rs#L89)）。引擎刷新后 setter 仍持旧快照 → `MonitorNotFound` 直接返回，不进入 refresh+retry。
- [daemon.rs](../crates/dwall/src/daemon.rs) 的 `build_engine` 现在只建一个 `Rc<DisplayMonitorProvider>`，[WallpaperSetter::new](../crates/dwall/src/platform/windows/wallpaper.rs) 接收它，引擎与 setter 共用同一份快照（守护进程单线程，`Rc` 足够）。
- 另外补了防御：`set_monitor_wallpaper_internal` 在快照里找不到显示器时先 `refresh_monitors()` 再查一次（只走失败路径，且用延迟初始化借用刷新后的表，不产生每周期克隆），把"缓存过期"从数分钟变成一次重试。

### D6 ✅ 变化检测不再每周期全量枚举

- [monitor.rs](../crates/dwall/src/platform/windows/monitor.rs)：新增 `DisplayConfigShape`（来自一次 `GetDisplayConfigBufferSizes`）与 `shape_forces_full_check()`——形状未变且距上次全量比对不足 60 s 时直接返回"未变化"，**不再做 `QueryDisplayConfig` + 每显示器 SetupAPI 枚举**；形状变化或 60 s 到期才走原来的键比对。语义依据：墙纸按设备路径寻址，分辨率变化不需要重新应用，插拔才会改变路径数；"数量与键都相同"的极小概率情形由 60 s 的兜底比对覆盖。
- 缓存新增 `shape` / `last_full_check` 两个字段，由变化检测维护（`update()` 不碰它们——有测试钉住）。
- 顺带修掉守护进程里**唯一一处错误完全不留痕**的地方：[theme_engine.rs](../crates/dwall/src/theme_engine.rs) 的 `has_configuration_changed().unwrap_or(false)` 现在会 `warn!`；并改正了 `CACHE_EXPIRY_SECONDS` 上那句已不成立的注释（"减少 90% API 调用"）。
- 测试（7 个）：`the_first_check_always_enumerates`、`an_unchanged_shape_within_the_interval_skips_the_enumeration`、`attaching_or_detaching_a_monitor_forces_the_enumeration`、`a_changed_mode_count_forces_the_enumeration`、`the_full_comparison_still_runs_once_the_interval_elapses`、`the_monitor_cache_stops_serving_the_list_once_it_expires`、`refreshing_the_list_keeps_the_shape_check_intact`。

### D4 ✅ 逐显示器失败计入周期结果

- 原先 `process_monitors` 把每个显示器的错误降级成 `warn!` 后返回 `Ok(success_count)`，`process_theme_cycle` 永远 `Ok(())`，`run_once` 永远 `Ok(true)`——失败既不触发退避，也不进设置端会读的 ERROR 行。
- [theme_engine.rs](../crates/dwall/src/theme_engine.rs)：拆出两个可测的纯函数——`monitor_targets()`（产出 `(显示器, 主题)` 目标，无配置的显示器跳过并 `debug!`）与 `apply_monitor_targets()`（统计 `applied`/`failed`，失败仍逐条 `warn!`）；**只有"配置过的显示器全部失败"才让周期失败**（新错误 `ThemeError::AllConfiguredMonitorsFailed { failed }`），且错误在颜色方案更新**之后**才返回，避免坏主题连带停掉日出日落切换。
- 于是默认空配置（没有任何显示器主题）仍是 `Ok`、`--once` 依旧退出 0；只有"配了但应用不了"才是错误，并进入带退避的重试。
- `ThemeEngine::run_once` 的返回类型由无意义的 `DwallResult<bool>`（永远 `Ok(true)`）改为 `DwallResult<()>`。
- **文档同步**：[docs/headless.md](./headless.md) 的退出码表补上"`--once` 对所有已配置显示器都失败时退出 1"，故障排查表新增一行说明退避与 `--once` 行为，并把原文那句**错误**的"守护进程保留上一份配置"改成实际行为（继续运行并每 5 秒重试；配置不可读期间不应用任何墙纸）。
- 测试（6 个）：`a_monitor_without_a_configured_theme_is_not_a_target`、`every_monitor_is_a_target_of_the_all_variant`、`a_configuration_without_a_monitor_map_has_no_targets`、`every_configured_monitor_failure_is_counted`、`one_successful_monitor_keeps_the_cycle_from_failing`、`nothing_to_do_is_not_a_failure`。

### 本轮验证命令与结果

| 命令 | 结果 |
| --- | --- |
| `cargo fmt --all -- --check` | exit 0 |
| `cargo clippy -- -D warnings` / `cargo clippy -r -- -D warnings` | exit 0 / 0 |
| `cargo clippy -r -p dwall --features build-script,log-max-level-info --all-targets -- -D warnings` | exit 0 |
| `DWALL_SKIP_BUILD=1 cargo clippy -p dwall-settings --all-targets -- -D warnings` | exit 0 |
| `DWALL_SKIP_BUILD=1 cargo test` | **252 passed / 0 failed**（批次 1/2 后为 239；dwall 108→121） |
| `bun run check`（tsgo + biome，385 文件） | exit 0，0 error / 0 warning |
| `bun run test` | **32 passed** |
| `bun run build:vite` | 成功；JS 231.3→232.6 KB、CSS 121.9→122.2 KB（批次 1/2 新增的错误面板与两条 i18n；批次 3 不涉及前端） |

### 仍未做（后续批次）

S3/S4/S5、`.zip` 泄漏、取消不重试、日志轮转上限与 `GetTimeZoneInformation` 缓存、设置端写盘顺序与会话过滤（批次 6）、第 8 节其余测试（含 8.1 的 `ThemeEngine` 泛型化与 `bin/daemon.rs` 的退出码集成测试）、第 9 节批次 8/9（CI 门禁与 P3 清理）。
另外仍**没有**把守护进程的"最后一条 ERROR"暴露到设置端状态区（`get_engine_status` 只返回 `bool`）：D1/D4 之后守护进程会带错误继续运行，用户想看到原因仍需点开日志目录——建议作为批次 5 的一项。
