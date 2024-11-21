use std::cell::RefCell;

use adw::subclass::window::AdwWindowImpl;
use glib::subclass::InitializingObject;
use gtk::glib::Properties;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{glib, CompositeTemplate};

#[derive(CompositeTemplate, Properties, Default, Debug)]
#[template(resource = "/in/wobbl/commashell/ui/top.ui")]
#[properties(wrapper_type = super::Top)]
pub struct Top {
    #[property(get, set, type = String)]
    pub main_text: RefCell<String>,
    #[property(get, set, type = String)]
    pub time: RefCell<String>,
    #[property(get, set, type = bool)]
    pub reveal: RefCell<bool>,
}

#[glib::object_subclass]
impl ObjectSubclass for Top {
    // `NAME` needs to match `class` attribute of template
    const NAME: &'static str = "Top";
    type Type = super::Top;
    type ParentType = adw::Window;

    fn class_init(klass: &mut Self::Class) {
        klass.bind_template();
    }

    fn instance_init(obj: &InitializingObject<Self>) {
        obj.init_template();
    }
}

#[glib::derived_properties]
impl ObjectImpl for Top {}

impl AdwWindowImpl for Top {}
impl WindowImpl for Top {}
impl WidgetImpl for Top {}
