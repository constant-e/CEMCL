---
description: "CEMCL 项目结构与规范：6 个 crate 与依赖图、UICommand/UIUpdate 消息流、账号头像（皮肤头部正面）与缓存、~/.cemcl 数据文件与镜像替换、命名/错误/i18n 约定、sha1 哈希校验、已修复怪癖、2 条 TODO 精确位置"
---

# CEMCL 项目总结（结构、规范、风格）

> 依源码核对于 2026-09-22。来源 `.github/instructions/CEMCL-project-summary.instructions.md`，其中依赖图已修正。
> 仓库 constant-e/CEMCL，版本 0.3.1，Rust edition 2024。

## 1. 项目概览

CEMCL（CE Minecraft Launcher）是一个 Rust + Slint 的 Minecraft 启动器。Workspace 含 6 个 crate：

| Crate | 类型 | 职责 |
|---|---|---|
| `app` | bin（`cemcl`） | 应用入口与运行时：账号/版本/Java 管理器、账号头像、事件循环、启动流程编排 |
| `frontend` | lib | Slint UI 封装：`AppWindow`、`UICommand`/`UIUpdate`、各页面数据模型转换 |
| `mc` | lib | MC 核心：版本清单解析、启动参数、认证（OAuth/离线）、皮肤与头像、下载清单构建（自带 `TaskInfo`） |
| `downloader` | lib | 异步下载引擎：`DownloadTask` 状态机、`TaskSet`、`DownloadManager` 广播状态 |
| `java` | lib | Java 运行时探测与版本解析（`JavaInstallation`、`JavaVersion`） |
| `utils` | lib | 文件工具（目录列举、规则检查、下载辅助） |

**实际依赖（以各 Cargo.toml 为准）**：`app → {frontend, mc, downloader, java, utils}`（全部直接依赖）；`mc → utils`；`frontend` / `downloader` / `java` 不依赖任何内部 crate。`java` 仅被 app 使用，frontend 自带独立的 `JavaInfo`（`crates/frontend/src/java.rs`）。
注意：`mc::download::TaskInfo` 与 `downloader::task::TaskInfo` 是两个独立类型（`crates/mc/src/download/mod.rs` / `crates/downloader/src/task.rs`），由 app 侧转换；mc 不依赖 downloader。

## 2. 架构与消息流

```
Slint UI (res/ui/*.slint)
  │ callback
  ▼
frontend::AppWindow ──UICommand──▶ tokio mpsc unbounded ──▶ AppRuntime::run (tokio::select!)
  ▲                                                              │
  │ UIUpdate（第二条 unbounded；AppWindow::handle + upgrade_in_event_loop）
  │                                                              │
DownloadManager ◀── add/start/pause/resume/cancel ────────────────┤
（broadcast task-set 状态，500ms，容量 64）──▶ Runtime ────────────┤
mc::download_*（返回 mc::TaskInfo 列表）──▶ Runtime 转成 downloader 任务
```

- **UI → App**：`UICommand` 枚举（`crates/frontend/src/app_window.rs`），经 `tokio::sync::mpsc::unbounded_channel` 发送，`AppRuntime::run` 中 `tokio::select!` 消费。
- **App → UI**：`UIUpdate` 枚举，经另一条 unbounded channel；`AppWindow::new` 内 spawn 的分发任务 `while let Some(update) = update_rx.recv().await` 调 `AppWindow::handle`，逐项用 `upgrade_in_event_loop` 更新主窗口/对话框。
- **下载状态**：`DownloadManager` 内部 `broadcast::Sender<TaskSetStatusInfo>`（`channel(64)`），`start_broadcast` 每 500ms tick 广播全部任务集状态；runtime 在 select! 中接收并转成 `UIUpdate::SetTaskSetList`。
- **账号头像**：正版账号的皮肤在启动时于后台更新（`app/src/avatar.rs`，见 §5），结果经第三条 unbounded channel 回到 `AppRuntime::run` 的 select! 分支，写入 `AvatarManager` 后 `refresh_ui_acc_list()`（整个列表带头像重发）。
- **对话框**：5 个对话框的 `slint::Weak` 存于 `Arc<Mutex<Option<slint::Weak<...>>>>` 字段；`XxxDialog::new()` + `show()` 创建展示，`ForgeDownloadDialog` 在 UI 回调内用 `slint::invoke_from_event_loop` 创建后存入。

## 3. 配置与数据文件

程序启动时 `main.rs` 将工作目录切到 `~/.cemcl`（不存在则创建），所有配置以 JSON 文件存放：

