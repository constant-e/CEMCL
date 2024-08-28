# 下载

## Release
### v0.1.1
**注意**：这是一个预览版。仅实现了离线功能，并且未经过充分测试。

更新日志：
1. 修复 默认配置不全
2. 修复 不保存设置
3. 修复 判断数组下标是否越界的逻辑
4. 修复 更改设置时不重新加载游戏列表

**注意**：这是最后一个0.1.x版本。我正在开发下载相关的功能，这可能引入一些不向下兼容的改变（如config.json的修改）。并且在这些功能完成前，我不会发布新版本（也就不会发布新的修复bug版本）。v0.1.x不应被正式使用。

下载链接：
[Linux](https://github.com/constant-e/CEMCL/releases/download/v0.1.1/cemcl-0.1.1-linux) |
[macOS](https://github.com/constant-e/CEMCL/releases/download/v0.1.1/cemcl-0.1.1-macos) |
[Windows](https://github.com/constant-e/CEMCL/releases/download/v0.1.1/cemcl-0.1.1-windows.exe)

## GitHub CI
请前往[GitHub Actions](https://github.com/constant-e/CEMCL/actions)获取。

## 自行构建
1. 安装Rust（与Cargo）。
2. 克隆仓库:
   ```sh
   git clone https://github.com/constant-e/CEMCL.git && cd CEMCL
   ```
3. 构建:
   ```sh
   # Build debug:
   cargo build
   # Build release:
   cargo build --release
   ```
