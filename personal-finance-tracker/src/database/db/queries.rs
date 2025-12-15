use sqlx::{Pool, Sqlite};
use rust_decimal::Decimal;
use std::str::FromStr; 
use sqlx::Row;
use chrono::NaiveDateTime;
use crate::database::models::{
        Account, Category, Transaction, RecurringTransaction, 
        Budget, SavingsGoal, CategorySpending 
};

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
    sqlx::query!(
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
    category_id: i64,
    amount: Decimal,
    base_amount: Decimal,
    currency: String,
    is_expense: bool,
    description: Option<&str>,
    transacted_at: NaiveDateTime, // scheduled transaction time
) -> Result<i64, sqlx::Error> {
    // Start database transaction (ACID)
    let mut tx = pool.begin().await?;
    let amount_str = amount.to_string();
    let base_amount_str = base_amount.to_string();

    // insert transaction record
    let trans_id_record = sqlx::query!(
        r#"
        INSERT INTO transactions (
            account_id, category_id, amount, base_amount, is_expense, 
            description, currency, transacted_at, trans_create_at
        )
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, datetime('now'))
        RETURNING transaction_id
        "#,
        account_id,
        category_id,
        amount_str,
        base_amount_str,
        is_expense,
        description,
        currency,
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
        let base_amount_decimal = Decimal::from_str(&amount_text)
            .map_err(|e| sqlx::Error::Decode(format!("Invalid Decimal format for amount: {}", e).into()))?;


        Ok(Transaction {
            transaction_id: row.get("transaction_id"),
            account_id: row.get("account_id"),
            category_id: row.get("category_id"),
            amount: amount_decimal,
            base_amount:base_amount_decimal,
            is_expense: row.get("is_expense"),
            description: row.get("description"),
            payee: row.get("payee"),
            currency: row.get("currency"),
            transacted_at: row.get("transacted_at"),
            trans_create_at: row.get("trans_create_at"),
        })
    })
    .collect::<Result<Vec<Transaction>, sqlx::Error>>()
}

/* ====================Recurring Queries====================== */
pub async fn create_recurring(
    pool: &Pool<Sqlite>,
    account_id: i64,
    amount: Decimal,
    currency: &str,
    category_id: Option<i64>,
    description: Option<&str>,
    recurrence_rule: String,
    next_run_date: NaiveDateTime
) -> Result<i64, sqlx::Error>{
    let amount_str = amount.to_string();

    let recurring_id = sqlx::query!(
        r#"
        INSERT INTO recurring_transactions(
            account_id, amount, currency, category_id, description, recurrence_rule, next_run_date
        )
        VALUES (?, ?, ?, ?, ?, ?, ?)
        "#,
        account_id,
        amount_str,
        currency,
        category_id,
        description,
        recurrence_rule,
        next_run_date
    )
    .execute(pool)
    .await?
    .last_insert_rowid();

    Ok(recurring_id)
}

pub async fn get_due(pool: &Pool<Sqlite>, today: &str) 
    -> Result<Vec<RecurringTransaction>, sqlx::Error> {
    let recs = sqlx::query(
        r#"
        SELECT * FROM recurring_transactions
        WHERE next_run_date <= ?
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
    pool: &Pool<Sqlite>,
    recurring_id: i64,
    new_date: &str
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        UPDATE recurring_transactions
        SET next_run_date = ?
        WHERE recurring_id = ?
        "#,
        new_date,
        recurring_id
    )
    .execute(pool)
    .await?;

    Ok(())
}

/*====================Budget Queries====================== */ 
pub async fn create_budget(pool: &Pool<Sqlite>, b: &Budget) -> Result<i64, sqlx::Error> {
    let amount_str = b.amount.to_string();
    let id = sqlx::query!(
        r#"
        INSERT INTO budgets 
        (account_id, category_id, period, amount, currency, start_date)
        VALUES (?, ?, ?, ?, ?, ?)
        "#,
        b.account_id,
        b.category_id,
        b.period,
        amount_str, 
        b.currency,
        b.start_date, 
    )
    .execute(pool)
    .await?
    .last_insert_rowid();

    Ok(id)
}

pub async fn list_by_account(pool: &Pool<Sqlite>, acc_id: i64) -> Result<Vec<Budget>,sqlx::Error> {
    sqlx::query(
        r#"
        SELECT * FROM budgets 
        WHERE account_id = ?
        "#
    )
    .bind(acc_id)
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|row|{
        let amount_text: String = row.get("amount");
        let amount_decimal = Decimal::from_str(&amount_text)
            .map_err(|e| sqlx::Error::Decode(format!("Invalid Decimal format for amount: {}", e).into()))?;
        Ok(Budget{
            budget_id: row.get("budget_id"),
            account_id: row.get("account_id"),
            category_id: row.get("category_id"),
            amount: amount_decimal,
            period: row.get("period"),
            currency: row.get("currency"),
            start_date: row.get("start_date"), 
        })
    })
    .collect::<Result<Vec<Budget>, sqlx::Error>>()
}

/*====================Saving Goal Queries====================== */ 
pub async fn create_saving_goal(pool: &Pool<Sqlite>, g: &SavingsGoal) -> Result<i64, sqlx::Error> {
    let target_amount_str = g.target_amount.to_string();
    let current_amount_str = g.current_amount.to_string();

    let new_saving_id = sqlx::query!(
        r#"
        INSERT INTO savings_goals
        (account_id, goal_name, target_amount, current_amount, deadline)
        VALUES (?, ?, ?, ?, ?)
        RETURNING goal_id
        "#,
        g.account_id,
        g.goal_name,
        target_amount_str,
        current_amount_str,
        g.deadline,
    )
    .fetch_one(pool)
    .await?
    .goal_id;

    Ok(new_saving_id)
}

