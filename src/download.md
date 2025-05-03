# 下载
**强烈建议从最新源代码构建。**

## Release

### v0.2.3
**这是一个预览版，未经过充分测试。**

#### 更新日志
1. 支持安装forge
2. 部分支持fabric
3. 修复启动forge时的一系列问题
4. UI改进
5. 修复一些小Bug

**下载链接：**
[Linux](https://github.com/constant-e/CEMCL/releases/download/v0.2.3/cemcl-0.2.3-linux-x86_64.zip) |
[macOS](https://github.com/constant-e/CEMCL/releases/download/v0.2.3/cemcl-0.2.3-macos-x86_64.zip) |
[Windows](https://github.com/constant-e/CEMCL/releases/download/v0.2.3/cemcl-0.2.3-windows-x86_64.zip)

## GitHub CI
请前往[GitHub Actions](https://github.com/constant-e/CEMCL/actions)获取。

## 构建
1. 安装Rust和Qt（若需要Qt后端）
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
