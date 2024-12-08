use std::{path::PathBuf, process::Command};

use astal_io::{prelude::VariableExt, Variable};
use futures_util::StreamExt;
use gtk::{
    gdk::Monitor,
    gio::{self, prelude::*},
    glib::{self, clone::Downgrade, Closure, Object, SendWeakRef, Value},
    prelude::*,
    subclass::prelude::*,
    LevelBar,
};
use gtk4_layer_shell::{Edge, LayerShell};
use wallpaper::WallpaperEntryObject;
use weather::WeatherService;

use crate::{app::App, TOKIO_RUNTIME};

mod imp;
mod weather;

glib::wrapper! {
    pub struct Top(ObjectSubclass<imp::Top>)
        @extends adw::Window, gtk::Window, gtk::Widget,
        @implements gio::ActionGroup, gio::ActionMap, gtk::Accessible, gtk::Buildable,
                    gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager;
}

impl Top {
    pub fn new(app: &App, monitor: &Monitor) -> Self {
        let time = Variable::new(&mut "woag".to_value());

        time.pollfn(
            1000,
            &Closure::new(move |_| {
                glib::DateTime::now_local()
                    .and_then(|dt| dt.format("%H:%M:%S · %A %d/%m"))
                    .map(|s| Value::from(s))
                    .ok()
            }),
        )
        .expect("Expected to create polling function");

        let current: Self = Object::builder()
            .property("application", app)
            .property("main-text", "Hi!")
            .property("default-width", monitor.geometry().width())
            .build();

        current
            .bind_property("reveal", app, "top-reveal")
            .bidirectional()
            .build();

        let time_ref = SendWeakRef::from(Downgrade::downgrade(&time));

        time.bind_property("value", &current, "time")
            .transform_to(move |_, _: Value| {
                time_ref
                    .upgrade()
                    .and_then(|v| v.value().get::<String>().ok())
            })
            .bidirectional()
            .build();

        current.init_layer_shell();
        let anchors = [
            (Edge::Left, true),
            (Edge::Right, true),
            (Edge::Top, true),
            (Edge::Bottom, false),
        ];

        for (anchor, state) in anchors {
            current.set_anchor(anchor, state);
        }

        current.set_default_height(1);
        current.set_height_request(1);
        current.auto_exclusive_zone_enable();

        current.present();

        glib::spawn_future_local(glib::clone!(
            #[weak]
            current,
            async move {
                // every 1 hour
                let mut stream = glib::interval_stream(std::time::Duration::from_secs(1 * 60 * 60));
                // TODO: configurable weather location
                let weather = weather::WeatherService::new();

                current.update_weather(&weather).await;

                while let Some(_) = stream.next().await {
                    current.update_weather(&weather).await;
                }
            }
        ));

        current
    }

    async fn update_weather(&self, weather: &WeatherService) {
        let weather = TOKIO_RUNTIME.block_on(weather.get_weather());
        match weather {
            Ok(weather) => {
                let temp = weather.temperature_slider();
                // TODO: configurable icon path
                let desc = weather
                    .current_condition()
                    .temperature(weather::TemperatureUnit::Celsius);
                let icon = weather.weather_icon();
                let location = weather.nearest_area().location();

                let uv_index = weather.current_condition().uv_index();
                let uv_index_color = match uv_index {
                    0..=2 => "green",
                    3..=5 => "yellow",
                    6..=7 => "orange",
                    8..=10 => "red",
                    11..=12 => "purple",
                    _ => "black",
                };

                self.set_weather_temp(temp.value as f32);
                self.set_weather_desc(weather.current_condition().desc().trim_end().trim());
                self.set_weather_temp_min(temp.min as f32);
                self.set_weather_temp_max(temp.max as f32);
                self.set_weather_temp_desc(desc);
                self.set_location(location);
                self.set_feels_like(format!(
                    "Feels like {}°C",
                    weather.current_condition().feels_like()
                ));
                self.set_uv(format!(
                    r#"UV Index: <span color="{}">{}</span>"#,
                    uv_index_color, uv_index
                ));
                self.set_cloud_cover(format!(
                    "Cloud cover: {}%",
                    weather.current_condition().cloud_cover()
                ));
                self.set_humidity(format!(
                    "Humidity: {}%",
                    weather.current_condition().humidity()
                ));

                self.set_weather_icon(match icon {
                    Ok(icon) => icon,
                    Err(e) => {
                        eprintln!("Failed to get weather icon for code {e}");
                        "question-round-outlined-symbolic"
                    }
                });

                self.hourly_weather_entries().remove_all();

                for day in weather.weather() {
                    let temp = day.temperature_slider();
                    let desc = day.desc();
                    let icon = day.icon();
                    let day = day.date();

                    let icon = match icon {
                        Ok(icon) => icon,
                        Err(e) => {
                            eprintln!("Failed to get weather icon for code {e}");
                            "question-round-outlined-symbolic"
                        }
                    };

                    let container = gtk::Box::new(gtk::Orientation::Vertical, 8);
                    container.add_css_class("daily-weather-entry");

                    let formatted_day = day.format("%-d %b").to_string();
                    let day_label = gtk::Label::new(Some(&formatted_day));
                    container.append(&day_label);

                    let icon = gtk::Image::from_icon_name(icon);
                    icon.set_pixel_size(32);
                    container.append(&icon);

                    let description_label = gtk::Label::new(Some(&desc));
                    container.append(&description_label);

                    // TODO: celcius/fahrenheit config
                    let temperature_slider = gtk::LevelBar::new();
                    temperature_slider.set_value(temp.value as f64);
                    temperature_slider.set_min_value(temp.min as f64);
                    temperature_slider.set_max_value(temp.max as f64);
                    container.append(&temperature_slider);

                    let temperature_label =
                        gtk::Label::new(Some(format!("{}°C / {}°C", temp.min, temp.max).as_str()));
                    container.append(&temperature_label);

                    self.hourly_weather_entries().append(&container);
                }
            }
            Err(e) => eprintln!("Failed to get weather: {:?}", e),
        }
    }

