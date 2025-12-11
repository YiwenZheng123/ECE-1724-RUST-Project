use rust_decimal::Decimal;
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct CurrencyRate {
    pub currency: String,
    pub rate_to_base: Decimal,
}
