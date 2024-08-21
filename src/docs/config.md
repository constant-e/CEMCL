# 配置文件说明
语言: 简体中文 | [English](config_en.md)

## account.json
账号配置文件。

数据类型：`[object]`

模板：[docs/account_template.json](account_template.json)

样例：
```json
[
    {
        "account_type": "Legacy",
        "token": "xxx",
        "uuid": "xxx",
        "user_name": "Steve"
    }, 
    {
        "account_type": "msa",
        "token": "xxx",
        "uuid": "xxx",
        "user_name": "Alex"
    }
]
```
说明：
1. `account_type`：登录方式，将直接填入启动参数。数据类型：`string`。
2. `token`：用户的token，将直接填入启动参数。数据类型：`string`。
3. `uuid`：用户的uuid。数据类型：`string`。  
**离线用户注意：uuid必须符合格式，否则无法启动；uuid不要更改，否则可能无法识别您的角色数据。**
4. `user_name`：用户名，将直接填入启动参数。数据类型：`string`。

## config.json（位于启动器目录下）
启动器配置文件。

数据类型：`object`

模板：[docs/config_launcher_template.json](config_launcher_template.json)

样例：
```json
{
    "close_after_launch": false,
    "fabric_source": "https://maven.fabricmc.net",
    "forge_source": "https://maven.minecraftforge.net",
    "game_path": ".minecraft",
    "height": "600",
    "java_path": "java",
    "game_source": "https://piston-meta.mojang.com",
    "optifine_source": "https://optifine.net",
    "width": "800",
    "xms": "1G", 
    "xmx": "2G"
}
```

说明：
1. `close_after_launch`：游戏启动后是否关闭启动器。数据类型：`bool`。
2. `..._source`：相应软件的下载源。数据类型：`string`。
3. `game_path`：`.minecraft`文件夹的位置。数据类型：`string`。
4. `height`：默认游戏窗口高度。数据类型：`string`。
5. `java_path`：`java`的路径。数据类型：`string`。
6. `width`：默认游戏窗口宽度。数据类型：`string`。
7. `xms`：为jvm分配的最小内存。数据类型：`string`。  
**将直接填入启动参数，格式如：`1024M`、`2G`等。**
8. `xmx`：为jvm分配的最大内存。数据类型：`string`。  
**将直接填入启动参数，格式如：`1024M`、`2G`等。**

## config.json（位于游戏目录下）
相应游戏版本的自定义配置文件

数据类型：`object`

模板：[docs/config_game_template.json](config_game_template.json)

样例：
```json
{
    "args": "",
    "description": "",
    "height": "",
    "java_path": "",
    "seperated": false,
    "width": "",
    "xms": "1G",
    "xmx": "2G"
}
```
说明：
1. `args`：自定义启动参数。若此项非空，将直接填入启动参数，并且**不会再添加其他参数**。数据类型：`string`。
2. `description`：备注。数据类型：`string`。
3. `seperated`：是否启用版本隔离。数据类型：`bool`。
4. 其余同*config.json（位于启动器目录下）*。
