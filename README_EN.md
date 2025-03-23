# CE Minecraft Launcher (CEMCL)
constant-e's Minecraft: Java Edition Launcher

Language: [简体中文](README.md) | English

## Notice
Some of the functions are still under development.

## Introduction
A Minecraft: Java Edition launcher using Rust and Slint.

## Downloads
**It is strongly recommended to build CEMCL from the latest source code.**

### Release
Release versions are stable versions which are recommended for most of the users.
1. Download from [Github](https://github.com/constant-e/CEMCL/releases)
2. Download from [website](https://constant-e.github.io/CEMCL/en/download.html)

### CI
CI (Continuous Integration) versions are automatically built by GitHub Actions after committing. They are updated more frequently and sometimes unstable.

Go to [GitHub Actions](https://github.com/constant-e/CEMCL/actions) to download CI versions.

## Build
1. Install Rust and Qt if you want a Qt backend
2. Clone this repository
   ```sh
   git clone https://github.com/constant-e/CEMCL.git
   ```
3. Build
   ```sh
   # Build Debug
   cargo build
   # Build Release
   cargo build --release
   ```


## Documents
[Documents](https://constant-e.github.io/CEMCL/en/docs)

## Translating
Run `update_tranlations.sh` to update the .po files.

## Roadmap
1. (Completed) Launch Minecraft
2. (Completed) Support customized configures
3. (Completed) Multi-language support
4. (Completed) Support installing Minecraft
5. (Completed) Support online login
6. (Developing) Support installing modified Minecraft
7. Others (More log, UI improvements, etc.)

## Credits
1. [BMCLAPI2](https://bmclapidoc.bangbang93.com/): Forge downloading
2. [clipboard](https://crates.io/crates/clipboard): clipboard
3. [env_logger](https://crates.io/crates/env_logger): logs
4. [futures](https://crates.io/crates/futures): async
5. [log](https://crates.io/crates/log): logs
6. [reqwest](https://crates.io/crates/reqwest): downloading
7. [serde_json](https://crates.io/crates/serde_json): JSON parsing
8. [slint](https://crates.io/crates/slint): GUI framework
9. [tokio](https://crates.io/crates/tokio): async
10. [uuid](https://crates.io/crates/uuid): UUID generating
11. [webbrowser](https://crates.io/crates/webbrowser): opening web browser
12. [zip](https://crates.io/crates/zip): decompressing

## License
Apache License 2.0
