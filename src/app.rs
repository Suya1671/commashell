use astal::subclass::prelude::*;
use astal_io::prelude::ApplicationExt as AstalIOApplicationExt;
use gtk::{
    gio,
    glib::{self, Object},
    prelude::*,
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

    pub fn disable_launcher(&self) {
        self.set_launcher_reveal(false);
        if let Err(e) = self.toggle_window("Launcher") {
            eprintln!("Launcher Error: {}", e);
        };
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
    use std::{cell::RefCell, sync::RwLock};

    use astal::prelude::ApplicationExt as AstalApplicationExt;
    use astal_io::{
        functions::write_sock, prelude::ApplicationExt as AstalIOApplicationExt,
        subclass::prelude::AstalIOApplicationImpl,
    };
    use glib::Properties;

    use crate::CSS_STYLE;

    use super::*;

    #[derive(Properties, Default, Debug)]
    #[properties(wrapper_type = super::App)]
    pub struct App {
        #[property(get, set)]
        pub top_reveal: RefCell<bool>,
        #[property(get, set)]
        pub right_reveal: RefCell<bool>,
        #[property(get, set)]
        pub launcher_reveal: RwLock<bool>,
    }

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
            let obj = self.obj();

            match msg {
                "launcher" => {
                    // change order depending on the current state
                    if obj.launcher_reveal() {
                        obj.disable_launcher();
                    } else {
                        if let Err(e) = obj.toggle_window("Launcher") {
                            eprintln!("Launcher Error: {}", e);
                        }
                        let window = obj.window("Launcher").unwrap();
                        window.set_width_request(500);
                        window.set_default_width(500);
                        obj.set_launcher_reveal(!obj.launcher_reveal());
                    }
                }
                "toggle top" => {
                    obj.set_top_reveal(!obj.top_reveal());
                }
                "toggle right" => {
                    obj.set_right_reveal(!obj.right_reveal());
                }
                "toggle all" => {
                    obj.set_top_reveal(!obj.top_reveal());
                    obj.set_right_reveal(!obj.right_reveal());
                }
                "on all" => {
                    obj.set_top_reveal(true);
                    obj.set_right_reveal(true);
                }
                "off all" => {
                    obj.set_top_reveal(false);
                    obj.set_right_reveal(false);
                }
                msg => {
                    write_sock(conn, msg, |res| {
                        if let Err(err) = res {
                            eprintln!("Error: {}", err);
                        }
                    });
                    return Ok(());
                }
            }

            // force refresh css
            self.obj().apply_css(CSS_STYLE, false);

            Ok(())
        }
    }
    impl GtkApplicationImpl for App {}
    impl ApplicationImpl for App {}

    #[glib::derived_properties]
    impl ObjectImpl for App {
        fn constructed(&self) {
            self.parent_constructed();
            self.obj().set_top_reveal(false);
            self.obj().set_right_reveal(false);
            self.obj().set_launcher_reveal(false);
        }
    }
}
