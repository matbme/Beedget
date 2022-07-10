use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{glib, CompositeTemplate};
use glib::{ParamFlags, ParamSpec, ParamSpecFloat};

use adw::subclass::{
    action_row::ActionRowImpl,
    preferences_row::PreferencesRowImpl
};

use once_cell::sync::Lazy;

use std::cell::RefCell;

use crate::models::*;

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/com/github/matbme/beedget/ui/transaction-row.ui")]
    pub struct TransactionRow {
        #[template_child]
        pub amount_label: TemplateChild<gtk::Label>,

        #[template_child]
        pub options: TemplateChild<gtk::Button>,

        pub amount: RefCell<f32>
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
                vec![ParamSpecFloat::new(
                    "amount",
                    "amount",
                    "amount",
                    std::f32::MIN,
                    std::f32::MAX,
                    0.0,
                    ParamFlags::CONSTRUCT | ParamFlags::READWRITE
                )]
            });

            PROPERTIES.as_ref()
        }

        fn set_property(&self, _obj: &Self::Type, _id: usize, value: &glib::Value, pspec: &ParamSpec) {
            match pspec.name() {
                "amount" => {
                    if let Ok(input) = value.get() {
                        self.amount.replace(input);
                    }
                }
                _ => unimplemented!()
            }
        }

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> glib::Value {
            match pspec.name() {
                "amount" => self.amount.borrow().to_value(),
                _ => unimplemented!()
            }
        }

        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            obj.set_amount();
        }
    }

    impl WidgetImpl for TransactionRow {}
    impl ListBoxRowImpl for TransactionRow {}
    impl PreferencesRowImpl for TransactionRow {}
    impl ActionRowImpl for TransactionRow {}
}

glib::wrapper! {
    pub struct TransactionRow(ObjectSubclass<imp::TransactionRow>)
        @extends gtk::ListBoxRow, gtk::Widget,
        @implements gtk::Accessible, gtk::Actionable, gtk::Buildable, gtk::ConstraintTarget;
}

impl TransactionRow {
    pub fn new(transaction: &Transaction) -> Self {
        glib::Object::new(&[
            ("title", &transaction.name),
            ("subtitle", &transaction.relative_date()),
            ("amount", &transaction.signed_amount())
        ]).expect("Failed to create `TransactionRow`.")
    }

    fn set_amount(&self) {
        self.imp().amount_label.set_label(&format!("{:.2}", self.imp().amount.borrow().abs()));

        if self.imp().amount.borrow().gt(&0.0) {
            self.imp().amount_label.add_css_class("income");
        } else {
            self.imp().amount_label.add_css_class("expense");
        }
    }
}
