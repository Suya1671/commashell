use adw::Application;
use gtk::{
    gio,
    glib::{self, Object},
};

mod imp;

glib::wrapper! {
    pub struct Shell(ObjectSubclass<imp::Shell>)
        @extends astal::Window, gtk::Window, gtk::Widget,
        @implements gio::ActionGroup, gio::ActionMap, gtk::Accessible, gtk::Buildable,
                    gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager;
}

impl Shell {
    pub fn new(app: &Application) -> Self {
        Object::builder().property("application", app).build()
    }
}
