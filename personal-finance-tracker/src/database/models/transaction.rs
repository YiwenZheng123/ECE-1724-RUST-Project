// use serde::{Deserialize, Serialize};
use sqlx::FromRow;
// use bigdecimal::BigDecimal;
use rust_decimal::Decimal;
use chrono::NaiveDateTime;

#[derive(FromRow, Debug)]
pub struct Transaction {
    pub transaction_id: i64,                //key
    pub account_id: i64,
    pub category_id: Option<i64>,
    pub amount: Decimal,
    pub is_expense: bool,
    pub description: Option<String>,
    pub currency: Option<String>,
    pub transacted_at: NaiveDateTime,
    pub trans_create_at: NaiveDateTime,
}