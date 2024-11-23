use std::cell::RefCell;
use std::cmp::min;
use std::process::Command;

use adw::subclass::window::AdwWindowImpl;
use astal_apps::prelude::{ApplicationExt, AppsExt};
use astal_apps::Apps;
use glib::subclass::InitializingObject;
use gtk::glib::property::PropertyGet;
use gtk::glib::Properties;
use gtk::subclass::prelude::*;
use gtk::{gdk, gio, prelude::*};
use gtk::{glib, CompositeTemplate};

use crate::app::App;
use crate::app_entry::AppEntry;

#[derive(CompositeTemplate, Properties, Default, Debug)]
#[template(resource = "/in/wobbl/commashell/ui/launcher.ui")]
#[properties(wrapper_type = super::Launcher)]
pub struct Launcher {
    #[property(get, set, type = bool)]
    pub reveal: RefCell<bool>,
    pub app_entries: RefCell<Option<gio::ListStore>>,
    #[property(get, set, nullable)]
    pub application: RefCell<Option<App>>,
    #[property(get)]
    pub apps: Apps,
    #[template_child]
    pub entry: TemplateChild<gtk::Entry>,
    #[template_child]
    pub list: TemplateChild<gtk::ListBox>,
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

// about the limit for what ListBoxes can handle
const MAX_ITEMS: usize = 200;

#[gtk::template_callbacks]
impl Launcher {
    #[template_callback]
    fn on_search_change(&self, entry: &gtk::Entry) {
        let model = self.obj().app_entries();
        model.remove_all();
        let text = entry.text();

        // calc
        if let Some(text) = text.strip_prefix("= ") {
            // run qalculate on the remaining text
            let result = Command::new("qalc")
                .arg(text)
                .output()
                .expect("failed to execute process");

            let output = String::from_utf8(result.stdout).unwrap();
            let output = output.trim_end();

            let application = self.obj().application().as_ref().unwrap().clone();

            let output = gtk::Button::with_label(&output);
            output.add_css_class("app-entry");
            output.set_valign(gtk::Align::Center);
            output
                .child()
                .unwrap()
                .downcast::<gtk::Label>()
                .unwrap()
                .set_wrap(true);

            output.connect_clicked(move |button| {
                application.disable_launcher();

                let display = gdk::Display::default().unwrap();
                let clipboard = display.clipboard();
                clipboard.set_text(&button.label().unwrap());
            });
            model.append(&output);
        } else {
            // fuzzy search
            let list = self.apps.fuzzy_query(Some(&text));
            let slice = &list[..min(MAX_ITEMS, list.len())];
            let application = self.obj().application().as_ref().unwrap().clone();
            for app in slice {
                let app = AppEntry::new(app, &application);
                model.append(&app);
            }
        }
    }

    #[template_callback]
    fn on_search_activate(&self, _entry: &gtk::Entry) {
        self.app_entries.borrow().as_ref().map(|model| {
            if let Some(row) = model.item(0) {
                if let Some(button) = row.downcast_ref::<gtk::Button>() {
                    button.activate();
                }
            }
        });
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
