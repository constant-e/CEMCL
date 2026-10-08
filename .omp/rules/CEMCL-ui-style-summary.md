---
description: "CEMCL Slint UI 约定（改 crates/frontend/res/ui 前必读）：29 个 .slint 文件组织、无边框窗口（自绘标题栏 + 1px 窗口描边）、侧栏/页面切换骨架、组件规格、Palette/间距/交互/对话框/i18n 模式"
globs: crates/frontend/res/ui/**/*.slint
---

# CEMCL UI 设计风格与布局规范

> 依 `crates/frontend/res/ui/` 全部 29 个 `.slint` 文件核对于 2026-10-08（Slint 1.18.1）；§7 的账号头像位图 2026-09-22 复核。来源 `.github/instructions/CEMCL-ui-style-summary.instructions.md`（已迁至 `.omp/rules/`），已修正若干偏差。
> 配套：`rule://CEMCL-project-summary`。

## 1. 文件组织（29 个文件）

```
res/ui/
├── app-window.slint                  # 主窗口：无边框 + SideBar + TitleBar + 页面切换；in-out 属性/回调汇聚；page-titles 表；re-export 对话框与 Config 类型
├── components/
│   ├── components.slint              # 总出口
│   ├── general/general.slint         # 子出口（转 text.slint + background.slint）
│   ├── general/text.slint            # Title / SubTitle / SectionTitle
│   ├── general/background.slint      # RoundedBackground
│   ├── spacing/spacing.slint         # HorizontalSpacing / VerticalSpacing
│   ├── settings-group.slint          # SettingsGroup / MyText / MySpinBox
│   ├── java-version-selector.slint   # JavaVersionSelector（继承 ComboBox）
│   ├── title-bar.slint               # TitleBar（页面标题 + 最小化/最大化/关闭）
│   ├── window-frame.slint            # WindowFrame（窗口背景 + 1px 描边，内容走 @children）
│   └── sidebar/{sidebar.slint, sidebar-item.slint}
├── pages/
│   ├── pages.slint                   # 总出口
│   ├── home.slint / games.slint / downloader.slint / java.slint / settings.slint
│   └── accounts/{accounts.slint, accounts-list.slint, account-item.slint, account-inner.slint}
└── dialogs/
    ├── dialogs.slint                 # 总出口 + AskDialog + AskID
    ├── add-java-dialog.slint / login-dialog.slint
    └── game/{add-game-dialog.slint, edit-game-dialog.slint, forge-download-dialog.slint, game.slint(MCConfig)}
```

**约定**：每个目录一个"总出口"（`components.slint` / `pages.slint` / `dialogs.slint`），新组件必须 export 进对应总出口；`components.slint` 经 `general/general.slint` 二层转发。现存例外：`app-window.slint` 直接 `import { AccountIconSpec, AccountInner } from "pages/accounts/accounts.slint"`（这些类型未经 pages.slint 导出）。

## 2. 整体布局骨架

