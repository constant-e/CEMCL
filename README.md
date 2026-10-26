# CE Minecraft Launcher (CEMCL)
constant-e's Minecraft: Java Edition Launcher

语言：简体中文 | [English](README_EN.md)

## 项目介绍
使用Rust和Slint开发的Minecraft Java版启动器

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

## 文档
[文档](https://constant-e.github.io/CEMCL/docs)

## 翻译
运行`crates/frontend/update_tranlations.sh`以更新po文件。

## 路线图
1. 基础功能开发（0.1.x及0.2.x已基本完成）
2. UI优化、功能完善（0.3.x正在开发）
3. 更多模块

## 鸣谢
1. [BMCLAPI2](https://bmclapidoc.bangbang93.com/)：forge下载
2. [base64](https://crates.io/crates/base64)：皮肤数据解码
3. [clipboard](https://crates.io/crates/clipboard)：剪切板
4. [env_logger](https://crates.io/crates/env_logger)：输出日志
5. [futures](https://crates.io/crates/futures)：异步
6. [log](https://crates.io/crates/log)：输出日志
7. [png](https://crates.io/crates/png)：皮肤贴图解码
8. [reqwest](https://crates.io/crates/reqwest)：下载
9. [serde_json](https://crates.io/crates/serde_json)：JSON解析
10. [sha1](https://crates.io/crates/sha1)：文件哈希校验
11. [slint](https://crates.io/crates/slint)：GUI框架
12. [tokio](https://crates.io/crates/tokio)：异步
13. [uuid](https://crates.io/crates/uuid)：UUID生成
14. [webbrowser](https://crates.io/crates/webbrowser)：打开浏览器
15. [zip](https://crates.io/crates/zip)：解压缩

## 许可证
Apache License 2.0
