# Drawio desktop for Windows

A Rust and GPUI Kit window hosts the bundled draw.io web editor in WebView2. Use the editor's File menu to open and save local diagrams. The app serves its own assets and works without a local HTTP server.

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

The executable requires WebView2. For a build that includes the runtime, use
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
