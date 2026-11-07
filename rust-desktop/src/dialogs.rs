use std::path::PathBuf;

#[cfg(target_os = "linux")]
use gtk::prelude::*;

pub fn error(title: &str, message: &str) {
    #[cfg(not(target_os = "linux"))]
    rfd::MessageDialog::new()
        .set_title(title)
        .set_description(message)
        .show();
    #[cfg(target_os = "linux")]
    {
        gtk::init().expect("Cannot initialize GTK dialogs");
        let dialog = message_dialog(title, message, gtk::ButtonsType::Ok);
        dialog.run();
        dialog.close();
    }
}

pub fn confirm_close(window: &gpui_kit::Window) -> bool {
    let message = "Save any changes using the File menu before closing.";
    #[cfg(not(target_os = "linux"))]
    {
        rfd::MessageDialog::new()
            .set_parent(window)
            .set_title("Close Drawio?")
            .set_description(message)
            .set_buttons(rfd::MessageButtons::OkCancel)
            .show()
            == rfd::MessageDialogResult::Ok
    }
    #[cfg(target_os = "linux")]
    {
        let _ = window;
        let dialog = message_dialog("Close Drawio?", message, gtk::ButtonsType::OkCancel);
        let accepted = dialog.run() == gtk::ResponseType::Ok;
        dialog.close();
        accepted
    }
}

pub fn save_file(name: &str) -> Option<PathBuf> {
    #[cfg(not(target_os = "linux"))]
    {
        rfd::FileDialog::new().set_file_name(name).save_file()
    }
    #[cfg(target_os = "linux")]
    {
        let dialog = gtk::FileChooserDialog::new(
            Some("Save As"),
            None::<&gtk::Window>,
            gtk::FileChooserAction::Save,
            &[
                ("Cancel", gtk::ResponseType::Cancel),
                ("Save", gtk::ResponseType::Accept),
            ],
        );
        dialog.set_current_name(name);
        dialog.set_do_overwrite_confirmation(true);
        dialog.set_keep_above(true);
        let path = if dialog.run() == gtk::ResponseType::Accept {
            dialog.filename()
        } else {
            None
        };
        dialog.close();
        path
    }
}

#[cfg(target_os = "linux")]
fn message_dialog(title: &str, message: &str, buttons: gtk::ButtonsType) -> gtk::MessageDialog {
    // Wry owns GTK on this thread; rfd's separate GTK thread can deadlock its main context.
    let dialog = gtk::MessageDialog::new(
        None::<&gtk::Window>,
        gtk::DialogFlags::MODAL,
        gtk::MessageType::Warning,
        buttons,
        message,
    );
    dialog.set_title(title);
    dialog.set_default_response(gtk::ResponseType::Ok);
    dialog.set_keep_above(true);
    dialog
}
