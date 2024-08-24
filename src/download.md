# 下载

## Release
### v0.1.0
注意：这是一个预览版。仅实现了离线功能，并且未经过充分测试。

功能：
1. 从.minecraft文件夹启动MC（包括forge版等，但不能下载依赖）。
2. 添加或修改账户（在线用户需要手动输入token和uuid）。
3. 修改MC（调整java路径、窗口大小、jvm内存或手动输入启动参数）。支持版本隔离。

下面上传的文件UI似乎有些问题（因为它们是使用GitHub Actions构建的）。如果可以，请自行构建。

下载连接：
[Linux](https://github.com/constant-e/CEMCL/releases/download/v0.1.0/cemcl-0.1.0-linux-x86_64) |
[macOS](https://github.com/constant-e/CEMCL/releases/download/v0.1.0/cemcl-0.1.0-macos-x86_64) |
[Windows](https://github.com/constant-e/CEMCL/releases/download/v0.1.0/cemcl-0.1.0-windows-x86_64.exe)

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
