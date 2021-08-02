use {
    crate::{get_obj, resource},
    gio::{prelude::*, SettingsBindFlags, SettingsExt, SimpleAction, SimpleActionGroup},
    glib::clone,
    gtk::{
        prelude::*, AboutDialog, ApplicationWindow, Builder, Button, Clipboard, Entry,
        FileChooserAction, FileChooserNative, FileFilter, InfoBar, Label, ResponseType, Stack,
        TextBuffer, TextView,
    },
    std::{cell::RefCell, fs, path::PathBuf, rc::Rc},
};

#[derive(Debug)]
pub struct Ui {
    main_window: ApplicationWindow,
    input: TextView,
    input_buffer: TextBuffer,
    result: Entry,
    stack: Stack,
    about_button: Button,
    about_dialog: AboutDialog,
    info_bar: InfoBar,
    info_bar_label: Label,
    open_dialog: FileChooserNative,
    save_dialog: FileChooserNative,
    edited: RefCell<bool>,
    path: RefCell<Option<PathBuf>>,
    settings: gio::Settings,
}

impl Ui {
    pub fn new(settings: &gio::Settings) -> Rc<Self> {
        let b = Builder::from_resource(resource!("ui/main"));

        settings.bind(
            "ask-save-on-exit",
            &get_obj!(b, gtk::Switch, "ask-save-switch"),
            "active",
            SettingsBindFlags::DEFAULT,
        );

        let this = Rc::new(Self {
            main_window: get_obj!(b, "main-window"),
            input: get_obj!(b, "input"),
            input_buffer: get_obj!(b, "input-buffer"),
            result: get_obj!(b, "result"),
            stack: get_obj!(b, "stack"),
            about_button: get_obj!(b, "about-button"),
            about_dialog: get_obj!(b, "about-dialog"),
            info_bar: get_obj!(b, "info-bar"),
            info_bar_label: get_obj!(b, "info-bar-label"),
            open_dialog: FileChooserNative::new(
                None,
                Some(&get_obj!(b, ApplicationWindow, "main-window")),
                FileChooserAction::Open,
                None,
                None,
            ),
            save_dialog: FileChooserNative::new(
                None,
                Some(&get_obj!(b, ApplicationWindow, "main-window")),
                FileChooserAction::Save,
                None,
                None,
            ),
            edited: RefCell::new(false),
            path: RefCell::new(None),
            settings: settings.clone(),
        });

        let file_filter = FileFilter::new();
        file_filter.add_pattern("*.mee");
        this.open_dialog.set_filter(&file_filter);

        // Infobar close button
        this.info_bar.connect_response(|ib, _| {
            ib.set_visible(false);
            ib.set_revealed(false);
        });

        // About dialog
        this.about_button
            .connect_clicked(clone!(@strong this => move |_| {
                this.about_dialog.run();
                this.about_dialog.hide();
            }));

        // Copy result (secondary icon click)
        this.result.connect_icon_release(|entry, pos, _evt_btn| {
            if pos == gtk::EntryIconPosition::Secondary {
                Clipboard::get(&gdk::SELECTION_CLIPBOARD).set_text(&entry.get_text().to_string());
            }
        });

        // Do math
        this.input_buffer
            .connect_changed(clone!(@strong this => move |_| {
                this.edited.replace(true);
                this.update_title();
                this.eval();
            }));

        let file_ag = this.new_action_group("file");

        let open_action = SimpleAction::new("open", None);
        open_action.connect_activate(clone!(@strong this => move |_, _| {
            if this.stack.get_visible_child_name() == Some("math".into()) {
                if *this.edited.borrow() && this.settings.get_boolean("ask-save-on-exit") {
                    if this.ask_save_file() {
                        this.save_file();
                        this.open_file();
                    } else {
                        this.open_file();
                    }
                } else {
                    this.open_file();
                }
            }
        }));
        file_ag.add_action(&open_action);

        let save_action = SimpleAction::new("save", None);
        save_action.connect_activate(clone!(@strong this => move |_, _| {
            if *this.edited.borrow() {
                this.save_file();
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
        self.stack.set_visible_child_name("math");
    }

    pub fn show_help(&self) {
        self.stack.set_visible_child_name("help");
    }

    pub fn show_info(&self, msg: &str) {
        self.info_bar.set_message_type(gtk::MessageType::Info);
        self.info_bar_label.set_text(msg);
        self.info_bar.set_visible(true);
        self.info_bar.set_revealed(true);
    }

    pub fn show_error(&self, msg: &str) {
        self.info_bar.set_message_type(gtk::MessageType::Error);
        self.info_bar_label.set_text(msg);
        self.info_bar.set_visible(true);
        self.info_bar.set_revealed(true);
    }

    pub fn show(&self) {
        self.main_window.show_all();
        self.update_title();
    }

    pub fn set_edited(&self, edited: bool) {
        self.edited.replace(edited);
    }

    pub fn set_path(&self, path: PathBuf) {
        self.path.replace(Some(path));
    }

    pub fn quit(&self) {
        if *self.edited.borrow()
            && self.settings.get_boolean("ask-save-on-exit")
            && self.ask_save_file()
        {
            self.save_file()
        }
    }

    fn ask_save_file(&self) -> bool {
        let dialog = gtk::MessageDialog::new(
            Some(&self.main_window),
            gtk::DialogFlags::MODAL,
            gtk::MessageType::Question,
            gtk::ButtonsType::YesNo,
            "Save file?",
        );
        if dialog.run() == ResponseType::Yes {
            dialog.hide();
            true
        } else {
            dialog.hide();
            false
        }
    }

    fn open_file(&self) {
        if self.open_dialog.run() == ResponseType::Accept {
            let path = self.open_dialog.get_filename().unwrap();
            match fs::read_to_string(&path) {
                Ok(content) => {
                    self.set_path(path);
                    self.set_input(&content);
                    self.update_title();
                }
                Err(e) => self.show_error(&e.to_string()),
            }
        }
    }

    fn save_file(&self) {
        let borrow = self.path.borrow();
        let cloned_path = borrow.clone();
        drop(borrow);
        match cloned_path {
            Some(ref path) => match fs::write(path, &self.get_content()) {
                Ok(_) => {
                    self.set_edited(false);
                    self.show_info("File saved");
                    self.update_title();
                }
                Err(e) => self.show_error(&e.to_string()),
            },
            None => {
                if self.save_dialog.run() == ResponseType::Accept {
                    let path = self.save_dialog.get_filename().unwrap();
                    match fs::write(&path, &self.get_content()) {
                        Ok(_) => {
                            self.set_edited(false);
                            self.set_path(path);
                            self.show_info("File saved");
                            self.update_title();
                        }
                        Err(e) => self.show_error(&e.to_string()),
                    }
                }
            }
        }
    }

    fn get_content(&self) -> String {
        let (iter_start, iter_end) = self.input_buffer.get_bounds();
        self.input_buffer
            .get_text(&iter_start, &iter_end, true)
            .map(|c| c.to_string())
            .unwrap_or_default()
    }

    fn update_title(&self) {
        let text = match self
            .path
            .borrow()
            .as_ref()
            .and_then(|p| p.to_str().map(|p| p.to_string()))
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
        match evalexpr::eval_with_context_mut(
            &self.get_content(),
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
