# Release rules

- Name the Windows release installer exactly `Drawio-Setup-x64.exe`.
- Keep `rust-desktop/installer.iss` configured with `OutputBaseFilename=Drawio-Setup-x64`; do not override it with alternate release names.
- Deliver the release installer with the local draw.io editor and WebView2 runtime bundled so installation and use work offline.
