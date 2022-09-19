use once_cell::sync::{Lazy, OnceCell};
use std::path::{Path, PathBuf};

use adw::subclass::prelude::*;
use glib::{clone, subclass::Signal};
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gdk, gio, glib};

use crate::config::VERSION;
use crate::models::{Group, SaveData};
use crate::BeedgetWindow;

pub static CLOCK_FORMAT: Lazy<String> = Lazy::new(|| {
    let desktop_settings = gio::Settings::new("org.gnome.desktop.interface");

    desktop_settings.string("clock-format").to_string()
});

mod imp {
    use super::*;

    #[derive(Debug)]
    pub struct BeedgetApplication {
        pub settings: OnceCell<gio::Settings>,
        pub data: OnceCell<SaveData>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for BeedgetApplication {
        const NAME: &'static str = "BeedgetApplication";
        type Type = super::BeedgetApplication;
        type ParentType = adw::Application;
    }

    impl Default for BeedgetApplication {
        fn default() -> Self {
            Self {
                settings: OnceCell::with_value(gio::Settings::new("com.github.matbme.beedget")),
                data: OnceCell::new(),
            }
        }
    }

    impl ObjectImpl for BeedgetApplication {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            obj.load_data();
            obj.setup_gactions();
            obj.load_css();
            obj.set_accels_for_action("app.quit", &["<primary>q"]);
            obj.set_accels_for_action("win.start-group-search", &["<Ctrl>f"]);

            obj.connect_closure(
                "save-group",
                false,
                glib::closure_local!(move |application: Self::Type, group: &Group| {
                    if let Some(data) = application.imp().data.get() {
                        data.save_group(group);
                    }
                }),
            );
        }

        fn signals() -> &'static [glib::subclass::Signal] {
            static SIGNALS: Lazy<Vec<Signal>> = Lazy::new(|| {
                vec![Signal::builder(
                    "save-group",
                    &[Group::static_type().into()],
                    <()>::static_type().into(),
                )
                .build()]
            });
            SIGNALS.as_ref()
        }
    }

    impl ApplicationImpl for BeedgetApplication {
        fn activate(&self, application: &Self::Type) {
            // Get the current window or create one if necessary
            let window = if let Some(window) = application.active_window() {
                window
            } else {
                let window = BeedgetWindow::new(application);
                window.upcast()
            };

            // Ask the window manager/compositor to present the window
            window.present();
        }
    }

    impl GtkApplicationImpl for BeedgetApplication {}
    impl AdwApplicationImpl for BeedgetApplication {}
}

glib::wrapper! {
    pub struct BeedgetApplication(ObjectSubclass<imp::BeedgetApplication>)
        @extends gio::Application, gtk::Application, adw::Application,
        @implements gio::ActionGroup, gio::ActionMap;
}

impl BeedgetApplication {
    pub fn new(application_id: &str, flags: &gio::ApplicationFlags) -> Self {
        glib::Object::new(&[("application-id", &application_id), ("flags", flags)])
            .expect("Failed to create BeedgetApplication")
    }

    pub fn data(&self) -> &SaveData {
        self.imp().data.get().expect("Save data not loaded")
    }

    fn setup_gactions(&self) {
        let quit_action = gio::SimpleAction::new("quit", None);
        quit_action.connect_activate(clone!(@weak self as app => move |_, _| {
            app.quit();
        }));
        self.add_action(&quit_action);

        let about_action = gio::SimpleAction::new("about", None);
        about_action.connect_activate(clone!(@weak self as app => move |_, _| {
            app.show_about();
        }));
        self.add_action(&about_action);
    }

    fn show_about(&self) {
        let window = self.active_window().unwrap();
        let dialog = gtk::AboutDialog::builder()
            .transient_for(&window)
            .modal(true)
            .program_name("Beedget")
            .version(VERSION)
            .authors(vec!["Mateus Melchiades".into()])
            .build();

        dialog.present();
    }

    /// Load save data from disk and populate save-path setting if empty
    fn load_data(&self) {
        let save_path = self.imp().settings.get().unwrap().string("save-path");
        let data_buf = if save_path.is_empty() {
            let mut dbf = glib::user_data_dir();
            dbf.push(r"beedget");
            match self
                .imp()
                .settings
                .get()
                .unwrap()
                .set_string("save-path", dbf.as_path().to_str().unwrap())
            {
                Ok(()) => dbf,
                Err(error) => {
                    panic!("{}", error)
                }
            }
        } else {
            let mut dbf = PathBuf::new();
            dbf.push(Path::new(&save_path));
            dbf
        };

        self.imp()
            .data
            .set(SaveData::new(&data_buf))
            .expect("Failed to load save data.");
    }

    fn load_css(&self) {
        self.connect_startup(glib::clone!(@weak self as app => move |_| {
            let provider = gtk::CssProvider::new();
            provider.load_from_data(include_bytes!("../data/ui/style.css"));

            gtk::StyleContext::add_provider_for_display(
                &gdk::Display::default().expect("Failed to get default display"),
                &provider,
                gtk::STYLE_PROVIDER_PRIORITY_APPLICATION);
        }));
    }
}
