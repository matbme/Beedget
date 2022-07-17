use serde::{Deserialize, Serialize};
use uuid::Uuid;

use gtk::prelude::*;
use gtk::gdk::RGBA;
use gtk::gio;

use std::cell::RefCell;

use crate::models::*;
use crate::widgets::*;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Group {
    pub id: Uuid,
    pub name: String,
    pub emoji: String,
    pub color: Vec<f32>,
    transactions: RefCell<Vec<Transaction>>,
}

impl DataObject for Group {
    fn filename(&self) -> String {
        self.id.to_string() + &String::from(".json")
    }
}

impl Group {
    pub fn new(emoji: &str, color: RGBA, name: &str) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: String::from(name),
            emoji: String::from(emoji),
            color: vec![color.red(), color.green(), color.blue(), color.alpha()],
            transactions: RefCell::new(vec![]),
        }
    }

    pub fn rgba_color(&self) -> RGBA {
        RGBA::new(
            self.color[0],
            self.color[1],
            self.color[2],
            self.color[3]
        )
    }

    pub fn new_transaction(&self, transaction: Transaction) {
        self.transactions.borrow_mut().push(transaction);
    }

    pub fn delete_transaction(&self, transaction_id: Uuid) {
        let mut idx = 0;

        for transaction in self.transactions.borrow().iter() {
            if transaction.id == transaction_id {
                break;
            } else {
                idx += 1;
            }
        }

        self.transactions.borrow_mut().remove(idx);
    }

    pub fn transaction_model(&self) -> gio::ListStore {
        let ls = gio::ListStore::new(TransactionRow::static_type());

        for transaction in self.transactions.borrow().iter() {
            let row = TransactionRow::new(transaction);
            ls.append(&row);
        }

        ls
    }
}
