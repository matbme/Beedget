use serde::{Deserialize, Serialize};
use uuid::Uuid;

use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::glib;
use glib::{DateTime, ParamSpec, ParamSpecString, ParamSpecFloat};

use once_cell::sync::Lazy;

use derivative::Derivative;

use std::cell::RefCell;

use crate::application::CLOCK_FORMAT;

#[derive(Debug, Serialize, Deserialize)]
pub enum TransactionType {
    EXPENSE,
    INCOME
}

pub fn transaction_type_to_string(tr_type: &TransactionType) -> String {
    match tr_type {
        TransactionType::EXPENSE => String::from("EXPENSE"),
        TransactionType::INCOME => String::from("INCOME"),
    }
}

pub fn transaction_type_from_string(tr_str: &str) -> TransactionType {
    match tr_str {
        "EXPENSE" => TransactionType::EXPENSE,
        "INCOME" => TransactionType::INCOME,
        _ => unimplemented!()
    }
}

mod imp {
    use super::*;

    #[derive(Derivative, Debug, Serialize, Deserialize)]
    #[derivative(Default)]
    pub struct TransactionInner {
        pub id: Uuid,
        pub name: String,
        #[derivative(Default(value="TransactionType::EXPENSE"))]
        pub tr_type: TransactionType,
        pub amount: f32,
        pub date: String
    }

    #[derive(Default)]
    pub struct Transaction {
        pub inner: RefCell<TransactionInner>
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Transaction {
        const NAME: &'static str = "Transaction";
        type Type = super::Transaction;
        type ParentType = glib::Object;
    }

    impl ObjectImpl for Transaction {
        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![
                    ParamSpecString::builder("uid").build(),
                    ParamSpecString::builder("name").build(),
                    ParamSpecString::builder("tr-type").build(),
                    ParamSpecFloat::builder("amount").build(),
                    ParamSpecString::builder("date").build(),
                ]
            });

            PROPERTIES.as_ref()
        }

        fn set_property(&self, _obj: &Self::Type, _id: usize, value: &glib::Value, pspec: &ParamSpec) {
            match pspec.name() {
                "uid" => self.inner.borrow_mut().id = Uuid::parse_str(value.get().unwrap()).unwrap(),
                "name" => self.inner.borrow_mut().name = value.get().unwrap(),
                "tr-type" => self.inner.borrow_mut().tr_type = transaction_type_from_string(value.get().unwrap()),
                "amount" => self.inner.borrow_mut().amount = value.get().unwrap(),
                "date" => self.inner.borrow_mut().date = value.get().unwrap(),
                _ => unimplemented!()
            }
        }

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> glib::Value {
            match pspec.name() {
                "uid" => self.inner.borrow().id.to_string().to_value(),
                "name" => self.inner.borrow().name.to_value(),
                "tr-type" => transaction_type_to_string(&self.inner.borrow().tr_type).to_value(),
                "amount" => self.inner.borrow().amount.to_value(),
                "date" => self.inner.borrow().date.to_value(),
                _ => unimplemented!()
            }
        }
    }
}

glib::wrapper! {
    pub struct Transaction(ObjectSubclass<imp::Transaction>);
}

impl Default for Transaction {
    fn default() -> Self {
        Self::empty()
    }
}

impl Serialize for Transaction {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where
        S: serde::Serializer {
        self.imp().inner.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Transaction {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where
        D: serde::Deserializer<'de> {
        let inner = imp::TransactionInner::deserialize(deserializer)?;
        let transaction: Self = glib::Object::new(&[])
            .expect("Failed to create Transaction");

        transaction.imp().inner.replace(inner);

        Ok(transaction)
    }
}

impl Transaction {
    pub fn new(name: &str, tr_type: TransactionType, amount: f32, date: DateTime) -> Self {
        glib::Object::new(&[
            ("uid", &Uuid::new_v4().to_string()),
            ("name", &name),
            ("tr-type", &transaction_type_to_string(&tr_type)),
            ("amount", &amount),
            ("date", &date.format_iso8601().unwrap().as_str())
        ]).expect("Failed to create Transaction")
    }

    pub fn empty() -> Self {
        glib::Object::new(&[]).expect("Failed to create Transaction")
    }

    pub fn id(&self) -> Uuid {
        self.imp().inner.borrow().id
    }

    pub fn name(&self) -> String {
        self.imp().inner.borrow().name.clone()
    }

    pub fn amount(&self) -> f32 {
        self.imp().inner.borrow().amount
    }

    pub fn tr_type(&self) -> TransactionType {
        match self.imp().inner.borrow().tr_type {
            TransactionType::EXPENSE => TransactionType::EXPENSE,
            TransactionType::INCOME => TransactionType::INCOME,
        }
    }

    pub fn date(&self) -> DateTime {
        DateTime::from_iso8601(&self.imp().inner.borrow().date, None).unwrap()
    }

    pub fn set_name(&self, name: &str) {
        self.imp().inner.borrow_mut().name = name.to_string();
        self.notify("name");
    }

    pub fn change_tr_type(&self, tr_type: TransactionType) {
        self.imp().inner.borrow_mut().tr_type = tr_type;
        self.notify("tr-type");
    }

    pub fn set_amount(&self, amount: f32) {
        self.imp().inner.borrow_mut().amount = amount;
        self.notify("amount");
    }

    pub fn set_date(&self, date: DateTime) {
        self.imp().inner.borrow_mut().date = String::from(date.format_iso8601().unwrap().as_str());
        self.notify("date");
    }

    pub fn signed_amount(&self) -> f32 {
        match self.imp().inner.borrow().tr_type {
            TransactionType::EXPENSE => -self.imp().inner.borrow().amount,
            TransactionType::INCOME => self.imp().inner.borrow().amount,
        }
    }

    pub fn relative_date(&self) -> String {
        let date_obj = self.date();
        let now = DateTime::now_local().unwrap();

        let day_component = match now.day_of_year() - date_obj.day_of_year() {
            0 => String::from("Today"),
            1 => String::from("Yesterday"),
            _ => date_obj.format("%x").unwrap().to_string()
        };

        let time_component = if CLOCK_FORMAT.as_str() == "12h" {
            date_obj.format("%r").unwrap().to_string()
        } else {
            date_obj.format("%R").unwrap().to_string()
        };

        day_component + ", " + &time_component
    }
}
