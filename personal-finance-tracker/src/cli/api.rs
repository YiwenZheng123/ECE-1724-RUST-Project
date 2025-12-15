use anyhow::Result;
use chrono::{NaiveDate, NaiveDateTime};
use rust_decimal::Decimal;
use sqlx::{sqlite::SqlitePoolOptions, Pool, Row, Sqlite};

use super::state::{
    AccountDto, AccountType, CategoryDto, CategoryType,
    CreateAccountReq, CreateTxnReq, Money, TransactionDto, SavingGoalDto, CategorySpendingDto,
};

#[derive(Clone)]
pub struct Client {
    pool: Pool<Sqlite>,
}

pub struct CreateGoalReq {
    pub account_id: i64, 
    pub name: String,
    pub target_amount: super::state::Money,
    pub current_amount: super::state::Money,
    pub deadline: Option<chrono::NaiveDateTime>,
}

impl Client {
    pub async fn sqlite(db_url: &str) -> Result<Self> {
        
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(db_url)
            .await?;

        sqlx::query("PRAGMA journal_mode = DELETE;")
            .execute(&pool)
            .await?;


        Ok(Self { pool })
    }
    pub async fn delete_goal(&self, id: i64) -> anyhow::Result<()> {
        sqlx::query("DELETE FROM savings_goals WHERE goal_id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub fn pool(&self) -> &Pool<Sqlite> {
        &self.pool
    }

    pub async fn update_goal(&self, id: i64, req: &CreateGoalReq) -> anyhow::Result<()> {
        let amount_str = req.target_amount.0.to_string();
        let current_str = req.current_amount.0.to_string();
        let deadline_str = req.deadline.map(|d| d.format("%Y-%m-%d %H:%M:%S").to_string());

        sqlx::query(
            r#"
            UPDATE savings_goals 
            SET goal_name=?, target_amount=?, current_amount=?, deadline=?
            WHERE goal_id=?
            "#
        )
        .bind(&req.name)
        .bind(&amount_str)
        .bind(&current_str)
        .bind(deadline_str)
        .bind(id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn create_goal(&self, req: &CreateGoalReq) -> anyhow::Result<()> {
        let amount_str = req.target_amount.0.to_string();
        let current_str = req.current_amount.0.to_string();
        
        // Handle deadline: if None, insert NULL, otherwise insert string
        let deadline_str = req.deadline.map(|d| d.format("%Y-%m-%d %H:%M:%S").to_string());

        sqlx::query(
            r#"
            INSERT INTO savings_goals (account_id, goal_name, target_amount, current_amount, deadline)
            VALUES (?, ?, ?, ?, ?)
            "#
        )
        .bind(req.account_id)
        .bind(&req.name)
        .bind(&amount_str)
        .bind(&current_str)
        .bind(deadline_str)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
   
    // Accounts
    pub async fn list_accounts(&self) -> Result<Vec<AccountDto>> {
        let rows = sqlx::query("SELECT account_id, account_name, account_type, currency, balance, account_created_at FROM accounts ORDER BY account_id")
            .fetch_all(&self.pool).await?;

        let mut list = Vec::new();
        for r in rows {
             list.push(AccountDto {
                id: r.try_get("account_id")?,
                name: r.try_get("account_name")?,
                r#type: map_account_type(&r.try_get::<String, _>("account_type")?),
                currency: r.try_get("currency")?,
                opening_balance: Money(Decimal::from_str_exact(&r.try_get::<String, _>("balance")?).unwrap_or(Decimal::ZERO)),
                created_at: r.try_get("account_created_at")?,
            });
        }
        Ok(list)
    }

    async fn recompute_balance_exec<'e, E>(&self, executor: E, account_id: i64) -> anyhow::Result<()>
    where
        E: sqlx::Executor<'e, Database = Sqlite>,
    {
        sqlx::query(
            r#"
            UPDATE accounts
            SET balance = IFNULL((
                SELECT ROUND(SUM(
                    CASE WHEN t.is_expense = 1 THEN -CAST(t.amount AS NUMERIC)
                        ELSE CAST(t.amount AS NUMERIC)
                    END
                ), 2)
                FROM transactions t
                WHERE t.account_id = ?
            ), 0.00)
            WHERE account_id = ?
            "#
        )
        .bind(account_id)
        .bind(account_id)
        .execute(executor)
        .await?;
        Ok(())
    }

    pub async fn create_account(&self, req: &CreateAccountReq) -> Result<AccountDto> {
        let mut tx = self.pool.begin().await?;
        
        let row = sqlx::query("INSERT INTO accounts (account_name, account_type, balance, currency, account_created_at) VALUES (?, ?, '0', ?, strftime('%Y-%m-%dT%H:%M:%SZ','now')) RETURNING account_id, account_name, account_type, currency, account_created_at")
            .bind(&req.name)
            .bind(req.r#type.as_str())
            .bind(&req.currency)
            .fetch_one(&mut *tx).await?;
            
        let aid: i64 = row.try_get("account_id")?;
        
        if !req.opening_balance.0.is_zero() {
             let amount_str = req.opening_balance.0.to_string();
             
             let cat_id_row = sqlx::query("SELECT category_id FROM categories WHERE category_name = 'Initial Balance'")
                .fetch_optional(&mut *tx).await?;
                
             let cat_id: i64 = match cat_id_row {
                 Some(r) => r.try_get("category_id")?,
                 None => {
                     let r = sqlx::query("INSERT INTO categories (category_name, category_type, icon) VALUES ('Initial Balance', 'INCOME', '💰') RETURNING category_id")
                        .fetch_one(&mut *tx).await?;
                     r.try_get("category_id")?
                 }
             };

             sqlx::query("INSERT INTO transactions (account_id, category_id, amount, base_amount, is_expense, description, currency, transacted_at, trans_create_at) VALUES (?, ?, ?, ?, 0, 'Opening Balance', ?, strftime('%Y-%m-%dT%H:%M:%SZ','now'), strftime('%Y-%m-%dT%H:%M:%SZ','now'))")
                .bind(aid)
                .bind(cat_id)
                .bind(&amount_str)
                .bind(&amount_str)
                .bind(&req.currency)
                .execute(&mut *tx).await.ok(); 
        }
        
        self.recompute_balance_exec(&mut *tx, aid).await?;
        tx.commit().await?;

        Ok(AccountDto {
            id: aid,
            name: row.try_get("account_name")?,
            r#type: map_account_type(&row.try_get::<String, _>("account_type")?),
            currency: row.try_get("currency")?,
            opening_balance: req.opening_balance.clone(),
            created_at: row.try_get("account_created_at")?,
        })
    }

    pub async fn update_account(&self, id: i64, name: &str, atype: &str, currency: &str) -> Result<()> {
        sqlx::query("UPDATE accounts SET account_name = ?, account_type = ?, currency = ? WHERE account_id = ?")
            .bind(name).bind(atype).bind(currency).bind(id)
            .execute(&self.pool).await?;
        Ok(())
    }

    pub async fn delete_account(&self, id: i64) -> Result<()> {
        let mut tx = self.pool.begin().await?;
        sqlx::query("DELETE FROM transactions WHERE account_id = ?").bind(id).execute(&mut *tx).await?;
        sqlx::query("DELETE FROM accounts WHERE account_id = ?").bind(id).execute(&mut *tx).await?;
        tx.commit().await?;
        Ok(())
    }

    // ================= Categories =================
    pub async fn list_categories(&self) -> Result<Vec<CategoryDto>> {
         let rows = sqlx::query("SELECT category_id, category_name, category_type, icon FROM categories ORDER BY category_name")
            .fetch_all(&self.pool).await?;
            
         let mut out = Vec::new();
         for r in rows {
             out.push(CategoryDto {
                 id: r.try_get("category_id")?,
                 name: r.try_get("category_name")?,
                 r#type: if r.try_get::<String, _>("category_type")?.eq_ignore_ascii_case("INCOME") { CategoryType::Income } else { CategoryType::Expense },
                 icon: r.try_get("icon")?,
             });
         }
         Ok(out)
    }

    // ================= Transactions =================
    pub async fn list_transactions(&self, account_id: i64, limit: Option<i64>, offset: Option<i64>) -> Result<Vec<TransactionDto>> {
        let rows = sqlx::query(
            r#"
            SELECT
              t.transaction_id,
              t.account_id,
              t.category_id,
              t.amount,
              t.is_expense,
              t.description,
              t.payee,
              t.currency,
              t.transacted_at
            FROM transactions t
            WHERE t.account_id = ?
            ORDER BY t.transacted_at DESC, t.transaction_id DESC
            LIMIT ? OFFSET ?
            "#
        )
        .bind(account_id)
        .bind(limit.unwrap_or(200))
        .bind(offset.unwrap_or(0))
        .fetch_all(&self.pool)
        .await?;

        let mut out = Vec::with_capacity(rows.len());
        for r in rows {
            let id: i64 = r.try_get("transaction_id")?;
            let amount_s: String = r.try_get("amount")?;
            let is_expense_i: i64 = r.try_get("is_expense")?;
            let txn_date_s: String = r.try_get("transacted_at")?;

            let mut amt = Decimal::from_str_exact(&amount_s).unwrap_or(Decimal::ZERO);
            if is_expense_i != 0 { amt = -amt; }

            out.push(TransactionDto {
                id,
                account_id: r.try_get("account_id")?,
                category_id: r.try_get("category_id")?,
                amount: Money(amt),
                memo: r.try_get("description")?,
                payee: r.try_get("payee")?,
                currency: r.try_get::<Option<String>, _>("currency")?.unwrap_or("CAD".into()),
                txn_date: parse_date_any(&txn_date_s),
                cleared: false,
                reconciled: false,
            });
        }
        Ok(out)
    }

    pub async fn create_transaction(&self, req: &CreateTxnReq) -> Result<TransactionDto> {
        let is_expense = if req.amount.0.is_sign_negative() { 1 } else { 0 };
        let amount_abs = req.amount.0.abs().to_string();
        let date_str = req.transacted_at.format("%Y-%m-%dT%H:%M:%S").to_string();

        let mut tx = self.pool.begin().await?;

        let row = sqlx::query(
            r#"
            INSERT INTO transactions
              (account_id, category_id, amount, base_amount, is_expense, description, payee, currency, transacted_at, trans_create_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, strftime('%Y-%m-%dT%H:%M:%SZ','now'))
            RETURNING
              transaction_id, account_id, category_id, amount, is_expense, description, payee, currency, transacted_at
            "#
        )
        .bind(req.account_id)
        .bind(req.category_id)
        .bind(&amount_abs)
        .bind(&amount_abs)
        .bind(is_expense)
        .bind(req.description.as_deref())
        .bind(req.payee.as_deref())
        .bind(&req.currency)
        .bind(&date_str)
        .fetch_one(&mut *tx)
        .await?;

        self.recompute_balance_exec(&mut *tx, req.account_id).await?;
        tx.commit().await?;

        let amount_s: String = row.try_get("amount")?;
        let is_expense_i: i64 = row.try_get("is_expense")?;
        let mut amt = Decimal::from_str_exact(&amount_s).unwrap_or(Decimal::ZERO);
        if is_expense_i != 0 { amt = -amt; }

        Ok(TransactionDto {
            id: row.try_get("transaction_id")?,
            account_id: row.try_get("account_id")?,
            category_id: row.try_get("category_id")?,
            amount: Money(amt),
            memo: row.try_get("description")?,
            payee: row.try_get("payee")?,
            currency: row.try_get::<Option<String>, _>("currency")?.unwrap_or("CAD".into()),
            txn_date: parse_date_any(&row.try_get::<String, _>("transacted_at")?),
            cleared: false,
            reconciled: false,
        })
    }
    
    pub async fn update_transaction(&self, id: i64, req: &CreateTxnReq) -> Result<()> {
        let is_expense = if req.amount.0.is_sign_negative() { 1 } else { 0 };
        let amount_abs = req.amount.0.abs().to_string();
        let mut tx = self.pool.begin().await?;
        
        sqlx::query("UPDATE transactions SET category_id = ?, amount = ?, base_amount = ?, is_expense = ?, description = ?, payee = ?, currency = ?, transacted_at = ? WHERE transaction_id = ?")
            .bind(req.category_id)
            .bind(&amount_abs)
            .bind(&amount_abs)
            .bind(is_expense)
            .bind(&req.description)
            .bind(&req.payee)
            .bind(&req.currency)
            .bind(&req.transacted_at)
            .bind(id)
            .execute(&mut *tx).await?;
            
        self.recompute_balance_exec(&mut *tx, req.account_id).await?;
        tx.commit().await?;
        Ok(())
    }
    
    pub async fn delete_transaction(&self, transaction_id: i64) -> anyhow::Result<()> {
        let mut tx = self.pool.begin().await?;
        
        let row = sqlx::query("SELECT account_id FROM transactions WHERE transaction_id = ?")
            .bind(transaction_id)
            .fetch_one(&mut *tx).await?;
        let aid: i64 = row.try_get("account_id")?;
        
        sqlx::query("DELETE FROM transactions WHERE transaction_id = ?")
            .bind(transaction_id)
            .execute(&mut *tx).await?;
            
        self.recompute_balance_exec(&mut *tx, aid).await?;
        tx.commit().await?;
        Ok(())
    }

    pub async fn list_goals(&self) -> Result<Vec<SavingGoalDto>> {
        let rows = sqlx::query("SELECT goal_id, goal_name, target_amount, current_amount, deadline FROM savings_goals ORDER BY deadline ASC").fetch_all(&self.pool).await?;
        let mut out = Vec::new();
        for r in rows {
            out.push(SavingGoalDto {
                id: r.try_get("goal_id")?,
                name: r.try_get("goal_name")?,
                target_amount: Money(Decimal::from_str_exact(&r.try_get::<String, _>("target_amount")?).unwrap_or(Decimal::ZERO)),          
                current_amount: Money(Decimal::from_str_exact(&r.try_get::<String, _>("current_amount")?).unwrap_or(Decimal::ZERO)),
                deadline: r.try_get("deadline")?,
            });
        }
        Ok(out)
    }

    pub async fn get_monthly_report(&self) -> Result<Vec<CategorySpendingDto>> {
        let now = chrono::Utc::now();
        let start_str = format!("{}-01 00:00:00", now.format("%Y-%m")); 
        let end_str = now.format("%Y-%m-%d %H:%M:%S").to_string();
        let rows = sqlx::query(
            r#"
            SELECT 
                c.category_name, 
                SUM(ABS(CAST(t.amount AS REAL))) as total
            FROM transactions t 
            JOIN categories c ON t.category_id = c.category_id 
            WHERE t.is_expense = 1 
                AND t.transacted_at >= ?1
                AND t.transacted_at <= ?2
            GROUP BY c.category_name 
            ORDER BY total DESC
            "#
        ).bind(start_str)
        .bind(end_str)
        .fetch_all(&self.pool)
        .await?;
        let mut out = Vec::new();
        for r in rows {
            out.push(CategorySpendingDto {
                category: r.try_get("category_name")?,
                total_amount: Money(Decimal::from_f64_retain(r.try_get("total")?).unwrap_or(Decimal::ZERO)),
            });
        }
        Ok(out)
    }

} 

// Helpers
fn map_account_type(s: &str) -> AccountType {
    if s.eq_ignore_ascii_case("checking") { AccountType::Checking } 
    else if s.eq_ignore_ascii_case("credit") { AccountType::Credit }
    else if s.eq_ignore_ascii_case("savings") { AccountType::Savings }
    else if s.eq_ignore_ascii_case("cash") { AccountType::Cash }
    else { AccountType::Other }
}

fn parse_date_any(s: &str) -> NaiveDate {
    if let Ok(d) = NaiveDate::parse_from_str(s, "%Y-%m-%d") { return d; }
    if let Ok(dt) = NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S") { return dt.date(); }
    if let Ok(dt) = NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%SZ") { return dt.date(); }
    if let Ok(dt) = NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S") { return dt.date(); }
    if let Ok(d) = NaiveDate::parse_from_str(s, "%Y/%m/%d") { return d; }
    chrono::Utc::now().date_naive()
}