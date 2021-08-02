use {
    error::MEEResult,
    gio::{prelude::*, ApplicationFlags, Resource, SimpleAction},
    glib::{clone, Bytes},
    gtk::{
        prelude::*, Application, CssProvider, CssProviderExt, StyleContext,
        STYLE_PROVIDER_PRIORITY_APPLICATION,
    },
    std::fs,
    ui::Ui,
};

mod error;
mod macros;
mod ui;

const RESOURCE_BYTES: &[u8] = include_bytes!("../out/mathexpreval.gresource");

fn main() -> MEEResult<()> {
    // Load resources
    gio::resources_register(&Resource::from_data(&Bytes::from_static(RESOURCE_BYTES))?);

    gtk::init()?;

    // Load CSS
    let provider = CssProvider::new();
    provider.load_from_resource(resource!("css/app.css"));
    StyleContext::add_provider_for_screen(
        &gdk::Screen::get_default().expect("Error initializing gtk css provider."),
        &provider,
        STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    // Create app
    let app = Application::new(
        Some("net.olback.MathExprEval"),
        ApplicationFlags::HANDLES_OPEN | ApplicationFlags::NON_UNIQUE,
    )?;

    // Keyboard shortcuts
    // app.set_accels_for_action("result.copy", &["<CTRL>C"]);
    app.set_accels_for_action("file.open", &["<CTRL>O"]);
    app.set_accels_for_action("file.save", &["<CTRL>S"]);
    app.set_accels_for_action("app.math", &["<CTRL>M"]);
    app.set_accels_for_action("app.help", &["<CTRL>H"]);
    app.set_accels_for_action("app.quit", &["<CTRL>Q", "<CTRL>W"]);

    app.add_main_option(
        "eval",
        glib::Char::new('e').unwrap(),
        glib::OptionFlags::NONE,
        glib::OptionArg::Filename,
        "Run file",
        Some("FILE"),
    );

    // Load settings
    let settings = gio::Settings::new("net.olback.MathExprEval");

    // Create ui
    let ui_ref = Ui::new(&settings);

    // Handle args
    app.connect_handle_local_options(|_, dict| {
        if let Some(path) = dict
            .lookup_value("eval", None)
            .map(|v| v.get_data_as_bytes())
            .and_then(|b| {
                std::str::from_utf8(&(*b)[..b.len() - 1])
                    .ok()
                    .map(|s| s.trim().to_string())
            })
        {
            match fs::read_to_string(path) {
                Ok(content) => match evalexpr::eval_with_context_mut(
                    &content,
                    &mut evalexpr::math_consts_context!().unwrap(),
                ) {
                    Ok(result) => println!("{}", result),
                    Err(e) => eprintln!("{}", e),
                },
                Err(e) => eprintln!("{}", e),
            }
            0
        } else {
            -1
        }
    });

    // Handle when the app is run with a file
    app.connect_open(glib::clone!(@strong ui_ref => move |app, files, _| {
        if files.len() == 1 {
            const C: Option<&'static gio::Cancellable> = None;
            let mut buf = [0u8; 1024 * 1024];
            let content_bytes = files[0].read(C).and_then(|stream| stream.read_all(&mut buf, C)).map(|(len, _)| &buf[0..len]);
            match content_bytes {
                Ok(bytes) => match std::str::from_utf8(bytes) {
                    Ok(s) => {
                        ui_ref.set_input(s);
                        if let Some(path) = files[0].get_path() {
                            ui_ref.set_path(path);
                        }
                        ui_ref.set_app(app);
                        ui_ref.show();
                    },
                    Err(e) => eprint!("{}", e)
                },
                Err(e) => eprintln!("{}", e)
            }
        } else {
            eprintln!("Expected 1 file, got {}", files.len());
        }
    }));

    let app_ag = ui_ref.new_action_group("app");

    let to_math_action = SimpleAction::new("math", None);
    to_math_action.connect_activate(clone!(@strong ui_ref => move |_, _| {
        ui_ref.show_math();
    }));
    app_ag.add_action(&to_math_action);

    let to_help_action = SimpleAction::new("help", None);
    to_help_action.connect_activate(clone!(@strong ui_ref => move |_, _| {
        ui_ref.show_help();
    }));
    app_ag.add_action(&to_help_action);

    let quit_action = SimpleAction::new("quit", None);
    quit_action.connect_activate(clone!(@strong app => move |_, _| {
        app.quit();
    }));
    app_ag.add_action(&quit_action);

    app.connect_activate(clone!(@strong ui_ref => move |app| {
        ui_ref.set_app(app);
        ui_ref.show();
    }));

    app.connect_shutdown(move |_| {
        ui_ref.quit();
    });

    app.run(&std::env::args().collect::<Vec<String>>());

    Ok(())
}
