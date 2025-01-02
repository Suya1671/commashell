use std::cell::RefCell;

use astal_notifd::prelude::*;
use glib::subclass::InitializingObject;
use gtk::glib::Properties;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{glib, CompositeTemplate};

#[derive(CompositeTemplate, Properties, Default)]
#[template(resource = "/in/wobbl/commashell/ui/notification.ui")]
#[properties(wrapper_type = super::Notification)]
pub struct Notification {
    #[template_child]
    pub event_controller: TemplateChild<gtk::EventControllerMotion>,
    #[template_child]
    pub icon_holder: TemplateChild<adw::Bin>,
    #[template_child]
    pub image_holder: TemplateChild<adw::Bin>,
    #[template_child]
    pub actions_holder: TemplateChild<adw::Bin>,
    #[property(get, set)]
    pub app_name: RefCell<String>,
    #[property(get, set)]
    pub summary: RefCell<String>,
    #[property(get, set)]
    pub time: RefCell<String>,
    #[property(get, set)]
    pub body: RefCell<String>,
    #[property(get, set)]
    pub notification_id: RefCell<u32>,
    pub notification: RefCell<Option<astal_notifd::Notification>>,
}

#[glib::object_subclass]
impl ObjectSubclass for Notification {
    // `NAME` needs to match `class` attribute of template
    const NAME: &'static str = "Notification";
    type Type = super::Notification;
    type ParentType = gtk::Box;

    fn class_init(klass: &mut Self::Class) {
        klass.bind_template();
        klass.bind_template_callbacks();
    }

    fn instance_init(obj: &InitializingObject<Self>) {
        obj.init_template();
    }
}

#[gtk::template_callbacks]
impl Notification {
    #[template_callback]
    fn on_hover_leave(&self, _controller: &gtk::EventControllerMotion) {
        // self.notification.borrow().as_ref().map(|n| n.dismiss());
    }

    #[template_callback]
    fn on_dismiss(&self, _button: &gtk::Button) {
        if let Some(n) = self.notification.borrow().as_ref() {
            n.dismiss()
        }
    }
}

#[glib::derived_properties]
impl ObjectImpl for Notification {}

impl BoxImpl for Notification {}
impl WidgetImpl for Notification {}
