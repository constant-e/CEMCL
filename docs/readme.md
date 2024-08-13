# 配置文件说明

## account.json
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
说明：为一个数组，每个元素作为一个账户被加载。其中`token`是启动参数中的accessToken，对于离线用户请勿留空，建议输入None；`type`是登录方式，直接作为启动参数，`Legacy`代表离线，`msa`代表微软，`Mojang`代表Mojang。暂未支持其他方式。`user_name`和`uuid`分别是用户名和UUID。

## config.json（位于启动器目录下）
模板：[docs/config_launcher_template.json](config_launcher_template.json)

样例：
```json
{
    "close_after_launch": false,
    "forge_source": "",
    "game_dir": ".minecraft",
    "height": 600,
    "java_path": "java",
    "mc_source": "",
    "width": 800,
    "xms": "1G", 
    "xmx": "2G"
}
```
说明：启动器配置文件。

## config.json（位于游戏目录下）
模板：[docs/config_game_template.json](config_game_template.json)

样例：
```json
{
    "args": "",
    "description": "",
    "height": -1,
    "java_path": "",
    "seperated": false,
    "width": -1,
    "xms": "1G",
    "xmx": "2G"
}
```
说明：保存在每个版本中，用于储存用户的自定义配置。
