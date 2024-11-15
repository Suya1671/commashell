mod shell;

use std::sync::LazyLock;

use adw::Application;
use config::{APP_ID, RESOURCES_BYTES, RESOURCES_PATH};
use gtk::prelude::*;
use gtk::{gdk, gio, glib};
use shell::Shell;

#[rustfmt::skip]
mod config;


pub static TOKIO_RUNTIME: LazyLock<tokio::runtime::Runtime> =
    LazyLock::new(|| tokio::runtime::Runtime::new().unwrap());

fn init_resources() {
    let gbytes = gtk::glib::Bytes::from_static(RESOURCES_BYTES);
    let resource = gtk::gio::Resource::from_data(&gbytes).unwrap();

    gtk::gio::resources_register(&resource);
}

fn init_icons<P: IsA<gdk::Display>>(display: &P) {
    let icon_theme = gtk::IconTheme::for_display(display);

    icon_theme.add_resource_path("/");
    icon_theme.add_resource_path(RESOURCES_PATH);
}

fn main() -> glib::ExitCode {
    init_resources();
    // Create a new application
    let app = Application::builder()
        .application_id(APP_ID)
        .resource_base_path(RESOURCES_PATH)
        .build();

    match app.register(gio::Cancellable::NONE) {
        Ok(_) => {}
        Err(err) => eprintln!("Registration error, {}", err),
    }

    build_ui(&app);

    // Run the application
    app.run()
}

fn build_ui(app: &Application) {
    let window = Shell::new(app);
    init_icons(&<Shell as RootExt>::display(&window));

    window.present();
    app.connect_activate(move |_| {
        window.present();
    });
}
