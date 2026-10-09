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
**For offline users: the UUID must match the required format, otherwise the game cannot start. Do not change the UUID, or your account may not be recognized.**
5. `user_name`: The username; it will be written directly into the launch arguments.

## config.json
Launcher configuration file

Example:
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
Explanation (every field is required except `progress_mode`; a missing field prevents the launcher from starting):
1. `close_after_launch`: Whether to close the launcher after launching the game.
2. `concurrency`: The maximum download concurrency.
3. `..._source`: The download source for the corresponding component (`assets`, `fabric`, `forge`, `game`, `libraries`).
4. `game_path`: The location of the `.minecraft` folder.
5. `height`: Default game window height.
6. `width`: Default game window width.
7. `progress_mode`: How download progress is displayed: `BySize`, `ByNumber`, or `Both`; when missing or set to another value, `BySize` is used.
8. `wrapper`: Wrapper.
9. `xms`: The minimum memory allocated to the JVM.
**This will be written directly into the launch arguments in formats such as `1024M`, `2G`, and so on.**
10. `xmx`: The maximum memory allocated to the JVM.
**This will be written directly into the launch arguments in formats such as `1024M`, `2G`, and so on.**

`java_path` from older versions has been removed; Java installations are now managed by [java.json](#java-json).

## java.json
Java installation list configuration file

Example:
```json
{
    "default": 0,
    "java_list": [
        "/usr/lib/jvm/java-17-openjdk",
        "/usr/lib/jvm/java-21-openjdk"
    ]
}
```
Explanation:
1. `java_list`: Java home directories, each containing `bin/java` (`bin/java.exe` on Windows); the list is sorted by Java version in descending order.
2. `default`: The `index` of the default Java in the list (pre-selected when adding a version); `null` or missing means none is selected.
3. `java_index` in `versions.json` indexes into this list.

## versions.json
Custom configuration file for game versions

CEMCL also stores the official launcher format's `launcher_profiles.json`, but it only reads configuration from `versions.json`.

Example:
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
Explanation:
1. `current`: The current version's `index` (the list is sorted by version name).
2. The `key` under `versions`: The version number, matching the folder name and unique.
3. `version`: The version number, the same as the `key`.
4. `game_type`: The game type: `release`, `snapshot`, `old_alpha`, or `old_beta`; when omitted, it is read from the `type` field of the version JSON.
5. `description`: Notes.
6. `game_args` and `jvm_args`: Custom launch arguments.
7. `java_index`: The index into the `java.json` list; when omitted, none is selected (the `java` from the system `PATH` is used at launch).
8. `separated`: Whether version isolation is enabled.
9. `height`, `width`, `wrapper`, `xms`, `xmx`: Same as in `config.json`.

When an option is missing, `null`, or empty (`0`/empty string/empty list), the value from `config.json` is used; when saving, options equal to the defaults are omitted so that they keep following `config.json`.

At startup, CEMCL checks the entries against `.minecraft`: entries without a matching version JSON are removed, other versions found in `.minecraft` are added, and `current` is re-anchored to the same version.
