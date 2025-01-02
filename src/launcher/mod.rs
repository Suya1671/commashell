use gtk::{
    gdk::Monitor,
    gio,
    glib::{self, Object},
    prelude::{MonitorExt, ObjectExt},
    subclass::prelude::ObjectSubclassIsExt,
};
use gtk4_layer_shell::{Edge, LayerShell};
use vte4::{Cast, EditableExt, WidgetExt};

use crate::app::App;

mod imp;
mod launchers;

glib::wrapper! {
    pub struct Launcher(ObjectSubclass<imp::Launcher>)
        @extends adw::Window, gtk::Window, gtk::Widget,
        @implements gio::ActionGroup, gio::ActionMap, gtk::Accessible, gtk::Buildable,
                    gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager;
}

impl Launcher {
    pub fn new(app: &App, monitor: &Monitor) -> Self {
        let current: Self = Object::builder()
            .property("application", app)
            .property("default-width", monitor.geometry().width())
            .build();

        current.set_monitor(monitor);

        current.setup_app_entries();

        current
            .bind_property("reveal", app, "launcher-reveal")
            .bidirectional()
            .build();

        current.init_layer_shell();
        let anchors = [
            (Edge::Left, false),
            (Edge::Right, false),
            (Edge::Top, false),
            (Edge::Bottom, false),
        ];

        for (anchor, state) in anchors {
            current.set_anchor(anchor, state);
        }

        current.set_width_request(1);
        current.set_height_request(1);
        current.set_keyboard_mode(gtk4_layer_shell::KeyboardMode::OnDemand);
        current.connect_show(|launcher| {
            launcher.imp().entry.set_text("");
            launcher.imp().entry.grab_focus();
        });

        current
    }

    fn app_entries(&self) -> gio::ListStore {
        self.imp()
            .app_entries
            .borrow()
            .clone()
            .expect("App entries not set")
    }

    fn setup_app_entries(&self) {
        let model = gio::ListStore::new::<gtk::Widget>();

        self.imp().app_entries.replace(Some(model.clone()));

        self.imp()
            .list
            .bind_model(Some(&self.app_entries()), move |item| {
                <gtk::glib::Object as Clone>::clone(item)
                    .downcast::<gtk::Widget>()
                    .unwrap()
            });
    }
}
