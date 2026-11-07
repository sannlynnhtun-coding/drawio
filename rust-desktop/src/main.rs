#![cfg_attr(not(test), windows_subsystem = "windows")]

use gpui_kit::*;
use std::{borrow::Cow, path::PathBuf};
use wry::{NewWindowResponse, WebView, WebViewBuilder, WebViewBuilderExtWindows};

const ORIGIN: &str = "https://drawio.localhost/";
const CSP: &str = "default-src 'self'; script-src 'self' 'unsafe-inline' 'unsafe-eval' blob:; style-src 'self' 'unsafe-inline'; img-src 'self' data: blob:; font-src 'self' data:; worker-src 'self' blob:; object-src 'none'; base-uri 'none'; form-action 'none'";

#[derive(rust_embed::RustEmbed)]
#[folder = "../src/main/webapp/"]
struct EditorAssets;

fn asset(path: &str) -> Option<(&'static str, Cow<'static, [u8]>)> {
    let decoded = percent_encoding::percent_decode_str(path)
        .decode_utf8()
        .ok()?;
    let relative = decoded.strip_prefix('/')?;
    if relative.split('/').any(|part| {
        part.is_empty() || part == "." || part == ".." || part.contains(['\\', ':', '\0'])
    }) {
        return None;
    }
    let mime = mime_guess::from_path(relative)
        .first_raw()
        .unwrap_or("application/octet-stream");
    Some((mime, EditorAssets::get(relative)?.data))
}

fn response(uri: &wry::http::Uri) -> wry::http::Response<Cow<'static, [u8]>> {
    let data = asset(uri.path());
    let builder = wry::http::Response::builder().header("Content-Security-Policy", CSP);
    match data {
        Some((mime, bytes)) => builder.header("Content-Type", mime).body(bytes).unwrap(),
        None => builder
            .status(404)
            .body(Cow::Borrowed([].as_slice()))
            .unwrap(),
    }
}

fn bounds(window: &Window) -> wry::Rect {
    let size = window.viewport_size();
    wry::Rect {
        position: wry::dpi::LogicalPosition::new(0.0, 0.0).into(),
        size: wry::dpi::LogicalSize::new(
            f64::from(size.width).max(1.0),
            f64::from(size.height).max(1.0),
        )
        .into(),
    }
}

struct Desktop(WebView);

impl Desktop {
    fn new(window: &Window) -> Result<Self, wry::Error> {
        use windows::Win32::UI::WindowsAndMessaging::{
            GWL_STYLE, GetWindowLongPtrW, ICON_BIG, ICON_SMALL, LoadIconW, SWP_FRAMECHANGED,
            SWP_NOMOVE, SWP_NOSIZE, SWP_NOZORDER, SendMessageW, SetWindowLongPtrW, SetWindowPos,
            WM_SETICON, WS_CLIPCHILDREN,
        };
        use wry::raw_window_handle::{HasWindowHandle, RawWindowHandle};
        if let RawWindowHandle::Win32(handle) = HasWindowHandle::window_handle(window)
            .expect("Missing native window")
            .as_raw()
        {
            let hwnd = windows::Win32::Foundation::HWND(handle.hwnd.get() as *mut _);
            // Keep the parent renderer from painting over WebView2's child window.
            unsafe {
                let instance = windows::Win32::System::LibraryLoader::GetModuleHandleW(None)
                    .expect("Cannot find Drawio module");
                let icon = LoadIconW(
                    Some(instance.into()),
                    windows::core::PCWSTR(1usize as *const u16),
                )
                .expect("Cannot load Drawio icon");
                SendMessageW(
                    hwnd,
                    WM_SETICON,
                    Some(windows::Win32::Foundation::WPARAM(ICON_BIG as usize)),
                    Some(windows::Win32::Foundation::LPARAM(icon.0 as isize)),
                );
                SendMessageW(
                    hwnd,
                    WM_SETICON,
                    Some(windows::Win32::Foundation::WPARAM(ICON_SMALL as usize)),
                    Some(windows::Win32::Foundation::LPARAM(icon.0 as isize)),
                );
                let style = GetWindowLongPtrW(hwnd, GWL_STYLE);
                SetWindowLongPtrW(hwnd, GWL_STYLE, style | WS_CLIPCHILDREN.0 as isize);
                let _ = SetWindowPos(
                    hwnd,
                    None,
                    0,
                    0,
                    0,
                    0,
                    SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_FRAMECHANGED,
                );
            }
        }
        let web = WebViewBuilder::new()
            .with_https_scheme(true)
            .with_incognito(true)
            .with_initialization_script(include_str!("../assets/branding.js"))
            .with_additional_browser_args("--disable-background-networking --proxy-server=127.0.0.1:9 --proxy-bypass-list=drawio.localhost")
            .with_custom_protocol("drawio".into(), |_, request| response(request.uri()))
            .with_navigation_handler(|url| url.starts_with(ORIGIN))
            .with_new_window_req_handler(|_, _| NewWindowResponse::Deny)
            .with_download_started_handler(|url, destination| {
                if !url.starts_with("blob:https://drawio.localhost/") && !url.starts_with("data:") {
                    return false;
                }
                let name = destination.file_name().unwrap_or_default().to_string_lossy();
                if let Some(path) = rfd::FileDialog::new().set_file_name(name.as_ref()).save_file() {
                    *destination = path;
                    true
                } else {
                    false
                }
            })
            .with_bounds(bounds(window))
            .with_url("drawio://localhost/index.html?offline=1&local=1&splash=0")
            .build_as_child(window)?;
        Ok(Self(web))
    }
}

