use std::cell::RefCell;

use adw::prelude::*;
use adw::subclass::prelude::AdwApplicationWindowImpl;
use glib::subclass::InitializingObject;
use gtk::glib::Properties;
use gtk::subclass::prelude::*;
use gtk::{glib, CompositeTemplate, Label, TemplateChild};

// Object holding the state
#[derive(CompositeTemplate, Properties, Default, Debug)]
#[template(resource = "/in/wobbl/commashell/ui/window.ui")]
#[properties(wrapper_type = super::Window)]
pub struct Window {
    #[template_child]
    pub label: TemplateChild<Label>,
    #[property(get, set, type = String)]
    pub main_text: RefCell<String>,
}

// The central trait for subclassing a GObject
#[glib::object_subclass]
impl ObjectSubclass for Window {
    // `NAME` needs to match `class` attribute of template
    const NAME: &'static str = "Window";
    type Type = super::Window;
    type ParentType = adw::ApplicationWindow;

    fn class_init(klass: &mut Self::Class) {
        klass.bind_template();
    }

    fn instance_init(obj: &InitializingObject<Self>) {
        obj.init_template();
    }
}

#[glib::derived_properties]
impl ObjectImpl for Window {
    fn constructed(&self) {
        let obj = self.obj();
        self.parent_constructed();
        obj.set_property("main-text", "Hello, world!");
        obj.set_main_text("Hello, woag!");
    }
}

impl AdwApplicationWindowImpl for Window {}
impl ApplicationWindowImpl for Window {}
impl WindowImpl for Window {}
impl WidgetImpl for Window {}
