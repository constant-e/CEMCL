# CEMCL UI 设计风格与布局规范总结

> 基于 `crates/frontend/res/ui/` 下全部 27 个 `.slint` 文件（Slint 1.17.1）分析整理。
> 配套文档：[CEMCL-project-summary.instructions.md](./CEMCL-project-summary.instructions.md)

## 1. 文件组织

```
res/ui/
├── app-window.slint            # 主窗口：SideBar + 页面切换，所有 in-out 属性与回调的汇聚点
├── components/
│   ├── components.slint        # 组件总出口（re-export）
│   ├── general/text.slint      # Title / SubTitle / SectionTitle
│   ├── general/background.slint# RoundedBackground
│   ├── spacing/spacing.slint   # HorizontalSpacing / VerticalSpacing
│   ├── settings-group.slint    # SettingsGroup / MyText / MySpinBox
│   ├── java-version-selector.slint # JavaVersionSelector（继承 ComboBox）
│   └── sidebar/                # SideBar / SideBarItem
├── pages/
│   ├── pages.slint             # 页面总出口
│   ├── home.slint / games.slint / downloader.slint / java.slint / settings.slint
│   └── accounts/               # accounts.slint / accounts-list.slint / account-item.slint / account-inner.slint
└── dialogs/
    ├── dialogs.slint           # 对话框总出口 + AskDialog + AskID
    ├── add-java-dialog.slint / login-dialog.slint
    └── game/                   # add-game-dialog / edit-game-dialog / forge-download-dialog / game.slint(MCConfig)
```

**约定**：每个目录有一个"总出口"文件（`components.slint`、`pages.slint`、`dialogs.slint`），其他文件只被总出口引用；`app-window.slint` 只 import 总出口。新增组件/页面/对话框时必须在对应总出口中 `export`。

## 2. 整体布局骨架

- **主窗口**：`HorizontalBox { padding: 0px; }`，左侧 `SideBar`（`min-width: 150px`），右侧内容 `Rectangle { preferred-width: 100%; }`。
- **页面切换**：不用 `TabWidget`，而是 `if (side-bar.current-index == N): xxx-page := XxxPage { ... }`，每个页面用 `:=` 命名以便绑定属性。
- **页面结构**：所有页面继承 `VerticalBox`，顶部 `Title`，中间内容，底部操作按钮行。
- **页面与主窗口通信**：页面属性用 `<=>` 双向绑定到 AppWindow 的 in-out 属性；页面回调转发到 AppWindow 回调（`xxx => { root.xxx() }`），再由 Rust 侧 `on_xxx` 处理。

## 3. 组件与样式规范

| 组件 | 规格 | 用途 |
|---|---|---|
| `Title` | 24px / 700 | 页面标题 |
| `SubTitle` | 16px | 对话框标题、区块标题 |
| `SectionTitle` | 12px / 700 | 小节标题（下载页"未完成/已完成"） |
| `RoundedBackground` | `Palette.control-background` + 圆角 8px | 卡片容器 |
| `HorizontalSpacing` / `VerticalSpacing` | 100% 拉伸、透明 | 弹性占位，把按钮推到右侧/底部 |
| `SettingsGroup` | `GroupBox` + `GridLayout { spacing: StyleMetrics.layout-spacing; @children }` | 表单分组 |
| `MyText` | `horizontal-stretch: 0; vertical-alignment: center; max-height: 20px` | 表单标签 |
| `MySpinBox` | min 0 / max 2147483647 | 数值输入 |
| `JavaVersionSelector` | 继承 `ComboBox`，模型为 `[string]` | Java 选择（见 §7） |

- **颜色**：一律使用 `Palette`（`background` / `alternate-background` / `control-background`），不硬编码颜色。
- **间距**：一律使用 `StyleMetrics.layout-spacing`，不硬编码像素间距。
- **卡片**：`Rectangle { background: Palette.control-background; border-radius: 8px; vertical-stretch: 0; }`（TaskSetItem、AccountsPage 头部卡片同款）。
- **按钮行**：`HorizontalBox { padding: 0px; HorizontalSpacing {} Button... Button... }`——`HorizontalSpacing` 把按钮推到最右；按钮之间靠布局默认间距。
- **表单行**：`SettingsGroup` 内用 `Row { MyText { text: ... } 输入控件 }`；整行控件（如 Switch）用 `colspan: 2`。

## 4. 状态与交互模式

