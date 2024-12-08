use std::cell::RefCell;

use adw::subclass::window::AdwWindowImpl;
use glib::subclass::InitializingObject;
use gtk::glib::Properties;
use gtk::subclass::prelude::*;
use gtk::{gio, prelude::*};
use gtk::{glib, CompositeTemplate};

#[derive(CompositeTemplate, Properties, Default, Debug)]
#[template(resource = "/in/wobbl/commashell/ui/top.ui")]
#[properties(wrapper_type = super::Top)]
pub struct Top {
    #[property(get, set)]
    pub main_text: RefCell<String>,
    #[property(get, set)]
    pub time: RefCell<String>,
    #[property(get, set)]
    pub reveal: RefCell<bool>,

    #[property(get, set)]
    pub weather_temp: RefCell<f32>,
    #[property(get, set)]
    pub weather_temp_min: RefCell<f32>,
    #[property(get, set)]
    pub weather_temp_max: RefCell<f32>,

    #[property(get, set)]
    pub weather_temp_desc: RefCell<String>,
    #[property(get, set)]
    pub weather_icon: RefCell<String>,

    #[property(get, set)]
    pub location: RefCell<String>,

    #[property(get, set)]
    pub weather_desc: RefCell<String>,
    #[property(get, set)]
    pub feels_like: RefCell<String>,
    #[property(get, set)]
    pub cloud_cover: RefCell<String>,
    #[property(get, set)]
    pub humidity: RefCell<String>,
    #[property(get, set)]
    pub uv: RefCell<String>,

    #[template_child]
    pub wallpaper_items: TemplateChild<gtk::ListBox>,
    #[template_child]
    pub hourly_weather: TemplateChild<gtk::Box>,

    pub wallpaper_entries: RefCell<Option<gio::ListStore>>,
    pub hourly_weather_entries: RefCell<Option<gio::ListStore>>,
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
impl ObjectImpl for Top {
    fn constructed(&self) {
        // Call "constructed" on parent
        self.parent_constructed();

        // Setup
        let obj = self.obj();
        obj.setup_wallpaper_entries();
        obj.setup_hourly_weather_entries();
    }
}

impl AdwWindowImpl for Top {}
impl WindowImpl for Top {}
impl WidgetImpl for Top {}
