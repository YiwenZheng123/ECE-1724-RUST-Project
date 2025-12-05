use sqlx::FromRow;

#[derive(FromRow, Debug)]
pub struct Category {
    pub category_id: i64,
    pub category_name: String,
    pub category_type: String,
    pub icon: String
}