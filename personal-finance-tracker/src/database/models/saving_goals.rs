use serde::{Serialize, Deserialize};
use rust_decimal::Decimal;
use chrono::NaiveDateTime;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct SavingsGoal {
    pub goal_id: i64,
    pub account_id: i64,
    pub goal_name: String,
    pub target_amount: Decimal,
    pub current_amount: Decimal,
    pub deadline: NaiveDateTime,
}
