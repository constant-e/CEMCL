# CE Minecraft Launcher (CEMCL)
constant-e's Minecraft (Java Edition) Launcher

## 注意
项目开发初期，绝大部分尚未完成，暂时不可用

## 简介
一个使用C++ Qt编写的小型MC Java版启动器（仍在开发中）

## 构建
1. 安装cmake和Qt6
2. 克隆此仓库
    ```bash
    git clone https://github.com/constant-e/CEMCL.git
    ```
3. 创建`build`文件夹
    ```bash
    mkdir CEMCL/build && cd CEMCL/build
    ```
4. 运行`cmake`
   ```bash
   cmake ..
   ```
5. 运行`make`
    ```bash
    make
    ```

## 项目结构
```
CEMCL
├── build                       # 编译文件夹（自行创建）
├── docs                        # 文档文件夹
│   ├── account_template.json   # account.json示例
│   ├── config_template.json    # config.json示例
│   ├── index_template.json     # index.json示例
│   ├── oldui                   # 曾用于生成xxxUI.h文件的源文件
│   │   ├── AddAccDialog.ui
│   │   └── ...
│   └── readme.md               # 开发者文档
├── res                         # 资源文件夹
│   ├── pic
│   │   └── icon.jpg            # 图标
│   ├── text                    # 应用内的文档（md格式）
│   │   ├── about.md
│   │   └── ...
│   └── resource.qrc            # qrc文件
├── src                         # 源代码文件夹
│   ├── AddAccDialog            # 一个带GUI的模块 添加账号对话框
│   │   ├── strings             # 字符串
│   │   │   ├── strings.cpp     # 字符串定义（根据语言）
│   │   │   └── strings.hpp     # 字符串声明
│   │   ├── AddAccDialog.cpp    # 后端源文件
│   │   ├── AddAccDialog.hpp    # 后端头文件
│   │   └── AddAccDialogUI.hpp  # UI头文件
│   ├── AddVerDialog            # 添加游戏版本对话框
│   ├── CEMCL                   # 启动器主页面
│   ├── config
│   │   └── config.h.in         # 通过CMake获取信息，生成config.h
│   ├── EditAccDialog           # 编辑账号对话框
│   ├── EditVerDialog           # 编辑游戏版本对话框
│   ├── file                    # 一个不带GUI的模块 文件及文件系统相关
│   │   ├── file.cpp            # 定义
│   │   └── file.hpp            # 声明
│   ├── MCCore                  # MC相关功能
│   ├── network                 # 网络相关功能
│   ├── Settings                # 设置界面
│   ├── sonic                   # JSON相关功能 来自bytedance/sonic-cpp
│   ├── strTools                # 字符串处理相关功能
│   └── main.cpp                # 程序入口
├── CMakeLists.txt              # CMakeLists文件
├── LICENSE
└── README.md
```

## Credits
bytedance/sonic-cpp  
Qt Group/Qt6

## 开源许可
Apache License 2.0
