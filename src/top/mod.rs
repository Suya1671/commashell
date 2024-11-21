use astal_io::{prelude::VariableExt, Variable};
use gtk::{
    gdk::Monitor,
    gio,
    glib::{self, clone::Downgrade, Closure, Object, SendWeakRef, Value},
    prelude::{GtkWindowExt, MonitorExt, ObjectExt, ToValue},
};
use gtk4_layer_shell::{Edge, LayerShell};
use vte4::WidgetExt;

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
}
