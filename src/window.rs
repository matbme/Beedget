use glib::clone;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gio, glib, CompositeTemplate};

use adw::prelude::BinExt;
use adw::subclass::application_window::AdwApplicationWindowImpl;

use uuid::Uuid;

use crate::widgets::*;
use crate::dialogs::*;

use beedget::app_data;

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/com/github/matbme/beedget/ui/window.ui")]
    pub struct BeedgetWindow {
        #[template_child]
        pub main_headerbar: TemplateChild<adw::HeaderBar>,

        #[template_child]
        pub pane: TemplateChild<adw::Leaflet>,

        #[template_child]
        pub search_bar: TemplateChild<gtk::SearchBar>,

        #[template_child]
        pub sidebar: TemplateChild<gtk::ListView>,

        #[template_child]
        pub content_pane: TemplateChild<gtk::Box>,

        #[template_child]
        pub content: TemplateChild<adw::Bin>
    }

    #[glib::object_subclass]
    impl ObjectSubclass for BeedgetWindow {
        const NAME: &'static str = "BeedgetWindow";
        type Type = super::BeedgetWindow;
        type ParentType = adw::ApplicationWindow;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
            Self::Type::bind_template_callbacks(klass);
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

#[gtk::template_callbacks]
impl BeedgetWindow {
    pub fn new<P: glib::IsA<gtk::Application>>(application: &P) -> Self {
        glib::Object::new(&[("application", application)])
            .expect("Failed to create BeedgetWindow")
    }

    #[template_callback]
    fn open_create_transaction_dialog(&self) {
        let dialog = TransactionDialog::new(self.upcast_ref());
        dialog.present();
    }

    #[template_callback]
    fn filter_group_list(&self, entry: &gtk::SearchEntry) {
        self.imp().sidebar
            .model().unwrap()
            .downcast_ref::<gtk::SingleSelection>().unwrap()
            .model().unwrap()
            .downcast_ref::<gtk::FilterListModel>().unwrap()
            .filter().unwrap()
            .downcast_ref::<gtk::StringFilter>().unwrap()
            .set_search(Some(&entry.text()));
    }

    fn setup_gactions(&self) {
        let open_create_group_dialog_action = gio::SimpleAction::new("open-create-group-dialog", None);
        open_create_group_dialog_action.connect_activate(clone!(@weak self as win => move |_, _| {
            win.show_create_group_dialog();
        }));
        self.add_action(&open_create_group_dialog_action);

        let open_create_transaction_dialog_action = gio::SimpleAction::new("open-create-transaction-dialog", None);
        open_create_transaction_dialog_action.connect_activate(clone!(@weak self as win => move |_, _| {
            win.open_create_transaction_dialog();
        }));
        self.add_action(&open_create_transaction_dialog_action);
    }

    fn show_create_group_dialog(&self) {
        let dialog = GroupDialog::new(self.upcast_ref());
        dialog.present();
    }

    /// Window only receives application after construction, so we wait to
    /// initialize content when we're sure it has been set.
    fn connect_init_sidebar(&self) {
        self.connect_application_notify(clone!(@weak self as win => move |_| {
            win.init_sidebar();
        }));
    }

    /// Initialize sidebar with groups from application data
    fn init_sidebar(&self) {
        app_data!(|data| {
            data.init_group_model();

            let filter = gtk::StringFilter::new(Some(GroupRow::search_expression()));
            let filter_model = gtk::FilterListModel::new(data.group_model.get(), Some(&filter));
            let selection_model = gtk::SingleSelection::new(Some(&filter_model));

            selection_model.connect_selected_notify(clone!(@weak self as win => move |model| {
                win.set_content_page(model);
            }));

            self.imp().sidebar.set_model(Some(&selection_model));
        });

        self.imp().sidebar.set_factory(Some(&GroupRow::factory()));

        // Fill content with element selected by default
        self.set_content_page(&self.imp().sidebar.model().unwrap()
            .downcast_ref::<gtk::SingleSelection>().unwrap());
    }

    /// Crates content page for selected group
    fn set_content_page(&self, model: &gtk::SingleSelection) {
        if model.selected() != gtk::INVALID_LIST_POSITION {
            let filtered_model = model.model().unwrap();
            let selected_object = filtered_model
                .item(model.selected()).unwrap();
            let selected_group = selected_object
                .downcast_ref::<GroupRow>().unwrap()
                .imp().group.borrow();

            // If we try to create a new GroupContent from the same group as
            // the currently constructed view, the application freezes
            if let Some(child) = self.imp().content.child() {
                let child_id = Uuid::parse_str(
                    &child
                        .downcast_ref::<GroupContent>().unwrap()
                        .imp()
                        .group
                        .get().unwrap()
                        .property::<glib::GString>("uid").to_string()
                );

                let new_id = Uuid::parse_str(
                    &selected_group
                        .property::<glib::GString>("uid").to_string()
                );

                if child_id != new_id {
                    let content_page = GroupContent::new(&selected_group);
                    self.imp().content.set_child(Some(&content_page));
                }
            } else {
                let content_page = GroupContent::new(&selected_group);
                self.imp().content.set_child(Some(&content_page));
            }
        }
    }
}
