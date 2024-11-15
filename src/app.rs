use astal::subclass::prelude::*;
use gtk::{
    gio,
    glib::{self, Object},
    subclass::prelude::*,
};

glib::wrapper! {
    pub struct App(ObjectSubclass<imp::App>)
        @extends astal::Application, gtk::Application, gio::Application,
        @implements gio::ActionGroup, gio::ActionMap, astal_io::Application;
}

impl App {
    pub fn new() -> Self {
        Object::builder().build()
    }

    pub fn builder() -> AppBuilder {
        AppBuilder::new()
    }
}

pub struct AppBuilder {
    builder: glib::object::ObjectBuilder<'static, App>,
}

impl AppBuilder {
    fn new() -> Self {
        Self {
            builder: glib::object::Object::builder(),
        }
    }

    pub fn application_id(self, application_id: impl Into<glib::GString>) -> Self {
        Self {
            builder: self
                .builder
                .property("application-id", application_id.into()),
        }
    }

    pub fn instance_name(self, instance_name: impl Into<glib::GString>) -> Self {
        Self {
            builder: self.builder.property("instance-name", instance_name.into()),
        }
    }

    pub fn resource_base_path(self, resource_base_path: impl Into<glib::GString>) -> Self {
        Self {
            builder: self
                .builder
                .property("resource-base-path", resource_base_path.into()),
        }
    }

    #[must_use = "Building the object from the builder is usually expensive and is not expected to have side effects"]
    pub fn build(self) -> App {
        self.builder.build()
    }
}

mod imp {
    use astal_io::{functions::write_sock, subclass::prelude::AstalIOApplicationImpl};

    use super::*;

    #[derive(Default, Debug)]
    pub struct App;

    #[glib::object_subclass]
    impl ObjectSubclass for App {
        const NAME: &'static str = "App";
        type Type = super::App;
        type ParentType = astal::Application;
        type Interfaces = (astal_io::Application,);
    }

    impl AstalApplicationImpl for App {}
    impl AstalIOApplicationImpl for App {
        fn request(&self, msg: &str, conn: &gio::SocketConnection) -> Result<(), glib::Error> {
            write_sock(conn, msg, |res| {
                if let Err(err) = res {
                    eprintln!("Error: {}", err);
                }
            });
            Ok(())
        }
    }
    impl GtkApplicationImpl for App {}
    impl ApplicationImpl for App {}
    impl ObjectImpl for App {}
}
