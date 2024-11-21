mod app;
mod cava;
mod notification;
mod notifications;
mod right;
mod top;

use std::sync::LazyLock;

use app::App;
use astal::prelude::ApplicationExt as AstalApplicationExt;
use astal_io::prelude::ApplicationExt as AstalIOApplicationExt;
use config::{APP_ID, RESOURCES_BYTES, RESOURCES_PATH};
use gtk::glib::clone;
use gtk::prelude::*;
use gtk::{gio, glib};
use top::Top;
#[rustfmt::skip]
mod config;

pub static TOKIO_RUNTIME: LazyLock<tokio::runtime::Runtime> =
    LazyLock::new(|| tokio::runtime::Runtime::new().unwrap());

fn init_resources() {
    let gbytes = gtk::glib::Bytes::from_static(RESOURCES_BYTES);
    let resource = gtk::gio::Resource::from_data(&gbytes).unwrap();

    gtk::gio::resources_register(&resource);
}

const CSS_STYLE: &str = include_str!("../data/resources/style.css");

fn main() -> glib::ExitCode {
    init_resources();
    adw::init().expect("To initialize Adwaita");

    // Create a new application
    let app = App::builder()
        .application_id(APP_ID)
        .resource_base_path(RESOURCES_PATH)
        .build();

    app.acquire_socket().expect("To acquire socket");

    match app.register(gio::Cancellable::NONE) {
        Ok(_) => {}
        Err(err) => eprintln!("Registration error, {}", err),
    }

    app.connect_activate(clone!(
        #[strong]
        app,
        move |_| {
            for monitor in &app.monitors() {
                let top = Top::new(&app, monitor);
                app.add_window(&top);

                let right = right::Right::new(&app, monitor);
                app.add_window(&right);

                let notifications = notifications::Notifications::new(&app, monitor);
                app.add_window(&notifications);
            }

            app.apply_css(CSS_STYLE, false);
        }
    ));

    app.run()
}