- **SideBarItem**：`states [ pressed / hover / selected ]` 控制背景 `opacity`（0.8 / 0.6 / 1），背景默认 `opacity: 0`，`animate opacity { duration: 150ms; }` 做淡入淡出。
- **条件布局**：用 `if (cond):` 而非 `visible` 做互斥区块（AccountsPage 的 `edit` / `switch` 两个 bool 属性互斥切换；TaskSetItem 的 `if !finished:` 隐藏按钮行）。
- **状态文本**：用 `states [ xxx when cond: { text.text: ... } ]` 驱动（HomePage 的 State、AskDialog 的 AskID、AddJavaDialog 的 JavaCheckResult）。
- **按钮可用性**：用 `enabled: 条件表达式` 控制（如 `enabled: state == State.Spare && combo-box.model.length != 0`）。
- **列表**：`StandardTableView`（`min-width: 250px`，列用 `{ title: @tr(...) }`）+ 底部按钮行；`ListView` 用于账号列表，`preferred-height: accounts.length * (48px + spacing)`、`max-height: 3 * 48px + 2 * spacing` 限制最多显示 3 行。
- **进度显示**：`progress-mode` 约定 `0 = BySize, 1 = ByNumber, 2 = Both`，由 `progress-text()` / `progress-value()` 函数统一计算（home.slint 与 downloader.slint 各有一份，逻辑相同）。

## 5. 对话框规范

- 全部继承 `Dialog`，用 `StandardButton { kind: ok/cancel }`（或 `yes/no`）收尾；自定义按钮用 `dialog-button-role: action`（EditGameDialog 的 Delete）。
- 尺寸：AddGameDialog 800×600；AddJavaDialog 450×200；EditGameDialog / ForgeDownloadDialog 400×200；LoginDialog 未指定（自适应）。
- 布局：`VerticalBox { padding: 0px; }` 或 `GridBox { padding: 0px; }`；GridBox 中跨列用 `colspan: 3`。
- 对话框数据流：Rust 侧 `XxxDialog::new()` → `show()` → 存 `slint::Weak` 到 `Arc<Mutex<Option<...>>>`；App 通过 `UIUpdate` + `upgrade_in_event_loop` 设置属性；用户操作通过回调 → `UICommand` 发回 App。

## 6. 翻译（i18n）约定

- 所有用户可见文本用 `@tr("...")`；带参数用 `@tr("Edit {}", name)` 或 `@tr("Java {0} detected", version)`。
- 翻译文件：`res/translation/frontend.pot`（模板）+ `zh_CN/LC_MESSAGES/frontend.po`（中文），build.rs 用 `with_bundled_translations` 打包，语言按系统 locale 自动选择。
- **重要限制**：Slint 的 `@tr()` 只能在 .slint 内使用；Rust 侧无法直接调用捆绑翻译（`private_unstable_api::translate` 不走 bundled 路径）。需要 Rust 侧翻译的字符串，必须在 .slint 中声明 `out property <string> xxx-text: @tr("...");`（必须是 `out`，普通 `property` 生成的 getter 是私有的），Rust 用 `ui.get_xxx_text()` 读取。实例：AppWindow 的 `not-compatible-text`。
- 更新翻译：`update_translations.sh`（依赖 `slint-tr-extractor` + `msgmerge`）。

## 7. 数据流与模型约定

- **UI→App**：`UICommand` 枚举（tokio mpsc unbounded channel）。
- **App→UI**：`UIUpdate` 枚举，在 `AppWindow::handle` 中分发，用 `upgrade_in_event_loop` 设置属性。
- **模型构建**：Rust 侧用 `ui_*` 前缀函数把 `Vec<T>` 转成 Slint 模型（`ModelRc<ModelRc<StandardListViewItem>>` 用于表格；`Vec<String>` 用于 ComboBox）。
- **ComboBox 模型只能是 `[string]`**（fluent 风格不支持 struct 模型），所以 Java 选择器的"版本 + (不兼容)"后缀必须在 Rust 侧格式化好再传入。
- **配置结构**：`Config { general, dl, mc }` 在 .slint 中定义为 struct，与 Rust 侧 `frontend::Config` 一一对应，用 `<=>` 双向绑定 + `set-config` 回调保存。

## 8. 注意事项（新增 UI 时）

1. 新组件/页面/对话框必须在对应总出口文件 export，并只从总出口 import。
2. 文本必须 `@tr()`，并同步更新 frontend.pot 与 zh_CN frontend.po。
3. 间距用 `StyleMetrics.layout-spacing`，颜色用 `Palette`，圆角统一 8px。
4. 页面切换沿用 `if (side-bar.current-index == N)` 模式，不要引入新的导航机制。
5. 需要 Rust 侧翻译的字符串，走 `out property <string> ...-text: @tr(...)` 桥接（见 §6）。
6. 按钮行用 `HorizontalSpacing {}` 右对齐；表单用 `SettingsGroup` + `Row` + `MyText`。
7. 对话框用 `StandardButton`，自定义动作按钮加 `dialog-button-role: action`。
8. 进度相关统一走 `progress-mode`（0/1/2）约定。
