# 配置文件说明

## account.json
模板：docs/account_template.json

样例：
```json
[
    {
        "token": "xxx",
        "type": "Legacy",
        "userName": "Steve",
        "uuid": "xxx",
        
    }, 
    {
        "token": "xxx",
        "type": "msa",
        "userName": "Steve",
        "uuid": "xxx",
        
    }, 
    ...
]
```
说明：为一个数组，每个元素作为一个账户被加载。其中`token`是启动参数中的accessToken，对于离线用户请勿留空，建议输入None；`type`是登录方式，直接作为启动参数，`Legacy`代表离线，`msa`代表微软，`Mojang`代表Mojang。暂未支持其他方式。`userName`和`uuid`分别是用户名和UUID。

## config.json
模板：docs/config_template.json

样例：
```json
{
    "closeAfterLaunch": false,
    "forgeSource": "",
    "gameDir": ".minecraft",
    "height": 600,
    "javaDir": "java",
    "MCSource": "",
    "width": 800,
    "xms": "1G", 
    "xmx": "2G"
}
```
说明：启动器配置文件。

## index.json
模板：docs/index_template.json

样例：
```json
[
    {
        "args": "",
        "describe": "",
        "height": 800,
        "sha1": "",
        "type": "release",
        "version": "1.12.2",
        "width": 600,
        "xms": "1G",
        "xmx": "2G"
    },
    {
        "args": "",
        "describe": "",
        "height": 800,
        "sha1": "",
        "type": "release",
        "version": "1.20.2",
        "width": 600,
        "xms": "1G",
        "xmx": "2G"
    }
]
```
说明：预生成的版本信息，同时保存用户添加的备注和自定义参数。
