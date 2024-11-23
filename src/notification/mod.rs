use std::fs;

use adw::prelude::*;
use astal_notifd::prelude::*;
use gtk::{
    gio,
    glib::{self, Object},
    subclass::prelude::*,
};

mod imp;

glib::wrapper! {
    pub struct Notification(ObjectSubclass<imp::Notification>)
        @extends gtk::Box, gtk::Widget,
        @implements gio::ActionGroup, gio::ActionMap, gtk::Accessible, gtk::Buildable,
                    gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager;
}

impl Notification {
    pub fn new(notification: astal_notifd::Notification) -> Self {
        let current: Self = Object::builder()
            .property(
                "app-name",
                &notification.app_name().unwrap_or("Unknown app".into()),
            )
            .property(
                "summary",
                &notification.summary().unwrap_or("No summary".into()),
            )
            .property("time", &format_time(notification.time()))
            .property("body", &notification.body().unwrap_or("No body".into()))
            .build();

        current
            .imp()
            .notification
            .replace(Some(notification.clone()));

        current.add_css_class(match notification.urgency() {
            astal_notifd::Urgency::Low => "low",
            astal_notifd::Urgency::Normal => "normal",
            astal_notifd::Urgency::Critical => "critical",
            _ => "other",
        });

        current.add_controller(current.imp().event_controller.get());

        if let Some(icon) = notification
            .app_icon()
            .or_else(|| notification.desktop_entry())
            .filter(|icon| !icon.is_empty())
        {
            let icon = gtk::Image::from_icon_name(&icon);
            icon.set_icon_size(gtk::IconSize::Large);
            icon.add_css_class("app-icon");
            current.imp().icon_holder.set_child(Some(&icon));
        }

        let display: gtk::gdk::Display = WidgetExt::display(&current);

        if let Some(image) = notification.image() {
            if fs::exists(&image).unwrap_or_default() {
                let image = gtk::Picture::for_filename(image);
                image.add_css_class("image");
                image.set_can_shrink(true);
                image.set_content_fit(gtk::ContentFit::Contain);
                image.set_width_request(128);
                image.set_height_request(128);
                current.imp().image_holder.set_child(Some(&image));
            } else if gtk::IconTheme::for_display(&display).has_icon(&image) {
                let image = gtk::Image::from_icon_name(&image);
                image.add_css_class("icon-image");
                current.imp().image_holder.set_child(Some(&image));
            }
        }

        let actions = notification.actions();

        if actions.len() > 0 {
            let ui_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
            ui_box.add_css_class("actions");
            current.imp().actions_holder.set_child(Some(&ui_box));
            for action in actions {
                let button = gtk::Button::builder()
                    .hexpand(true)
                    .child(
                        &gtk::Label::builder()
                            .label(action.label())
                            .halign(gtk::Align::Center)
                            .hexpand(true)
                            .build(),
                    )
                    .build();

                button.connect_clicked(glib::clone!(
                    #[weak]
                    notification,
                    move |_button| {
                        notification.invoke(&action.id());
                    }
                ));

                ui_box.append(&button);
            }
        }

        current
    }
}

fn format_time(time: i64) -> glib::GString {
    glib::DateTime::from_unix_local(time)
        .unwrap()
        .format("%H:%M:%S")
        .unwrap()
}