- **无边框窗口**：**所有窗口**（`AppWindow` 与 6 个对话框）都设 `no-frame: true` + `resize-border-width: 5px`（winit 后端在窗口边缘 5px 内做缩放命中测试，其余后端忽略）。窗口装饰（标题栏、描边）完全自绘。
- **窗口描边（WindowFrame）**：每个窗口的最外层是 `WindowFrame { width: 100%; height: 100%; ... }`（`components/window-frame.slint`）：`background: Palette.background` + `@children` + 末尾一个 `Rectangle { width/height: 100%; border-width: 1px; border-color: Palette.border; }` 覆盖层。描边画在最外层 1px 且**最后绘制**（Rectangle 不拦截输入），所以内容不需要为描边留内边距。主窗口里 WindowFrame 是 Window 的直接子元素；对话框里它是唯一的主子元素（见 §5）。
- **主窗口** `AppWindow`（preferred 800×600，title "CE Minecraft Launcher"）：`WindowFrame` 内是 `HorizontalBox { padding: 0px; }`，左侧 `side-bar := SideBar`（`min-width: 150px`，`header-height: title-bar.height`），右侧 `VerticalBox { padding: 0px; }` = `title-bar := TitleBar` + 页面容器 `Rectangle { preferred-width: 100%; vertical-stretch: 1; }`。
- **标题栏（TitleBar）**：`title-text: root.page-titles[side-bar.current-index]`；右上角三个按钮在同一行。窗口控制只在根元素里生效：`minimize => { root.minimized = true; }`、`maximize => { root.maximized = !root.maximized; }`、`close => { root.close(); }`、`drag => { root.drag-window(); }`（`drag-window` 是 Rust 回调，见 §4）。
- **页面标题**：不再放在页面里（页面顶部已无 `Title`），统一由 AppWindow 的私有属性 `page-titles: [@tr("Account"), "CE Minecraft Launcher", @tr("Minecraft Version"), @tr("Downloader"), @tr("Java Installations"), @tr("Settings")]` 按侧栏索引提供，顺序必须与 SideBar `model` 一致。
- **SideBar**：`model: [@tr("Account"), @tr("Home"), @tr("Games"), @tr("Downloader"), @tr("Java"), @tr("Settings")]`，初始 `current-index: 1`；`index-changed(from, to)` 在点击切换前触发（from=旧索引）；app-window 里用 `from == 5`（离开设置页）触发 `set-config` 保存。顶部 `header := Rectangle { height: root.header-height; }` 内含居中加粗的 `label := SubTitle`（`title <=> label.text`），高度与标题栏一致以保证 CEMCL 与页面标题对齐。
- **页面切换**：不用 TabWidget 做导航，用 `if (side-bar.current-index == N): xxx-page := XxxPage { ... }`（0..5 对应上表），以 `:=` 命名以便绑定属性。
- **页面结构**：所有页面 `inherits VerticalBox`，顶部直接是内容（标题已上移到标题栏），底部操作按钮行。
- **页面与主窗口通信**：页面属性 `<=>` AppWindow 的 in-out 属性；页面回调转发到 root（`xxx(index) => { root.xxx(index) }`），Rust 侧 `on_xxx` 处理。

## 3. 组件与样式

| 组件 | 规格 | 用途 |
|---|---|---|
| `Title` | 24px / 700 | 标题栏标题（旧页面标题样式） |
| `SubTitle` | 16px | 对话框、区块标题、侧栏 CEMCL（实例加 700 加粗） |
| `SectionTitle` | 12px / 700 | 小节标题 |
| `TitleBar` | 高 40px，背景 `Palette.background`（不区分背景色），左侧 `Title`（左内边距 `StyleMetrics.layout-padding`、`overflow: elide`）；按钮 46×40 | 自绘标题栏：`title-text` + `show-minimize` / `show-maximize` / `maximized`，回调 `minimize` / `maximize` / `close` / `drag`（对话框只用 close/drag） |
| `WindowFrame` | `Palette.background` + 末尾 1px `Palette.border` 覆盖层（无圆角） | 无边框窗口的窗口框：填充整个窗口、画描边，内容走 `@children` |
| `RoundedBackground` | `Palette.control-background` + 圆角 8px（无 vertical-stretch） | 卡片容器 |
| `HorizontalSpacing` / `VerticalSpacing` | 100% 拉伸、透明 | 弹性占位，推按钮到右侧/底部 |
| `SettingsGroup` | `GroupBox` + `GridLayout { spacing: StyleMetrics.layout-spacing; @children }` | 表单分组 |
| `MyText` | `horizontal-stretch: 0; vertical-alignment: center; max-height: 20px` | 表单标签 |
| `MySpinBox` | min 0 / max 2147483647 | 数值输入 |
| `JavaVersionSelector` | 继承 `ComboBox`，`[string]` 模型（预格式化字符串） | Java 选择 |

