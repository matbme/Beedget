use serde::{Deserialize, Serialize};
use uuid::Uuid;

use gtk::glib::DateTime;

use crate::application::CLOCK_FORMAT;

#[derive(Debug, Serialize, Deserialize)]
pub enum TransactionType {
    EXPENSE,
    INCOME
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Transaction {
    pub id: Uuid,
    pub name: String,
    pub tr_type: TransactionType,
    pub amount: f32,
    date: String
}

impl Transaction {
    pub fn new(name: &str, tr_type: TransactionType, amount: f32, date: DateTime) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: String::from(name),
            tr_type,
            amount,
            date: String::from(date.format_iso8601().unwrap().as_str())
        }
    }

    pub fn date(&self) -> DateTime {
        DateTime::from_iso8601(&self.date, None).unwrap()
    }

    pub fn signed_amount(&self) -> f32 {
        match self.tr_type {
            TransactionType::EXPENSE => -self.amount,
            TransactionType::INCOME => self.amount,
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
