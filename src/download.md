# 下载

## Release
### v0.2.0
**注意**：这是一个预览版，未经过充分测试。

由于使用Qt做为后端，需要先安装Qt5。

更新日志：
1. 新增 支持下载原版MC，并支持使用镜像
2. 新增 在启动时显示一个弹出窗口
3. 新增 添加图标
4. 新增 强制使用Qt作为后端
5. 修复 .minecraft不存在时的错误
6. 修复 某些版本的natives解压异常
7. 修复 Windows系统下-cp参数异常
8. 修复 不保存自定义游戏配置
9. 修复 旧版本的assets参数
10. UI改进
11. 移除 内置OptiFine下载

下载链接：
[Linux](https://github.com/constant-e/CEMCL/releases/download/v0.2.0/cemcl-0.2.0-linux-x86_64) |
[macOS](https://github.com/constant-e/CEMCL/releases/download/v0.2.0/cemcl-0.2.0-macos-x86_64) |
[Windows](https://github.com/constant-e/CEMCL/releases/download/v0.2.0/cemcl-0.2.0-windows-x86_64.exe)

## GitHub CI
请前往[GitHub Actions](https://github.com/constant-e/CEMCL/actions)获取。

## 构建
1. 安装Rust
2. 克隆此仓库
   ```sh
   git clone https://github.com/constant-e/CEMCL.git
   ```
3. 构建
   ```sh
   # 构建Debug版
   cargo build
   # 构建Release版
   cargo build --release
   ```
