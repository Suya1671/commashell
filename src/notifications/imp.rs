use std::cell::RefCell;

use adw::subclass::window::AdwWindowImpl;
use glib::subclass::InitializingObject;
use gtk::subclass::prelude::*;
use gtk::{gio, prelude::*};
use gtk::{glib, CompositeTemplate};

#[derive(CompositeTemplate, Default, Debug)]
#[template(resource = "/in/wobbl/commashell/ui/notifications.ui")]
pub struct Notifications {
    #[template_child]
    pub container: TemplateChild<gtk::ListBox>,
    pub notifications: RefCell<Option<gio::ListStore>>,
}

#[glib::object_subclass]
impl ObjectSubclass for Notifications {
    // `NAME` needs to match `class` attribute of template
    const NAME: &'static str = "Notifications";
    type Type = super::Notifications;
    type ParentType = adw::Window;

    fn class_init(klass: &mut Self::Class) {
        klass.bind_template();
    }

    fn instance_init(obj: &InitializingObject<Self>) {
        obj.init_template();
    }
}

// Trait shared by all GObjects
impl ObjectImpl for Notifications {
    fn constructed(&self) {
        // Call "constructed" on parent
        self.parent_constructed();

        // Setup
        let obj = self.obj();
        obj.setup_notification_widgets();
    }
}

impl AdwWindowImpl for Notifications {}
impl WindowImpl for Notifications {}
impl WidgetImpl for Notifications {}
