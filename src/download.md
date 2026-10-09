# 下载
**强烈建议从最新源代码构建。**

## Release

### v0.3.1

#### 更新日志
1. 新增Java管理器：管理已安装Java；首次自动探测系统Java；按MC版本自动检查Java。
2. 下载管理器改进：任务分未完成、已完成管理，支持按大小/数量/两者显示进度；新增Forge下载进度弹窗。
3. 账号管理器改进：按皮肤头部生成、缓存并显示头像；离线账号名取默认皮肤名。
4. 窗口改进：窗口改为自绘标题栏。
5. 修复：不再重复解压natives。
6. 新增哈希值校验：下载后、启动前进行sha1校验。
7. 版本配置管理改进。
8. 修复一些小bug。

**下载链接：**
[Linux](https://github.com/constant-e/CEMCL/releases/download/v0.3.1/cemcl-0.3.1-linux-x86_64) |
[macOS](https://github.com/constant-e/CEMCL/releases/download/v0.3.1/cemcl-0.3.1-macos-arm64) |
[Windows](https://github.com/constant-e/CEMCL/releases/download/v0.3.1/cemcl-0.3.1-windows-x86_64.exe)

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
