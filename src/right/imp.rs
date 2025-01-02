use std::cell::RefCell;

use adw::subclass::window::AdwWindowImpl;
use astal_mpris::prelude::PlayerExt;
use glib::subclass::InitializingObject;
use gtk::glib::{Properties, SpawnFlags};
use gtk::subclass::prelude::*;
use gtk::{gdk, gio, prelude::*};
use gtk::{glib, CompositeTemplate};
use vte4::{PtyFlags, TerminalExt, TerminalExtManual};

use crate::cava::Cava;

#[derive(CompositeTemplate, Properties, Default, Debug)]
#[template(resource = "/in/wobbl/commashell/ui/right.ui")]
#[properties(wrapper_type = super::Right)]
pub struct Right {
    #[template_child]
    pub default_text: TemplateChild<gtk::Label>,
    #[template_child(id = "player")]
    pub player_overlay: TemplateChild<gtk::Overlay>,
    #[template_child(id = "lyrics")]
    pub lyrics_overlay: TemplateChild<gtk::Overlay>,
    #[template_child]
    pub player_content: TemplateChild<gtk::Box>,
    #[template_child]
    pub lyrics_content: TemplateChild<gtk::Box>,
    #[template_child]
    pub lyrics_terminal: TemplateChild<vte4::Terminal>,
    #[template_child]
    pub seeker: TemplateChild<astal::Slider>,
    #[template_child]
    pub play: TemplateChild<gtk::Button>,
    // properties
    #[property(get, set)]
    pub reveal: RefCell<bool>,
    #[property(get, set)]
    length: RefCell<f64>,
    #[property(get, set)]
    position: RefCell<f64>,
    #[property(get, set)]
    playing: RefCell<bool>,
    #[property(get, set, nullable)]
    player: RefCell<Option<astal_mpris::Player>>,
}

#[gtk::template_callbacks]
impl Right {
    #[template_callback]
    fn on_play_clicked(&self, _button: &gtk::Button) {
        let obj = self.obj();
        if let Some(player) = obj.player().as_ref() {
            player.play_pause()
        }
    }

    #[template_callback]
    fn on_next_clicked(&self, _button: &gtk::Button) {
        let obj = self.obj();
        if let Some(player) = obj.player().as_ref() {
            player.next()
        }
    }

    #[template_callback]
    fn on_back_clicked(&self, _button: &gtk::Button) {
        let obj = self.obj();
        if let Some(player) = obj.player().as_ref() {
            player.previous()
        }
    }
}

#[glib::object_subclass]
impl ObjectSubclass for Right {
    // `NAME` needs to match `class` attribute of template
    const NAME: &'static str = "Right";
    type Type = super::Right;
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
impl ObjectImpl for Right {
    fn constructed(&self) {
        self.parent_constructed();

        self.player_overlay.add_overlay(&self.player_content.get());
        self.lyrics_overlay.add_overlay(&self.lyrics_content.get());

        let cava = Cava::new();
        self.lyrics_overlay.set_child(Some(&cava));

        let obj = self.obj();

        obj.connect_length_notify(glib::clone!(
            #[weak(rename_to = right)]
            self,
            move |obj| {
                if obj.length() > 0.0 {
                    right.seeker.set_range(0.0, obj.length());
                } else {
                    right.seeker.set_range(0.0, 1.0);
                }
            }
        ));

        obj.connect_playing_notify(glib::clone!(
            #[weak(rename_to = right)]
            self,
            move |obj| {
                right.play.set_icon_name(if obj.playing() {
                    "pause-large-symbolic"
                } else {
                    "play-large-symbolic"
                });
            }
        ));

        obj.connect_position_notify(glib::clone!(
            #[weak(rename_to = right)]
            self,
            move |obj| {
                right.seeker.set_value(obj.position());
            }
        ));

        self.seeker.connect_value_changed(glib::clone!(
            #[weak]
            obj,
            move |seeker| {
                obj.set_position(seeker.value());
            }
        ));

        self.lyrics_terminal
            .set_color_background(&gdk::RGBA::new(0.0, 0.0, 0.0, 0.0));

        self.lyrics_terminal.spawn_async(
            PtyFlags::DEFAULT,
            glib::home_dir().to_str(),
            &[
                "sptlrx",
                "--current",
                "bold",
                "--before",
                "#a6adc8,faint,italic",
                "--after",
                "104,faint",
            ],
            &[],
            SpawnFlags::SEARCH_PATH,
            || {},
            i32::MAX,
            None::<&gio::Cancellable>,
            |output| {
                if let Err(err) = output {
                    eprintln!("Error when setting up lyrics: {}", err);
                }
            },
        );
    }
}

impl AdwWindowImpl for Right {}
impl WindowImpl for Right {}
impl WidgetImpl for Right {}
