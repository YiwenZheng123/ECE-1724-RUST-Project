use sqlx::{Pool, Sqlite};
use rust_decimal::Decimal;
use std::str::FromStr; 
use sqlx::Row;
use chrono::NaiveDateTime;
use crate::database::models::{Account, Category, Transaction, Tag};
/*
This file contains the specific SQL query, 
CRUD (Create, Read, Update, Delete) logic 
and is responsible for interacting with the database.
 */

 /*==========Account Queries=========== */

// Create account
pub async fn create_account(
    pool: &Pool<Sqlite>, 
    account_name: &str, 
    account_type: &str,
    currency: &str,
) -> Result<i64, sqlx::Error> {
    let acc_id = sqlx::query!(
        r#"
        INSERT INTO accounts (account_name, account_type, balance, currency, account_created_at)
        VALUES (?, ?, ?, ?, datetime('now'))
        RETURNING account_id
        "#,
        account_name,
        account_type,
        "0", // The initial balance is 0. can be adjusted it by creating an "Income" transaction with the "initial balance".
        currency
    )
    .fetch_one(pool)
    .await?
    .account_id;

    Ok(acc_id)
}

// Get account by id
pub async fn get_account_by_id(pool: &Pool<Sqlite>, account_id: i64) -> Result<Account, sqlx::Error> {
    // Retrieves a row of data (returns a Row type)
    let row = sqlx::query(
        r#"
        SELECT 
            account_id, 
            account_name, 
            account_type, 
            balance, 
            currency, 
            account_created_at
        FROM accounts
        WHERE account_id = ?
        "#
    )
    .bind(account_id) 
    .fetch_one(pool) 
    .await?;          // convert Result<Row, Error> to Row or return Error.

    let balance_text: String = row.get("balance");
    let balance_decimal = Decimal::from_str(&balance_text)
        .map_err(|e| sqlx::Error::Decode(format!("Invalid Decimal format for balance: {}", e).into()))?;

    Ok(Account {
        account_id: row.get("account_id"),
        account_name: row.get("account_name"),
        account_type: row.get("account_type"),
        balance: balance_decimal,
        currency: row.get("currency"),
        account_created_at: row.get("account_created_at"),
    })
}

// Get all accounts
pub async fn get_all_accounts(pool: &Pool<Sqlite>) -> Result<Vec<Account>, sqlx::Error> {
    sqlx::query(
        r#"
        SELECT 
            account_id, 
            account_name, 
            account_type, 
            balance, 
            currency, 
            account_created_at
        FROM accounts
        ORDER BY account_id ASC
        "#
    )
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|row| {
        let balance_text: String = row.get("balance");

        //convert the string to Decimal
        let balance_decimal = Decimal::from_str(&balance_text)
            .map_err(|e| sqlx::Error::Decode(format!("Invalid Decimal format for balance: {}", e).into()))?;

        Ok(Account {
            account_id: row.get("account_id"),
            account_name: row.get("account_name"),
            account_type: row.get("account_type"),
            balance: balance_decimal,
            currency: row.get("currency"),
            account_created_at: row.get("account_created_at"),
        })
    })
    .collect::<Result<Vec<Account>, sqlx::Error>>()
}


