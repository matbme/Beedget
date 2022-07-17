use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gdk, glib, CompositeTemplate};
use glib::{ParamFlags, ParamSpec, ParamSpecPointer, ParamSpecObject};
use glib::types::Pointee;

use adw::subclass::prelude::*;

use uuid::Uuid;
use once_cell::sync::{Lazy, OnceCell};

use std::ptr::NonNull;

use crate::widgets::*;
use crate::models::*;
use beedget::app_data;

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/com/github/matbme/beedget/ui/transaction-dialog.ui")]
    pub struct TransactionDialog {
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
        pub amount_entry: TemplateChild<gtk::Entry>,

        pub edit_transaction: OnceCell<*const Transaction>,
        pub current_group: OnceCell<Group>
    }

    #[glib::object_subclass]
    impl ObjectSubclass for TransactionDialog {
        const NAME: &'static str = "TransactionDialog";
        type Type = super::TransactionDialog;
        type ParentType = adw::Window;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
            Self::Type::bind_template_callbacks(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for TransactionDialog {
        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![ParamSpecPointer::new(
                    "transaction",
                    "transaction",
                    "transaction",
                    ParamFlags::CONSTRUCT | ParamFlags::READWRITE
                ),
                ParamSpecObject::new(
                    "group",
                    "group",
                    "group",
                    Group::static_type(),
                    ParamFlags::CONSTRUCT | ParamFlags::READWRITE
                )]
            });

            PROPERTIES.as_ref()
        }

        fn set_property(&self, obj: &Self::Type, _id: usize, value: &glib::Value, pspec: &ParamSpec) {
            match pspec.name() {
                "transaction" => {
                    if let Ok(input) = value.get::<NonNull::<Pointee>>() {
                        let ptr_cast = input.cast::<Transaction>();
                        match self.edit_transaction.set(ptr_cast.as_ptr()) {
                            Ok(_) => obj.populate_transaction_values(),
                            Err(_) => panic!("Transaction pointer was already set!")
                        }
                    }
                }
                "group" => {
                    if let Ok(input) = value.get::<Group>() {
                        match self.current_group.set(input) {
                            Ok(_) => obj.set_current_group(),
                            Err(_) => panic!("Group pointer was already set!")
                        }
                    }
                }
                _ => unimplemented!()
            }
        }

        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            if self.edit_transaction.get().is_none() {
                self.expense_check_button.set_active(true);
            }

            obj.connect_key_event_controller();
            obj.connect_add_button_valid();

            if self.group_select.model().is_none() {
                obj.populate_group_select_dropdown();
            }
        }
    }
    impl WidgetImpl for TransactionDialog {}
    impl WindowImpl for TransactionDialog {}
    impl AdwWindowImpl for TransactionDialog {}
}

glib::wrapper! {
    pub struct TransactionDialog(ObjectSubclass<imp::TransactionDialog>)
        @extends gtk::Window, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget,
                    gtk::Native, gtk::Root, gtk::ShortcutManager;
}

#[gtk::template_callbacks]
impl TransactionDialog {
    pub fn new(parent: &gtk::Window) -> Self {
        glib::Object::new(&[
            ("transient-for", &Some(parent))
        ]).expect("Failed to create `TransactionDialog`.")
    }

    pub fn edit(parent: &gtk::Window, edit_transaction: *const Transaction, group: &Group) -> Self {
        let transaction_ptr = NonNull::new(edit_transaction as *mut Transaction)
            .expect("Invalid pointer to transaction");

        glib::Object::new(&[
            ("transient-for", &Some(parent)),
            ("transaction", &transaction_ptr.cast::<Pointee>().to_value()),
            ("group", &group)
        ]).expect("Failed to create `TransactionDialog`.")
    }

    #[template_callback]
    fn close_window(&self) {
        self.destroy();
    }

