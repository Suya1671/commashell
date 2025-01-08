use std::{path::PathBuf, process::Command};

use futures_util::StreamExt;
use gtk::{
    gdk::Monitor,
    gio::{self, prelude::*, Settings, SettingsBindFlags},
    glib::{self, Object},
    prelude::*,
    subclass::prelude::*,
};
use gtk4_layer_shell::{Edge, LayerShell};
use sysinfo::System;
use wallpaper::WallpaperEntryObject;

use crate::{app::App, config::APP_ID, TOKIO_RUNTIME};

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
        let current: Self = Object::builder()
            .property("application", app)
            .property("default-width", monitor.geometry().width())
            .build();

        current.set_monitor(monitor);

        current
            .bind_property("reveal", app, "top-reveal")
            .bidirectional()
            .build();

        let settings = Settings::new(APP_ID);
        settings
            .bind("wallpaper-folder", &current, "wallpaper-folder")
            .flags(SettingsBindFlags::GET | SettingsBindFlags::SET)
            .build();

        settings
            .bind("wallpaper-command", &current, "wallpaper-command")
            .flags(SettingsBindFlags::GET | SettingsBindFlags::SET)
            .build();

        settings
            .bind("location", &current, "location")
            .flags(SettingsBindFlags::GET | SettingsBindFlags::SET)
            .build();

        settings
            .bind("use-metric-units", &current, "use-metric-units")
            .flags(SettingsBindFlags::GET | SettingsBindFlags::SET)
            .build();

        current.imp().location_entry.set_text(&current.location());
        current
            .imp()
            .wallpaper_command_entry
            .set_text(&current.wallpaper_command());

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
        current.set_keyboard_mode(gtk4_layer_shell::KeyboardMode::OnDemand);
        current.auto_exclusive_zone_enable();

        current.present();

        // update system stats
        glib::spawn_future_local(glib::clone!(
            #[weak]
            current,
            async move {
                let mut sys = System::new_all();
                // every 5 seconds
                let mut stream = glib::interval_stream(std::time::Duration::from_secs(5));
                sys.refresh_all();
                std::thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);
                sys.refresh_cpu_usage();

                current.update_system_stats(&mut sys);

                while (stream.next().await).is_some() {
                    sys.refresh_all();
                    current.update_system_stats(&mut sys);
                }
            }
        ));

        // update weather
        glib::spawn_future_local(glib::clone!(
            #[weak]
            current,
            async move {
                // every 1 hour
                let mut stream = glib::interval_stream(std::time::Duration::from_secs(60 * 60));
                current.update_weather().await;

                while (stream.next().await).is_some() {
                    current.update_weather().await;
                }
            }
        ));

        // update time
        glib::spawn_future_local(glib::clone!(
            #[weak]
            current,
            async move {
                let mut stream = glib::interval_stream(std::time::Duration::from_secs(1));

                while (stream.next().await).is_some() {
                    if let Ok(time) =
                        glib::DateTime::now_local().and_then(|dt| dt.format("%H:%M:%S · %A %d/%m"))
                    {
                        current.set_time(time);
                    };
                }
            }
        ));

        current
    }

    fn update_system_stats(&self, sys: &mut System) {
        self.set_cpu_usage_value(sys.global_cpu_usage());
        self.set_cpu_usage(format!("{:.0}%", sys.global_cpu_usage()));

        let used_memory = sys.used_memory() as f64 / sys.total_memory() as f64;
        self.set_ram_usage_value(used_memory as f32);
        self.set_ram_usage(format!(
            " {:.0}GB / {:.0}GB",
            // bytes to gigabytes
            sys.used_memory() / 1_000_000_000,
            sys.total_memory() / 1_000_000_000
        ));
    }

    async fn update_weather(&self) {
        let weather =
            TOKIO_RUNTIME.block_on(self.imp().weather_service.get_weather(&self.location()));

        match weather {
            Ok(weather) => {
                let temp = weather.temperature_slider();
                let desc = weather
                    .current_condition()
                    .temperature(self.temperature_unit());
                let icon = weather.weather_icon();

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

                self.daily_weather_entries().remove_all();

                for day in weather.weather() {
                    let temp = day.temperature_slider(self.temperature_unit());
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

                    let description_label = gtk::Label::new(Some(desc));
                    container.append(&description_label);

                    let temperature_slider = gtk::LevelBar::new();
                    temperature_slider.set_value(temp.value);
                    temperature_slider.set_min_value(temp.min);
                    temperature_slider.set_max_value(temp.max);
                    container.append(&temperature_slider);

                    let temperature_label = gtk::Label::new(Some(&match self.temperature_unit() {
                        weather::TemperatureUnit::Celsius => {
                            format!("{}°C ({}°C / {}°C)", temp.value, temp.min, temp.max)
                        }
                        weather::TemperatureUnit::Fahrenheit => {
                            format!("{}°F ({}°F / {}°F)", temp.value, temp.min, temp.max)
                        }
                    }));
                    container.append(&temperature_label);

                    self.daily_weather_entries().append(&container);
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

    fn daily_weather_entries(&self) -> gio::ListStore {
        self.imp()
            .daily_weather_entries
            .borrow()
            .clone()
            .expect("Daily weather entries not set")
    }

    fn setup_daily_weather_entries(&self) {
        let model = gio::ListStore::new::<gtk::Widget>();

        self.imp()
            .daily_weather_entries
            .replace(Some(model.clone()));

        // TODO: proper glib object for daily weather
        self.daily_weather_entries()
            .connect_items_changed(glib::clone!(
                #[weak(rename_to = current)]
                self,
                move |entries, _position, _added, _removed| {
                    // the most cursed way to remove all children from a list store
                    while let Some(child) = current.imp().daily_weather.first_child() {
                        current.imp().daily_weather.remove(&child);
                    }

                    for item in entries.iter::<gtk::Widget>().filter_map(|item| item.ok()) {
                        current.imp().daily_weather.append(&item);
                    }
                }
            ));
    }

    fn setup_wallpaper_entries(&self) {
        let model = gio::ListStore::new::<WallpaperEntryObject>();

        self.imp().wallpaper_entries.replace(Some(model.clone()));

        self.connect_wallpaper_folder_notify(|current| {
            current.wallpaper_entries().remove_all();

            let wallpaper_dir = current.wallpaper_folder();
            let formats: Vec<_> = gtk::gdk_pixbuf::Pixbuf::formats()
                .iter()
                .filter_map(|f| f.name())
                .collect();

            if let Ok(dir) = std::fs::read_dir(wallpaper_dir) {
                let items = dir.filter_map(Result::ok).filter_map(|entry| {
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
                });

                for item in items {
                    current.wallpaper_entries().append(&item);
                }
            }
        });

        // reference counting shenanigans
        let obj = self.clone();
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
                button.connect_clicked(glib::clone!(
                    #[weak]
                    obj,
                    move |_button| {
                        let command = obj
                            .wallpaper_command()
                            .replace("{path}", &image_path.to_string_lossy());

                        let (command, args) = command.split_once(' ').unwrap();

                        let args = args.split(' ');
                        if let Err(command) = Command::new(command).args(args).spawn() {
                            eprintln!("Failed to execute command: {:?}", command);
                        };
                    }
                ));

                button.into()
            });

        self.notify_wallpaper_folder();
    }

    fn temperature_unit(&self) -> weather::TemperatureUnit {
        if self.use_metric_units() {
            weather::TemperatureUnit::Celsius
        } else {
            weather::TemperatureUnit::Fahrenheit
        }
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