// Delete account
pub async fn delete_account(pool: &Pool<Sqlite>, account_id: i64) -> Result<bool, sqlx::Error> {
    let mut tx = pool.begin().await?;

    // First need to delete all transaction records associated with the account
    let trans_result = sqlx::query!(
        r#"
        DELETE FROM transactions
        WHERE account_id = ?
        "#,
        account_id
    )
    .execute(&mut *tx) 
    .await?;

    // Delete all scheduled transaction records associated with the account ('scheduled_transactions' table)

    // delete the account
    let acc_result = sqlx::query!(
        r#"
        DELETE FROM accounts
        WHERE account_id = ?
        "#,
        account_id
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(acc_result.rows_affected() > 0)
}


// Update account
pub async fn update_account(
    pool: &Pool<Sqlite>,
    account_id: i64,
    account_name: String,
    account_type: String,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query!(
        r#"
        UPDATE accounts
        SET account_name = ?, account_type = ?
        WHERE account_id = ?
        "#,
        account_name,
        account_type,
        account_id
    )
    .execute(pool)
    .await?;

    Ok(result.rows_affected() > 0)
}

 /*==========Category Queries=========== */
 // Create a category
 pub async fn create_category(pool: &Pool<Sqlite>,category_name: &str,
    category_type: &str, // "Income" or "Expense"
    icon: &str
) -> Result<i64, sqlx::Error> {
    let catid = sqlx::query!(
        r#"
        INSERT INTO categories (category_name, category_type, icon)
        VALUES (?, ?, ?)
        RETURNING category_id
        "#,
        category_name,
        category_type,
        icon
    )
    .fetch_one(pool)
    .await?
    .category_id;

    Ok(catid)
}

pub async fn get_all_categories(pool: &Pool<Sqlite>) -> Result<Vec<Category>, sqlx::Error> {
    sqlx::query_as!(Category,
        "SELECT * FROM categories ORDER BY category_name ASC"
    )
    .fetch_all(pool)
    .await
}

 /*==========Transaction Queries=========== */

 //create a transaction

/* The core logic of creating a transaction:
It is an atomic operation: it inserts the transaction record and automatically updates the account balance.
If any step fails, the database will roll back to ensure the safety of funds. */

pub async fn create_transaction(
pool: &Pool<Sqlite>,
    account_id: i64,
    category_id: Option<i64>,
    amount: Decimal,
    is_expense: bool,
    description: Option<&str>,
    transacted_at: NaiveDateTime, // scheduled transaction time
) -> Result<i64, sqlx::Error> {
    // Start database transaction (ACID)
    let mut tx = pool.begin().await?;
    let amount_str = amount.to_string();

    // insert transaction record
    let trans_id_record = sqlx::query!(
        r#"
        INSERT INTO transactions (
            account_id, category_id, amount, is_expense, 
            description, transacted_at, trans_create_at
        )
        VALUES (?, ?, ?, ?, ?, ?, datetime('now'))
        RETURNING transaction_id
        "#,
        account_id,
        category_id,
        amount_str,
        is_expense,
        description,
        transacted_at
    )
    // .map(|row| row.get::<i64, _>("transaction_id"))
    .fetch_one(&mut *tx)
    .await?;

    let trans_id = trans_id_record.transaction_id;

    // change in balance: expense(-amount), income(+amount)
    let balance_change = if is_expense { -amount } else { amount };
    let balance_change_str = balance_change.to_string();
    //update balance
    sqlx::query!(
        r#"
        UPDATE accounts
        SET balance = balance + ?
        WHERE account_id = ?
        "#,
        balance_change_str,
        account_id
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(trans_id)
}

// Get all transactions of a specific account
pub async fn get_transactions_by_account(
    pool: &Pool<Sqlite>, 
    account_id: i64
) -> Result<Vec<Transaction>, sqlx::Error> {
    sqlx::query(
        r#"
        SELECT *
        FROM transactions
        WHERE account_id = ?
        ORDER BY transacted_at DESC
        "#
    )
    .bind(account_id) 
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|row| {
        let amount_text: String = row.get("amount");

        let amount_decimal = Decimal::from_str(&amount_text)
            .map_err(|e| sqlx::Error::Decode(format!("Invalid Decimal format for amount: {}", e).into()))?;

        Ok(Transaction {
            transaction_id: row.get("transaction_id"),
            account_id: row.get("account_id"),
            category_id: row.get("category_id"),
            amount: amount_decimal,
            is_expense: row.get("is_expense"),
            description: row.get("description"),
            currency: row.get("currency"),
            transacted_at: row.get("transacted_at"),
            trans_create_at: row.get("trans_create_at"),
        })
    })
    .collect::<Result<Vec<Transaction>, sqlx::Error>>()
}

// ====================Tag Queries======================
// create Tag
pub async fn create_tag(pool: &Pool<Sqlite>, tag: &str) -> Result<i64, sqlx::Error> {
    let id = sqlx::query!(
        "INSERT INTO tags (tag) VALUES (?) RETURNING tag_id",
        tag
    )
    .fetch_one(pool)
    .await?
    .tag_id;
    Ok(id)
}

// get all Tags
pub async fn get_all_tags(pool: &Pool<Sqlite>) -> Result<Vec<Tag>, sqlx::Error> {
    sqlx::query_as!(Tag, "SELECT * FROM tags").fetch_all(pool).await
}

// bind Tag with Transaction
pub async fn add_tag_to_transaction(
    pool: &Pool<Sqlite>,
    transaction_id: i64,
    tag_id: i64
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "INSERT INTO transaction_tags (transaction_id, tag_id) VALUES (?, ?)",
        transaction_id,
        tag_id
    )
    .execute(pool)
    .await?;
    Ok(())
}