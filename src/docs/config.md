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
    "assets_source": "https://resources.download.minecraft.net",
    "close_after_launch": false,
    "concurrency": 10,
    "fabric_source": "https://maven.fabricmc.net",
    "forge_source": "https://files.minecraftforge.net",
    "game_path": ".minecraft",
    "game_source": "https://piston-meta.mojang.com",
    "height": 600,
    "libraries_source": "https://libraries.minecraft.net",
    "progress_mode": "BySize",
    "width": 800,
    "wrapper": "",
    "xms": "1G",
    "xmx": "2G"
}
```
说明（除`progress_mode`外均为必填项，缺失会导致启动器无法启动）：
1. `close_after_launch`：游戏启动后是否关闭启动器。
2. `concurrency`：下载时的最大并发数量。
3. `..._source`：相应软件的下载源（`assets`、`fabric`、`forge`、`game`、`libraries`）。
4. `game_path`：`.minecraft`文件夹的位置。
5. `height`：默认游戏窗口高度。
6. `width`：默认游戏窗口宽度。
7. `progress_mode`：下载进度的显示方式：`BySize`（按大小）、`ByNumber`（按数量）或`Both`（均显示）；缺失或为其他值时使用`BySize`。
8. `wrapper`：封装器。
9. `xms`：为jvm分配的最小内存。
**将直接填入启动参数，格式如：`1024M`、`2G`等。**
10. `xmx`：为jvm分配的最大内存。
**将直接填入启动参数，格式如：`1024M`、`2G`等。**

旧版本中的`java_path`已被移除，Java安装改由[java.json](#java-json)管理。

## java.json
Java安装列表配置文件

样例：
```json
{
    "default": 0,
    "java_list": [
        "/usr/lib/jvm/java-17-openjdk",
        "/usr/lib/jvm/java-21-openjdk"
    ]
}
```
说明：
1. `java_list`：Java安装目录（java home）列表，目录中须有`bin/java`（Windows上为`bin/java.exe`）；列表按Java版本降序排列。
2. `default`：默认使用的Java在列表中的`index`（添加版本时默认选中）；为`null`或缺失表示未选择。
3. `versions.json`中的`java_index`索引此列表。

## versions.json
游戏版本的自定义配置文件

CEMCL也会储存官方启动器格式的`launcher_profiles.json`，但只会从`versions.json`读取配置。

样例：
```json
{
    "current": 0,
    "versions": {
        "1.20.1": {
            "version": "1.20.1",
            "game_type": "release",
            "description": "",
            "game_args": [],
            "jvm_args": [],
            "java_index": 0,
            "separated": true,
            "height": 720,
            "width": 1280,
            "wrapper": "",
            "xms": "2G",
            "xmx": "4G"
        }
    }
}
```
说明：
1. `current`：当前版本的`index`（列表按版本名排序）。
2. `versions`下的`key`：版本号，与文件夹名称一致，且不重复。
3. `version`：版本号，与`key`相同。
4. `game_type`：游戏类型：`release`、`snapshot`、`old_alpha`或`old_beta`；留空时从版本json的`type`中读取。
5. `description`：备注。
6. `game_args`和`jvm_args`：自定义启动参数。
7. `java_index`：`java.json`列表中的索引；留空表示未选择（启动时使用系统`PATH`中的`java`）。
8. `separated`：是否启用版本隔离。
9. `height`、`width`、`wrapper`、`xms`、`xmx`：含义同`config.json`。

配置项缺省、为`null`或留空（`0`/空串/空列表）时，使用`config.json`中的默认值；保存时，与默认值相同的配置项会被省略，以继续跟随`config.json`。

启动时CEMCL会与`.minecraft`核对：在`.minecraft`中找不到对应版本json的项会被删除，`.minecraft`中的其他版本会被加入；`current`会重新定位到原来的版本。
