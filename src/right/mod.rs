use astal_mpris::{
    prelude::{MprisExt, PlayerExt},
    Mpris, PlaybackStatus, Player,
};
use gtk::{
    gdk::Monitor,
    gio,
    glib::{self, Object},
    prelude::{GtkWindowExt, MonitorExt, ObjectExt},
};
use gtk4_layer_shell::{Edge, LayerShell};

use crate::app::App;

mod imp;

glib::wrapper! {
    pub struct Right(ObjectSubclass<imp::Right>)
        @extends adw::Window, gtk::Window, gtk::Widget,
        @implements gio::ActionGroup, gio::ActionMap, gtk::Accessible, gtk::Buildable,
                    gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager;
}

impl Right {
    pub fn new(app: &App, monitor: &Monitor) -> Self {
        let mpris = astal_mpris::functions::default().expect("Expected to get default Mpris");

        let current: Self = Object::builder()
            .property("application", app)
            .property("default-height", monitor.geometry().height())
            .build();

        current
            .bind_property("reveal", app, "right-reveal")
            .bidirectional()
            .build();

        connect_players(&mpris, &current);

        mpris.connect_players_notify(glib::clone!(
            #[weak]
            current,
            move |mpris| connect_players(mpris, &current)
        ));

        current.init_layer_shell();
        let anchors = [
            (Edge::Left, false),
            (Edge::Right, true),
            (Edge::Top, true),
            (Edge::Bottom, true),
        ];

        for (anchor, state) in anchors {
            current.set_anchor(anchor, state);
        }

        current.set_default_width(1);
        current.auto_exclusive_zone_enable();

        current.present();

        current
    }
}

fn connect_players(mpris: &Mpris, current: &Right) {
    let Some(player) = mpris
        .players()
        .iter()
        .max_by_key(|player| {
            let i = player.identity();
            if i.as_ref().is_some_and(|i| i.contains("Feishin")) {
                return 200;
            } else if i.as_ref().is_some_and(|i| i.contains("strawberry")) {
                return 100;
            } else {
                return 0;
            }
        })
        .cloned()
    else {
        current.set_player(None::<Player>);
        return;
    };

    if current.player().is_some_and(|p| p == player) {
        return;
    }

    current.set_player(Some(player.clone()));

    if let Some(cover_art) = player.cover_art() {
        current.set_cover_art(cover_art);
    }

    current.set_length(player.length());

    // setup template binds to new player
    player.connect_cover_art_notify(glib::clone!(
        #[weak]
        current,
        move |player| {
            if let Some(cover_art) = player.cover_art() {
                current.set_cover_art(cover_art);
            }
        }
    ));

    player.connect_length_notify(glib::clone!(
        #[weak]
        current,
        move |player| {
            current.set_length(player.length());
        }
    ));

    player
        .bind_property("position", current, "position")
        .bidirectional()
        .sync_create()
        .build();

    player
        .bind_property("title", current, "title")
        .sync_create()
        .build();

    player
        .bind_property("artist", current, "artist")
        .sync_create()
        .build();

    player.connect_playback_status_notify(glib::clone!(
        #[weak]
        current,
        move |player| {
            current.set_playing(player.playback_status() == PlaybackStatus::Playing);
        }
    ));
}
