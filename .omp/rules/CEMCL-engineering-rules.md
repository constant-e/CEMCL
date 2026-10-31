---
description: "CEMCL 工程规则（写代码/加测试/改前端前必读）：测试只放 crates/*/tests、禁止源码内 #[cfg(test)]；禁止硬编码开发机路径（如 /usr/lib/jvm/xx），只读环境变量/配置；测试必须自包含且跨平台，不依赖开发设备环境；所有用户可见前端文案走 @tr 与翻译文件"
alwaysApply: true
---

# CEMCL 工程规则

## 1. 测试位置

- 所有测试 MUST 放在对应 crate 的 `tests/` 目录：`crates/<crate>/tests/*.rs`。
- NEVER 在 `src/**` 中写 `#[cfg(test)] mod tests` 或其他内联测试。
- 需要访问私有实现时，应设计窄接口（如测试用构造器/依赖注入）或重构模块，而不是把测试搬回 `src/`。
- 同一测试二进制中如果有多个测试会切换进程工作目录或环境变量，MUST 串行执行（如静态 `Mutex`）或拆分到不同测试文件。

## 2. 禁止硬编码路径

- NEVER 在代码中硬编码开发机、发行版或用户目录路径（例如 `/usr/lib/jvm/java-17-openjdk`、`/usr/bin/java`、`/home/<user>/...`、`C:\Program Files\...`）。
- 运行期路径 MUST 来自环境变量、配置文件、函数参数或标准库 API（如 `JAVA_HOME`、`PATH`、`std::env::temp_dir()`）。
- 平台差异 MUST 用 `#[cfg(unix)]` / `#[cfg(windows)]` 或标准库抽象处理，不要把某一平台的路径写死。
- 示例：Java 自动检测只读取 `JAVA_HOME` 和 `PATH` 中 `java` 报告的 `java.home`，不扫描 `/usr/lib/jvm` 等固定目录。

## 3. 测试不依赖开发设备环境，且跨平台

- 测试 MUST 自包含：使用 `std::env::temp_dir()` 下的唯一临时目录、假文件/假数据、依赖注入。
- NEVER 读取或依赖本机的 Java、`PATH`、`JAVA_HOME`、网络、浏览器、用户配置目录、已安装软件或机器特定状态。
- NEVER 依赖测试执行顺序、进程初始 cwd 或环境变量现值；需要 cwd/env 时自行设置并在结束时恢复。
- 外部网络请求、子进程和平台服务 MUST mock 或通过注入结果替代，测试不能因某台机器缺少软件而失败。
- 平台相关行为 MUST 用 `#[cfg(...)]` 隔离；默认测试逻辑必须在所有支持平台上可编译、可运行，不假设路径分隔符、软链接、权限或 shell。
- 参考实现：`JavaManager::new_with_detected` 注入检测结果，`crates/app/tests/java_manager.rs` 使用临时目录中的假 Java。

## 4. 前端实现与文案

- 所有用户可见文案 MUST 写在 `.slint` 中并使用 `@tr("...")`；NEVER 在 Rust 侧硬编码用户可见字符串（原始技术明细/异常详情除外）。
- 需要 Rust 侧读取译文时 MUST 通过 `out property <string> xxx-text: @tr("...")` 桥接，并在 UI 线程读取。
- 新增或修改文案 MUST 同步 `crates/frontend/res/translation` 下的 `frontend.pot` 与 `zh_CN/LC_MESSAGES/frontend.po`（运行 `crates/frontend/update_translations.sh`），并删除 `#, fuzzy` 标记。
- UI 结构、组件、交互与对话框规范见 `rule://CEMCL-ui-style-summary`。