- `config.json` — 全局配置（通用/下载/游戏）；`frontend::Config*` ↔ `app::Config*` ↔ `downloader::Config` 三层结构重复，靠手写 `From` 转换（结构性重复，有意保留）。
- `account.json` — 账号列表 + `current` 索引。
- `avatar/<uuid>.png` — 账号头像缓存（皮肤头部正面，8×8 PNG；uuid 小写去连字符，见 §5）。
- `versions.json` — MC 安装列表（CEMCL 格式）；另存 `<游戏路径>/launcher_profiles.json` 兼容原版启动器。2026-10-09 起：启动时与 `<游戏路径>/versions` 核对，删除无效项（缺版本 json）并写回，`current` 以版本名锚定、列表按名字排序；版本配置项缺省、为 null 或留空（0/空串/空列表）时使用 `config.json` 的默认值，与默认值相同的配置保存时省略（`app/src/version.rs`，测试 `cargo test -p app`）。
- `java.json` — Java 安装列表 + `default` 索引。

**镜像系统**：下载 URL 含 `{assets_source}`、`{fabric_source}`、`{game_source}`、`{libraries_source}`、`{forge_source}` 占位符（mc 生成，app 写入 `mirrors` 映射），`DownloadManager::add_taskset` 对每个键 `format!("{{{k}}}")` 后 `url.replace`。

## 4. 命名与代码规范

- **私有辅助函数**：`i_` 前缀（`i_load`、`i_save_config`、`i_get_java_path`）。
- **转换函数命名**：`frontend_*`（app 侧，转成 UI 类型，如 `frontend_mc_config`、`frontend_account`）；`ui_*`（frontend 侧，构建 Slint 模型，如 `ui_acc_list`、`ui_java_list`）；枚举字符串转换 `to_*`（如 `to_account_type`）。
- **错误处理**：每个 crate 手写错误枚举 + `From` 转换 + `Display`（不用 thiserror/anyhow）；错误日志统一 `error!("{e}")`。
- **注释**：中文 `///` 文档注释与英文混用；行内注释英文为主。
- **格式化**：标准 `cargo fmt` 风格；`use` 按 crate 分组。
- **异步**：tokio full features；下载任务内部用 mpsc channel 做命令控制（Pause/Resume/Cancel），`try_lock` 保护状态。
- **测试**：集成测试在各 crate 的 `tests/` 目录（`cargo test --workspace`）；`app` 为 lib + bin 双 target（`src/lib.rs` 声明模块、`src/main.rs` 只留入口），因此 `VersionManager` 等可直接用 `app::version::…` 在 `tests/` 中测试。
- **Slint**：入口 `res/ui/app-window.slint`，页面/对话框/组件见 `rule://CEMCL-ui-style-summary`；`build.rs` 用 `slint_build::compile_with_config` + `with_bundled_translations("res/translation")` + `with_style("fluent")`。
- **无边框窗口**：全部窗口（主窗口 + 6 个对话框）自绘标题栏（`no-frame`），最外层统一包 `components/window-frame.slint` 的 `WindowFrame`（窗口背景 + 1px `Palette.border` 描边），对话框只显示右上角关闭按钮（与主窗口同款 46×40、悬停变红）。移动窗口走 `.slint` 回调 `drag-window` → `frontend/src/ui.rs` 的 `drag_window()` → `slint::winit_030::WinitWindowAccessor` + `winit::Window::drag_window()`（`crates/frontend/Cargo.toml` 启用 slint 的 `unstable-winit-030` feature；每个窗口的 Rust 构造函数都要接 `on_drag_window`）；因为在 WM 交互移动期间应用收不到抬起事件，`drag_window()` 还会在事件循环里补发一次窗口外的 `PointerReleased` 复位 Slint 输入状态（细节见 `rule://CEMCL-ui-style-summary` §4–§5）。最小化/最大化/关闭直接在根元素上改 `root.minimized` / `root.maximized` / `root.close()`。
- **翻译更新**：`crates/frontend/update_translations.sh`（`slint-tr-extractor` 生成 frontend.pot，`msgmerge` 更新 zh_CN po）。

## 5. 账号头像（皮肤头部正面，2026-09-22 加入）

- **与游戏一致**（最新 Java 版）：默认皮肤取 `DefaultPlayerSkin.get(uuid)` = `DEFAULT_SKINS[Math.floorMod(uuid.hashCode(), 18)]`（9 个名字 alex…zuri × slim/wide；`getDefaultSkin()` = slim/steve）；贴图处理对齐 `SkinTextureDownloader.processLegacySkin`：只接受 64×32 / 64×64（HD 视为无效皮肤），内层头部强制不透明（保留 RGB），第二层（帽子层）按源 alpha 叠加在内层上，旧版 64×32 贴图右半部分整块不透明时清成透明（`doNotchTransparencyHack`）。实现：`crates/mc/src/account/skin.rs`；26.3 客户端的 9 张默认皮肤打包在 `crates/mc/res/skins/*.png`（wide 变体，头部像素与 slim 完全相同，头像只需头部）。
- **头像 = 头部正面 8×8 RGBA**（`mc::account::skin::Avatar`）：正版账号来自 `sessionserver.mojang.com/session/minecraft/profile/<uuid>` 返回的 `textures.SKIN.url`（204、无 SKIN 都视为没有自定义皮肤 → 按 uuid 选默认皮肤，与游戏内一致）；离线账号（`Legacy`）直接用 uuid 对应的默认皮肤；`Other` 与「加载失败且无缓存」用游戏默认皮肤 Steve。
- **缓存与后台更新**（`crates/app/src/avatar.rs`）：启动时先读 `avatar/<uuid>.png`（没有就先用默认头像），正版账号再用 `tokio::spawn` 拉最新皮肤，结果经独立 unbounded channel 回运行时；网络失败保留原头像、缓存不写坏。删除账号会一并删除缓存文件。
- **离线账号默认名**：`Account::default()` 生成 uuid 后取该 uuid 的默认皮肤名（Steve、Alex、Zuri…）作为 `user_name`。
- Rust 侧测试：`cargo test -p mc`（默认皮肤名向量来自 JDK 上的 `DefaultPlayerSkin` 算法；合成/旧版/尺寸校验用例）。

