use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gdk, glib, CompositeTemplate};

use adw::subclass::window::AdwWindowImpl;

use crate::widgets::*;
use crate::models::*;
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

        #[template_child]
        pub dt_picker: TemplateChild<DateTimePicker>,

        #[template_child]
        pub expense_check_button: TemplateChild<gtk::CheckButton>,

        #[template_child]
        pub income_check_button: TemplateChild<gtk::CheckButton>,

        #[template_child]
        pub amount_entry : TemplateChild<gtk::Entry>,
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

            self.expense_check_button.set_active(true);

            obj.connect_key_event_controller();
            obj.connect_add_button_valid();
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
        app_data!(|data| {
            let transaction = Transaction::new(
                &self.imp().transaction_name.text(),
                {
                    if self.imp().expense_check_button.is_active() {
                        TransactionType::EXPENSE
                    } else {
                        TransactionType::INCOME
                    }
                },
                self.amount_entry_value().unwrap(),
                self.imp().dt_picker.property("selected-date")
            );

            let selected_group_idx = self.imp().group_select.selected();
            let selected_group = &data.groups.borrow()[selected_group_idx as usize];

            selected_group.new_transaction(transaction);
            data.save_group(selected_group);

            self.destroy();
        });
    }

    /// Disables button if name and/or amount entries are empty
    fn connect_add_button_valid(&self) {
        // Set initial
        self.imp().add_button.set_sensitive(self.imp().transaction_name.text_length() > 0);

        // Subscribe to changes
        self.imp().transaction_name.buffer()
            .connect_length_notify(glib::clone!(@weak self as parent => move |_| {
                parent.imp().add_button.set_sensitive(
                    parent.imp().transaction_name.text_length() > 0 &&
                    parent.amount_entry_value().is_some()
                );
        }));

        self.imp().amount_entry.buffer()
            .connect_text_notify(glib::clone!(@weak self as parent => move |_| {
                parent.imp().add_button.set_sensitive(
                    parent.amount_entry_value().is_some() &&
                    parent.imp().transaction_name.text_length() > 0
                );
        }));
    }

    fn amount_entry_value(&self) -> Option<f32> {
        if let Ok(amount) = self.imp().amount_entry.text().as_str().parse::<f32>() {
            if amount > 0.0 {
                self.imp().amount_entry.remove_css_class("error");
                Some(amount)
            } else {
                self.imp().amount_entry.add_css_class("error");
                None
            }
        } else {
            self.imp().amount_entry.add_css_class("error");
            None
        }
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
        self.imp().group_select.set_factory(Some(&GroupListRowContent::factory()));
        app_data!(|data| self.imp().group_select.set_model(data.group_model.get()));
        self.imp().group_select.set_expression(Some(&GroupListRowContent::search_expression()));
    }
}
