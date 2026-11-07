use gpui_kit::{App, Application, Window, WindowingModes};
use wry::raw_window_handle::{
    HandleError, HasWindowHandle, RawWindowHandle, WindowHandle, XlibWindowHandle,
};

pub fn application() -> Application {
    // Both GPUI and the embedded GTK view must use the same X11 server.
    unsafe { std::env::set_var("GDK_BACKEND", "x11") };
    gtk::init().expect("Drawio requires an X11 or XWayland desktop and GTK 3");
    gpui_kit::platform::linux(WindowingModes::X11)
}

pub fn pump_events(cx: &App) {
    cx.spawn(async move |cx| {
        loop {
            for _ in 0..100 {
                if !gtk::events_pending() {
                    break;
                }
                gtk::main_iteration_do(false);
            }
            cx.background_executor()
                .timer(std::time::Duration::from_millis(16))
                .await;
        }
    })
    .detach();
}

pub struct Parent(XlibWindowHandle);

impl HasWindowHandle for Parent {
    fn window_handle(&self) -> Result<WindowHandle<'_>, HandleError> {
        // GPUI owns the X11 window for the lifetime of the WebView.
        Ok(unsafe { WindowHandle::borrow_raw(RawWindowHandle::Xlib(self.0)) })
    }
}

pub fn parent(window: &Window) -> Result<Parent, wry::Error> {
    let handle = HasWindowHandle::window_handle(window)?;
    match handle.as_raw() {
        RawWindowHandle::Xlib(handle) => Ok(Parent(handle)),
        RawWindowHandle::Xcb(handle) => {
            Ok(Parent(XlibWindowHandle::new(handle.window.get().into())))
        }
        _ => Err(wry::Error::UnsupportedWindowHandle),
    }
}
