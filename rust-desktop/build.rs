use std::{env, fs, path::PathBuf};

fn main() {
    println!("cargo:rerun-if-changed=Drawio.rc");
    println!("cargo:rerun-if-changed=assets/Drawio.ico");
    embed_resource::compile("Drawio.rc", embed_resource::NONE)
        .manifest_optional()
        .expect("Cannot embed Drawio icon");
    println!("cargo:rerun-if-changed=../src/main/webapp");
    println!("cargo:rerun-if-env-changed=DRAWIO_WEBVIEW2_CAB");
    let runtime = env::var_os("DRAWIO_WEBVIEW2_CAB")
        .map(|cab| {
            let cab = PathBuf::from(cab)
                .canonicalize()
                .expect("Cannot find WebView2 CAB");
            let folder = cab
                .file_stem()
                .and_then(|name| name.to_str())
                .expect("Invalid CAB filename");
            let version = folder
                .strip_prefix("Microsoft.WebView2.FixedVersionRuntime.")
                .and_then(|name| name.strip_suffix(".x64"))
                .expect("Use Microsoft's original x64 CAB filename");
            assert!(
                version.split('.').count() == 4
                    && version.split('.').all(|n| n.parse::<u32>().is_ok()),
                "Invalid WebView2 version"
            );
            println!("cargo:rerun-if-changed={}", cab.display());
            format!(
                "Some(({folder:?}, include_bytes!({:?})))",
                cab.to_str().expect("Invalid CAB path")
            )
        })
        .unwrap_or_else(|| "None".into());
    let output = PathBuf::from(env::var_os("OUT_DIR").unwrap()).join("webview2.rs");
    fs::write(
        output,
        format!("const BUNDLED_RUNTIME: Option<(&str, &[u8])> = {runtime};"),
    )
    .unwrap();
}
