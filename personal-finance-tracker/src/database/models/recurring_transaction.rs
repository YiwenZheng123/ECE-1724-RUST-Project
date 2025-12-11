use serde::{Serialize, Deserialize};
use chrono::NaiveDateTime;
use rust_decimal::Decimal;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct RecurringTransaction {
    pub recurring_id: i64,
    pub account_id: i64,
    pub amount: Decimal,
    pub currency: String,
    pub category_id: Option<i64>,
    pub description: Option<String>,
    pub recurrence_rule: String,
    pub next_run_date: NaiveDateTime, // ISO date string
}