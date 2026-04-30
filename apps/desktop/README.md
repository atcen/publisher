# Publisher Desktop Application

This directory contains the Tauri desktop application shell for Publisher.

## Building

### Linux Distributions

#### Debian/Ubuntu (.deb)
```bash
cargo tauri build -- --target x86_64-unknown-linux-gnu
```
Output: `target/x86_64-unknown-linux-gnu/release/bundle/deb/`

#### Red Hat/Fedora (.rpm)
```bash
cargo tauri build -- --target x86_64-unknown-linux-gnu
```
Output: `target/x86_64-unknown-linux-gnu/release/bundle/rpm/`

#### AppImage
```bash
cargo tauri build -- --target x86_64-unknown-linux-gnu
```
Output: `target/x86_64-unknown-linux-gnu/release/bundle/appimage/`

#### Flatpak
Build using the manifest with `flatpak-builder`:
```bash
flatpak-builder --repo=repo --force-clean build-dir ../desktop/com.publisher.app.yml
flatpak build-bundle repo publisher.flatpak com.publisher.app
```

Or submit `com.publisher.app.yml` to [Flathub](https://flathub.org) for distribution through their infrastructure.

### macOS (.dmg)
```bash
cargo tauri build -- --target aarch64-apple-darwin
```
Output: `target/aarch64-apple-darwin/release/bundle/dmg/`

### Windows (.msi)
```bash
cargo tauri build -- --target x86_64-pc-windows-msvc
```
Output: `target/x86_64-pc-windows-msvc/release/bundle/msi/`

## Distribution

Each platform's binaries are available as separate downloads in [GitHub Releases](https://github.com/atcen/publisher/releases):

- **macOS**: `.dmg`
- **Windows**: `.msi`
- **Linux**: `.deb`, `.rpm`, `.AppImage`
- **Flatpak**: Available via [Flathub](https://flathub.org) or built locally from `com.publisher.app.yml`
