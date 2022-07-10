use glib::clone;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gio, glib, CompositeTemplate};

use adw::subclass::application_window::AdwApplicationWindowImpl;

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
        pub content: TemplateChild<gtk::Box>
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
        let dialog = CreateTransactionDialog::new(self.upcast_ref());
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

    /// Initialize sidebar with groups from application data
    fn init_sidebar(&self) {
        app_data!(|data| {
            data.init_group_model();

            let filter = gtk::StringFilter::new(Some(GroupListRowContent::search_expression()));
            let filter_model = gtk::FilterListModel::new(data.group_model.get(), Some(&filter));
            let selection_model = gtk::SingleSelection::new(Some(&filter_model));

            selection_model.connect_selected_notify(clone!(@weak self as win => move |model| {
                win.transaction_list(model.selected());
            }));

            self.imp().sidebar.set_model(Some(&selection_model));
        });

        self.imp().sidebar.set_factory(Some(&GroupListRowContent::factory()));
    }

    fn transaction_list(&self, selected: u32) {
        app_data!(|data| {
            let transaction_list = gtk::ListBox::new();
            transaction_list.bind_model(
                Some(data.groups.borrow()[selected as usize].transaction_model()),
                clone!(@weak self as win => @default-panic, move |item| {
                    let row = item.downcast_ref::<TransactionRow>().unwrap().clone();
                    row.upcast::<gtk::Widget>()
                })
            );

            self.imp().content.append(&transaction_list);
        });
    }
}
