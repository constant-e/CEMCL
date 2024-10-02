# CE Minecraft Launcher (CEMCL)
constant-e's Minecraft: Java Edition Launcher

语言：简体中文 | [English](README_EN.md)

## 注意
项目开发初期，尚未完成，暂时不完全可用

## 项目介绍
使用Rust和Slint开发的Minecraft Java版启动器

## 下载
目前存在Release、CI版两种版本。

### Release版
Release版是相对稳定的版本，适合大多数用户。
1. 前往[Github](https://github.com/constant-e/CEMCL/releases)下载
2. 前往[官网](https://constant-e.github.io/CEMCL/download.html)下载

### CI版
CI（持续集成）版是在GitHub中提交commit后，由GitHub Actions自动构建的版本。它更新频率更快，但可能存在Bug。

前往[GitHub Actions](https://github.com/constant-e/CEMCL/actions)下载CI版。

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
4. 生成语言文件（对于非Windows系统）
   ```sh
   # 对于Debug版
   ./update_translations.sh
   # 对于Release版
   ./update_translations.sh --release
   # 或使用--all同时为Debug和Release生成
   ```

## 文档
[文档](https://constant-e.github.io/CEMCL/docs)

## 翻译
运行`update_tranlations.sh`以更新po文件并生成mo文件。翻译后需要再次运行以生成新的mo文件。

**注意**：由于Slint的原因，翻译暂不支持Windows。

## 路线图
1. （已完成）启动Minecraft
2. （已完成）在各个版本内部储存自定义设置
3. （已完成）完善配置系统
4. （已完成）支持版本隔离
5. （部分完成）多语言支持
6. （部分完成）支持下载原版Minecraft
7. 支持下载Mod版Minecraft
8. 正版登录
9. 其他（完善log和报错，UI改进等）

## 鸣谢
1. [env_logger](https://crates.io/crates/env_logger)：输出日志
2. [futures](https://crates.io/crates/futures)：异步
3. [log](https://crates.io/crates/log)：输出日志
4. [reqwest](https://crates.io/crates/reqwest)：下载
5. [serde_json](https://crates.io/crates/serde_json)：JSON解析
6. [slint](https://crates.io/crates/slint)：GUI框架
7. [tokio](https://crates.io/crates/tokio)：异步
8. [uuid](https://crates.io/crates/uuid)：UUID生成
9. [zip](https://crates.io/crates/zip)：解压缩

## 许可证
Apache License 2.0
