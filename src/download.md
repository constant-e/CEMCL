# 下载
**强烈建议从最新源代码构建。**

## Release

### v0.2.2
**这是一个预览版，未经过充分测试。**

#### 更新日志
1. 新增 启动时显示进度条，替换之前的弹出窗口
2. 新增 添加一个下载器，替换旧的下载方法（仍在开发，未完成GUI部分）
3. 新增 Windows系统下的多语言支持（并将所有版本的语言文件修改为内置）
4. 重新支持自定义启动
5. 部分支持下载forge
6. 移除Windows下的控制台窗口
7. 将UI风格切换至fluent

**下载链接：**
[Linux](https://github.com/constant-e/CEMCL/releases/download/v0.2.2/cemcl-0.2.2-linux-x86_64.zip) |
[macOS](https://github.com/constant-e/CEMCL/releases/download/v0.2.2/cemcl-0.2.2-macos-x86_64.zip) |
[Windows](https://github.com/constant-e/CEMCL/releases/download/v0.2.2/cemcl-0.2.2-windows-x86_64.zip)

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
