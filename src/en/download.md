# Download

## Release
### v0.2.1
**Warning**: This is an alpha release which haven't been fully tested.

#### Change Log
[New]
1. support online login
2. partially support multi-language
3. support limiting the concurrency of downloading
4. show a warning dialog if init failed
5. support load forge list (without downloading)

[Fix]
1. fix wrong index when deleting game
2. fix index out of range

[Other]
1. prepare dlls for windows
2. better logs

**Download Links:**
[Linux](https://github.com/constant-e/CEMCL/releases/download/v0.2.1/cemcl-0.2.1-linux-x86_64.zip) |
[macOS](https://github.com/constant-e/CEMCL/releases/download/v0.2.1/cemcl-0.2.1-macos-x86_64.zip) |
[Windows](https://github.com/constant-e/CEMCL/releases/download/v0.2.1/cemcl-0.2.1-windows-x86_64.zip)

## GitHub CI
Get it in [GitHub Actions](https://github.com/constant-e/CEMCL/actions).

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
