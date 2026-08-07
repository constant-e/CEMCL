# Configuration Files

## account.json
Account configuration file

Example:
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

Explanation:
1. `current`: The current account's `index`.
2. `account_type`: The login method; it will be written directly into the launch arguments.
3. `token`: The user's refresh token.
4. `uuid`: The user's UUID.
**For offline users: the UUID must match the required format; otherwise the game cannot start. Do not change the UUID, or your account not be recognized.**
1. `user_name`: The username; it will be written directly into the launch arguments.

## config.json
Launcher configuration file

Example:
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

Explanation:
1. `close_after_launch`: Whether to close the launcher after launching the game.
2. `concurrency`: The maximum download concurrency.
3. `..._source`: The download source for the corresponding software.
4. `game_path`: The location of the `.minecraft` folder.
5. `height`: Default game window height.
6. `java_path`: The path to `java`.
7. `width`: Default game window width.
8. `wrapper`: Wrapper.
9. `xms`: The minimum memory allocated to the JVM.
**This will be written directly into the launch arguments in formats such as `1024M`, `2G`, and so on.**
10. `xmx`: The maximum memory allocated to the JVM.
**This will be written directly into the launch arguments in formats such as `1024M`, `2G`, and so on.**

## versions.json
Custom configuration file for game versions

CEMCL also stores the official launcher format's `launcher_profiles.json`, but it only reads configuration from `versions.json`.

Example:
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

Explanation:
1. `current`: The current version's `index`.
2. The `key` under `versions`: The version number, which must match the folder name and be unique.
3. `game_args` and `jvm_args`: Custom launch arguments.
4. `description`: Notes.
5. `separated`: Whether version isolation is enabled.
6. The rest are the same as `config.json`.
