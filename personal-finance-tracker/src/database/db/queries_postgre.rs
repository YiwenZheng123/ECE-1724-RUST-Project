use sqlx::{Pool, Postgres};
use rust_decimal::Decimal;
use std::str::FromStr; 
use sqlx::Row;
use chrono::NaiveDateTime;
use crate::database::models::{Account, Category, Transaction, NewTransaction, Tag, RecurringTransaction, Budget, SavingsGoal, CurrencyRate};
use rust_decimal::prelude::*;

/*
This file contains the specific PostgreSQL query, 
CRUD (Create, Read, Update, Delete) logic 
and is responsible for interacting with the database.
 */

 /*==========Account Queries=========== */

// Create account
pub async fn create_account(
    pool: &Pool<Postgres>, 
    account_name: &str, 
    account_type: &str,
    currency: &str,
) -> Result<i64, sqlx::Error> {
    let acc_id = sqlx::query!(
        r#"
        INSERT INTO accounts (account_name, account_type, balance, currency, account_created_at)
        VALUES ($1, $2, $3::NUMERIC, $4, NOW())
        RETURNING account_id
        "#,
        account_name,
        account_type,
        Decimal::from_str("0").unwrap(), // Postgres TEXT to NUMERIC implicit cast works, but ::NUMERIC is safer
        currency
    )
    .fetch_one(pool)
    .await?
    .account_id;

    Ok(acc_id.into())
}