## 6. 已修复怪癖（2026-09-09 修复；2026-09-22 复核仍成立）

- `utils::get_parent_dir` 改用 `Path::parent()`。
- `mc::download::libraries::extract_lib` 重写：直接按平台/架构匹配从 jar 中读出 natives（`.dll`/`.dylib`/`.so`），目标已存在则跳过；不再整包解压到 `temp_dir`（旧实现在每次启动时会写/删约 3.7 万个临时文件，占启动准备阶段 90% 以上耗时，并在解压失败时泄漏临时目录）。调用点统一走 `extract_lib_logged`（失败记日志、不阻断启动）。
- `utils::download` 改用共享 `reqwest::Client`（连接 10s / 整体 60s 超时 + 退避重试），此前 `reqwest::get` 无超时，网络黑洞时可在「启动中 0%」卡到 TCP 超时。
- Java 不兼容后缀翻译：AppWindow `out property <string> not-compatible-text: @tr("not compatible")` 桥接，Rust 读取后格式化。
- `AccountType::Other` 序列化为 `"Other"`。
- `account.rs` 的 `save()` 中无空 `error!` 日志。
- `open-edit-java-dialog` 回调已声明但未实现（JavaPage 无 Edit 按钮，当前不可触发）。

## 7. 哈希校验（sha1，2026-10-08 加入）

下载的每个可校验文件都会在下载后、以及启动准备时（本地已存在时）核对 sha1；不一致的文件会被删除并重新下载，坏文件不会留在磁盘上（下载器以追加方式写文件，重取前必须先删除）。

- **utils**：`sha1_file`（流式读取，小写十六进制）、`to_hex`；`download(url, path, max, sha1)` 在写入后校验，失配时删除文件并按 `max` 重试；`DLError` 补了 `Display`/`Debug`。依赖 `sha1 = "0.11.0"`（与 zip 已用的同版本）。
- **downloader**：`task::TaskInfo`/`DownloadTask` 增 `sha1`（预期哈希）与 `sha1_url`（maven 风格的 `<url>.sha1`，二者择一，`sha1` 优先）；任务流写完后 `verify()`：有 `sha1` 直接比对，只有 `sha1_url` 时先取回哈希（网络错误重试 3 次；取不到只 `warn` 并跳过校验，兼容不提供 .sha1 的镜像）并缓存到 `<save_path>.sha1` 供之后离线校验；不一致时删除文件、状态置 `Failed` 并返回 `DownloadTaskError::Failed`。`DownloadManager::add_taskset` 的镜像占位符替换同时作用于 `sha1_url`。
- **mc**：`download::needs_redownload`（缺失或哈希不符，`pub`，供测试与调用方使用）、`remove_file_if_exists`、`read_cached_sha1`（解析 maven 格式，允许大写与文件名）。
  - `download_libraries`：natives classifier 与 artifact 按 json 中的 `sha1`；fabric 库的 profile json 不带哈希，下载时从 `{fabric_source}/<path>.sha1` 校验，之后按 `<jar>.sha1` 缓存离线校验（无缓存视为无法校验，保留文件）；fabric 分支现在会先建下载目录（此前缺失，首个 fabric 库会因父目录不存在而下载失败）。两个函数及 `download_assets` 因在校验时读文件改为 `async`。
  - `download_assets`：按资源索引中 `objects.<name>.hash` 校验。
  - `launch.rs`：客户端 jar 用 `downloads.client.sha1`、资源索引用 `assetIndex.sha1`（mod 继承时取父版本 json）。
  - `manifest.rs`：`MCDL` 增 `sha1`（版本清单 `versions[].sha1`），`download_mc` 下载/校验版本 json；fabric profile json 与 forge 安装器无哈希来源，不校验。
- **测试**：`cargo test --workspace`（各 crate 的 `tests/`：sha1 已知向量、本地 HTTP 服务下的下载校验/重试/缓存，以及各处“缺失/一致/不一致”的决策用例；maven 格式的 `.sha1` 解析由 fabric 缓存与 sha1_url 用例覆盖）。

## 8. 剩余 TODO（截至 2026-10-08，共 2 条）

1. `crates/frontend/src/app_window.rs:248` — `// TODO: implement edit java dialog`
2. `crates/mc/src/account/account.rs:7` — `Other, // TODO: implement other account type`
