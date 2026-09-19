---
description: "CEMCL 项目结构与规范：6 个 crate 与依赖图、UICommand/UIUpdate 消息流、~/.cemcl 数据文件与镜像替换、命名/错误/i18n 约定、已修复怪癖、6 条 TODO 精确位置"
---

# CEMCL 项目总结（结构、规范、风格）

> 依源码核对于 2026-09-18（所有行号同为 2026-09-18 实测）。来源 `.github/instructions/CEMCL-project-summary.instructions.md`，其中依赖图已修正。
> 仓库 constant-e/CEMCL，版本 0.3.0，Rust edition 2024。

## 1. 项目概览

CEMCL（CE Minecraft Launcher）是一个 Rust + Slint 的 Minecraft 启动器。Workspace 含 6 个 crate：

| Crate | 类型 | 职责 |
|---|---|---|
| `app` | bin（`cemcl`） | 应用入口与运行时：账号/版本/Java 管理器、事件循环、启动流程编排 |
| `frontend` | lib | Slint UI 封装：`AppWindow`、`UICommand`/`UIUpdate`、各页面数据模型转换 |
| `mc` | lib | MC 核心：版本清单解析、启动参数、认证（OAuth/离线）、下载清单构建（自带 `TaskInfo`） |
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
- **对话框**：5 个对话框的 `slint::Weak` 存于 `Arc<Mutex<Option<slint::Weak<...>>>>` 字段；`XxxDialog::new()` + `show()` 创建展示，`ForgeDownloadDialog` 在 UI 回调内用 `slint::invoke_from_event_loop` 创建后存入。

## 3. 配置与数据文件

程序启动时 `main.rs` 将工作目录切到 `~/.cemcl`（不存在则创建），所有配置以 JSON 文件存放：

- `config.json` — 全局配置（通用/下载/游戏）；`frontend::Config*` ↔ `app::Config*` ↔ `downloader::Config` 三层结构重复，靠手写 `From` 转换（结构性重复，有意保留）。
- `account.json` — 账号列表 + `current` 索引。
- `versions.json` — MC 安装列表（CEMCL 格式）；另存 `<游戏路径>/launcher_profiles.json` 兼容原版启动器。
- `java.json` — Java 安装列表 + `default` 索引。

**镜像系统**：下载 URL 含 `{assets_source}`、`{fabric_source}`、`{game_source}`、`{libraries_source}`、`{forge_source}` 占位符（mc 生成，app 写入 `mirrors` 映射），`DownloadManager::add_taskset` 对每个键 `format!("{{{k}}}")` 后 `url.replace`。

## 4. 命名与代码规范

- **私有辅助函数**：`i_` 前缀（`i_load`、`i_save_config`、`i_get_java_path`）。
- **转换函数命名**：`frontend_*`（app 侧，转成 UI 类型，如 `frontend_mc_config`、`frontend_account`）；`ui_*`（frontend 侧，构建 Slint 模型，如 `ui_acc_list`、`ui_java_list`）；枚举字符串转换 `to_*`（如 `to_account_type`）。
- **错误处理**：每个 crate 手写错误枚举 + `From` 转换 + `Display`（不用 thiserror/anyhow）；错误日志统一 `error!("{e}")`。
- **注释**：中文 `///` 文档注释与英文混用；行内注释英文为主。
- **格式化**：标准 `cargo fmt` 风格；`use` 按 crate 分组。
- **异步**：tokio full features；下载任务内部用 mpsc channel 做命令控制（Pause/Resume/Cancel），`try_lock` 保护状态。
- **Slint**：入口 `res/ui/app-window.slint`，页面/对话框/组件见 `rule://CEMCL-ui-style-summary`；`build.rs` 用 `slint_build::compile_with_config` + `with_bundled_translations("res/translation")` + `with_style("fluent")`。
- **无边框窗口**：主窗口与 AddGameDialog 自绘标题栏（`no-frame`）。移动窗口走 `.slint` 回调 `drag-window` → `frontend/src/ui.rs` 的 `drag_window()` → `slint::winit_030::WinitWindowAccessor` + `winit::Window::drag_window()`（`crates/frontend/Cargo.toml` 启用 slint 的 `unstable-winit-030` feature）；因为在 WM 交互移动期间应用收不到抬起事件，`drag_window()` 还会在事件循环里补发一次窗口外的 `PointerReleased` 复位 Slint 输入状态（细节见 `rule://CEMCL-ui-style-summary` §4）。最小化/最大化/关闭直接在根元素上改 `root.minimized` / `root.maximized` / `root.close()`。
- **翻译更新**：`crates/frontend/update_translations.sh`（`slint-tr-extractor` 生成 frontend.pot，`msgmerge` 更新 zh_CN po）。

## 5. 已修复怪癖（2026-09-09 修复；2026-09-18 复核仍成立）

- `utils::get_parent_dir` 改用 `Path::parent()`。
- `extract_lib` 改用 `std::env::temp_dir()/cemcl-{id}-{uuid}` 唯一目录。
- Java 不兼容后缀翻译：AppWindow `out property <string> not-compatible-text: @tr("not compatible")` 桥接，Rust 读取后格式化。
- `AccountType::Other` 序列化为 `"Other"`。
- `account.rs` 的 `save()` 中无空 `error!` 日志。
- `open-edit-java-dialog` 回调已声明但未实现（JavaPage 无 Edit 按钮，当前不可触发）。

## 6. 剩余 TODO（行号截至 2026-09-18，共 6 条；grep 全仓无其它 TODO/FIXME）

1. `crates/mc/src/download/libraries.rs:78` — `// TODO: check hash`（natives 库已存在时校验哈希）
2. `crates/mc/src/download/libraries.rs:139` — `// TODO: check hash`（fabric 库已存在时校验哈希）
3. `crates/mc/src/download/assets.rs:34` — `// TODO: check hash`（资源已存在时校验哈希）
4. `crates/frontend/src/app_window.rs:248` — `// TODO: implement edit java dialog`
5. `crates/mc/src/account/account.rs:7` — `Other, // TODO: implement other account type`
6. `crates/frontend/res/ui/pages/accounts/account-item.slint:21` — `// TODO: User avatar`（当前用 icon.png 占位）
