use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::prelude::*;

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
    pub date: DateTime<Utc>
}

impl Transaction {
    fn new(name: &str, amount: f32, date: DateTime<Utc>) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: String::from(name),
            amount,
            date
        }
    }
}