// Get account by id
pub async fn get_account_by_id(pool: &Pool<Postgres>, account_id: i64) -> Result<Account, sqlx::Error> {
    // Retrieves a row of data (returns a Row type)
    // Note: Added ::TEXT cast to balance to ensure it returns as String for your existing logic
    let row = sqlx::query(
        r#"
        SELECT 
            account_id, 
            account_name, 
            account_type, 
            balance::TEXT, 
            currency, 
            account_created_at
        FROM accounts
        WHERE account_id = $1
        "#
    )
    .bind(account_id) 
    .fetch_one(pool) 
    .await?;          

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
pub async fn get_all_accounts(pool: &Pool<Postgres>) -> Result<Vec<Account>, sqlx::Error> {
    sqlx::query(
        r#"
        SELECT 
            account_id, 
            account_name, 
            account_type, 
            balance::TEXT, 
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
pub async fn delete_account(pool: &Pool<Postgres>, account_id: i64) -> Result<bool, sqlx::Error> {
    let mut tx = pool.begin().await?;

    // First need to delete all transaction records associated with the account
    let _trans_result = sqlx::query!(
        r#"
        DELETE FROM transactions
        WHERE account_id = $1
        "#,
        account_id
    )
    .execute(&mut *tx) 
    .await?;

    // delete the account
    let acc_result = sqlx::query!(
        r#"
        DELETE FROM accounts
        WHERE account_id = $1
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
    pool: &Pool<Postgres>,
    account_id: i64,
    account_name: String,
    account_type: String,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query!(
        r#"
        UPDATE accounts
        SET account_name = $1, account_type = $2
        WHERE account_id = $3
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
 pub async fn create_category(pool: &Pool<Postgres>,category_name: &str,
    category_type: &str, // "Income" or "Expense"
    icon: &str
) -> Result<i64, sqlx::Error> {
    let catid = sqlx::query!(
        r#"
        INSERT INTO categories (category_name, category_type, icon)
        VALUES ($1, $2, $3)
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

pub async fn get_all_categories(pool: &Pool<Postgres>) -> Result<Vec<Category>, sqlx::Error> {
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
    pool: &Pool<Postgres>,
    t: &NewTransaction
) -> Result<i64, sqlx::Error> {

    // Start SQL transaction (ACID)
    let mut tx = pool.begin().await?;

    // step 1: get currency rate
    let currency = t.currency.as_deref().unwrap_or("CAD"); 
    let rate = get_rate(pool, currency).await?;

    // step 2: Decimal → f64 to compute base_amount
    let amount_f64 = t.amount.to_f64().unwrap_or(0.0);
    let base_amount_f64 = amount_f64 * rate;

    // step 3: insert transaction into DB
    let inserted = sqlx::query!(
        r#"
        INSERT INTO transactions (
            account_id,
            category_id,
            amount,
            base_amount,
            currency,
            is_expense,
            description,
            transacted_at
        )
        VALUES ($1, $2, $3::NUMERIC, $4, $5, $6, $7, $8)
        RETURNING transaction_id
        "#,
        t.account_id,
        t.category_id,
        t.amount,                  // Decimal → NUMERIC
        base_amount_f64,           // f64 → DOUBLE PRECISION
        t.currency,
        t.is_expense,
        t.description,
        t.transacted_at
    )
    .fetch_one(&mut *tx)
    .await?;

    let new_id = inserted.transaction_id;

    // step 4: determine balance change
    let balance_delta = if t.is_expense {-t.amount} else {t.amount};

    // step 5: update account balance
    sqlx::query!(
        r#"
        UPDATE accounts
        SET balance = balance + $1::NUMERIC
        WHERE account_id = $2
        "#,
        balance_delta,
        t.account_id
    )
    .execute(&mut *tx)
    .await?;

    // step 6: commit SQL transaction
    tx.commit().await?;

    Ok(new_id)
}

// Get all transactions of a specific account
pub async fn get_transactions_by_account(
    pool: &Pool<Postgres>, 
    account_id: i64
) -> Result<Vec<Transaction>, sqlx::Error> {
    let rows = sqlx::query(
        r#"
        SELECT 
            transaction_id, account_id, category_id, 
            amount::TEXT, is_expense, description, 
            currency, transacted_at, trans_create_at
        FROM transactions
        WHERE account_id = $1
        ORDER BY transacted_at DESC
        "#
    )
    .bind(account_id)
    .fetch_all(pool)
    .await?;

    // map SQL rows → Transaction structs
    let transactions = rows.into_iter()
        .map(|row| {
            let amount_text: String = row.get("amount");
            let amount = Decimal::from_str(&amount_text)
                .map_err(|e| sqlx::Error::Decode(format!("Invalid Decimal: {}", e).into()))?;

            Ok(Transaction {
                transaction_id: row.get("transaction_id"),
                account_id: row.get("account_id"),
                category_id: row.get("category_id"),
                amount,
                is_expense: row.get("is_expense"),
                description: row.get("description"),
                currency: row.get("currency"),
                transacted_at: row.get("transacted_at"),
                trans_create_at: row.get("trans_create_at"),
            })
        })
        .collect::<Result<Vec<Transaction>, sqlx::Error>>()?;

    Ok(transactions)
}

// ====================Tag Queries======================
// create Tag
pub async fn create_tag(pool: &Pool<Postgres>, tag: &str) -> Result<i64, sqlx::Error> {
    let id = sqlx::query!(
        "INSERT INTO tags (tag) VALUES ($1) RETURNING tag_id",
        tag
    )
    .fetch_one(pool)
    .await?
    .tag_id;
    Ok(id)
}

// get all Tags
pub async fn get_all_tags(pool: &Pool<Postgres>) -> Result<Vec<Tag>, sqlx::Error> {
    sqlx::query_as!(Tag, "SELECT * FROM tags").fetch_all(pool).await
}

// bind Tag with Transaction
pub async fn add_tag_to_transaction(
    pool: &Pool<Postgres>,
    transaction_id: i64,
    tag_id: i64
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "INSERT INTO transaction_tags (transaction_id, tag_id) VALUES ($1, $2)",
        transaction_id,
        tag_id
    )
    .execute(pool)
    .await?;
    Ok(())
}

// ====================Recurring Queries======================
pub async fn create_recurring(
    pool: &Pool<Postgres>,
    account_id: i64,
    amount: Decimal,
    currency: &str,
    category_id: Option<i64>,
    description: Option<&str>,
    recurrence_rule: String,
    next_run_date: NaiveDateTime
) -> Result<i64, sqlx::Error>{
    // let amount_str = amount.to_string();

    // Postgres: Cannot use last_insert_rowid(). 
    // Changed to RETURNING recurring_id and using fetch_one.
    let recurring_id = sqlx::query!(
        r#"
        INSERT INTO recurring_transactions(
            account_id, amount, currency, category_id, description, recurrence_rule, next_run_date
        )
        VALUES ($1, $2::NUMERIC, $3, $4, $5, $6, $7)
        RETURNING recurring_id
        "#,
        account_id,
        amount,
        currency,
        category_id,
        description,
        recurrence_rule,
        next_run_date
    )
    .fetch_one(pool)
    .await?
    .recurring_id;

    Ok(recurring_id)
}

pub async fn get_due(pool: &Pool<Postgres>, today: &str) 
    -> Result<Vec<RecurringTransaction>, sqlx::Error> {
    // Note: Assuming 'today' is a string formatted as date/datetime. 
    // Explicit casting to TIMESTAMP might be needed if Postgres complains, e.g. $1::TIMESTAMP
    // For now assuming the driver handles the string->timestamp mapping or string->string comparison.
    // Also added amount::TEXT casting.
    let recs = sqlx::query(
        r#"
        SELECT 
            recurring_id, account_id, amount::TEXT, 
            currency, category_id, description, 
            recurrence_rule, next_run_date
        FROM recurring_transactions
        WHERE next_run_date <= $1::TIMESTAMP
        "#
    )
    .bind(today)
    .fetch_all(pool)
    .await?;
    
    //Use .map to convert Row to RecurringTransaction
    recs.into_iter()
    .map(|row| {
        let amount_text: String = row.get("amount");

    let amount_decimal = Decimal::from_str(&amount_text)
            .map_err(|e| sqlx::Error::Decode(format!("Invalid Decimal format for amount: {}", e).into()))?;

        Ok(RecurringTransaction {
            recurring_id: row.get("recurring_id"),
            account_id: row.get("account_id"),
            amount: amount_decimal,
            currency: row.get("currency"),
            category_id: row.get("category_id"),
            description: row.get("description"),
            recurrence_rule: row.get("recurrence_rule"),
            next_run_date: row.get("next_run_date"),
        })
    })
    .collect::<Result<Vec<RecurringTransaction>, sqlx::Error>>()
}

pub async fn update_next_date(
    pool: &Pool<Postgres>,
    id: i64,
    new_date: &NaiveDateTime
) -> Result<(), sqlx::Error> {
    // Note: cast new_date to TIMESTAMP
    sqlx::query!(
        r#"
        UPDATE recurring_transactions
        SET next_run_date = $1::TIMESTAMP
        WHERE recurring_id = $2
        "#,
        new_date,
        id
    )
    .execute(pool)
    .await?;

    Ok(())
}

// ====================Budget Queries======================
pub async fn create_budget(pool: &Pool<Postgres>, b: &Budget) -> Result<i64, sqlx::Error> {
    
    // Postgres: Cannot use last_insert_rowid(). 
    // Changed to RETURNING budget_id.
    let id = sqlx::query!(
        r#"
        INSERT INTO budgets 
        (account_id, category_id, period, amount, currency, start_date)
        VALUES ($1, $2, $3, $4::NUMERIC, $5, $6)
        RETURNING budget_id
        "#,
        b.account_id,
        b.category_id,
        b.period,
        b.amount,
        b.currency,
        b.start_date,
    )
    .fetch_one(pool)
    .await?
    .budget_id;

    Ok(id)
}

pub async fn list_by_account(pool: &Pool<Postgres>, acc_id: i64) -> Result<Vec<Budget>,sqlx::Error> {
    let rows = sqlx::query_as!(
        Budget,
        r#"
        SELECT * FROM budgets 
        WHERE account_id = $1
        "#,
        acc_id
    )
    .fetch_all(pool)
    .await?;

    Ok(rows)
}

// ====================Saving Queries======================
pub async fn create_saving(pool: &Pool<Postgres>, g: &SavingsGoal) -> Result<i64, sqlx::Error> {
    let new_saving_id = sqlx::query!(
        r#"
        INSERT INTO savings_goals
        (account_id, goal_name, target_amount, current_amount, deadline)
        VALUES ($1, $2, $3::NUMERIC, $4::NUMERIC, $5)
        RETURNING goal_id
        "#,
        g.account_id,
        g.goal_name,
        g.target_amount,
        g.current_amount,
        g.deadline,
    )
    .fetch_one(pool)
    .await?
    .goal_id;

    Ok(new_saving_id)
}

pub async fn update_current_amount(
    pool: &Pool<Postgres>,
    goal_id: i64,
    new_amount: Decimal
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        UPDATE savings_goals
        SET current_amount = $1
        WHERE goal_id = $2
        "#,
        new_amount,
        goal_id
    )
    .execute(pool)
    .await?;

    Ok(())
}

// ====================currency Queries======================
pub async fn get_rate(pool: &Pool<Postgres>, currency: &str) -> Result<f64, sqlx::Error> {
    let row = sqlx::query!(
        r#"
        SELECT rate_to_base as rate_to_base
        FROM currency_rates
        WHERE currency = $1
        "#,
        currency
    )
    .fetch_one(pool)
    .await?;

    Ok(row.rate_to_base)
}

pub async fn upsert_rate(
    pool: &Pool<Postgres>,
    currency: &str,
    rate: f64
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO currency_rates (currency, rate_to_base)
        VALUES ($1, $2)
        ON CONFLICT(currency) DO UPDATE SET rate_to_base = excluded.rate_to_base
        "#,
        currency,
        rate
    )
    .execute(pool)
    .await?;

    Ok(())
}

// ====================reports Queries======================
pub async fn monthly_summary(
    pool: &Pool<Postgres>,
    start: &NaiveDateTime,
    end: &NaiveDateTime
) -> Result<Vec<(i64, f64)>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"
        SELECT category_id, SUM(base_amount) as total
        FROM transactions
        WHERE trans_create_at BETWEEN $1 AND $2
        GROUP BY category_id
        "#,
        start,
        end
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| (r.category_id, r.total.unwrap_or(0.0)))
        .collect())
}

pub async fn net_savings(pool: &Pool<Postgres>) -> Result<f64, sqlx::Error> {
    let row = sqlx::query!(
        r#"
        SELECT 
            (SELECT SUM(base_amount) FROM transactions WHERE amount > 0) -
            (SELECT SUM(base_amount) FROM transactions WHERE amount < 0)
        AS net
        "#
    )
    .fetch_one(pool)
    .await?;

    Ok(row.net.unwrap_or(0.0))
}