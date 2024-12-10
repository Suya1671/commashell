use std::cell::RefCell;

use adw::subclass::window::AdwWindowImpl;
use glib::subclass::InitializingObject;
use gtk::gdk::Rectangle;
use gtk::glib::Properties;
use gtk::subclass::prelude::*;
use gtk::{gio, prelude::*};
use gtk::{glib, CompositeTemplate};

use super::weather;

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

    #[template_child]
    pub location_entry: TemplateChild<gtk::Entry>,
    #[property(get, set)]
    pub location: RefCell<String>,

    #[property(get, set)]
    pub wallpaper_folder: RefCell<String>,
    #[template_child]
    pub wallpaper_dialog: TemplateChild<gtk::FileDialog>,

    #[template_child]
    pub weather_right_click: TemplateChild<gtk::Popover>,
    #[template_child]
    pub wallpaper_right_click: TemplateChild<gtk::Popover>,

    #[template_child]
    pub weather_button: TemplateChild<gtk::MenuButton>,
    #[template_child]
    pub wallpaper_button: TemplateChild<gtk::MenuButton>,

    #[template_child]
    pub weather_click: TemplateChild<gtk::GestureClick>,
    #[template_child]
    pub wallpaper_click: TemplateChild<gtk::GestureClick>,

    pub weather_service: weather::WeatherService,
}

#[gtk::template_callbacks]
impl Top {
    #[template_callback]
    pub fn set_location_folder(&self) {
        let obj = self.obj();

        let path = gio::File::for_path(&obj.wallpaper_folder());
        if path.path().is_some_and(|p| p.exists()) {
            self.wallpaper_dialog.set_initial_folder(Some(&path));
        }

        glib::spawn_future_local(glib::clone!(
            #[weak(rename_to = current)]
            self,
            #[weak]
            obj,
            async move {
                let window = obj.upcast_ref::<gtk::Window>();

                let res = current
                    .wallpaper_dialog
                    .select_folder_future(Some(window))
                    .await;

                match res {
                    Ok(folder) => {
                        if let Some(path) = folder.path() {
                            obj.set_wallpaper_folder(path.to_string_lossy().to_string());
                        }
                    }
                    Err(err) => {
                        eprintln!("Error selecting folder: {}", err);
                    }
                }
            }
        ));
    }

    #[template_callback]
    pub fn refresh_location_folder(&self) {
        let obj = self.obj();
        obj.notify_wallpaper_folder();
    }

    #[template_callback]
    pub fn show_weather_right_click(&self, _n_press: i32, x: f64, y: f64) {
        let x_rounded = x.round() as i32;
        let y_rounded = y.round() as i32;

        self.weather_right_click
            .set_pointing_to(Some(&Rectangle::new(x_rounded, y_rounded, 1, 1)));

        self.weather_right_click.popup();
    }

    #[template_callback]
    pub fn show_wallpaper_right_click(&self, _n_press: i32, x: f64, y: f64) {
        let x_rounded = x.round() as i32;
        let y_rounded = y.round() as i32;

        self.wallpaper_right_click
            .set_pointing_to(Some(&Rectangle::new(x_rounded, y_rounded, 1, 1)));
        self.wallpaper_right_click.popup();
    }

    #[template_callback]
    pub fn on_location_change(&self) {
        let obj = self.obj();
        obj.set_location(self.location_entry.text().to_string());
    }

    #[template_callback]
    pub fn refresh_location(&self) {
        let obj = self.obj();

        glib::spawn_future_local(glib::clone!(
            #[weak]
            obj,
            async move {
                obj.update_weather().await;
            }
        ));
    }
}

#[glib::object_subclass]
impl ObjectSubclass for Top {
    // `NAME` needs to match `class` attribute of template
    const NAME: &'static str = "Top";
    type Type = super::Top;
    type ParentType = adw::Window;

    fn class_init(klass: &mut Self::Class) {
        klass.bind_template();
        klass.bind_template_callbacks();
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

        self.weather_right_click
            .set_parent(&self.weather_button.get());
        self.wallpaper_right_click
            .set_parent(&self.wallpaper_button.get());
    }
}

impl AdwWindowImpl for Top {}
impl WindowImpl for Top {}
impl WidgetImpl for Top {}
