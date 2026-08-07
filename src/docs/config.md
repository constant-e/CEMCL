# 配置文件说明

## account.json
账号配置文件

样例：
```json
{
    "accounts": [
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
    ],
    "current": 0
}

```
说明：
1. `current`：当前账号的`index`。
2. `account_type`：登录方式，将直接填入启动参数。
3. `token`：用户的refresh token。
4. `uuid`：用户的uuid。
**离线用户注意：uuid必须符合格式，否则无法启动；uuid不要更改，否则可能无法识别您的角色数据。**
5. `user_name`：用户名，将直接填入启动参数。

## config.json
启动器配置文件

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
    "height": 600,
    "java_path": "java",
    "libraries_source": "https://libraries.minecraft.net",
    "width": 800,
    "wrapper": "",
    "xms": "1G",
    "xmx": "2G"
}
```

说明：
1. `close_after_launch`：游戏启动后是否关闭启动器。
2. `concurrency`：下载时的最大并发数量。
3. `..._source`：相应软件的下载源。
4. `game_path`：`.minecraft`文件夹的位置。
5. `height`：默认游戏窗口高度。
6. `java_path`：`java`的路径。
7. `width`：默认游戏窗口宽度。
8. `wrapper`：封装器。
9. `xms`：为jvm分配的最小内存。
**将直接填入启动参数，格式如：`1024M`、`2G`等。**
10. `xmx`：为jvm分配的最大内存。 
**将直接填入启动参数，格式如：`1024M`、`2G`等。**

## versions.json
游戏版本的自定义配置文件

CEMCL也会储存官方启动器格式的`launcher_profiles.json`，但只会从`versions.json`读取配置。

样例：
```json
{
    "current": 0,
    "versions": {
        "xxx": {
            "description": "",
            "game_args": [],
            "height": 600,
            "java_path": "java",
            "jvm_args": [],
            "separated": false,
            "width": 800,
            "wrapper": "",
            "xms": "1G",
            "xmx": "2G"
        }
    }
}
```
说明：
1. `current`：当前版本的`index`。
2. `versions`下的`key`：版本号，与文件夹名称一致，且不重复。
3. `game_args`和`jvm_args`：自定义启动参数。
4. `description`：备注。
5. `separated`：是否启用版本隔离。
6. 其余同`config.json`。
