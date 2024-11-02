# CE Minecraft Launcher (CEMCL)
constant-e's Minecraft: Java Edition Launcher

Language: [简体中文](README.md) | English

## Warning
Some of the functions are still under development.

## Introduction
A Minecraft: Java Edition launcher using Rust and Slint.

## Downloads
There are two kinds of version, Release and CI.

### Release
Release versions are stable versions, which is recommended for most of the users.
1. Go to [Github](https://github.com/constant-e/CEMCL/releases)
2. Go to [website](https://constant-e.github.io/CEMCL/en/download.html)

### CI
CI (Continuous Integration) versions are automatically built by GitHub Actions after committing. They are updated more frequently and sometimes unstable.

Go to [GitHub Actions](https://github.com/constant-e/CEMCL/actions) to download CI versions.

## Build
1. Install Rust and Qt
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
4. Generate language file (for non-Windows users)
   ```sh
   # For Debug
   ./update_translations.sh
   # For Release
   ./update_translations.sh --release
   # Or use --all to generate for both Debug and Release
   ```

## Documents
[Documents](https://constant-e.github.io/CEMCL/en/docs)

## Translating
Run `update_tranlations.sh` to update the .po files and generate .mo files. After translating you need to run it again to generate new .mo files.

**Warning**: It doesn't support Windows because of Slint.

## Roadmap
1. (Completed) Launch Minecraft
2. (Completed) Store configuration files for each version in its directory
3. (Completed) Support configurations editing
4. (Completed) Support separated versions
5. (Partially) Multi-language support
6. (Partially) Support installing Minecraft
7. (Completed, Applying for permission) Support online login
8. (Developing)Support installing modified Minecraft
9. Others (More log, UI improvements, etc.)

## Credits
1. [clipboard](https://crates.io/crates/clipboard): clipboard
2. [env_logger](https://crates.io/crates/env_logger): logs
3. [futures](https://crates.io/crates/futures): async
4. [log](https://crates.io/crates/log): logs
5. [reqwest](https://crates.io/crates/reqwest): downloading
6. [serde_json](https://crates.io/crates/serde_json): JSON parsing
7. [slint](https://crates.io/crates/slint): GUI framework
8. [tokio](https://crates.io/crates/tokio): async
9. [uuid](https://crates.io/crates/uuid): UUID generating
10. [zip](https://crates.io/crates/zip): decompressing

## License
Apache License 2.0
