mod application;
mod config;
mod util;
mod window;

mod dialogs;
mod models;
mod widgets;

use self::application::BeedgetApplication;
use self::window::BeedgetWindow;

use config::{GETTEXT_PACKAGE, LOCALEDIR, PKGDATADIR};
use gettextrs::{bind_textdomain_codeset, bindtextdomain, textdomain};
use gtk::gio;
use gtk::prelude::*;

fn main() {
    // Set up gettext translations
    bindtextdomain(GETTEXT_PACKAGE, LOCALEDIR).expect("Unable to bind the text domain");
    bind_textdomain_codeset(GETTEXT_PACKAGE, "UTF-8")
        .expect("Unable to set the text domain encoding");
    textdomain(GETTEXT_PACKAGE).expect("Unable to switch to the text domain");

    // Load resources
    let resources = gio::Resource::load(PKGDATADIR.to_owned() + "/beedget.gresource")
        .expect("Could not load resources");
    gio::resources_register(&resources);

    // Run application
    let app = BeedgetApplication::new("com.github.matbme.beedget", &gio::ApplicationFlags::empty());
    std::process::exit(app.run());
}
