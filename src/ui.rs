use {
    crate::{get_obj, resource},
    gio::{prelude::*, SimpleAction, SimpleActionGroup},
    glib::clone,
    gtk::{
        prelude::*, AboutDialog, ApplicationWindow, Builder, Button, Clipboard, Entry, Notebook,
        TextBuffer, TextView,
    },
    std::{cell::RefCell, path::PathBuf, rc::Rc},
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
    edited: RefCell<bool>,
    path: RefCell<Option<PathBuf>>,
}

impl Ui {
    pub fn new() -> Rc<Self> {
        let b = Builder::from_resource(resource!("ui/main"));

        let this = Rc::new(Self {
            main_window: get_obj!(b, "main-window"),
            input: get_obj!(b, "input"),
            input_buffer: get_obj!(b, "input-buffer"),
            result: get_obj!(b, "result"),
            notebook: get_obj!(b, "notebook"),
            about_button: get_obj!(b, "about-button"),
            about_dialog: get_obj!(b, "about-dialog"),
            edited: RefCell::new(false),
            path: RefCell::new(None),
        });

        // About dialog
        this.about_button
            .connect_clicked(clone!(@strong this => move |_| {
                this.about_dialog.run();
                this.about_dialog.hide();
            }));

        // Copy result (secondary icon click)
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

        let file_ag = this.new_action_group("file");

        let open_action = SimpleAction::new("open", None);
        open_action.connect_activate(clone!(@strong this => move |_, _| {
            if *this.edited.borrow() {
                println!("open, edited");
            } else {
                println!("open, unedited");
            }
        }));
        file_ag.add_action(&open_action);

        let save_action = SimpleAction::new("save", None);
        save_action.connect_activate(clone!(@strong this => move |_, _| {
            if *this.edited.borrow() {
                println!("save, edited");
            } /* else {} */ // No point in saving if no changes are made
        }));
        file_ag.add_action(&save_action);

        this
    }

    pub fn set_input(&self, content: &str) {
        self.input_buffer.set_text(content);
    }

    pub fn set_result(&self, result: &str) {
        self.result.set_text(result);
    }

    pub fn set_app(&self, app: &gtk::Application) {
        self.main_window.set_application(Some(app));
    }

    pub fn new_action_group(&self, name: &str) -> SimpleActionGroup {
        let ag = SimpleActionGroup::new();
        self.main_window.insert_action_group(name, Some(&ag));
        ag
    }

    pub fn show_math(&self) {
        self.notebook.set_current_page(Some(0));
    }

    pub fn show_help(&self) {
        self.notebook.set_current_page(Some(1));
    }

    pub fn show(&self) {
        self.main_window.show_all();
        self.update_title();
    }

    pub fn set_path(&self, path: PathBuf) {
        self.path.replace(Some(path));
    }

    fn update_title(&self) {
        let text = match self
            .path
            .borrow()
            .as_ref()
            .and_then(|p| p.to_str().and_then(|p| Some(p.to_string())))
        {
            Some(p) => format!("Math Expr Eval - {}", p),
            None => String::from("Math Expr Eval"),
        };
        match *self.edited.borrow() {
            true => self.main_window.set_title(&format!("⏺ {}", text)),
            false => self.main_window.set_title(&text),
        }
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
                    self.set_result("");
                }
                v => {
                    self.set_result(&v.to_string());
                }
            },
            Err(e) => {
                self.set_result(&e.to_string());
            }
        }
    }
}
