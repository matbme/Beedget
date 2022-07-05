use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gdk, gio, glib, CompositeTemplate};

use adw::subclass::window::AdwWindowImpl;

use crate::widgets::*;
use beedget::app_data;

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/com/github/matbme/beedget/ui/create-transaction-dialog.ui")]
    pub struct CreateTransactionDialog {
        #[template_child]
        pub add_button: TemplateChild<gtk::Button>,

        #[template_child]
        pub cancel_button: TemplateChild<gtk::Button>,

        #[template_child]
        pub transaction_name: TemplateChild<gtk::Entry>,

        #[template_child]
        pub group_select: TemplateChild<gtk::DropDown>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for CreateTransactionDialog {
        const NAME: &'static str = "CreateTransactionDialog";
        type Type = super::CreateTransactionDialog;
        type ParentType = adw::Window;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
            Self::Type::bind_template_callbacks(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for CreateTransactionDialog {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            obj.connect_key_event_controller();
            obj.connect_add_button_to_entry_size();
            obj.populate_group_select_dropdown();
        }
    }
    impl WidgetImpl for CreateTransactionDialog {}
    impl WindowImpl for CreateTransactionDialog {}
    impl AdwWindowImpl for CreateTransactionDialog {}
}

glib::wrapper! {
    pub struct CreateTransactionDialog(ObjectSubclass<imp::CreateTransactionDialog>)
        @extends gtk::Window, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget,
                    gtk::Native, gtk::Root, gtk::ShortcutManager;
}

#[gtk::template_callbacks]
impl CreateTransactionDialog {
    pub fn new(parent: &gtk::Window) -> Self {
        glib::Object::new(&[
            ("transient-for", &Some(parent))
        ]).expect("Failed to create `CreateTransactionDialog`.")
    }

    #[template_callback]
    fn close_window(&self) {
        self.destroy();
    }

    #[template_callback]
    fn create_transaction(&self) {
    }

    /// Disables button if name entry is empty
    fn connect_add_button_to_entry_size(&self) {
        // Set initial
        self.imp().add_button.set_sensitive(self.imp().transaction_name.text_length() > 0);

        // Subscribe to changes
        self.imp().transaction_name.buffer()
            .connect_length_notify(glib::clone!(@weak self as parent => move |_| {
                parent.imp().add_button.set_sensitive(parent.imp().transaction_name.text_length() > 0);
        }));
    }

    /// Handle keyboard events
    fn connect_key_event_controller(&self) {
        let key_controller = gtk::EventControllerKey::new();
        key_controller.connect_key_pressed(glib::clone!(@strong self as parent => move |_, keyval, _, _| {
            match keyval {
                gdk::Key::Escape => { // Esc closes dialog
                    parent.destroy();
                    gtk::Inhibit(true)
                }
                _ => { gtk::Inhibit(false) }
            }
        }));

        self.add_controller(&key_controller);
    }

    fn populate_group_select_dropdown(&self) {
        let model = gio::ListStore::new(GroupListRowContent::static_type());
        app_data!(|data| {
            for group in data.groups.borrow().iter() {
                let row = GroupListRowContent::new(
                    &group.emoji,
                    &group.rgba_color(),
                    &group.name
                );
                model.append(&row);
            }
        });

        self.imp().group_select.set_factory(Some(&GroupListRowContent::factory()));
        self.imp().group_select.set_model(Some(&model));
        self.imp().group_select.set_expression(Some(&GroupListRowContent::search_expression()));
    }
}
