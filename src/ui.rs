use {
    crate::{get_obj, resource},
    glib::clone,
    gtk::{
        prelude::*, AboutDialog, ApplicationWindow, Builder, Button, Clipboard, Entry, Notebook,
        TextBuffer, TextView,
    },
    std::rc::Rc,
};

#[derive(Debug)]
pub struct Ui {
    main_window: ApplicationWindow,
    input: TextView,
    input_buffer: TextBuffer,
    result: Entry,
    notebook: Notebook,
    about_button: Button,
    about_dialog: AboutDialog,
}

impl Ui {
    pub fn new() -> Option<Rc<Self>> {
        let b = Builder::from_resource(resource!("ui/main"));

        let this = Rc::new(Self {
            main_window: get_obj!(b, "main-window"),
            input: get_obj!(b, "input"),
            input_buffer: get_obj!(b, "input-buffer"),
            result: get_obj!(b, "result"),
            notebook: get_obj!(b, "notebook"),
            about_button: get_obj!(b, "about-button"),
            about_dialog: get_obj!(b, "about-dialog"),
        });

        // About dialog
        this.about_button
            .connect_clicked(clone!(@strong this => move |_| {
                this.about_dialog.run();
                this.about_dialog.hide();
            }));

        this.result
            .connect_icon_release(|entry, pos, _evt_btn| match pos {
                gtk::EntryIconPosition::Secondary => {
                    Clipboard::get(&gdk::SELECTION_CLIPBOARD)
                        .set_text(&entry.get_text().to_string());
                }
                _ => {}
            });

        // Do math
        this.input_buffer
            .connect_changed(clone!(@strong this => move |_| this.eval()));

        Some(this)
    }

    pub fn set_app(&self, app: &gtk::Application) {
        self.main_window.set_application(Some(app));
    }

    pub fn show(&self) {
        self.main_window.show_all();
    }

    fn eval(&self) {
        let (iter_start, iter_end) = self.input_buffer.get_bounds();
        let content = self
            .input_buffer
            .get_text(&iter_start, &iter_end, true)
            .map(|c| c.to_string())
            .unwrap_or(String::new());

        match evalexpr::eval_with_context_mut(
            &content,
            &mut evalexpr::math_consts_context!().unwrap(),
        ) {
            Ok(val) => match val {
                evalexpr::Value::Empty => {
                    self.result.set_text("");
                }
                v => {
                    self.result.set_text(&v.to_string());
                }
            },
            Err(e) => {
                self.result.set_text(&e.to_string());
            }
        }
    }
}
