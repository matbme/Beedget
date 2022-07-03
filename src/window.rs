use glib::clone;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gio, glib, CompositeTemplate};

use adw::subclass::application_window::AdwApplicationWindowImpl;

use crate::widgets::*;
use crate::dialogs::*;
use crate::models::*;

use beedget::app_data;

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/com/github/matbme/beedget/ui/window.ui")]
    pub struct BeedgetWindow {
        #[template_child]
        pub header_bar: TemplateChild<adw::HeaderBar>,

        #[template_child]
        pub pane: TemplateChild<adw::Leaflet>,

        #[template_child]
        pub sidebar: TemplateChild<gtk::ListBox>,

        #[template_child]
        pub content: TemplateChild<gtk::Box>
    }

    #[glib::object_subclass]
    impl ObjectSubclass for BeedgetWindow {
        const NAME: &'static str = "BeedgetWindow";
        type Type = super::BeedgetWindow;
        type ParentType = adw::ApplicationWindow;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for BeedgetWindow {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
            obj.setup_gactions();

            obj.connect_init_sidebar();
        }
    }

    impl WidgetImpl for BeedgetWindow {}
    impl WindowImpl for BeedgetWindow {}
    impl ApplicationWindowImpl for BeedgetWindow {}
    impl AdwApplicationWindowImpl for BeedgetWindow {}
}

glib::wrapper! {
    pub struct BeedgetWindow(ObjectSubclass<imp::BeedgetWindow>)
        @extends gtk::Widget, gtk::Window, gtk::ApplicationWindow,
        @implements gio::ActionGroup, gio::ActionMap, gtk::Accessible,
                    gtk::Buildable, gtk::ConstraintTarget, gtk::Native,
                    gtk::Root, gtk::ShortcutManager;
}

impl BeedgetWindow {
    pub fn new<P: glib::IsA<gtk::Application>>(application: &P) -> Self {
        glib::Object::new(&[("application", application)])
            .expect("Failed to create BeedgetWindow")
    }

    fn setup_gactions(&self) {
        let open_create_group_dialog_action = gio::SimpleAction::new("open-create-group-dialog", None);
        open_create_group_dialog_action.connect_activate(clone!(@weak self as win => move |_, _| {
            win.show_create_group_dialog();
        }));
        self.add_action(&open_create_group_dialog_action);
    }

    fn show_create_group_dialog(&self) {
        let dialog = CreateGroupDialog::new(self.upcast_ref());
        dialog.present();
    }

    /// Window only receives application after construction, so we wait to
    /// initialize content when we're sure it has been set.
    fn connect_init_sidebar(&self) {
        self.connect_application_notify(clone!(@weak self as win => move |_| {
            win.init_sidebar();
        }));
    }

    fn init_sidebar(&self) {
        app_data!(|data| {
            for group in data.groups.borrow().iter() {
                let row = GroupListRowContent::new(
                    &group.emoji,
                    &group.rgba_color(),
                    &group.name
                );
                self.imp().sidebar.append(&row);
            }
        });

        let empty_dialog = EmptyDialog::new();
        self.imp().content.append(&empty_dialog);

        // Callbacks for updating sidebar when a group changes
        app_data!(|data| {
            data.add_group_update_callback(clone!(@weak self as parent => move |group, update_type| {
                match update_type {
                    UpdateType::Added => {
                        let row = GroupListRowContent::new(
                            &group.emoji,
                            &group.rgba_color(),
                            &group.name
                        );
                        parent.imp().sidebar.append(&row);
                    }
                    UpdateType::Removed => {

                    }
                    UpdateType::Changed => {

                    }
                }
            }));
        });
    }
}
