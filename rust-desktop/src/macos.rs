use gpui_kit::{App, Application, AsyncApp};
use std::{cell::RefCell, path::PathBuf, rc::Rc};

#[derive(Default)]
pub struct Opens {
    context: RefCell<Option<AsyncApp>>,
    urls: RefCell<Vec<String>>,
}

pub fn register(application: &Application) -> Rc<Opens> {
    let opens = Rc::new(Opens::default());
    let callback = opens.clone();
    application.on_open_urls(move |urls| {
        callback.urls.borrow_mut().extend(urls);
        if let Some(cx) = callback.context.borrow().clone() {
            let opens = callback.clone();
            cx.spawn(async move |cx| {
                cx.update(|cx| drain(&opens, cx));
            })
            .detach();
        }
    });
    opens
}

pub fn ready(opens: Rc<Opens>, cx: &mut App) -> bool {
    *opens.context.borrow_mut() = Some(cx.to_async());
    drain(&opens, cx)
}

fn file_path(value: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
    url::Url::parse(value)?
        .to_file_path()
        .map_err(|_| "Choose a local .drawio file".into())
}

fn drain(opens: &Opens, cx: &mut App) -> bool {
    let mut opened = false;
    for value in std::mem::take(&mut *opens.urls.borrow_mut()) {
        match file_path(&value).and_then(|path| super::launch_diagram(&path)) {
            Ok(diagram) => {
                super::open_diagram_window(Some(diagram), cx);
                opened = true;
            }
            Err(error) => {
                rfd::MessageDialog::new()
                    .set_title("Cannot open diagram")
                    .set_description(error.to_string())
                    .show();
            }
        }
    }
    opened
}

#[cfg(test)]
mod tests {
    #[test]
    fn finder_urls_keep_spaces_and_unicode_and_reject_remote_urls() {
        let path = std::path::PathBuf::from("/tmp/diagram မြန်မာ #1.drawio");
        let url = url::Url::from_file_path(&path).unwrap();
        assert_eq!(super::file_path(url.as_str()).unwrap(), path);
        assert!(super::file_path("https://example.com/file.drawio").is_err());
        assert!(super::file_path("file://example.com/file.drawio").is_err());
    }
}
