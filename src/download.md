# 下载
**强烈建议从最新源代码构建。**

## Release

### v0.3.0
**这是一个预览版，未经过充分测试。**

#### 更新日志
1. **破坏性变化** 更新了各个配置文件的格式，并将各版本的配置文件统一储存在.cemcl下
2. 构建了全新的分页式UI，包含一个全新的账号页
3. 支持在启动时添加封装器
4. 支持记忆上次选中的账号和游戏版本
5. 修复一些小Bug

**下载链接：**
[Linux](https://github.com/constant-e/CEMCL/releases/download/v0.3.0/cemcl-0.3.0-linux-x86_64) |
[macOS](https://github.com/constant-e/CEMCL/releases/download/v0.3.0/cemcl-0.3.0-macos-arm64) |
[Windows](https://github.com/constant-e/CEMCL/releases/download/v0.3.0/cemcl-0.3.0-windows-x86_64)

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
