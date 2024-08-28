# Download

## Release
### v0.1.1
**Warning**: This is a alpha release. Only features not requiring network connection are implemented. And all the features haven't been fully tested.

Change Log:
1. fix not loading all default configs when config.json not exists
2. fix not saving changes for config.json
3. fix the logic when judging if the index is out of range
4. fix not reloading game list when changing settings

**Warning**: This is the latest version of 0.1.x. I'm currently working on features related to downloading, which may import some breaking changes (e.g. new keys in config.json). And there won't be any new releases before these features are completed (so you won't get bug fix versions for v0.1.x). This version is still not ready to use.

Download Links:
[Linux](https://github.com/constant-e/CEMCL/releases/download/v0.1.1/cemcl-0.1.1-linux) |
[macOS](https://github.com/constant-e/CEMCL/releases/download/v0.1.1/cemcl-0.1.1-macos) |
[Windows](https://github.com/constant-e/CEMCL/releases/download/v0.1.1/cemcl-0.1.1-windows.exe)

## GitHub CI
Get it in [GitHub Actions](https://github.com/constant-e/CEMCL/actions).

## Build
1. Install Rust (with Cargo).
2. Clone the repository:
   ```sh
   git clone https://github.com/constant-e/CEMCL.git && cd CEMCL
   ```
3. Build:
   ```sh
   # Build debug:
   cargo build
   # Build release:
   cargo build --release
   ```
