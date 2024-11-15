use gtk::{
    gdk::Monitor,
    gio,
    glib::{self, Object},
    prelude::{GtkWindowExt, MonitorExt},
};

use crate::app::App;

mod imp;

glib::wrapper! {
    pub struct Top(ObjectSubclass<imp::Top>)
        @extends astal::Window, gtk::Window, gtk::Widget,
        @implements gio::ActionGroup, gio::ActionMap, gtk::Accessible, gtk::Buildable,
                    gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager;
}

impl Top {
    pub fn new(app: &App, monitor: &Monitor) -> Self {
        let current: Self = Object::builder()
            .property("application", app)
            .property("gdkmonitor", monitor)
            .property("main-text", "Hi!")
            .property("default-width", monitor.geometry().width())
            .build();

        current.present();

        current
    }
}