- **颜色**：一律用 `Palette.*`（全仓唯一硬编码色值：`title-bar.slint` 里 UWP 关闭按钮悬停的 `#e81123` / `#ffffff`，Palette 无对应语义色）。
- **间距**：布局 `spacing` 一律 `StyleMetrics.layout-spacing`；条目尺寸类硬编码可接受（账号行高 48px、图标 44px 等）。
- **卡片**：手写 `Rectangle { background: Palette.control-background; border-radius: 8px; vertical-stretch: 0; }`（TaskSetItem、AccountsPage 头部卡片同款；`RoundedBackground` 自身不含 stretch）。
- **按钮行**：`HorizontalBox { padding: 0px; ... HorizontalSpacing {} ... }` 或 `HorizontalLayout + HorizontalSpacing`，把按钮推到最右。
- **表单行**：`SettingsGroup` 内 `Row { MyText { ... } 输入控件 }`；整行控件用 `colspan: 2`（如 Switch）。

## 4. 状态与交互模式

- **标题栏按钮（Win10 UWP 风格）**：无背景色，`hover` 时最小化/最大化用 `Palette.alternate-background` 高亮、关闭按钮为红底白图标；图标用 1px `Rectangle` / `Path` 自绘（10×10，含最大化后的“还原”双框，前框用标题栏底色挖空）。a11y 上每个按钮是 `accessible-role: button` + `accessible-label`（`@tr("Minimize"/"Maximize"/"Restore"/"Close")`）。
- **窗口拖动**：标题栏左侧空白带的 `TouchArea.pointer-event`（按下）→ `drag()` → 根元素 `root.drag-window()` → frontend `ui::drag_window()` → winit `Window::drag_window()`（需要 slint 的 `unstable-winit-030` feature；非 winit 后端为空操作）。**每个窗口都要在 Rust 侧接上这个回调**（`ui.on_drag_window(move || { if let Some(ui) = ui_weak.upgrade() { ui::drag_window(&ui); } })`，见 `app_window.rs` / `game.rs` / `java.rs` / `account.rs` / `msg_box.rs`），否则该窗口的标题栏无法拖动。两个必须遵守的细节：
  1. **拖动区不能与按钮重叠**。重叠的 `TouchArea` 会*同时*收到 `pointer-event`，而且下层的会抢走指针 grab（表现为"按标题栏按钮也在拖窗口"）。所以拖动区是布局里的兄弟节点（`horizontal-stretch: 1`），只覆盖按钮左侧，标题 `Title` 放在它内部并用 `x: StyleMetrics.layout-padding` 对齐页面内容。
  2. **WM 交互移动会吞掉抬起事件**。窗口管理器接管指针后 release 不会送达应用，Slint 会一直以为指针按下，把后续点击都路由给拖动区（表现为"点什么都在拖窗口"）。因此 `ui::drag_window()` 在事件循环里补发一次窗口外（`(-1,-1)`）的 `PointerReleased` 复位输入状态，拖动区的 `pointer-event` 再额外用 `mouse-x/mouse-y` 是否落在自身范围内做守卫（残留 grab 期间收到的事件坐标会越界）。
- **SideBarItem**：`states [ pressed / hover / selected ]` 控制背景 `opacity`（0.8 / 0.6 / 1），背景默认 `opacity: 0`，`animate opacity { duration: 150ms; }` 淡入淡出。
- **互斥区块**用 `if (cond):`（非 `visible`）：AccountsPage 的 `edit` / `switch` 两个 bool 互斥；TaskSetItem 的 `if !finished:` 隐藏按钮行。
- **状态文本**用 `states [ xxx when cond: {...} ]` 驱动：HomePage `State`（spare/downloading/launching/logging-in 改 state-text）、AskDialog `AskID`、AddJavaDialog `JavaCheckResult`（6 状态）。
- **按钮可用性**：`enabled: 条件表达式`，如 `enabled: state == State.Spare && combo-box.model.length != 0`；列表项 `enabled: index != current-index`。
- **表格**：`StandardTableView`，列 `{ title: @tr(...) }`；页面表格 `min-width: 250px`，AddGameDialog 内两张表 `min-width: 200px`。
- **账号列表**：`ListView`（AccountsList，GroupBox + ListView）；`preferred-height: accounts.length * (48px + StyleMetrics.layout-spacing)`，`max-height` 限 3 行。
- **进度**：`progress-mode` 约定 `0 = BySize, 1 = ByNumber, 2 = Both`（home.slint / downloader.slint / settings.slint 三处注释重复）。downloader.slint 有 `progress-text(info)` + `progress-value(info)`（含 total>0 防零除）；home.slint 只有 `progress-text()`，进度条直接绑 float `progress`。

