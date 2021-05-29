use {
    crate::{get_obj, resource},
    gtk::{prelude::*, AboutDialog, ApplicationWindow, Builder, Entry, Label},
    std,
};

#[derive(Debug)]
struct Ui {
    main_window: ApplicationWindow,
    input: Entry,
    result: Label,
    about_dialog: AboutDialog,
}

impl Ui {
    pub fn new() -> Option<Self> {
        let b = Builder::from_resource(resource!("ui"));
        todo!()
    }
}
