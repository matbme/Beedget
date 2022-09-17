use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gio, glib, CompositeTemplate};
use glib::{ParamFlags, ParamSpec, ParamSpecObject};

use adw::prelude::*;
use adw::subclass::prelude::*;

use once_cell::sync::{Lazy, OnceCell};

use std::cell::RefCell;

use crate::dialogs::*;
use crate::models::*;
use crate::widgets::*;

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/com/github/matbme/beedget/ui/transaction-row.ui")]
    pub struct TransactionRow {
        #[template_child]
        pub amount_label: TemplateChild<gtk::Label>,

        #[template_child]
        pub options_button: TemplateChild<gtk::MenuButton>,

        pub transaction: OnceCell<Transaction>,

        pub bindings: RefCell<Vec<glib::Binding>>
    }

    #[glib::object_subclass]
    impl ObjectSubclass for TransactionRow {
        const NAME: &'static str = "TransactionRow";
        type Type = super::TransactionRow;
        type ParentType = adw::ActionRow;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for TransactionRow {
        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![
                    ParamSpecObject::builder("transaction", Transaction::static_type())
                        .flags(ParamFlags::CONSTRUCT | ParamFlags::READWRITE)
                        .build()
                ]
            });

            PROPERTIES.as_ref()
        }

        fn set_property(&self, _obj: &Self::Type, _id: usize, value: &glib::Value, pspec: &ParamSpec) {
            match pspec.name() {
                "transaction" => {
                    if let Ok(input) = value.get::<Transaction>() {
                        self.transaction.set(input)
                            .expect("Transaction pointer was already set!");
                    }
                }
                _ => unimplemented!()
            }
        }

        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            let mut bindings = obj.imp().bindings.borrow_mut();
            let transaction = obj.imp().transaction.get()
                .expect("No transaction assigned to row");

            // Bind transaction amount to label
            let amount_binding = transaction
                .bind_property("amount", &obj.imp().amount_label.get(), "label")
                .transform_to(|_, value| {
                    if let Ok(amount) = value.get::<f32>() {
                        Some(format!("{:.2}", amount.abs()).to_value())
                    } else { None }
                })
                .flags(glib::BindingFlags::SYNC_CREATE)
                .build();
            bindings.push(amount_binding);

            // Listen for transaction type changes
            transaction.connect_notify_local(
                Some("tr-type"),
                glib::clone!(@weak obj as parent => move |transaction, _| {
                    parent.apply_css(transaction.tr_type());
                })
            );

            // Bind transaction name to title
            let name_binding = transaction
                .bind_property("name", obj, "title")
                .flags(glib::BindingFlags::SYNC_CREATE)
                .build();
            bindings.push(name_binding);

            // Listen for transaction date changes
            transaction.connect_notify_local(
                Some("date"),
                glib::clone!(@weak obj as parent => move |transaction, _| {
                    parent.set_subtitle(&transaction.relative_date());
                })
            );

            obj.apply_css(transaction.tr_type());
            obj.set_subtitle(&transaction.relative_date());

            obj.setup_gactions();
        }
    }

    impl WidgetImpl for TransactionRow {}
    impl ListBoxRowImpl for TransactionRow {}
    impl PreferencesRowImpl for TransactionRow {}
    impl ActionRowImpl for TransactionRow {}
}

glib::wrapper! {
    pub struct TransactionRow(ObjectSubclass<imp::TransactionRow>)
        @extends adw::ActionRow, adw::PreferencesRow, gtk::ListBoxRow, gtk::Widget,
        @implements gtk::Accessible, gtk::Actionable, gtk::Buildable, gtk::ConstraintTarget;
}

impl TransactionRow {
    pub fn new(transaction: &Transaction) -> Self {
        glib::Object::new(&[
            ("transaction", &transaction),
        ]).expect("Failed to create `TransactionRow`.")
    }

    pub fn transaction(&self) -> Option<&Transaction> {
        self.imp().transaction.get()
    }

    fn setup_gactions(&self) {
        let transaction_action_group = gio::SimpleActionGroup::new();

        let edit_action = gio::SimpleAction::new("edit", None);
        edit_action.connect_activate(glib::clone!(@weak self as parent => move |_, _| {
            let dialog = TransactionDialog::edit(
                parent.root().unwrap().downcast_ref::<gtk::Window>().unwrap(),
                parent.imp().transaction.get().unwrap(),
                parent
                    .parent().unwrap()
                    .parent().unwrap()
                    .downcast_ref::<GroupContent>().unwrap()
                    .imp().group.get().unwrap()
            );
            dialog.present();
        }));
        transaction_action_group.add_action(&edit_action);

        let delete_action = gio::SimpleAction::new("delete", None);
        delete_action.connect_activate(glib::clone!(@weak self as parent => move |_, _| {
            let group = parent.parent().unwrap();
            let group = group
                .parent().unwrap();
            let group = group
                .downcast_ref::<GroupContent>().unwrap()
                .imp().group.get().unwrap();

            let application = parent.root().unwrap().downcast_ref::<gtk::Window>().unwrap().application().unwrap();
            group.delete_transaction(parent.imp().transaction.get().unwrap().id());
            application.emit_by_name::<()>("save-group", &[&group]);
        }));
        transaction_action_group.add_action(&delete_action);

        self.insert_action_group("transaction", Some(&transaction_action_group));
    }

    fn apply_css(&self, tr_type: TransactionType) {
        if let Some(cls) = self.imp().amount_label.css_classes().last() {
            self.imp().amount_label.remove_css_class(cls);
        }

        match tr_type {
            TransactionType::EXPENSE => self.imp().amount_label.add_css_class("expense"),
            TransactionType::INCOME => self.imp().amount_label.add_css_class("income")
        }
    }
}