## 5. 对话框规范

- **所有对话框都是自绘标题栏的无边框窗口**：`no-frame: true` + `resize-border-width: 5px` + `padding: 0px`（Dialog 内置布局的 `StyleMetrics.layout-padding` 必须关掉，否则标题栏与描边无法贴到窗口边缘）。Dialog 只允许一个非按钮子元素，且该子元素在关闭内边距后即整窗大小 —— 因此唯一子元素固定为
  `WindowFrame { width: 100%; height: 100%; VerticalBox { padding: 0px; spacing: 0px; title-bar := TitleBar { show-minimize: false; show-maximize: false; ... }; <内容 VerticalBox（默认 8px padding/spacing）> } }`
- **标题栏内容一致**：所有对话框用同一个 `TitleBar`，只显示右上角关闭按钮（`show-minimize/show-maximize: false`，46×40、悬停变红，与主窗口一致，贴窗口右上角）。`title-text` 与窗口 `title` 用同一条 `@tr()` 文案；正文里不再重复放同名的 `Title`/`SubTitle`。
- **按钮不再交给 Dialog 自动布局**：`StandardButton`/动作按钮写进内容底部自己的 `HorizontalLayout { alignment: end; spacing: StyleMetrics.layout-spacing; ... }`，不能再用 `dialog-button-role`（非 Dialog 直接子元素会报错），也**不会**再生成 `<kind>-clicked` 自动回调 —— 需要 Rust 接的回调（如 `cancel-clicked`）在组件里显式声明，Rust 侧 `on_cancel_clicked` 调用方式不变。平台相关按钮顺序因此固定为声明顺序（ok → cancel；EditGameDialog 为 Delete → OK → Cancel）。
- 尺寸：AddGameDialog 800×600；AddJavaDialog 450×200；EditGameDialog / ForgeDownloadDialog 400×200；LoginDialog、AskDialog 自适应（AskDialog 无 preferred-*，靠内容撑开）。
- 内容容器：AddGameDialog 用 `VerticalBox` 包 `HorizontalBox { padding: 0px; }` + 设置区；AddJavaDialog 用 `GridBox { padding: 0px; }`（跨列 `colspan: 3`，末尾 `VerticalSpacing { colspan: 3; }` 把内容顶到上方）；EditGameDialog 是 `SettingsGroup`（`title: @tr("Edit {}", version)`）+ 底部按钮行；ForgeDownloadDialog / LoginDialog 用默认 8px 的 `VerticalBox`。
- AskDialog：`title: root.title-text`，标题栏文案由 `states [ DelAccConfirm / DelGameConfirm ]` 与正文一起设置（`@tr("Delete Account")` / `@tr("Delete Game")`）。
- 数据流：`XxxDialog::new()` → 设属性/绑定回调 → `show()`，`Weak` 存 `Arc<Mutex<Option<slint::Weak<...>>>>`；Rust 经 `UIUpdate` + `upgrade_in_event_loop` 设置属性；用户操作 → 回调 → `UICommand`。

## 6. 翻译（i18n）约定

- 所有用户可见文本用 `@tr("...")`；带参数 `@tr("Edit {}", name)` 或 `@tr("Java {0} detected", version)`。
- 文件：`res/translation/frontend.pot`（模板）+ `zh_CN/LC_MESSAGES/frontend.po`（中文）；build.rs `with_bundled_translations("res/translation")` + `with_style("fluent")`，按系统 locale 选择。
- **Rust 侧取译文**必须走 `.slint` 桥：`out property <string> xxx-text: @tr("...");`（普通 `property` 的 getter 对 Rust 不可见），Rust 用 `ui.get_xxx_text()` 读取。实例：AppWindow `not-compatible-text`。
- 更新翻译：`crates/frontend/update_translations.sh`（依赖 `slint-tr-extractor` + `msgmerge`）。

