# Release rules

- Name the Windows release installer exactly `Drawio-Setup-x64.exe`.
- Keep `rust-desktop/installer.iss` configured with `OutputBaseFilename=Drawio-Setup-x64`; do not override it with alternate release names.
- Deliver the release installer with the local draw.io editor and WebView2 runtime bundled so installation and use work offline.

- macOS release packages are `Drawio-macOS-arm64.dmg` and `Drawio-macOS-x64.dmg`; Linux x64 packages are `Drawio-Linux-x64.deb`. Include SHA-256 checksums.
- Build and test macOS/Linux packages on their native GitHub Actions runners before publishing. State Linux system-runtime requirements and macOS signing/notarization status in release notes.
