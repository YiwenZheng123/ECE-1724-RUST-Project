use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use rust_decimal::Decimal;

pub mod account;
pub mod category;
pub mod transaction;
pub mod recurring_transaction;
pub mod budget;
pub mod saving_goals;
pub mod currency_rate;


pub use account::Account;
pub use category::Category;
pub use transaction::Transaction;
pub use recurring_transaction::RecurringTransaction;
pub use budget::Budget;
pub use currency_rate::CurrencyRate;
// pub use saving_goals::SavingsGoal;

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct CategorySpending {
    pub category: String,
    pub total_amount: Decimal,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct SavingsGoal {
    pub goal_id: i64,
    pub account_id: i64,
    pub goal_name: String,
    pub target_amount: Decimal,
    pub current_amount: Decimal,
    pub deadline: Option<String>,
}
