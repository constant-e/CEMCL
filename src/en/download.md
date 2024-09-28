# Download

## Release
### v0.2.0
**Warning**: This is an alpha release which haven't been fully tested.

This version uses qt as the backend. So you should install Qt5 first.

Change Log:
1. support downloading Minecraft (without mod loaders) and mirrors are supported too
2. show a popup window when launching Minecraft
3. add an icon
4. force to use qt as the backend
5. fix failures when .minecraft not exists
6. fix natives extracting for some versions
7. fix -cp args for windows
8. fix not saving game configs
9. fix assets for old versions
10. ui improvements
11. remove built-in optifine downloading support

Download Links:
[Linux](https://github.com/constant-e/CEMCL/releases/download/v0.2.0/cemcl-0.2.0-linux-x86_64) |
[macOS](https://github.com/constant-e/CEMCL/releases/download/v0.2.0/cemcl-0.2.0-macos-x86_64) |
[Windows](https://github.com/constant-e/CEMCL/releases/download/v0.2.0/cemcl-0.2.0-windows-x86_64.exe)

## GitHub CI
Get it in [GitHub Actions](https://github.com/constant-e/CEMCL/actions).

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
