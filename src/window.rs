use glib::clone;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gio, glib, CompositeTemplate};

use adw::prelude::BinExt;
use adw::subclass::application_window::AdwApplicationWindowImpl;

use crate::models::*;
use crate::widgets::*;
use crate::dialogs::*;
use crate::application;

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

            // Wait for window to receive application and initialize sidebar with save data
            obj.connect_application_notify(clone!(@weak obj as parent => move |_| {
                parent.init_sidebar();
            }));
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

    #[template_callback]
    fn open_transaction_dialog(&self) {
        let dialog = TransactionDialog::new(self.upcast_ref());
        dialog.present();
    }

    fn open_group_dialog(&self) {
        let dialog = GroupDialog::new(self.upcast_ref());
        dialog.present();
    }

    fn setup_gactions(&self) {
        let open_group_dialog_action = gio::SimpleAction::new("open-group-dialog", None);
        open_group_dialog_action.connect_activate(clone!(@weak self as win => move |_, _| {
            win.open_group_dialog();
        }));
        self.add_action(&open_group_dialog_action);

        let open_transaction_dialog_action = gio::SimpleAction::new("open-transaction-dialog", None);
        open_transaction_dialog_action.connect_activate(clone!(@weak self as win => move |_, _| {
            win.open_transaction_dialog();
        }));
        self.add_action(&open_transaction_dialog_action);
    }

    /// Initialize sidebar with groups from application data
    fn init_sidebar(&self) {
        let application = application!(self @as crate::BeedgetApplication);
        let model = application.data().group_model();

        let filter = gtk::StringFilter::new(Some(Group::search_expression()));
        let filter_model = gtk::FilterListModel::new(Some(model), Some(&filter));
        let selection_model = gtk::SingleSelection::new(Some(&filter_model));

        selection_model.connect_selected_notify(clone!(@weak self as win => move |model| {
            win.set_content_page(model);
        }));

        self.imp().sidebar.set_model(Some(&selection_model));

        self.imp().sidebar.set_factory(Some(&Group::factory()));

        // Fill content with element selected by default
        self.set_content_page(&self.imp().sidebar.model().unwrap()
            .downcast_ref::<gtk::SingleSelection>().unwrap());
    }

    /// Creates content page for selected group
    fn set_content_page(&self, model: &gtk::SingleSelection) {
        if model.selected() != gtk::INVALID_LIST_POSITION {
            let filtered_model = model.model().unwrap();
            let selected_object = filtered_model
                .item(model.selected()).unwrap();
            let selected_group = selected_object
                .downcast_ref::<Group>().unwrap();

            let content_page = GroupContent::new(&selected_group);
            self.imp().content.set_child(Some(&content_page));
        }
    }
}
