use serde::{Serialize, Deserialize};
use rust_decimal::Decimal;
use chrono::NaiveDateTime;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Budget {
    pub budget_id: i64,
    pub account_id: i64,
    pub category_id: Option<i64>,
    pub period: String,       // 'weekly' or 'monthly'
    pub amount: Decimal,
    pub currency: String,
    pub start_date: NaiveDateTime,
}
