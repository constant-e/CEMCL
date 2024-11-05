# 配置文件说明

## account.json
账号配置文件。

数据类型：`[object]`

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
2. `token`：用户的refresh token。数据类型：`string`。
3. `uuid`：用户的uuid。数据类型：`string`。  
**离线用户注意：uuid必须符合格式，否则无法启动；uuid不要更改，否则可能无法识别您的角色数据。**
1. `user_name`：用户名，将直接填入启动参数。数据类型：`string`。

## config.json（位于启动器目录下）
启动器配置文件。

数据类型：`object`

样例：
```json
{
    "assets_source": "http://resources.download.minecraft.net",
    "close_after_launch": false,
    "concurrency": 10,
    "fabric_source": "https://maven.fabricmc.net",
    "forge_source": "https://maven.minecraftforge.net",
    "game_path": ".minecraft",
    "game_source": "https://piston-meta.mojang.com",
    "height": "600",
    "java_path": "java",
    "libraries_source": "https://libraries.minecraft.net",
    "width": "800",
    "xms": "1G",
    "xmx": "2G"
}
```

说明：
1. `close_after_launch`：游戏启动后是否关闭启动器。数据类型：`bool`。
2. `concurrency`：下载时的最大并发数量。数据类型：`int`。
3. `..._source`：相应软件的下载源。数据类型：`string`。
4. `game_path`：`.minecraft`文件夹的位置。数据类型：`string`。
5. `height`：默认游戏窗口高度。数据类型：`string`。
6. `java_path`：`java`的路径。数据类型：`string`。
7. `width`：默认游戏窗口宽度。数据类型：`string`。
8. `xms`：为jvm分配的最小内存。数据类型：`string`。  
**将直接填入启动参数，格式如：`1024M`、`2G`等。**
1. `xmx`：为jvm分配的最大内存。数据类型：`string`。  
**将直接填入启动参数，格式如：`1024M`、`2G`等。**

## config.json（位于游戏目录下）
相应游戏版本的自定义配置文件

数据类型：`object`

样例：
```json
{
    "description": "",
    "game_args": [],
    "height": "600",
    "java_path": "java",
    "jvm_args": [],
    "separated": false,
    "width": "800",
    "xms": "1G",
    "xmx": "2G"
}
```
说明：
1. `game_args`和`jvm_args`：自定义启动参数。数据类型：`[string]`。
2. `description`：备注。数据类型：`string`。
3. `separated`：是否启用版本隔离。数据类型：`bool`。
4. 其余同*config.json（位于启动器目录下）*。
