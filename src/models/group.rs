use serde::{Deserialize, Serialize};
use uuid::Uuid;
use gtk::gdk::RGBA;

use crate::models::{DataObject, Transaction};

#[derive(Debug, Serialize, Deserialize)]
pub struct Group {
    pub id: Uuid,
    pub name: String,
    pub emoji: String,
    pub color: Vec<f32>,
    transactions: Vec<Transaction>
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
            transactions: vec![]
        }
    }
}
