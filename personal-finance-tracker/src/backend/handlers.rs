// src/backend/handlers.rs
use axum::{
    extract::State,
    Json,
    response::IntoResponse,
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use crate::backend::AppState;
use rust_decimal::Decimal;
use chrono::NaiveDateTime;
use rust_decimal::prelude::ToPrimitive; //converting Decimal to f64

#[derive(Debug, Deserialize, Serialize)]
pub struct CreateTransaction {
    pub account_id: i64,
    pub category_id: i64,      
    pub amount: Decimal,
    pub base_amount: Decimal,  
    pub is_expense: bool,
    pub description: Option<String>,
    pub currency: String,      
    pub transacted_at: NaiveDateTime,
}
#[derive(Debug, Deserialize, Serialize)]
pub struct SyncRequest {
    pub last_synced_at: Option<String>,
    pub transactions: Vec<CreateTransaction>,
}

pub async fn sync_handler(
    State(state): State<AppState>,
    Json(payload): Json<SyncRequest>,
) -> impl IntoResponse {
    println!(" Received sync request, processing {} transactions...", payload.transactions.len());

    let mut success_count = 0;

    for txn in payload.transactions {
        let amount_f64 = txn.amount.to_f64().unwrap_or(0.0);
        let base_amount_f64 = txn.base_amount.to_f64().unwrap_or(0.0);

        let now = chrono::Local::now().naive_local();

        let result = sqlx::query!(
            r#"
            INSERT INTO transactions (
                account_id, category_id, amount, base_amount, is_expense, 
                description, currency, transacted_at, trans_create_at
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            txn.account_id,
            txn.category_id,
            amount_f64,        
            base_amount_f64,  
            txn.is_expense,
            txn.description,
            txn.currency,
            txn.transacted_at,
            now
        )
        .execute(&state.db)
        .await;

        match result {
            Ok(_) => success_count += 1,
            Err(e) => {
                println!("Insert failed: {:?}", e);
            }
        }
    }

    println!("Sync complete! Successfully inserted {} records", success_count);

    (StatusCode::OK, Json(format!("Synced {} transactions successfully", success_count)))
}