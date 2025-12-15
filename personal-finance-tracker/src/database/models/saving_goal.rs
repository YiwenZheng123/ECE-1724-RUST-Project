use sqlx::FromRow;
use chrono::NaiveDate; 

#[derive(Debug, Clone, FromRow)]
pub struct SavingGoal {
    pub id: i64,
    pub name: String,
    pub target_amount: f64,
    pub current_amount: f64,
    pub deadline: Option<NaiveDate>, 
}

#[derive(Debug, Clone, FromRow)]
pub struct CategorySpending {
    pub category: String,
    pub total_amount: f64,
}