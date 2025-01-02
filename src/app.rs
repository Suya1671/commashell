use astal::{prelude::ApplicationExt, subclass::prelude::*};
use astal_io::prelude::ApplicationExt as AstalIOApplicationExt;
use gtk::{
    gio,
    glib::{self, Object},
    prelude::*,
    subclass::prelude::*,
};
use tokio::sync::mpsc::{Receiver, Sender};

glib::wrapper! {
    pub struct App(ObjectSubclass<imp::App>)
        @extends astal::Application, gtk::Application, gio::Application,
        @implements gio::ActionGroup, gio::ActionMap, astal_io::Application;
}

pub enum Message {
    Toggle { component: Component },
    On { component: Component },
    Off { component: Component },
    HideLauncher,
}

pub enum Component {
    Top,
    Right,
    Launcher,
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
        let sender = self.imp().signal_sender.read().unwrap().clone().unwrap();
        glib::spawn_future_local(async move {
            glib::timeout_future(std::time::Duration::from_millis(200)).await;
            if let Err(e) = sender.send(Message::HideLauncher).await {
                eprintln!("Launcher Error: {}", e);
            }
        });
    }

    pub fn enable_launcher(&self) {
        let window = self.window("Launcher").unwrap();
        window.set_width_request(500);
        window.set_default_width(500);
        if let Err(e) = self.toggle_window("Launcher") {
            eprintln!("Launcher Error: {}", e);
        }
        self.set_launcher_reveal(true);
    }

    pub fn handle_message(&self, msg: Message) {
        match msg {
            Message::Toggle { component } => match component {
                Component::Top => self.set_top_reveal(!self.top_reveal()),
                Component::Right => self.set_right_reveal(!self.right_reveal()),
                Component::Launcher => {
                    if self.launcher_reveal() {
                        self.disable_launcher();
                    } else {
                        self.enable_launcher();
                    }
                }
            },
            Message::On { component } => match component {
                Component::Top => self.set_top_reveal(true),
                Component::Right => self.set_right_reveal(true),
                Component::Launcher => self.enable_launcher(),
            },
            Message::Off { component } => match component {
                Component::Top => self.set_top_reveal(false),
                Component::Right => self.set_right_reveal(false),
                Component::Launcher => self.disable_launcher(),
            },
            Message::HideLauncher => {
                self.set_launcher_reveal(false);
                self.toggle_window("Launcher").unwrap();
            }
        }
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
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
    pub fn build(
        self,
        mut signal_receiver: Receiver<Message>,
        signal_sender: Sender<Message>,
    ) -> App {
        let app = self.builder.build();
        app.imp()
            .signal_sender
            .write()
            .unwrap()
            .replace(signal_sender);

        // start reading and handling messages
        glib::spawn_future_local(glib::clone!(
            #[weak]
            app,
            async move {
                while let Some(msg) = signal_receiver.recv().await {
                    app.handle_message(msg);
                }
            }
        ));

        app
    }
}

mod imp {
    use std::sync::RwLock;

    use astal_io::{functions::write_sock, subclass::prelude::AstalIOApplicationImpl};
    use glib::Properties;

    use super::*;

    #[derive(Properties, Default, Debug)]
    #[properties(wrapper_type = super::App)]
    pub struct App {
        #[property(get, set)]
        pub top_reveal: RwLock<bool>,
        #[property(get, set)]
        pub right_reveal: RwLock<bool>,
        #[property(get, set)]
        pub launcher_reveal: RwLock<bool>,
        pub signal_sender: RwLock<Option<Sender<Message>>>,
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

            let messages = match msg {
                "launcher" => {
                    vec![Message::Toggle {
                        component: Component::Launcher,
                    }]
                }
                "toggle top" => {
                    vec![Message::Toggle {
                        component: Component::Top,
                    }]
                }
                "toggle right" => {
                    vec![Message::Toggle {
                        component: Component::Right,
                    }]
                }
                "toggle all" => {
                    vec![
                        Message::Toggle {
                            component: Component::Top,
                        },
                        Message::Toggle {
                            component: Component::Right,
                        },
                    ]
                }
                "on all" => {
                    vec![
                        Message::On {
                            component: Component::Top,
                        },
                        Message::On {
                            component: Component::Right,
                        },
                    ]
                }
                "off all" => {
                    vec![
                        Message::Off {
                            component: Component::Top,
                        },
                        Message::Off {
                            component: Component::Right,
                        },
                    ]
                }
                msg => {
                    write_sock(conn, msg, |res| {
                        if let Err(err) = res {
                            eprintln!("Error: {}", err);
                        }
                    });
                    return Ok(());
                }
            };

            for message in messages {
                obj.handle_message(message);
            }

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
