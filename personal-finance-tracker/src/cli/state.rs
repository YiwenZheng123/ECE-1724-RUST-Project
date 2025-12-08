use chrono::NaiveDate;
use ratatui::widgets::{ListState, TableState};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
use crate::cli::api::Client;


#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AccountType {
    Checking,
    Credit,
    Savings,
    Cash,
    Other,
}
impl Default for AccountType {
    fn default() -> Self {
        AccountType::Cash
    }
}
impl AccountType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Checking => "CHECKING",
            Self::Credit => "CREDIT",
            Self::Savings => "SAVINGS",
            Self::Cash => "CASH",
            Self::Other => "OTHER",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Money(#[serde(with = "rust_decimal::serde::str")] pub Decimal);
impl Money {
    pub fn zero() -> Self {
        Self(Decimal::ZERO)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountDto {
    pub id: i64,
    pub name: String,
    pub r#type: AccountType,
    pub currency: String,
    pub opening_balance: Money,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CategoryType {
    Income,
    Expense,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryDto {
    pub id: i64,
    pub name: String,
    pub r#type: CategoryType,
    pub icon: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionDto {
    pub id: i64,
    pub account_id: i64,
    pub category_id: Option<i64>,
    pub amount: Money, 
    pub memo: Option<String>,
    pub currency: String,
    pub txn_date: NaiveDate,
    pub cleared: bool,
    pub reconciled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAccountReq {
    pub name: String,
    pub r#type: AccountType,
    pub currency: String,
    pub opening_balance: Money,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTxnReq {
    pub account_id: i64,
    pub category_id: Option<i64>,
    pub amount: Money, 
    pub memo: Option<String>,
    pub currency: String,
    pub txn_date: NaiveDate, 
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BalanceDto {
    pub account_id: i64,
    pub balance: Money,
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Accounts,
    Transactions,
    AddTxn,
    Help,
}


#[derive(Default, Clone)]
pub struct AccountForm {
    pub name: String,
    pub currency: String,
    pub opening: String,
    pub r#type: AccountType,
    pub error: Option<String>,
}

#[derive(Default)]
pub struct AccountsPage {
    pub list: Vec<AccountDto>,
    pub sel: ListState,
    pub creating: bool,
    pub form: AccountForm,
}

// Transactions 
#[derive(Default)]
pub struct TxnPage {
    pub account_id: Option<i64>,
    pub table: Vec<TransactionDto>,
    pub tsel: TableState,
    pub loading: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditField {
    Payee,
    Memo,
    Amount,
    Date,
}

// Add Transaction Page 
#[derive(Default, Clone)]
pub struct AddTxnForm {
    pub account_id: Option<i64>,
    pub date: String, 
    pub payee: String, 
    pub memo: String,
    pub categories: Vec<CategoryDto>,
    pub cat_sel: ListState,
    pub amount: String,
    pub is_expense: bool, 
    pub error: Option<String>,
    pub success: Option<String>,
    pub just_entered: bool,
    pub editing: Option<EditField>,
}

// App

pub struct App {
    pub api: Client,
    pub tab: Tab,
    pub status: String,
    pub quit: bool,

    pub accounts: AccountsPage,
    pub txn: TxnPage,
    pub add: AddTxnForm,
}

impl App {
    pub fn new(api: Client) -> Self {
        let today = chrono::Utc::now().date_naive();
        let mut add = AddTxnForm::default();
        add.date = today.format("%Y-%m-%d").to_string();

        Self {
            api,
            tab: Tab::Accounts,
            status: "Press ? for help | q to quit".into(),
            quit: false,
            accounts: AccountsPage::default(),
            txn: TxnPage::default(),
            add,
        }
    }


    pub async fn refresh_accounts(&mut self) -> anyhow::Result<()> {
        let data = self.api.list_accounts().await?;
        self.accounts.list = data;
        if self.accounts.sel.selected().is_none() && !self.accounts.list.is_empty() {
            self.accounts.sel.select(Some(0));
        }
        Ok(())
    }

    pub async fn refresh_txns(&mut self) -> anyhow::Result<()> {
        if let Some(aid) = self.current_account_id() {
            self.txn.loading = true;

            let rows = self.api.list_transactions(aid, Some(200), Some(0)).await?;

            self.txn.table = rows;
            if self.txn.tsel.selected().is_none() && !self.txn.table.is_empty() {
                self.txn.tsel.select(Some(0));
            }
            self.txn.loading = false;
        }
        Ok(())
        }

    pub async fn load_categories(&mut self) {
        if self.add.categories.is_empty() {
            if let Ok(list) = self.api.list_categories().await {
                self.add.categories = list;
            }
        }
    }


    pub fn ensure_cat_selected(&mut self) {
        // Initialize only once when you first enter AddTxn
        if !self.add.just_entered {
            return;
        }

        if self.add.categories.is_empty() {
            // If there is no category, do not select any items
            self.add.cat_sel.select(None);
        } else if self.add.cat_sel.selected().is_none() {
            // If there is a category but it hasn't been selected yet, choose the 0th one
            self.add.cat_sel.select(Some(0));
        }

        self.add.just_entered = false;
    }

    pub fn clamp_cat_selection(&mut self) {
        let len = self.add.categories.len();
        match (len, self.add.cat_sel.selected()) {
            (0, _) => self.add.cat_sel.select(None),
            (n, Some(i)) if i >= n => self.add.cat_sel.select(Some(n - 1)),
            _ => {}
        }
    }

    pub fn move_cat(&mut self, delta: i32) {
        let len = self.add.categories.len();
        if len == 0 {
            self.add.cat_sel.select(None);
            return;
        }
        let cur = self.add.cat_sel.selected().unwrap_or(0) as i32;
        let new = (cur + delta).rem_euclid(len as i32) as usize;
        self.add.cat_sel.select(Some(new));
    }

    pub fn current_account(&self) -> Option<&AccountDto> {
        let idx = self.accounts.sel.selected()?;
        self.accounts.list.get(idx)
    }
    pub fn current_account_id(&self) -> Option<i64> {
        self.current_account().map(|a| a.id)
    }

    fn current_txn_index(&self) -> Option<usize> {
        self.txn.tsel.selected()
    }
    fn current_txn_id(&self) -> Option<i64> {
        let idx = self.current_txn_index()?;
        self.txn.table.get(idx).map(|t| t.id)
    }
    fn move_txn(&mut self, delta: isize) {
        let n = self.txn.table.len();
        if n == 0 { 
            self.txn.tsel.select(None);
            return; 
        }
        let cur = self.txn.tsel.selected().unwrap_or(0) as isize;
        let next = (cur + delta).rem_euclid(n as isize) as usize;
        self.txn.tsel.select(Some(next));
    }



    fn next_field(&self, f: EditField) -> EditField {
        use EditField::*;
        match f { Payee => Amount, Amount => Memo, Memo => Date, Date => Payee }
    }
    fn prev_field(&self, f: EditField) -> EditField {
        use EditField::*;
        match f { Payee => Date, Date => Memo, Memo => Amount, Amount => Payee }
    }

    pub async fn handle_key(&mut self, k: KeyEvent) -> anyhow::Result<()> {
        if k.kind != KeyEventKind::Press {
        return Ok(());
        }
        match self.tab {
        Tab::Accounts => match k.code {
            KeyCode::Up => self.move_account(-1),
            KeyCode::Down => self.move_account(1),
            KeyCode::Enter => {
                self.tab = Tab::Transactions;
                self.txn.account_id = self.current_account_id();
                self.refresh_txns().await.ok();
            }
            KeyCode::Char('n') => {
                self.accounts.creating = true;
                self.accounts.form = AccountForm {
                    currency: "CAD".into(),
                    r#type: AccountType::Cash,
                    ..Default::default()
                };
            }
            KeyCode::Char('r') => {
                self.refresh_accounts().await.ok();
            }
            KeyCode::Esc => {
                self.accounts.creating = false;
                self.accounts.form.error = None;
            }
            KeyCode::Char('?') => self.tab = Tab::Help,
            _ => {}
        },

        Tab::Transactions => match k.code {
            KeyCode::Up   => self.move_txn(-1),
            KeyCode::Down => self.move_txn(1),
            KeyCode::Char('a') => {
                self.tab = Tab::AddTxn;
                self.add.account_id = self.current_account_id();
                self.add.just_entered = true;
                self.load_categories().await;
                self.ensure_cat_selected(); 
            }
            KeyCode::Char('r') => {
                self.refresh_txns().await.ok();
            }
            KeyCode::Char('x') | KeyCode::Delete => {
                if let Some(id) = self.current_txn_id() {
                    if let Err(e) = self.api.delete_transaction(id).await {
                        self.status = format!("Delete failed: {e}");
                    } else {
                        let old_idx = self.txn.tsel.selected().unwrap_or(0);
                        self.refresh_txns().await.ok();
                        let n = self.txn.table.len();
                        if n == 0 {
                            self.txn.tsel.select(None);
                        } else {
                            self.txn.tsel.select(Some(old_idx.min(n - 1)));
                        }
                        self.status = "Deleted.".into();
                    }
                } else {
                    self.status = "No transaction selected.".into();
                }
            }
            KeyCode::Char('b') => self.tab = Tab::Accounts,
            KeyCode::Char('?') => self.tab = Tab::Help,
            _ => {}
        },

        Tab::AddTxn => {

            if let Some(field) = self.add.editing {
                match k.code {
                    KeyCode::Char(c) => {
                        match field {
                            EditField::Payee  => self.add.payee.push(c),
                            EditField::Memo   => self.add.memo.push(c),
                            EditField::Amount => self.add.amount.push(c),
                            EditField::Date   => self.add.date.push(c),
                        }
                    }
                    KeyCode::Backspace => {
                        let s = match field {
                            EditField::Payee  => &mut self.add.payee,
                            EditField::Memo   => &mut self.add.memo,
                            EditField::Amount => &mut self.add.amount,
                            EditField::Date   => &mut self.add.date,
                        };
                        s.pop();
                    }
                    KeyCode::Enter => {
                        self.add.editing = None;
                    }
                    KeyCode::Tab => { 
                        self.add.editing = Some(self.next_field(field));
                    }
                    KeyCode::BackTab => { 
                        self.add.editing = Some(self.prev_field(field));
                    }
                    KeyCode::Esc => {
                        self.add.editing = None;
                    }
                    _ => {}
                }
                return Ok(()); 
            }

            match k.code {
                KeyCode::Up   => self.move_cat(-1),
                KeyCode::Down => self.move_cat(1),
                // Enter does not commit in non-edit mode
                KeyCode::Enter => { }

                KeyCode::Esc | KeyCode::Char('b') => {
                    self.tab = Tab::Transactions;
                    self.add.error = None;
                    self.add.success = None;
                }
                // Switch +/- (income/expenditure)
                KeyCode::Char('t') => {
                    self.add.is_expense = !self.add.is_expense;
                }

                // Enter editing mode
                KeyCode::Char('p') => self.add.editing = Some(EditField::Payee),
                KeyCode::Char('a') => self.add.editing = Some(EditField::Amount),
                KeyCode::Char('m') => self.add.editing = Some(EditField::Memo),
                KeyCode::Char('d') => self.add.editing = Some(EditField::Date),

                KeyCode::Tab => {
                    let cur = self.add.editing.unwrap_or(EditField::Payee);
                    self.add.editing = Some(self.next_field(cur));
                }
                KeyCode::BackTab => {
                    let cur = self.add.editing.unwrap_or(EditField::Payee);
                    self.add.editing = Some(self.prev_field(cur));
                }

                KeyCode::Char('s') => {
                    self.submit_txn().await.ok();
                }

                KeyCode::Char('?') => self.tab = Tab::Help,

                _ => {}
            }

        }


        Tab::Help => match k.code {
            KeyCode::Esc | KeyCode::Char('b') => self.tab = Tab::Accounts,
            _ => {}
        },
    }
    Ok(())
}
    fn move_account(&mut self, delta: isize) {
        let n = self.accounts.list.len();
        if n == 0 {
            return;
        }
        let cur = self.accounts.sel.selected().unwrap_or(0) as isize;
        let next = (cur + delta).rem_euclid(n as isize) as usize;
        self.accounts.sel.select(Some(next));
    }
// submit

    pub async fn submit_txn(&mut self) -> anyhow::Result<()> {
        let acc = if let Some(id) = self.add.account_id {
            id
        } else {
            self.add.error = Some("Please choose account".into());
            return Ok(());
        };

        let amt = if self.add.amount.trim().is_empty() {
            self.add.error = Some("The amount can not be empty".into());
            return Ok(());
        } else {
            match Decimal::from_str_exact(self.add.amount.trim()) {
                Ok(d) => Money(d),
                Err(_) => {
                    self.add.error = Some("The amount format was wrong".into());
                    return Ok(());
                }
            }
        };
        let final_amt = if self.add.is_expense {
            Money(-amt.0.abs())
        } else {
            Money(amt.0.abs())
        };

        let date = if self.add.date.trim().is_empty() {
            chrono::Utc::now().date_naive()
        } else {
            match NaiveDate::parse_from_str(&self.add.date, "%Y-%m-%d") {
                Ok(d) => d,
                Err(_) => {
                    self.add.error = Some("Date formate should be YYYY-MM-DD".into());
                    return Ok(());
                }
            }
        };

        let cat_id = self
            .add
            .cat_sel
            .selected()
            .and_then(|i| self.add.categories.get(i))
            .map(|c| c.id);

        let req = CreateTxnReq {
            account_id: acc,
            category_id: cat_id,
            amount: final_amt,
            memo: if self.add.memo.trim().is_empty() {
                None
            } else {
                Some(self.add.memo.clone())
            },
            currency: self
                .current_account()
                .map(|a| a.currency.clone())
                .unwrap_or_else(|| "CAD".into()),
            txn_date: date,
        };

        match self.api.create_transaction(&req).await {
            Ok(_) => {
                self.add.success = Some("Save ✓".into());
                self.add.error = None;
                self.add.amount.clear();
                self.add.memo.clear();
                self.refresh_txns().await.ok();
            }
            Err(e) => {
                self.add.error = Some(format!("Fail to save：{e}"));
                self.add.success = None;
            }
        }

        Ok(())
    }
}
