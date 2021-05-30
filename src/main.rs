use {
    error::{Error, MEEResult},
    gio::{prelude::*, Resource},
    glib::Bytes,
    gtk::{
        prelude::*, Application, CssProvider, CssProviderExt, StyleContext,
        STYLE_PROVIDER_PRIORITY_APPLICATION,
    },
    ui::Ui,
};

mod error;
mod macros;
mod ui;

const RESOURCE_BYTES: &'static [u8] = include_bytes!("../out/mathexpreval.gresource");

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
    let app = Application::new(Some("net.olback.MathExprEval"), Default::default())?;

    // Create ui
    let ui_ref = Ui::new().ok_or(Error::None)?;

    app.connect_activate(move |app| {
        ui_ref.set_app(app);
        ui_ref.show();
    });

    app.run(&std::env::args().collect::<Vec<String>>());

    Ok(())
}
