// src/cli/state.rs
use chrono::NaiveDate;
use ratatui::widgets::{ListState, TableState};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
use crate::cli::api::Client;
use std::str::FromStr; 

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
pub struct Money(pub Decimal);
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

// Adapted to new structure: base_amount required, category_id required
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionDto {
    pub id: i64,
    pub account_id: i64,
    pub category_id: i64,     
    pub amount: Money,
    pub base_amount: Money,   
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

// Request body adapted
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTxnReq {
    pub account_id: i64,
    pub category_id: i64,     
    pub amount: Money,
    pub base_amount: Money,   
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
    // 0: Name, 1: Type, 2: Currency, 3: Opening
    pub focus_index: usize, 
    pub error: Option<String>,
}

#[derive(Default)]
pub struct AccountsPage {
    pub list: Vec<AccountDto>,
    pub sel: ListState,
    pub creating: bool,
    pub form: AccountForm,
}

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
        if !self.add.just_entered { return; }
        if self.add.categories.is_empty() {
            self.add.cat_sel.select(None);
        } else if self.add.cat_sel.selected().is_none() {
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
        if n == 0 { self.txn.tsel.select(None); return; }
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
        if k.kind != KeyEventKind::Press { return Ok(()); }
        if self.accounts.creating {
            self.handle_creation_input(k).await?;
            return Ok(());
        }

      
        if self.tab == Tab::AddTxn && self.add.editing.is_some() {
            self.handle_add_txn_input(k)?; 
            return Ok(());
        }

     
        match k.code {
            KeyCode::Char('q') => {
                self.quit = true;
                return Ok(());
            }
           
            _ => {}
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
                        r#type: AccountType::Checking,
                        focus_index: 0, 
                        ..Default::default()
                    };
                }
                KeyCode::Char('r') => { self.refresh_accounts().await.ok(); }
                KeyCode::Char('?') => self.tab = Tab::Help,
                _ => {}
            },
            Tab::Transactions => match k.code {
                KeyCode::Up => self.move_txn(-1),
                KeyCode::Down => self.move_txn(1),
                KeyCode::Char('a') => {
                    self.tab = Tab::AddTxn;
                    self.add.account_id = self.current_account_id();
                    self.add.just_entered = true;
                    self.load_categories().await;
                    self.ensure_cat_selected();
                }
                KeyCode::Char('r') => { self.refresh_txns().await.ok(); }
                KeyCode::Char('x') | KeyCode::Delete => {
                    if let Some(id) = self.current_txn_id() {
                        if let Err(e) = self.api.delete_transaction(id).await {
                            self.status = format!("Delete failed: {e}");
                        } else {
                            self.refresh_txns().await.ok();
                            self.status = "Deleted.".into();
                        }
                    }
                }
                KeyCode::Char('b') | KeyCode::Esc => self.tab = Tab::Accounts,
                KeyCode::Char('?') => self.tab = Tab::Help,
                _ => {}
            },
            Tab::AddTxn => {
              
                match k.code {
                    KeyCode::Up => self.move_cat(-1),
                    KeyCode::Down => self.move_cat(1),
                    KeyCode::Esc | KeyCode::Char('b') => {
                        self.tab = Tab::Transactions;
                        self.add.error = None;
                    }
                    KeyCode::Char('t') => { self.add.is_expense = !self.add.is_expense; }
                    
                    KeyCode::Char('p') => self.add.editing = Some(EditField::Payee),
                    KeyCode::Char('a') => self.add.editing = Some(EditField::Amount),
                    KeyCode::Char('m') => self.add.editing = Some(EditField::Memo),
                    KeyCode::Char('d') => self.add.editing = Some(EditField::Date),
                    KeyCode::Char('s') => { self.submit_txn().await.ok(); }
                    KeyCode::Char('?') => self.tab = Tab::Help,
                    _ => {}
                }
            },
            Tab::Help => match k.code {
                KeyCode::Esc | KeyCode::Char('b') => self.tab = Tab::Accounts,
                _ => {}
            },
        }
        Ok(())
    }

    async fn handle_creation_input(&mut self, k: KeyEvent) -> anyhow::Result<()> {
        match k.code {
            KeyCode::Esc => {
                self.accounts.creating = false;
                self.accounts.form = AccountForm::default();
            }
            KeyCode::Enter => {
               
                if self.accounts.form.name.trim().is_empty() {
                    self.accounts.form.error = Some("Name is required".into());
                    return Ok(());
                }
               
                let balance = Decimal::from_str(&self.accounts.form.opening).unwrap_or(Decimal::ZERO);
                let req = CreateAccountReq {
                    name: self.accounts.form.name.clone(),
                    r#type: self.accounts.form.r#type,
                    currency: self.accounts.form.currency.clone(),
                    opening_balance: Money(balance),
                };
                match self.api.create_account(&req).await {
                    Ok(_) => {
                        self.accounts.creating = false;
                        self.refresh_accounts().await?;
                    }
                    Err(e) => {
                        self.accounts.form.error = Some(format!("Error: {}", e));
                    }
                }
            }
            KeyCode::Tab | KeyCode::Down => {
                self.accounts.form.focus_index = (self.accounts.form.focus_index + 1) % 4;
            }
            KeyCode::BackTab | KeyCode::Up => {
                if self.accounts.form.focus_index > 0 {
                    self.accounts.form.focus_index -= 1;
                } else {
                    self.accounts.form.focus_index = 3;
                }
            }
            
            KeyCode::Left if self.accounts.form.focus_index == 1 => {
                self.cycle_account_type(-1);
            }
            KeyCode::Right if self.accounts.form.focus_index == 1 => {
                self.cycle_account_type(1);
            }

            KeyCode::Char(c) => {
                match self.accounts.form.focus_index {
                    0 => self.accounts.form.name.push(c),
                    1 => {},
                    2 => self.accounts.form.currency.push(c.to_ascii_uppercase()),
                    3 => {
                        if c.is_digit(10) || c == '.' || c == '-' {
                            self.accounts.form.opening.push(c);
                        }
                    }
                    _ => {}
                }
            }

            KeyCode::Backspace => {
                match self.accounts.form.focus_index {
                    0 => { self.accounts.form.name.pop(); },
                    2 => { self.accounts.form.currency.pop(); },
                    3 => { self.accounts.form.opening.pop(); },
                    _ => {}
                }
            }
            _ => {}
        }
        Ok(())
    }

    
    fn handle_add_txn_input(&mut self, k: KeyEvent) -> anyhow::Result<()> {
        if let Some(field) = self.add.editing {
            match k.code {
                KeyCode::Char(c) => match field {
                    EditField::Payee => self.add.payee.push(c),
                    EditField::Memo => self.add.memo.push(c),
                    EditField::Amount => self.add.amount.push(c),
                    EditField::Date => self.add.date.push(c),
                },
                KeyCode::Backspace => {
                    let s = match field {
                        EditField::Payee => &mut self.add.payee,
                        EditField::Memo => &mut self.add.memo,
                        EditField::Amount => &mut self.add.amount,
                        EditField::Date => &mut self.add.date,
                    };
                    s.pop();
                }
                KeyCode::Enter | KeyCode::Esc => { self.add.editing = None; }
                KeyCode::Tab => { self.add.editing = Some(self.next_field(field)); }
                KeyCode::BackTab => { self.add.editing = Some(self.prev_field(field)); }
                _ => {}
            }
        }
        Ok(())
    }

    
    fn cycle_account_type(&mut self, delta: i32) {
        use AccountType::*;
        let types = [Checking, Savings, Credit, Cash, Other];
        
        let current_pos = types.iter().position(|&t| t == self.accounts.form.r#type).unwrap_or(0);
        let len = types.len() as i32;
        let new_pos = (current_pos as i32 + delta).rem_euclid(len) as usize;
        
        self.accounts.form.r#type = types[new_pos];
    }

    
  

    fn move_account(&mut self, delta: isize) {
        let n = self.accounts.list.len();
        if n == 0 { return; }
        let cur = self.accounts.sel.selected().unwrap_or(0) as isize;
        let next = (cur + delta).rem_euclid(n as isize) as usize;
        self.accounts.sel.select(Some(next));
    }

    // Rewrite submission logic
    pub async fn submit_txn(&mut self) -> anyhow::Result<()> {
        let acc = if let Some(id) = self.add.account_id {
            id
        } else {
            self.add.error = Some("Please choose account".into());
            return Ok(());
        };

        // 1. Get and enforce Category check
        let cat_id = self.add.cat_sel.selected()
            .and_then(|i| self.add.categories.get(i))
            .map(|c| c.id);

        let final_cat_id = if let Some(id) = cat_id {
            id
        } else {
            // If user didn't select category, return error directly 
            self.add.error = Some("Category is required!".into());
            return Ok(());
        };

        // 2. Parse amount
        let amt = if self.add.amount.trim().is_empty() {
            self.add.error = Some("Amount cannot be empty".into());
            return Ok(());
        } else {
            match Decimal::from_str(self.add.amount.trim()) {
                Ok(d) => Money(d),
                Err(_) => {
                    self.add.error = Some("Invalid amount format".into());
                    return Ok(());
                }
            }
        };

        // 3. Calculate sign
        let final_amt = if self.add.is_expense {
            Money(-amt.0.abs())
        } else {
            Money(amt.0.abs())
        };

        // 4. Calculate base_amount (currently simple handling: equal to original amount)
        let final_base_amt = final_amt;

        // 5. Parse date
        let date = if self.add.date.trim().is_empty() {
            chrono::Utc::now().date_naive()
        } else {
            match NaiveDate::parse_from_str(&self.add.date, "%Y-%m-%d") {
                Ok(d) => d,
                Err(_) => {
                    self.add.error = Some("Format: YYYY-MM-DD".into());
                    return Ok(());
                }
            }
        };

        // 6. Construct request
        let req = CreateTxnReq {
            account_id: acc,
            category_id: final_cat_id, 
            amount: final_amt,
            base_amount: final_base_amt, 
            memo: if self.add.memo.trim().is_empty() { None } else { Some(self.add.memo.clone()) },
            currency: self.current_account().map(|a| a.currency.clone()).unwrap_or_else(|| "CAD".into()),
            txn_date: date,
        };

        // 7. Send
        match self.api.create_transaction(&req).await {
            Ok(_) => {
                self.add.success = Some("Saved ✓".into());
                self.add.error = None;
                self.add.amount.clear();
                self.add.memo.clear();
                self.refresh_txns().await.ok();
            }
            Err(e) => {
                self.add.error = Some(format!("Save failed: {e}"));
                self.add.success = None;
            }
        }
        Ok(())
    }
}