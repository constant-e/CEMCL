# CE Minecraft Launcher (CEMCL)
constant-e's Minecraft (Java Edition) Launcher

## 注意
项目开发初期，绝大部分尚未完成，暂时不可用

## 项目介绍
使用Rust和Slint开发的Minecraft: Java Edition启动器

## 项目结构
```
CEMCL
├── docs                        # 文档
│   ├── account_template.json   # account.json的模板
│   ├── config_template.json    # config.json的模板
│   ├── index_template.json     # index.json的模板
│   └── readme.md               # 说明
├── res                         # 资源
│   ├── translate               # 翻译
│   │   ├── cemcl.pot           # 模板
│   │   └── zh_CN.po            # 简体中文
│   └── ui                      # UI
│       └── cemcl.slint         # 主窗口
├── src                         # 源代码
│   ├── cemcl.rs                # 主窗口
│   ├── file_tools.rs           # 文件相关
│   ├── main.rs                 # 程序入口
│   └── mc_core.rs              # Minecraft相关
├── build.rs                    # 编译相关
├── Cargo.lock                  # Cargo文件
├── Cargo.toml                  # Cargo文件
├── LICENSE                     # 许可证
└── README.md                   # 项目描述

```

## Credits
slint-ui/slint for GUI

## 开源许可
Apache License 2.0
