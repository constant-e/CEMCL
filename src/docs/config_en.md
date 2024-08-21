# Configuration Files
Language: [简体中文](config.md) | English

## account.json
Configuration file for accounts.

Type: `[object]`

Example:
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
Explanations:
1. `account_type`: Account type. It will be a part of launch arguments. Type: `string`.
2. `token`: Token. It will be a part of launch arguments. Type: `string`.
3. `uuid`: UUID. Type: `string`.  
**NOTICE for offline users: UUID must match the format, or MC won't be launched. Do not change UUID, or your character may not be recognized by MC.**
1. `user_name`: User name. It will be a part of launch arguments. Type: `string`.

## config.json (In the launcher's folder)
Configuration file for CEMCL.

Type: `object`

Example:
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

Explanations:
1. `close_after_launch`: Whether to close CEMCL after launching MC. Type: `bool`.
2. `..._source`: Download source for the software. Type: `string`.
3. `game_path`: The path of `.minecraft`. Type: `string`.
4. `height`: Default game window height. Type: `string`.
5. `java_path`: The path of `java`. Type: `string`.
6. `width`: Default game window width. Type: `string`.
7. `xms`: Minimum memory for jvm. Type: `string`.  
**It will be a part of launch arguments. Format: `1024M`, `2G`, etc.**
1. `xmx`: Maximum memory for jvm. Type: `string`.  
**It will be a part of launch arguments. Format: `1024M`, `2G`, etc.**

## config.json (In the game's folder)
Configuration file for the game.

Type: `object`

Example:
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
Explanations:
1. `args`: Customized launcher arguments. If not blank, it will be the launch arguments, and **CEMCL won't add other arguments**.Type: `string`.
2. `description`: Description. Type: `string`.
3. `seperated`: Whether to seperate different versions. Type: `bool`.
4. Others are the same as *config.json (In the launcher's folder)*.
