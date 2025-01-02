use std::cell::RefCell;

use adw::subclass::window::AdwWindowImpl;
use astal_apps::Apps;
use glib::subclass::InitializingObject;
use gtk::glib::Properties;
use gtk::subclass::prelude::*;
use gtk::{gio, prelude::*};
use gtk::{glib, CompositeTemplate};

use crate::app::App;

use super::launchers::{FuzzyAppSearch, Launcher as LauncherTrait, Qalculate, Thino};

#[derive(CompositeTemplate, Properties, Default, Debug)]
#[template(resource = "/in/wobbl/commashell/ui/launcher.ui")]
#[properties(wrapper_type = super::Launcher)]
pub struct Launcher {
    #[property(get, set, type = bool)]
    pub reveal: RefCell<bool>,
    #[property(get, set, nullable)]
    pub application: RefCell<Option<App>>,
    #[property(get)]
    pub apps: Apps,
    #[template_child]
    pub entry: TemplateChild<gtk::Entry>,
    #[template_child]
    pub list: TemplateChild<gtk::ListBox>,
    pub app_entries: RefCell<Option<gio::ListStore>>,
}

#[glib::object_subclass]
impl ObjectSubclass for Launcher {
    // `NAME` needs to match `class` attribute of template
    const NAME: &'static str = "Launcher";
    type Type = super::Launcher;
    type ParentType = adw::Window;

    fn class_init(klass: &mut Self::Class) {
        klass.bind_template();
        klass.bind_template_callbacks();
    }

    fn instance_init(obj: &InitializingObject<Self>) {
        obj.init_template();
    }
}

#[gtk::template_callbacks]
impl Launcher {
    #[template_callback]
    fn on_search_change(&self, entry: &gtk::Entry) {
        let model = self.obj().app_entries();
        model.remove_all();

        let text = entry.text();
        let application = self.obj().application().as_ref().unwrap().clone();

        // TODO: configurable set of launchers
        fn launch_if_can<L: LauncherTrait>(
            model: &gio::ListStore,
            launcher: L,
            text: &str,
        ) -> bool {
            if launcher.can_launch(text) {
                let widgets = launcher.launch(text);
                for widget in widgets {
                    model.append(&widget);
                }
                true
            } else {
                false
            }
        }

        let calc = Qalculate::new(application.clone());
        if launch_if_can(&model, calc, &text) {
            return;
        }

        let thino = Thino::new(application.clone());
        if launch_if_can(&model, thino, &text) {
            return;
        }

        let fuzzy_search = FuzzyAppSearch::new(application.clone(), self.obj().apps());
        if launch_if_can(&model, fuzzy_search, &text) {}
    }

    #[template_callback]
    fn on_search_activate(&self, _entry: &gtk::Entry) {
        if let Some(row) = self
            .app_entries
            .borrow()
            .as_ref()
            .and_then(|model| model.item(0))
        {
            // to save a clone, this isn't apart of the and_then stack
            if let Some(button) = row.downcast_ref::<gtk::Button>() {
                button.activate();
            }
        };
    }
}

#[glib::derived_properties]
impl ObjectImpl for Launcher {
    fn constructed(&self) {
        // Call "constructed" on parent
        self.parent_constructed();
    }
}

impl AdwWindowImpl for Launcher {}
impl WindowImpl for Launcher {}
impl WidgetImpl for Launcher {}
