use astal_apps::{prelude::AppsExt, Apps};
use gtk::{gdk, gio, prelude::*, UriLauncher};
use std::{cmp::min, process::Command};

use crate::app::App;

pub trait Launcher {
    fn can_launch(&self, term: &str) -> bool;
    fn launch(&self, term: &str) -> impl Iterator<Item = gtk::Widget>;
}

pub struct Qalculate {
    app: App,
}

impl Qalculate {
    pub fn new(app: App) -> Self {
        Self { app }
    }
}

impl Launcher for Qalculate {
    fn can_launch(&self, term: &str) -> bool {
        term.starts_with("= ")
    }

    fn launch(&self, term: &str) -> impl Iterator<Item = gtk::Widget> {
        let input = term.trim_start_matches("= ");

        let result = Command::new("qalc")
            .arg(input)
            .output()
            .expect("failed to execute process");

        let output = String::from_utf8(result.stdout).unwrap();
        let output = output.trim_end();

        let output = gtk::Button::with_label(&output);
        output.add_css_class("app-entry");
        output.set_valign(gtk::Align::Center);
        output
            .child()
            .unwrap()
            .downcast::<gtk::Label>()
            .unwrap()
            .set_wrap(true);

        // Reference clone
        let app = self.app.clone();
        output.connect_clicked(move |button| {
            app.disable_launcher();

            let display = gdk::Display::default().unwrap();
            let clipboard = display.clipboard();
            clipboard.set_text(&button.label().unwrap());
        });

        vec![output.into()].into_iter()
    }
}

pub struct FuzzyAppSearch {
    app: App,
    apps: Apps,
}

impl FuzzyAppSearch {
    pub fn new(app: App, apps: Apps) -> Self {
        Self { app, apps }
    }
}

impl Launcher for FuzzyAppSearch {
    fn can_launch(&self, _term: &str) -> bool {
        true
    }

    fn launch(&self, term: &str) -> impl Iterator<Item = gtk::Widget> {
        let list = self.apps.fuzzy_query(Some(&term));
        let slice = &list[..min(200, list.len())];

        slice
            .iter()
            .map(|app| {
                let app = crate::app_entry::AppEntry::new(app, &self.app);
                app.into()
            })
            .collect::<Vec<_>>()
            .into_iter()
    }
}

// use obsidian + thino to log thoughts! (pro version only. I might make this configurable to support other services)
pub struct Thino {
    app: App,
}

impl Thino {
    pub fn new(app: App) -> Self {
        Self { app }
    }
}

impl Launcher for Thino {
    fn can_launch(&self, term: &str) -> bool {
        term.starts_with("; ")
    }

    fn launch(&self, term: &str) -> impl Iterator<Item = gtk::Widget> {
        let text = term.trim_start_matches("; ");
        let uri = format!("obsidian://thino?content={}&type=daily&task=false", text);

        let output = gtk::Button::with_label("Send to Thino");
        output.add_css_class("app-entry");
        output.set_valign(gtk::Align::Center);
        output
            .child()
            .unwrap()
            .downcast::<gtk::Label>()
            .unwrap()
            .set_wrap(true);

        // Reference clone
        let app = self.app.clone();
        output.connect_clicked(move |_button| {
            app.disable_launcher();
            let uri_launcher = UriLauncher::new(&uri);
            uri_launcher.launch(None::<&gtk::Window>, None::<&gio::Cancellable>, |res| {
                if let Err(err) = res {
                    eprintln!("Error logging to thino: {}", err);
                }
            });
        });

        vec![output.into()].into_iter()
    }
}
