use std::{path::PathBuf, process::Command};

use astal_io::{prelude::VariableExt, Variable};
use gtk::{
    gdk::Monitor,
    gio::{self},
    glib::{self, clone::Downgrade, Closure, Object, SendWeakRef, Value},
    prelude::{GtkWindowExt, MonitorExt, ObjectExt, ToValue},
    subclass::prelude::ObjectSubclassIsExt,
};
use gtk4_layer_shell::{Edge, LayerShell};
use vte4::{ButtonExt, Cast, WidgetExt};

use crate::app::App;

mod imp;

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

        current
    }

    fn wallpaper_entries(&self) -> gio::ListStore {
        self.imp()
            .wallpaper_entries
            .borrow()
            .clone()
            .expect("Wallpaper entries not set")
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

glib::wrapper! {
    pub struct WallpaperEntryObject(ObjectSubclass<imp_wallpaper::WallpaperEntryObject>);
}

impl WallpaperEntryObject {
    pub fn new(path: PathBuf) -> Self {
        glib::Object::builder().property("path", path).build()
    }
}
