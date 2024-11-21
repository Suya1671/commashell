use astal_notifd::prelude::*;
use gtk::{
    gdk::Monitor,
    gio,
    glib::{self, Object},
    prelude::GtkWindowExt,
    subclass::prelude::*,
};
use gtk4_layer_shell::{Edge, LayerShell};
use vte4::{Cast, ListModelExtManual};

use crate::{app::App, notification::Notification};

mod imp;

glib::wrapper! {
    pub struct Notifications(ObjectSubclass<imp::Notifications>)
        @extends adw::Window, gtk::Window, gtk::Widget,
        @implements gio::ActionGroup, gio::ActionMap, gtk::Accessible, gtk::Buildable,
                    gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager;
}

impl Notifications {
    pub fn new(app: &App, monitor: &Monitor) -> Self {
        let current: Self = Object::builder().property("application", app).build();
        let notifd = astal_notifd::functions::default().expect("Expected to get default Notifd");

        notifd.connect_notified(glib::clone!(
            #[weak]
            current,
            move |notifd, notif, _replaced| {
                current
                    .notification_widgets()
                    .append(&notifd.notification(notif).unwrap());
            }
        ));

        notifd.connect_resolved(glib::clone!(
            #[weak]
            current,
            move |_notifd, notif, _reason| {
                let notifications = current.notification_widgets();
                let index = notifications
                    .iter::<astal_notifd::Notification>()
                    .position(|n| n.is_ok_and(|n| n.id() == notif))
                    .expect("Expected to get index");

                notifications.remove(index as u32);
                println!("Notification resolved: {}", notif);
            }
        ));

        current.init_layer_shell();
        let anchors = [
            (Edge::Left, false),
            (Edge::Right, true),
            (Edge::Top, true),
            (Edge::Bottom, false),
        ];

        for (anchor, state) in anchors {
            current.set_anchor(anchor, state);
        }

        current.auto_exclusive_zone_enable();

        current.present();

        current
    }

    fn notification_widgets(&self) -> gio::ListStore {
        self.imp()
            .notification_widgets
            .borrow()
            .clone()
            .expect("Expected to get current widgets")
    }

    fn setup_notification_widgets(&self) {
        let model = gio::ListStore::new::<astal_notifd::Notification>();

        self.imp().notification_widgets.replace(Some(model));

        self.imp()
            .container
            .bind_model(Some(&self.notification_widgets()), move |notification| {
                let notification = notification
                    .downcast_ref::<astal_notifd::Notification>()
                    .unwrap();

                let widget = Notification::new(notification.clone());

                widget.into()
            });
    }
}
