use serde::{Deserialize, Serialize};
use uuid::Uuid;

use gtk::glib::DateTime;

#[derive(Debug, Serialize, Deserialize)]
pub enum TransactionType {
    EXPENSE,
    INCOME
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Transaction {
    pub id: Uuid,
    pub name: String,
    pub amount: f32,
    date: String
}

impl Transaction {
    pub fn new(name: &str, amount: f32, date: DateTime) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: String::from(name),
            amount,
            date: String::from(date.format_iso8601().unwrap().as_str())
        }
    }

    pub fn date(&self) -> DateTime {
        DateTime::from_iso8601(&self.date, None).unwrap()
    }
}