impl Render for Desktop {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().size_full()
    }
}

include!(concat!(env!("OUT_DIR"), "/webview2.rs"));

fn unpack_webview2(folder: &str, cab: &[u8]) -> Result<PathBuf, Box<dyn std::error::Error>> {
    use std::{fs, os::windows::process::CommandExt, process::Command};

    let cache = PathBuf::from(std::env::var_os("LOCALAPPDATA").ok_or("Missing LOCALAPPDATA")?)
        .join("DrawioDesktop")
        .join("WebView2");
    let runtime = cache.join(folder);
    let ready = || runtime.join("msedgewebview2.exe").is_file();
    if ready() {
        return Ok(runtime);
    }

    fs::create_dir_all(&cache)?;
    let staging = tempfile::tempdir_in(&cache)?;
    let archive = staging.path().join("runtime.cab");
    fs::write(&archive, cab)?;
    let system32 =
        PathBuf::from(std::env::var_os("SystemRoot").ok_or("Missing SystemRoot")?).join("System32");
    let expand = Command::new(system32.join("expand.exe"))
        .creation_flags(0x08000000) // CREATE_NO_WINDOW
        .arg("-F:*")
        .arg(&archive)
        .arg(staging.path())
        .output()?;
    let extracted = staging.path().join(folder);
    if !expand.status.success() || !extracted.join("msedgewebview2.exe").is_file() {
        return Err(format!(
            "Cannot unpack WebView2: {}",
            String::from_utf8_lossy(&expand.stderr)
        )
        .into());
    }

    // Windows 10 runs the renderer in an AppContainer that needs read access here.
    let acl = Command::new(system32.join("icacls.exe"))
        .creation_flags(0x08000000)
        .arg(&extracted)
        .args([
            "/grant",
            "*S-1-15-2-2:(OI)(CI)(RX)",
            "*S-1-15-2-1:(OI)(CI)(RX)",
        ])
        .output()?;
    if !acl.status.success() {
        return Err(format!(
            "Cannot prepare WebView2 permissions: {}",
            String::from_utf8_lossy(&acl.stderr)
        )
        .into());
    }

    if let Err(error) = fs::rename(extracted, &runtime) {
        // Another launch may have published the same runtime while this one unpacked it.
        if !ready() {
            return Err(error.into());
        }
    }
    Ok(runtime)
}

fn main() {
    // Use the HWND renderer when hosting a native WebView2 child.
    unsafe { std::env::set_var("GPUI_DISABLE_DIRECT_COMPOSITION", "1") };
    let runtime = if let Some((folder, cab)) = BUNDLED_RUNTIME {
        match unpack_webview2(folder, cab) {
            Ok(runtime) => Some(runtime),
            Err(error) => {
                rfd::MessageDialog::new()
                    .set_title("Cannot prepare bundled WebView2")
                    .set_description(error.to_string())
                    .show();
                std::process::exit(1);
            }
        }
    } else {
        std::env::current_exe()
            .ok()
            .map(|exe| exe.with_file_name("WebView2Runtime"))
            .filter(|dir| dir.join("msedgewebview2.exe").is_file())
    };
    if let Some(runtime) = runtime {
        // Configure WebView2 before starting GPUI or any worker threads.
        unsafe { std::env::set_var("WEBVIEW2_BROWSER_EXECUTABLE_FOLDER", runtime) };
    }

    gpui_kit::application().run(move |cx| {
        gpui_kit::init(cx);
        cx.on_window_closed(|cx, _| {
            if cx.windows().is_empty() {
                cx.quit();
            }
        })
        .detach();
        gpui_kit::open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
                    None,
                    size(px(1200.), px(800.)),
                    cx,
                ))),
                titlebar: Some(TitlebarOptions {
                    title: Some("Drawio".into()),
                    ..Default::default()
                }),
                ..Default::default()
            },
            cx,
            move |window, cx| {
                let desktop = Desktop::new(window).unwrap_or_else(|error| {
                    rfd::MessageDialog::new()
                        .set_title("Cannot start local editor")
                        .set_description(error.to_string())
                        .show();
                    std::process::exit(1);
                });
                window.on_window_should_close(cx, |window, _| {
                    rfd::MessageDialog::new()
                        .set_parent(window)
                        .set_title("Close Drawio?")
                        .set_description("Save any changes using the File menu before closing.")
                        .set_buttons(rfd::MessageButtons::OkCancel)
                        .show()
                        == rfd::MessageDialogResult::Ok
                });
                cx.new(|cx: &mut Context<Desktop>| {
                    cx.observe_window_bounds(window, |this, window, _| {
                        let _ = this.0.set_bounds(bounds(window));
                    })
                    .detach();
                    desktop
                })
            },
        )
        .expect("Cannot open GPUI window");
        cx.activate(true);
    });
}

#[cfg(test)]
mod tests {
    use super::response;
    #[test]
    fn serves_local_editor_and_rejects_traversal() {
        let editor = response(&"/index.html?offline=1&local=1&splash=0".parse().unwrap());
        assert_eq!(editor.status(), 200);
        assert!(String::from_utf8_lossy(editor.body()).contains("js/bootstrap.js"));
        assert_eq!(
            response(&"/%2e%2e/index.html".parse().unwrap()).status(),
            404
        );
    }
}
