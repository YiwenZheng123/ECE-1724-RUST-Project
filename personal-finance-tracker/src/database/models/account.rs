use sqlx::FromRow;
// use bigdecimal::BigDecimal;
use rust_decimal::Decimal;
use chrono::NaiveDateTime;

#[derive(FromRow, Debug)]
pub struct Account {
    pub account_id: i64,
    pub account_name: String,               // account name defined by user (cash/RBC chequing)
    pub account_type: String,       // cash/debit/credit/other
    pub balance: Decimal,
    pub currency: Option<String>,
    pub account_created_at: NaiveDateTime,
}