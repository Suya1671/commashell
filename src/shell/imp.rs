use std::cell::RefCell;

use astal::subclass::prelude::*;
use glib::subclass::InitializingObject;
use gtk::glib::Properties;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{glib, CompositeTemplate, Label, TemplateChild};

// Object holding the state
#[derive(CompositeTemplate, Properties, Default, Debug)]
#[template(resource = "/in/wobbl/commashell/ui/shell.ui")]
#[properties(wrapper_type = super::Shell)]
pub struct Shell {
    #[template_child]
    pub label: TemplateChild<Label>,
    #[property(get, set, type = String)]
    pub main_text: RefCell<String>,
}

// The central trait for subclassing a GObject
#[glib::object_subclass]
impl ObjectSubclass for Shell {
    // `NAME` needs to match `class` attribute of template
    const NAME: &'static str = "Shell";
    type Type = super::Shell;
    type ParentType = astal::Window;

    fn class_init(klass: &mut Self::Class) {
        klass.bind_template();
    }

    fn instance_init(obj: &InitializingObject<Self>) {
        obj.init_template();
    }
}

#[glib::derived_properties]
impl ObjectImpl for Shell {
    fn constructed(&self) {
        let obj = self.obj();
        self.parent_constructed();
        obj.set_property("main-text", "Hello, world!");
        obj.set_main_text("Hello, woag!");
    }
}

impl AstalWindowImpl for Shell {}
impl WindowImpl for Shell {}
impl WidgetImpl for Shell {}
