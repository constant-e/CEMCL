# Download
**It is strongly recommended to build CEMCL from the latest source code.**

## Release

### v0.3.1
**This is an alpha version which hasn't been fully tested.**

#### ChangeLog
1. Added a Java Manager: manage installed Java; automatically detect system Java on first launch; automatically check Java based on the Minecraft version.
2. Improved the Download Manager: tasks are managed as incomplete and completed; progress can be displayed by size/count/both; added a Forge download progress popup.
3. Improved the Account Manager: generate, cache, and display avatars based on skin heads; offline account names use the default skin name.
4. Window improvements: the window now uses a custom-drawn title bar.
5. Fixed: natives are no longer repeatedly extracted.
6. Added hash verification: SHA-1 verification is performed after download and before launch.
7. Improved version configuration management.
8. Fixed some minor bugs.

**Download Links:** [Linux](https://github.com/constant-e/CEMCL/releases/download/v0.3.1/cemcl-0.3.1-linux-x86_64) |
[macOS](https://github.com/constant-e/CEMCL/releases/download/v0.3.1/cemcl-0.3.1-macos-arm64) |
[Windows](https://github.com/constant-e/CEMCL/releases/download/v0.3.1/cemcl-0.3.1-windows-x86_64.exe)

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
