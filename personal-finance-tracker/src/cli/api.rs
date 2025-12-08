use anyhow::Result;
use chrono::NaiveDate;
use rust_decimal::Decimal;
use sqlx::{sqlite::SqlitePoolOptions, Pool, Row, Sqlite};

use super::state::{
    AccountDto, AccountType, BalanceDto, CategoryDto, CategoryType,
    CreateAccountReq, CreateTxnReq, Money, TransactionDto,
};

#[derive(Clone)]
pub struct Client {
    pool: Pool<Sqlite>,
}

impl Client {
    pub async fn sqlite(db_url: &str) -> Result<Self> {
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(db_url)
            .await?;
        Ok(Self { pool })
    }

    // ============= Accounts =============

    pub async fn list_accounts(&self) -> Result<Vec<AccountDto>> {
        let rows = sqlx::query(
            r#"
            SELECT
              account_id         AS id,
              account_name       AS name,
              account_type       AS atype,
              currency           AS currency,
              balance            AS balance,
              account_created_at AS created_at
            FROM accounts
            ORDER BY account_id
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        let mut list = Vec::with_capacity(rows.len());
        for r in rows {
            let id: i64 = r.try_get("id")?;
            let name: String = r.try_get("name")?;
            let atype: String = r.try_get("atype")?;
            let currency: String = r.try_get("currency")?;
            let balance_s: String = r.try_get("balance")?;
            let created_at: String = r.try_get("created_at")?;

            list.push(AccountDto {
                id,
                name,
                r#type: map_account_type(&atype),
                currency,
                opening_balance: Money(Decimal::from_str_exact(&balance_s).unwrap_or(Decimal::ZERO)),
                created_at,
            });
        }
        Ok(list)
    }

    pub async fn create_account(&self, req: &CreateAccountReq) -> Result<AccountDto> {
        let row = sqlx::query(
            r#"
            INSERT INTO accounts (account_name, account_type, balance, currency, account_created_at)
            VALUES (?, ?, ?, ?, strftime('%Y-%m-%dT%H:%M:%SZ','now'))
            RETURNING
              account_id         AS id,
              account_name       AS name,
              account_type       AS atype,
              currency           AS currency,
              balance            AS balance,
              account_created_at AS created_at
            "#,
        )
        .bind(&req.name)
        .bind(req.r#type.as_str())
        .bind(req.opening_balance.0.to_string())
        .bind(&req.currency)
        .fetch_one(&self.pool)
        .await?;

        Ok(AccountDto {
            id: row.try_get("id")?,
            name: row.try_get("name")?,
            r#type: map_account_type(&row.try_get::<String, _>("atype")?),
            currency: row.try_get("currency")?,
            opening_balance: Money(Decimal::from_str_exact(&row.try_get::<String, _>("balance")?).unwrap_or(Decimal::ZERO)),
            created_at: row.try_get("created_at")?,
        })
    }

    pub async fn balance(&self, account_id: i64) -> Result<BalanceDto> {
        let opening: Option<String> = sqlx::query_scalar(
            "SELECT balance FROM accounts WHERE account_id = ?",
        )
        .bind(account_id)
        .fetch_optional(&self.pool)
        .await?;

        let bal = opening
            .and_then(|s| Decimal::from_str_exact(&s).ok())
            .unwrap_or(Decimal::ZERO);

        Ok(BalanceDto { account_id, balance: Money(bal) })
    }

    // ============= Categories =============

    pub async fn list_categories(&self) -> Result<Vec<CategoryDto>> {
        let rows = sqlx::query(
            r#"
            SELECT
              category_id   AS id,
              category_name AS name,
              category_type AS ctype,
              icon          AS icon
            FROM categories
            ORDER BY category_name
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        let mut out = Vec::with_capacity(rows.len());
        for r in rows {
            let id: i64 = r.try_get("id")?;
            let name: String = r.try_get("name")?;
            let ctype: String = r.try_get("ctype")?;
            let icon: String = r.try_get("icon")?;

            out.push(CategoryDto {
                id,
                name,
                r#type: if ctype.eq_ignore_ascii_case("INCOME") { CategoryType::Income } else { CategoryType::Expense },
                icon,
            });
        }
        Ok(out)
    }

    // ============= Transactions =============

    pub async fn list_transactions(
        &self,
        account_id: i64,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<Vec<TransactionDto>> {
        let rows = sqlx::query(
            r#"
            SELECT
              t.transaction_id AS id,
              t.account_id     AS account_id,
              t.category_id    AS category_id,
              t.amount         AS amount,
              t.is_expense     AS is_expense,
              t.description    AS memo,
              t.currency       AS currency,
              t.transacted_at  AS txn_date
            FROM transactions t
            WHERE t.account_id = ?
              AND (? IS NULL OR t.transacted_at >= ?)
              AND (? IS NULL OR t.transacted_at <= ?)
            ORDER BY t.transacted_at DESC, t.transaction_id DESC
            LIMIT COALESCE(?, 200) OFFSET COALESCE(?, 0)
            "#,
        )
        .bind(account_id)
        .bind(limit).bind(offset)
        .fetch_all(&self.pool)
        .await?;

        let mut out = Vec::with_capacity(rows.len());
        for r in rows {
            let id: i64 = r.try_get("id")?;
            let acc: i64 = r.try_get("account_id")?;
            let cat: Option<i64> = r.try_get("category_id")?;
            let amount_s: String = r.try_get("amount")?;
            let is_expense_i: i64 = r.try_get("is_expense")?;
            let memo: Option<String> = r.try_get("memo")?;
            let currency: Option<String> = r.try_get("currency")?;
            let txn_date_s: String = r.try_get("txn_date")?;

            let mut amt = Decimal::from_str_exact(&amount_s).unwrap_or(Decimal::ZERO);
            if is_expense_i != 0 { amt = -amt; }

            out.push(TransactionDto {
                id,
                account_id: acc,
                category_id: cat,
                amount: Money(amt),
                memo,
                currency: currency.unwrap_or_else(|| "CAD".into()),
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

        let row = sqlx::query(
            r#"
            INSERT INTO transactions
              (account_id, category_id, amount, is_expense, description, currency, transacted_at, trans_create_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, strftime('%Y-%m-%dT%H:%M:%SZ','now'))
            RETURNING
              transaction_id AS id,
              account_id     AS account_id,
              category_id    AS category_id,
              amount         AS amount,
              is_expense     AS is_expense,
              description    AS memo,
              currency       AS currency,
              transacted_at  AS txn_date
            "#,
        )
        .bind(req.account_id)
        .bind(req.category_id)
        .bind(amount_abs)
        .bind(is_expense)
        .bind(req.memo.as_deref()) 
        .bind(&req.currency)
        .bind(req.txn_date.format("%Y-%m-%d").to_string())
        .fetch_one(&self.pool)
        .await?;

        let amount_s: String = row.try_get("amount")?;
        let is_expense_i: i64 = row.try_get("is_expense")?;
        let mut amt = Decimal::from_str_exact(&amount_s).unwrap_or(Decimal::ZERO);
        if is_expense_i != 0 { amt = -amt; }

        Ok(TransactionDto {
            id: row.try_get("id")?,
            account_id: row.try_get("account_id")?,
            category_id: row.try_get("category_id")?,
            amount: Money(amt),
            memo: row.try_get("memo")?,
            currency: row.try_get::<Option<String>, _>("currency")?.unwrap_or_else(|| "CAD".into()),
            txn_date: parse_date_any(&row.try_get::<String, _>("txn_date")?),
            cleared: false,
            reconciled: false,
        })
    }


    pub async fn delete_transaction(&self, transaction_id: i64) -> anyhow::Result<()> {
    sqlx::query!(
        r#"DELETE FROM transactions WHERE transaction_id = ?"#,
        transaction_id
    )
    .execute(&self.pool)
    .await?;

    sqlx::query!(
        r#"
        UPDATE accounts
        SET balance = IFNULL((
          SELECT ROUND(SUM(
            CASE WHEN t.is_expense = 1
                 THEN -CAST(t.amount AS NUMERIC)
                 ELSE  CAST(t.amount AS NUMERIC)
            END
          ), 2)
          FROM transactions t
          WHERE t.account_id = accounts.account_id
        ), 0.00)
        "#
    )
    .execute(&self.pool)
    .await?;

    Ok(())
}
}




/* ========== 辅助函数 ========== */

fn map_account_type(s: &str) -> AccountType {
    if s.eq_ignore_ascii_case("checking") {
        AccountType::Checking
    } else if s.eq_ignore_ascii_case("credit") {
        AccountType::Credit
    } else if s.eq_ignore_ascii_case("savings") {
        AccountType::Savings
    } else if s.eq_ignore_ascii_case("cash") {
        AccountType::Cash
    } else {
        AccountType::Other
    }
}

fn parse_date_any(s: &str) -> NaiveDate {
    NaiveDate::parse_from_str(s, "%Y-%m-%d")
        .or_else(|_| NaiveDate::parse_from_str(s, "%Y/%m/%d"))
        .unwrap_or_else(|_| chrono::Utc::now().date_naive())
}
