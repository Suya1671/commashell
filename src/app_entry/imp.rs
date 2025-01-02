use std::cell::RefCell;

use glib::subclass::InitializingObject;
use gtk::glib::Properties;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{glib, CompositeTemplate};

use crate::app::App;

#[derive(CompositeTemplate, Properties, Default)]
#[template(resource = "/in/wobbl/commashell/ui/app-entry.ui")]
#[properties(wrapper_type = super::AppEntry)]
pub struct AppEntry {
    #[property(get, set, nullable)]
    pub app: RefCell<Option<astal_apps::Application>>,
    #[property(get, set, nullable)]
    pub application: RefCell<Option<App>>,
}

#[glib::object_subclass]
impl ObjectSubclass for AppEntry {
    // `NAME` needs to match `class` attribute of template
    const NAME: &'static str = "AppEntry";
    type Type = super::AppEntry;
    type ParentType = gtk::Button;

    fn class_init(klass: &mut Self::Class) {
        klass.bind_template();
        klass.bind_template_callbacks();
    }

    fn instance_init(obj: &InitializingObject<Self>) {
        obj.init_template();
    }
}

#[gtk::template_callbacks]
impl AppEntry {
    #[template_callback]
    fn on_app_clicked(&self) {
        let obj = self.obj();
        if let Some(app) = obj.application().as_ref() {
            app.disable_launcher();
        }

        if let Some(ref app) = obj.app() {
            use astal_apps::prelude::ApplicationExt;
            app.launch();
        };
    }
}

#[glib::derived_properties]
impl ObjectImpl for AppEntry {}

impl ButtonImpl for AppEntry {}
impl WidgetImpl for AppEntry {}