    fn wallpaper_entries(&self) -> gio::ListStore {
        self.imp()
            .wallpaper_entries
            .borrow()
            .clone()
            .expect("Wallpaper entries not set")
    }

    fn hourly_weather_entries(&self) -> gio::ListStore {
        self.imp()
            .hourly_weather_entries
            .borrow()
            .clone()
            .expect("Hourly weather entries not set")
    }

    fn setup_hourly_weather_entries(&self) {
        let model = gio::ListStore::new::<gtk::Widget>();

        self.imp()
            .hourly_weather_entries
            .replace(Some(model.clone()));

        // TODO: proper glib object for hourly weather
        self.hourly_weather_entries()
            .connect_items_changed(glib::clone!(
                #[weak(rename_to = current)]
                self,
                move |entries, _position, _added, _removed| {
                    while let Some(child) = current.imp().hourly_weather.first_child() {
                        current.imp().hourly_weather.remove(&child);
                    }

                    for item in entries.iter::<gtk::Widget>().filter_map(|item| item.ok()) {
                        current.imp().hourly_weather.append(&item);
                    }
                }
            ));
    }

    fn setup_wallpaper_entries(&self) {
        let model = gio::ListStore::new::<WallpaperEntryObject>();

        self.imp().wallpaper_entries.replace(Some(model.clone()));

        // TODO: configurable path to wallpapers & file watch
        let wallpaper_dir = glib::home_dir().join("commafiles/wallpapers");
        let formats: Vec<_> = gtk::gdk_pixbuf::Pixbuf::formats()
            .iter()
            .filter_map(|f| f.name())
            .collect();
        let items = std::fs::read_dir(wallpaper_dir)
            .expect("Expected to read wallpapers directory")
            .filter_map(Result::ok)
            .filter_map(|entry| {
                if entry.file_type().map(|ft| ft.is_file()).unwrap_or_default()
                    && entry
                        .path()
                        .extension()
                        .map(|ext| formats.iter().any(|f| *ext == **f))
                        .unwrap_or_default()
                {
                    Some(WallpaperEntryObject::new(entry.path()))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        for item in items {
            self.wallpaper_entries().append(&item);
        }

        self.imp()
            .wallpaper_items
            .bind_model(Some(&self.wallpaper_entries()), move |item| {
                let entry = item.downcast_ref::<WallpaperEntryObject>().unwrap();
                let image_path = entry.path();

                println!("Creating wallpaper entry: {:?}", image_path);
                let image = gtk::Picture::for_filename(&image_path);
                image.add_css_class("wallpaper-image");
                // due to ultrawide base res is 2560x1080
                // scale down so it actually doesn't take up the whole screen
                let scaling_factor = 0.2;
                image.set_size_request(
                    (2560.0 * scaling_factor) as i32,
                    (1080.0 * scaling_factor) as i32,
                );
                image.set_content_fit(gtk::ContentFit::Cover);

                let button = gtk::Button::new();
                button.set_child(Some(&image));
                button.add_css_class("wallpaper-entry");
                button.connect_clicked(move |_button| {
                    // TODO: configurable wallpaper launch command
                    let command = format!("swww img -t wave --transition-angle 30 --transition-bezier 0.41,0.26,0.98,1 --transition-step 180 --transition-fps 60 --transition-duration 1.2 {}", image_path.display());
                    let (command, args) = command.split_once(' ').unwrap();
                    let args = args.split(' ');
                    if let Err(command) = Command::new(command)
                        .args(args)
                        .spawn() {
                        eprintln!("Failed to execute command: {:?}", command);
                    };
                });

                button.into()
            });
    }
}

mod wallpaper {
    use super::*;

    glib::wrapper! {
        pub struct WallpaperEntryObject(ObjectSubclass<imp_wallpaper::WallpaperEntryObject>);
    }

    impl WallpaperEntryObject {
        pub fn new(path: PathBuf) -> Self {
            glib::Object::builder().property("path", path).build()
        }
    }

    mod imp_wallpaper {
        use std::{cell::RefCell, path::PathBuf};

        use gtk::{
            glib::{self, Properties},
            prelude::*,
            subclass::prelude::*,
        };

        // Object holding the state
        #[derive(Properties, Default)]
        #[properties(wrapper_type = super::WallpaperEntryObject)]
        pub struct WallpaperEntryObject {
            #[property(get, set)]
            pub path: RefCell<PathBuf>,
        }

        // The central trait for subclassing a GObject
        #[glib::object_subclass]
        impl ObjectSubclass for WallpaperEntryObject {
            const NAME: &'static str = "WallpaperEntryObject";
            type Type = super::WallpaperEntryObject;
        }

        // Trait shared by all GObjects
        #[glib::derived_properties]
        impl ObjectImpl for WallpaperEntryObject {}
    }
}