## 7. 数据流与模型约定

- **UI→App**：`UICommand`（tokio mpsc unbounded）；**App→UI**：`UIUpdate`（`upgrade_in_event_loop`），详见 `rule://CEMCL-project-summary`。
- **模型构建**：Rust 侧 `ui_*` 函数：表格 `ModelRc<ModelRc<StandardListViewItem>>`（`ui_game_list`、`ui_fabric_list`、`ui_forge_list`、`ui_game_dl_list`、`ui_java_list`）；`ui_combo_box_list` → `ModelRc<SharedString>`；`ui_java_combo_box_list` → `Vec<String>`；另有 `ui_acc_list`、`ui_task_set_list`。
- **账号头像位图**：`AccountInner.avatar`（`image`）由 Rust 生成 —— `mc` 给出皮肤头部正面 8×8（第二层已合成），frontend 按 `AccountIconSpec.size × window.scale_factor()`（`account::ui_avatar_size`）最近邻放大到设备像素后再 `Image::from_rgba8`，避免 Slint 放大位图导致模糊（`ui_acc_list(&list, icon_size)`）；显示尺寸只在 `account-item.slint` 的 `export global AccountIconSpec { out property <length> size: 48px; }` 里定义一次，Rust 经 AppWindow 的 `out property <length> account-icon-size: AccountIconSpec.size` 读取（同 §6 的 Rust 取译文桥接）。账号头像的加载/缓存见 `rule://CEMCL-project-summary` §5。
- **ComboBox 模型只能 `[string]`**（fluent 风格不支持 struct 模型）：Java "版本 + (不兼容)" 后缀在 Rust 侧格式化后传入。
- **Config 结构**：`Config`/`ConfigGeneral`/`ConfigDL`/`ConfigMC` 在 settings.slint 定义为 struct，与 Rust `frontend::Config*` 一一对应；settings 页用 `<=>` 绑定，每次编辑调 `set-config(root.config)` 保存。

## 8. 新增 UI 检查单

1. 新组件/页面/对话框必须在对应总出口 export，并只从总出口 import（现存唯一例外：AppWindow 直接从 accounts.slint import `AccountIconSpec` / `AccountInner`）。
2. 文本必须 `@tr()`，并同步 frontend.pot 与 zh_CN/LC_MESSAGES/frontend.po（跑 update_translations.sh）。注意 msgmerge 会把换了 context 的条目标成 `#, fuzzy`，fuzzy 条目不生效，必须手工去掉。
3. 颜色用 `Palette`；布局 `spacing` 用 `StyleMetrics.layout-spacing`；圆角统一 8px。
4. 页面切换沿用 `if (side-bar.current-index == N)` 模式，不要引入新的导航机制；新页面的标题写进 AppWindow 的 `page-titles`（索引与 SideBar `model` 对齐），页面内部不要再放 `Title`。
5. 需要 Rust 侧翻译的字符串，走 `out property <string> ...-text: @tr(...)` 桥接。
6. 按钮行用 `HorizontalSpacing {}` 右对齐；表单用 `SettingsGroup` + `Row` + `MyText`。
7. 对话框按钮写进内容自己的按钮行（`HorizontalLayout { alignment: end; }`），需要 Rust 处理的回调显式声明（如 `callback cancel-clicked();`）；不要再给对话框里的按钮加 `dialog-button-role`。
8. 进度相关统一走 `progress-mode`（0/1/2）约定。
9. 自绘标题栏/窗口框改动后确认：按钮点击区不被拖动区吞掉、关闭按钮悬停变红、描边在最外圈 1px 且四个角都在、标题栏与关闭按钮贴窗口右上角（对话框与主窗口一致）、`page-titles` 与侧栏索引一致、无边框窗口仍可用边缘拖动缩放。
10. 像素位图（如账号头像）：按「显示尺寸 × 窗口缩放因子」在 Rust 侧最近邻放大后再交给 `Image`，显示尺寸用 `out property` 桥接给 Rust（见 §7），不要直接把小位图丢给 Slint 缩放。
