use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gio, glib, CompositeTemplate};
use glib::{ParamFlags, ParamSpec, ParamSpecObject};

use adw::prelude::*;
use adw::subclass::prelude::*;

use once_cell::sync::{Lazy, OnceCell};

use crate::dialogs::*;
use crate::models::*;
use crate::widgets::*;

use beedget::app_data;

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/com/github/matbme/beedget/ui/transaction-row.ui")]
    pub struct TransactionRow {
        #[template_child]
        pub amount_label: TemplateChild<gtk::Label>,

        #[template_child]
        pub options_button: TemplateChild<gtk::MenuButton>,

        pub transaction: OnceCell<Transaction>
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
                vec![ParamSpecObject::new(
                    "transaction",
                    "transaction",
                    "transaction",
                    Transaction::static_type(),
                    ParamFlags::CONSTRUCT | ParamFlags::READWRITE
                )]
            });

            PROPERTIES.as_ref()
        }

        fn set_property(&self, obj: &Self::Type, _id: usize, value: &glib::Value, pspec: &ParamSpec) {
            match pspec.name() {
                "transaction" => {
                    if let Ok(input) = value.get::<Transaction>() {
                        match self.transaction.set(input) {
                            Ok(_) => obj.init_row(),
                            Err(_) => panic!("Transaction pointer was already set!")
                        }
                    }
                }
                _ => unimplemented!()
            }
        }

        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

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

    fn setup_gactions(&self) {
        let transaction_action_group = gio::SimpleActionGroup::new();

        let edit_action = gio::SimpleAction::new("edit", None);
        edit_action.connect_activate(glib::clone!(@weak self as parent => move |_, _| {
            let dialog = TransactionDialog::edit(
                parent.root().unwrap().downcast_ref::<gtk::Window>().unwrap(),
                parent.imp().transaction.get().unwrap(),
                parent.parent().unwrap().
                    parent().unwrap().
                    downcast_ref::<GroupContent>().unwrap().
                    imp().group.get().unwrap()
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

            group.delete_transaction(parent.imp().transaction.get().unwrap().id());
            parent.save_group(group);
        }));
        transaction_action_group.add_action(&delete_action);

        self.insert_action_group("transaction", Some(&transaction_action_group));
    }

    fn save_group(&self, group: &Group) {
        app_data!(|data| {
            data.save_group(group);
        });
    }

    fn init_row(&self) {
        let amount = self.imp().transaction.get().unwrap().signed_amount();

        self.imp().amount_label.set_label(&format!("{:.2}", amount.abs()));

        if amount.gt(&0.0) {
            self.imp().amount_label.add_css_class("income");
        } else {
            self.imp().amount_label.add_css_class("expense");
        }

        self.set_title(&self.imp().transaction.get().unwrap().name());
        self.set_subtitle(&self.imp().transaction.get().unwrap().relative_date());
    }
}
