# 下载

## Release
### v0.2.1
**注意**：这是一个预览版，未经过充分测试。

#### 更新日志
［新增］
1. 支持在线登录
2. 部分支持多语言
3. 支持限制下载器的并发数
4. 在初始化失败时显示对话框
5. 支持加载Forge列表（不能下载）

［修复］
1. 修复删除游戏时的错误下标
2. 修复下标越界

［其他］
1. 为windows打包dll
2. 优化日志

**下载链接：**
[Linux](https://github.com/constant-e/CEMCL/releases/download/v0.2.1/cemcl-0.2.1-linux-x86_64.zip) |
[macOS](https://github.com/constant-e/CEMCL/releases/download/v0.2.1/cemcl-0.2.1-macos-x86_64.zip) |
[Windows](https://github.com/constant-e/CEMCL/releases/download/v0.2.1/cemcl-0.2.1-windows-x86_64.zip)

## GitHub CI
请前往[GitHub Actions](https://github.com/constant-e/CEMCL/actions)获取。

## 构建
1. 安装Rust和Qt
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
