mod app;
mod shell;

use std::sync::LazyLock;

use app::App;
use astal::prelude::{ApplicationExt as AstalApplicationExt, WindowExt};
use astal_io::prelude::{ApplicationExt as AstalIOApplicationExt, *};
use config::{APP_ID, RESOURCES_BYTES, RESOURCES_PATH};
use gtk::glib::clone;
use gtk::prelude::{ApplicationExt, *};
use gtk::{gdk, gio, glib};
use shell::Top;
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
    let app = App::builder()
        .application_id(APP_ID)
        .resource_base_path(RESOURCES_PATH)
        .build();

    app.acquire_socket().expect("To acquire socket");

    gtk::init().expect("To initialize GTK");

    match app.register(gio::Cancellable::NONE) {
        Ok(_) => {}
        Err(err) => eprintln!("Registration error, {}", err),
    }

    app.connect_activate(clone!(
        #[strong]
        app,
        move |_| {
            for monitor in &app.monitors() {
                let window = Top::new(&app, monitor);
                init_icons(&<Top as RootExt>::display(&window));

                app.add_window(&window);
            }
        }
    ));

    // Run the application
    app.run()
}
