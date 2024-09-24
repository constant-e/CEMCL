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

## 路线图
1. （已完成）启动Minecraft
2. （已完成）在各个版本内部储存自定义设置
3. （已完成）完善配置系统
4. （已完成）支持版本隔离
5. （部分完成）支持下载原版Minecraft
6. 支持下载Mod版Minecraft
7. 正版登录
8. 多语言支持
9. 其他（完善log和报错，UI改进等）

## 鸣谢
[slint-ui/slint](https://github.com/slint-ui/slint)

## 许可证
Apache License 2.0
