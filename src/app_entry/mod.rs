use gtk::{
    gio,
    glib::{self, Object},
};

use crate::app::App;

mod imp;

glib::wrapper! {
    pub struct AppEntry(ObjectSubclass<imp::AppEntry>)
        @extends gtk::Box, gtk::Widget,
        @implements gio::ActionGroup, gio::ActionMap, gtk::Accessible, gtk::Buildable,
                    gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager;
}

impl AppEntry {
    pub fn new(app: &astal_apps::Application, application: &App) -> Self {
        let current: Self = Object::builder()
            .property("app", app)
            .property("application", application)
            .build();
        current
    }
}
