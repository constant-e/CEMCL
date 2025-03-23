# Download
**It is strongly recommended to build CEMCL from the latest source code.**

## Release

### v0.2.2
**This is an alpha version which hasn't been fully tested.**

#### Change Log
1. Add a progress bar when launching, replacing the old popup window
2. Add a downloader to replace old download methods (developing, the GUI of it hasn't work yet)
3. Multi-language support for windows (with bundled locales for all versions)
4. Fix the support of customized launch arguments
5. Partially support downloading forge
6. Remove the console for Windows
7. Switch the UI style to fluent

**Download Links:** [Linux](https://github.com/constant-e/CEMCL/releases/download/v0.2.2/cemcl-0.2.2-linux-x86_64.zip) |
[macOS](https://github.com/constant-e/CEMCL/releases/download/v0.2.2/cemcl-0.2.2-macos-x86_64.zip) |
[Windows](https://github.com/constant-e/CEMCL/releases/download/v0.2.2/cemcl-0.2.2-windows-x86_64.zip)

## GitHub CI
Get it in [GitHub Actions](https://github.com/constant-e/CEMCL/actions).

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
