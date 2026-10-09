# CE Minecraft Launcher (CEMCL)
constant-e's Minecraft: Java Edition Launcher

Language: [简体中文](README.md) | English

## Introduction
A Minecraft: Java Edition launcher using Rust and Slint.

## Downloads
**It is strongly recommended to build CEMCL from the latest source code.**

### Release
Release versions are stable versions which are recommended for most of the users.

Download from [Github](https://github.com/constant-e/CEMCL/releases)

### CI
CI (Continuous Integration) versions are automatically built by GitHub Actions after committing. They are updated more frequently and sometimes unstable.

Download from [GitHub Actions](https://github.com/constant-e/CEMCL/actions)

## Build
1. Install Rust
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
Run `crates/frontend/update_tranlations.sh` to update the .po files.

## Roadmap
1. Basic functions (v0.1.x and v0.2.x) (completed)
2. Better UI and function (v0.3.x) (developing)
3. More modules

## Credits
1. [BMCLAPI2](https://bmclapidoc.bangbang93.com/): Forge downloading
2. [base64](https://crates.io/crates/base64): skin data decoding
3. [clipboard](https://crates.io/crates/clipboard): clipboard
4. [env_logger](https://crates.io/crates/env_logger): logs
5. [futures](https://crates.io/crates/futures): async
6. [log](https://crates.io/crates/log): logs
7. [png](https://crates.io/crates/png): skin texture decoding
8. [reqwest](https://crates.io/crates/reqwest): downloading
9. [serde_json](https://crates.io/crates/serde_json): JSON parsing
10. [sha1](https://crates.io/crates/sha1): file hash verification
11. [slint](https://crates.io/crates/slint): GUI framework
12. [tokio](https://crates.io/crates/tokio): async
13. [uuid](https://crates.io/crates/uuid): UUID generating
14. [webbrowser](https://crates.io/crates/webbrowser): opening web browser
15. [zip](https://crates.io/crates/zip): decompressing

## License
Apache License 2.0