    #[template_callback]
    fn confirm_transaction(&self) {
        if self.imp().edit_transaction.get().is_some() {
            app_data!(|data| {
                let selected_group_idx = self.imp().group_select.selected();

                let selected_group = &data.group_model.get().unwrap()
                    .item(selected_group_idx).unwrap()
                    .downcast_ref::<GroupRow>().unwrap()
                    .property::<Group>("group");

                let selected_group_id = Uuid::parse_str(
                    &selected_group
                        .property::<glib::GString>("uid").to_string()
                );

                let current_group_id = Uuid::parse_str(
                    &self.imp().current_group
                        .get().unwrap()
                        .property::<glib::GString>("uid").to_string()
                );

                if selected_group_id != current_group_id {
                    // Edit transaction, change group
                    self.change_transaction_group();
                } else {
                    // Edit transaction, same group
                    self.edit_transaction();
                }
            });
        } else {
            // Create transaction
            self.create_transaction();
        }

        app_data!(|data| {
            let selected_group_idx = self.imp().group_select.selected();

            let selected_group = &data.group_model.get().unwrap()
                .item(selected_group_idx).unwrap()
                .downcast_ref::<GroupRow>().unwrap()
                .property::<Group>("group");

            data.save_group(selected_group);
        });

        self.destroy();
    }

    fn change_transaction_group(&self) {
        let transaction = unsafe {
            self.imp().edit_transaction.get().unwrap().clone().as_ref().unwrap()
        };

        let prev_group = self.imp().current_group.get().unwrap();
        prev_group.delete_transaction(transaction.id);

        app_data!(|data| {
            data.save_group(prev_group);
        });

        self.create_transaction();
    }

    fn edit_transaction(&self) {
        let transaction = unsafe {
            (self.imp().edit_transaction.get().unwrap().clone() as *mut Transaction).as_mut().unwrap()
        };

        transaction.set_name(&self.imp().transaction_name.text());
        transaction.change_tr_type(
            if self.imp().expense_check_button.is_active() {
                TransactionType::EXPENSE
            } else {
                TransactionType::INCOME
            }
        );
        transaction.set_amount(self.amount_entry_value().unwrap());
        transaction.set_date(
            glib::DateTime::from_iso8601(
                self.imp().dt_picker.property::<glib::GString>("selected-date").as_str(),
                None
            ).expect("Invalid date")
        );
    }

    fn create_transaction(&self) {
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
            glib::DateTime::from_iso8601(
                self.imp().dt_picker.property::<glib::GString>("selected-date").as_str(),
                None
            ).expect("Invalid date")
        );

        app_data!(|data| {
            let selected_group_idx = self.imp().group_select.selected();

            let selected_group = &data.group_model.get().unwrap()
                .item(selected_group_idx).unwrap()
                .downcast_ref::<GroupRow>().unwrap()
                .property::<Group>("group");

            selected_group.new_transaction(transaction);
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
        self.imp().group_select.set_factory(Some(&GroupRow::factory()));
        app_data!(|data| self.imp().group_select.set_model(data.group_model.get()));
        self.imp().group_select.set_expression(Some(&GroupRow::search_expression()));
    }

    /// Fill entries with transaction values for edit
    fn populate_transaction_values(&self) {
        assert!(self.imp().edit_transaction.get().is_some());

        let transaction = unsafe {
            self.imp().edit_transaction.get().unwrap().as_ref().unwrap()
        };

        self.imp().transaction_name.set_buffer(&gtk::EntryBuffer::new(
            Some(&transaction.name)
        ));

        self.imp().amount_entry.set_buffer(&gtk::EntryBuffer::new(
            Some(&format!("{:.2}", &transaction.amount))
        ));

        match transaction.tr_type {
            TransactionType::EXPENSE => self.imp().expense_check_button.set_active(true),
            TransactionType::INCOME => self.imp().income_check_button.set_active(true),
        }

        self.imp().dt_picker.set_property(
            "selected-date",
            transaction.date().format_iso8601().expect("Invalid date")
        );
    }

    /// Set group dropdown to specified group
    fn set_current_group(&self) {
        assert!(self.imp().current_group.get().is_some());

        if self.imp().group_select.model().is_none() {
            self.populate_group_select_dropdown();
        }

        let mut group_idx: u32 = 0;

        let current_group_row_id = Uuid::parse_str(
            &self.imp().current_group
                .get().unwrap()
                .property::<glib::GString>("uid").to_string()
        );

        for row in self.imp().group_select.model().unwrap().into_iter() {
            let row_group_id = Uuid::parse_str(
                &row
                    .downcast_ref::<GroupRow>().unwrap()
                    .imp()
                    .group
                    .borrow()
                    .property::<glib::GString>("uid").to_string()
            );

            if current_group_row_id == row_group_id {
                break;
            } else {
                group_idx += 1;
            }
        }

        self.imp().group_select.set_selected(group_idx);
    }
}