pub async fn update_goal_amount(
    pool: &Pool<Sqlite>,
    goal_id: i64,
    new_amount: Decimal
) -> Result<(), sqlx::Error> {
    let new_amount_str = new_amount.to_string();
    sqlx::query!(
        r#"
        UPDATE savings_goals
        SET target_amount = ?
        WHERE goal_id = ?
        "#,
        new_amount_str,
        goal_id
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_all_saving_goals(pool: &Pool<Sqlite>) -> Result<Vec<SavingsGoal>, sqlx::Error> {
    sqlx::query(
        r#"
        SELECT 
            goal_id, 
            account_id, 
            goal_name, 
            target_amount, 
            current_amount, 
            deadline
        FROM savings_goals
        ORDER BY deadline ASC
        "#
    )
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|row| {
        let target_str: String = row.get("target_amount");
        let current_str: String = row.get("current_amount");

        let target_dec = Decimal::from_str(&target_str).unwrap_or(Decimal::ZERO);
        let current_dec = Decimal::from_str(&current_str).unwrap_or(Decimal::ZERO);

        Ok(SavingsGoal {
            goal_id: row.get("goal_id"),
            account_id: row.get("account_id"),
            goal_name: row.get("goal_name"),
            target_amount: target_dec,
            current_amount: current_dec,
            deadline: row.get("deadline"), 
        })
    })
    .collect::<Result<Vec<SavingsGoal>, sqlx::Error>>()
}

// ====================currency Queries======================
pub async fn get_rate(
    pool: &Pool<Sqlite>,
    currency: &str
) -> Result<Decimal, sqlx::Error> {

    let row = sqlx::query(
        r#"
        SELECT rate_to_base
        FROM currency_rates
        WHERE currency = ?
        "#
    )
    .bind(currency)
    .fetch_one(pool)
    .await?;

    let rate_str: String = row.get("rate_to_base");

    Decimal::from_str(&rate_str)
        .map_err(|e| sqlx::Error::Decode(Box::new(e)))
}

pub async fn insert_rate(
    pool: &Pool<Sqlite>,
    currency: &str,
    rate: f64
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO currency_rates (currency, rate_to_base)
        VALUES (?, ?)
        ON CONFLICT(currency)
        DO UPDATE SET rate_to_base = excluded.rate_to_base
        "#
    )
    .bind(currency)
    .bind(rate)
    .execute(pool)
    .await?;

    Ok(())
}

// ====================reports Queries======================
pub async fn monthly_summary(
    pool: &Pool<Sqlite>,
    start: &NaiveDateTime,
    end: &NaiveDateTime
) -> Result<Vec<(i64, f64)>, sqlx::Error> {
    let rows = sqlx::query(
        r#"
        SELECT category_id, SUM(base_amount) AS total
        FROM transactions
        WHERE trans_create_at BETWEEN ? AND ?
        GROUP BY category_id
        "#
    )
    .bind(start)
    .bind(end)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| {
            let category_id: i64 = r.get("category_id");
            let total: f64 = r.get("total");
            (category_id, total)
        })
        .collect())
}

pub async fn net_savings(pool: &Pool<Sqlite>) -> Result<f64, sqlx::Error> {
    let row = sqlx::query!(
        r#"
        SELECT 
            ROUND(COALESCE(SUM(base_amount), 0.0), 2) AS net
        FROM transactions
        "#
    )
    .fetch_one(pool)
    .await?;
    
    Ok(row.net.unwrap_or(0.0))
}
pub async fn seed_fixed_categories(pool: &Pool<Sqlite>) -> Result<(), sqlx::Error> {
    let categories = vec![
        (1, "Salary", "Income"),
        (2, "Bonus", "Income"),
        (3, "Investment", "Income"),
        (4, "Food", "Expense"),
        (5, "Transport", "Expense"),
        (6, "Rent", "Expense"),
        (7, "Shopping", "Expense"),
        (8, "Utilities", "Expense"),
        (9, "Entertainment", "Expense"),
        (10, "Health", "Expense"),
        (11, "Education", "Expense"),
        (12, "Travel", "Expense"),
    ];

    for (id, name, cat_type) in categories {
        sqlx::query(
            r#"
            INSERT OR IGNORE INTO categories
            (category_id, category_name, category_type, icon)
            VALUES (?, ?, ?, '')
            "#
        )
        .bind(id)
        .bind(name)
        .bind(cat_type)
        .execute(pool)
        .await?;
    }
    Ok(())
}

pub async fn get_category_spending_report(
    pool: &Pool<Sqlite>,
    start_date: NaiveDateTime,
    end_date: NaiveDateTime,
) -> Result<Vec<CategorySpending>, sqlx::Error> {
    let rows = sqlx::query(
        r#"
        SELECT 
            c.category_name, 
            SUM(CAST(t.amount AS REAL)) as total
        FROM transactions t
        JOIN categories c ON t.category_id = c.category_id
        WHERE t.is_expense = 1 
          AND t.transacted_at >= ? 
          AND t.transacted_at <= ?
        GROUP BY c.category_name
        ORDER BY total DESC
        "#
    )
    .bind(start_date)
    .bind(end_date)
    .fetch_all(pool)
    .await?;

    let result = rows.into_iter().map(|row| {
        let total: f64 = row.try_get("total").unwrap_or(0.0);
        let category: String = row.get("category_name");

        CategorySpending {
            category,
            total_amount: Decimal::from_f64_retain(total).unwrap_or(Decimal::ZERO),
        }
    }).collect();

    Ok(result)
}