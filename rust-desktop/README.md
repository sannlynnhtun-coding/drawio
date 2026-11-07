# Drawio desktop

A Rust and GPUI Kit window hosts the bundled draw.io web editor using WebView2 on Windows, WKWebView on macOS, and WebKitGTK on Linux. Use the editor's File menu to open and save local diagrams. The app serves its own assets and works without a local HTTP server.

## Run

Install Rust for the Windows MSVC target, Visual Studio C++ Build Tools, and the Microsoft Edge WebView2 Runtime. From the repository root:

```powershell
cargo run --locked --manifest-path rust-desktop/Cargo.toml
```

The first build needs Cargo dependencies. Once built, the editor works offline.
Use **File → Open From → Device** to open a local diagram and **File → Save As**
to save a copy. Test with a disposable diagram before editing an original.

```powershell
cargo test --locked --manifest-path rust-desktop/Cargo.toml
```

The Windows executable requires WebView2. For a build that includes the runtime, use
the instructions below.

## Bundle WebView2 for offline deployment

Download the Windows x64 Fixed Version CAB from [Microsoft](https://developer.microsoft.com/en-us/microsoft-edge/webview2/#download-section). Extract it with `expand.exe -F:*` and put the folder containing `msedgewebview2.exe` beside `Drawio.exe` under the name `WebView2Runtime`. Keep the entire runtime folder when distributing the app. The app selects this runtime at startup if present and can use the installed runtime when distributed as an executable alone.

On Windows 10, apply the folder permissions listed in [Microsoft’s Fixed Version deployment instructions](https://learn.microsoft.com/en-us/microsoft-edge/webview2/concepts/distribution#the-fixed-version-runtime-distribution-mode) after extraction. The bundled runtime does not update itself. Replace the folder with a current Fixed Version package when redistributing.

To build a single executable with WebView2 included, point
`DRAWIO_WEBVIEW2_CAB` to the original Microsoft x64 CAB filename before running
`cargo build --release --locked --manifest-path rust-desktop/Cargo.toml`. The resulting
`Drawio.exe` unpacks the runtime into the current user's local app data
on first launch and grants the Windows 10 renderer read access automatically.
Subsequent launches reuse that version. Allow about 1 GB of free space for the
first launch, including the temporary CAB. The runtime does not auto-update.

## Windows installer

Build the single executable with the bundled x64 WebView2 CAB as described
above. Compile the Inno Setup script from the repository root:

```powershell
$exe = (Resolve-Path rust-desktop/target/release/Drawio.exe).Path
ISCC.exe "/DSourceExe=$exe" rust-desktop/installer.iss
```

Use the bundled build as the installer input so the installed app can start
offline. The installer is per user and adds a Start menu shortcut.

## Windows Open with

The installer registers Drawio as an available application for `.drawio` files without changing your default app. Right-click a diagram and choose **Open with > Drawio**, or browse to the installed `Drawio.exe`. Each launch opens its file in a new, maximized window. Filenames containing spaces and Unicode are supported.

The application loads a snapshot of the selected file. Use **File > Save As** to choose where to save edits; opening a file does not grant the embedded browser permission to overwrite the original.

## macOS and Linux releases

The **Desktop macOS and Linux** Actions workflow builds and tests natively on macOS Apple Silicon, macOS Intel, and Ubuntu 24.04 x64. It produces native packages and per-package SHA-256 files as workflow artifacts. Download the successful artifacts and attach them to the matching GitHub release; do not publish a package from a failed build.

- macOS 14+: `Drawio-macOS-arm64.dmg` and `Drawio-macOS-x64.dmg`. Drag Drawio.app to Applications. The bundle registers `.drawio` documents with Finder and uses the system WKWebView. Packages are ad-hoc signed, not Developer ID signed or notarized; Gatekeeper may block downloaded builds.
- Ubuntu 24.04 x64: `Drawio-Linux-x64.deb`. Install with `sudo apt install ./Drawio-Linux-x64.deb`. The package registers the desktop launcher, icon, and `.drawio` MIME type without forcing a default application. It requires GTK 3, WebKitGTK 4.1, Vulkan, and an X11 or XWayland desktop. Required system packages must be installed before offline use; the DEB is not a fully self-contained runtime bundle.

The editor assets are embedded on every platform. macOS and Linux do not use or download WebView2. File-menu open/save uses each system web engine's picker/download behavior; use Save As for files launched from the shell. Native GUI open/edit/save checks on macOS and Linux remain necessary in addition to CI builds/tests.

To package on the matching OS after a release build (Python 3.11+ and OS packaging tools required):

```sh
cargo build --release --locked --manifest-path rust-desktop/Cargo.toml
python3 rust-desktop/package_unix.py --binary rust-desktop/target/release/Drawio --arch x64
```

Use `--arch arm64` on Apple Silicon. Linux packaging derives shared-library dependencies with dpkg-shlibdeps. macOS packaging verifies the executable architecture, app signature, and DMG. Each package receives a `.sha256` checksum file.
