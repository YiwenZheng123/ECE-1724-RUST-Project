use chrono::{NaiveDate, NaiveDateTime};
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GoalField {
    Name,
    Target,
    Current,
    Deadline, // YYYY-MM-DD
}

#[derive(Default, Clone)]
pub struct GoalForm {
    pub name: String,
    pub target: String,
    pub current: String,
    pub deadline: String,
    pub error: Option<String>,
    pub editing: Option<GoalField>,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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
    pub payee: Option<String>,      //added payee
    pub currency: String,
    pub txn_date: NaiveDate,
    pub cleared: bool,
    pub reconciled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavingGoalDto {
    pub id: i64,
    pub name: String,
    pub target_amount: Money,
    pub current_amount: Money,
    pub deadline: Option<String>, // YYYY-MM-DD
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategorySpendingDto {
    pub category: String,
    pub total_amount: Money,
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
    pub category_id: i64,   
    pub amount: Money, 
    pub base_amount: Money,  
    pub is_expense: bool,    
    pub description: Option<String>,  
    pub payee: Option<String>,      //added payee
    pub currency: String,
    pub transacted_at: NaiveDateTime,
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
    Dashboard,
    Help,
}


#[derive(Default, Clone)]
pub struct AccountForm {
    pub name: String,
    pub currency: String,
    pub opening: String,
    pub r#type: AccountType,
    pub error: Option<String>,
    pub editing: Option<AccField>,
}

#[derive(Default)]
pub struct AccountsPage {
    pub list: Vec<AccountDto>,
    pub sel: ListState,
    pub creating: bool,
    pub form: AccountForm,
    pub editing_id: Option<i64>,   
    pub show_delete_confirm: bool,
}

// Transactions 
#[derive(Default)]
pub struct TxnPage {
    pub account_id: Option<i64>,
    pub table: Vec<TransactionDto>,
    pub tsel: TableState,
    pub loading: bool,
}

#[derive(Default)]
pub struct DashboardPage {
    pub goals: Vec<SavingGoalDto>,
    pub report: Vec<CategorySpendingDto>,
    pub loading: bool,
    pub creating: bool,
    pub editing_id: Option<i64>,
    pub form: GoalForm,
    pub selected_index: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccField {
    Name,
    Currency,
    Opening,
    Type,
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditField {
    Payee,
    Memo,
    Category,
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
    pub editing_txn_id: Option<i64>,
}

// App

pub struct App {
    pub api: Client,
    pub tab: Tab,
    pub status: String,
    pub quit: bool,

    pub accounts: AccountsPage,
    pub txn: TxnPage,
    pub dashboard: DashboardPage,
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
            dashboard: DashboardPage::default(),
            add,
        }
    }
    
    fn next_goal_field(&self, f: GoalField) -> GoalField {
        use GoalField::*;
        match f {
            Name => Target,
            Target => Current, 
            Current => Deadline, 
            Deadline => Name,
        }
    }

    pub async fn refresh_goals(&mut self) -> anyhow::Result<()> {
        let goals = self.api.list_goals().await.unwrap_or_default();
        self.dashboard.goals = goals;
        Ok(())
    }

    pub async fn refresh_monthly_report(&mut self) -> anyhow::Result<()> {
        let report = self.api.get_monthly_report().await.unwrap_or_default();
        self.dashboard.report = report;
        Ok(())
    }

    pub async fn refresh_dashboard(&mut self) -> anyhow::Result<()> {
        self.dashboard.loading = true;
        let goals = self.api.list_goals().await.unwrap_or_default();
        let report = self.api.get_monthly_report().await.unwrap_or_default();

        self.dashboard.goals = goals;
        self.dashboard.report = report;
        self.dashboard.loading = false;
        
        Ok(())
    }

    // fn current_category_type(&self) -> Option<CategoryType> {
    //     self.add
    //         .cat_sel
    //         .selected()
    //         .and_then(|i| self.add.categories.get(i))
    //         .map(|c| c.r#type.clone())
    // }
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
            self.load_categories().await;

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
        if let Ok(list) = self.api.list_categories().await {
            self.add.categories = list;
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
            self.add.is_expense = false;
        } else {
            if self.add.cat_sel.selected().is_none() {
                self.add.cat_sel.select(Some(0));
            }
            if let Some(i) = self.add.cat_sel.selected()
                .and_then(|idx| self.add.categories.get(idx))
            {
                self.add.is_expense = matches!(i.r#type, crate::cli::state::CategoryType::Expense);
            }
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
        if let Some(cat) = self.add.categories.get(new) {
            self.add.is_expense = matches!(cat.r#type, crate::cli::state::CategoryType::Expense);
        }
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



    // fn next_field(&self, f: EditField) -> EditField {
    //     use EditField::*;
    //     match f { 
    //         Date     => Payee,
    //         Payee    => Memo,
    //         Memo     => Category,
    //         Category => Amount,
    //         Amount   => Date
    //     }
    // }

    fn acc_next_field(&self, f: AccField) -> AccField {
        use AccField::*;
        match f { Name => Type, Type => Currency, Currency => Opening, Opening => Name }
    }

    fn cycle_account_type(&self, cur: AccountType, forward: bool) -> AccountType {
        use AccountType::*;
        if forward {
            match cur { Cash => Checking, Checking => Savings, Savings => Credit, Credit => Other, Other => Cash }
        } else {
            match cur { Cash => Other, Other => Credit, Credit => Savings, Savings => Checking, Checking => Cash }
        }
    }

    pub async fn handle_key(&mut self, k: KeyEvent) -> anyhow::Result<()> {
        use crate::cli::state::AccField;
        if k.kind != KeyEventKind::Press {
            return Ok(());
        }

        let is_typing = (self.tab == Tab::AddTxn && self.add.editing.is_some()) 
             || (self.tab == Tab::Accounts && self.accounts.creating);

        // Pressing q to exit is only allowed when it is not in typing mode
        if !is_typing {
            match k.code {
                KeyCode::Char('q') => {
                    self.quit = true;
                    return Ok(());
                }
                _ => {}
            }
        }
        match self.tab {
           Tab::Accounts => {
            if self.accounts.show_delete_confirm {
                match k.code {
                    KeyCode::Char('y') | KeyCode::Enter => {
                        if let Some(idx) = self.accounts.sel.selected() {
                            if let Some(acc) = self.accounts.list.get(idx) {
                                if let Err(e) = self.api.delete_account(acc.id).await {
                                    self.status = format!("Delete failed: {}", e);
                                } else {
                                    self.status = "Account deleted.".to_string();
                                    self.refresh_accounts().await.ok();
                                    let len = self.accounts.list.len(); 
                                    if len > 0 {
                                        self.accounts.sel.select(Some(len - 1));
                                    } else {
                                        self.accounts.sel.select(None);
                                    }
                                }
                            }
                        }
                        self.accounts.show_delete_confirm = false;
                    }
                    KeyCode::Char('n') | KeyCode::Esc => {
                        self.accounts.show_delete_confirm = false;
                        self.status = "Delete cancelled.".to_string();
                    }
                    _ => {}
                }
                return Ok(());
            }

            
            if self.accounts.creating {
                use crate::cli::state::{AccField};
                match k.code {
                    KeyCode::Char(c) => {
                        if let Some(f) = self.accounts.form.editing {
                            match f {
                                AccField::Name     => self.accounts.form.name.push(c),
                                AccField::Currency => self.accounts.form.currency.push(c),
                                AccField::Opening  => {
                                    if self.accounts.editing_id.is_none() {
                                        self.accounts.form.opening.push(c)
                                    }
                                },
                                AccField::Type     => {} 
                            }
                        }
                    }
                    KeyCode::Backspace => {
                        if let Some(f) = self.accounts.form.editing {
                            match f {
                                AccField::Name     => { self.accounts.form.name.pop(); }
                                AccField::Currency => { self.accounts.form.currency.pop(); }
                                AccField::Opening  => { 
                                    if self.accounts.editing_id.is_none() {
                                        self.accounts.form.opening.pop(); 
                                    }
                                }
                                AccField::Type     => {}
                            }
                        }
                    }
                    KeyCode::Tab => {
                        let cur = self.accounts.form.editing.unwrap_or(AccField::Name);
                        self.accounts.form.editing = Some(self.acc_next_field(cur));
                    }
                    KeyCode::Up => {
                    if matches!(self.accounts.form.editing, Some(AccField::Type)) {
                        self.accounts.form.r#type = self.cycle_account_type(self.accounts.form.r#type, true);
                    }
                    }
                    KeyCode::Down => {
                        if matches!(self.accounts.form.editing, Some(AccField::Type)) {
                            self.accounts.form.r#type = self.cycle_account_type(self.accounts.form.r#type, false);
                        }
                    }
                    KeyCode::Esc => {
                        self.accounts.creating = false;
                        self.accounts.editing_id = None; 
                        self.accounts.form.error = None;
                        self.accounts.form.editing = None;
                    }
                    KeyCode::Enter => {
                        let name = self.accounts.form.name.trim();
                        if name.is_empty() {
                            self.accounts.form.error = Some("Name cannot be empty".into());
                            return Ok(());
                        }

                        if let Some(edit_id) = self.accounts.editing_id {
                            match self.api.update_account(
                                edit_id, 
                                name, 
                                self.accounts.form.r#type.as_str(), 
                                &self.accounts.form.currency
                            ).await {
                                Ok(_) => {
                                    self.status = "Account updated.".to_string();
                                    self.accounts.creating = false;
                                    self.accounts.editing_id = None;
                                    self.refresh_accounts().await.ok();
                                }
                                Err(e) => self.accounts.form.error = Some(e.to_string()),
                            }
                        } else {
                            let opening = match Decimal::from_str_exact(self.accounts.form.opening.trim()) {
                                Ok(d) => d,
                                Err(_) => {
                                    self.accounts.form.error = Some("Invalid opening balance".into());
                                    return Ok(());
                                }
                            };
                            let req = CreateAccountReq {
                                name: name.to_string(),
                                r#type: self.accounts.form.r#type,
                                currency: if self.accounts.form.currency.trim().is_empty() { "CAD".into() } else { self.accounts.form.currency.trim().to_uppercase() },
                                opening_balance: Money(opening),
                            };
                            match self.api.create_account(&req).await {
                                Ok(_) => {
                                    self.accounts.creating = false;
                                    self.accounts.form.editing = None;
                                    self.refresh_accounts().await.ok();
                                    if !self.accounts.list.is_empty() {
                                        self.accounts.sel.select(Some(self.accounts.list.len().saturating_sub(1)));
                                    }
                                    self.status = "Account created.".into();
                                }
                                Err(e) => {
                                    eprintln!(">>> DB Error: {:?}", e);
                                    self.accounts.form.error = Some(format!("Create failed: {e}"));
                                }
                            }
                        }
                    }
                    _ => {}
                }
                return Ok(()); 
            }

            match k.code {
                KeyCode::Up => self.move_account(-1),
                KeyCode::Down => self.move_account(1),
                KeyCode::Enter => {
                    self.tab = Tab::Transactions;
                    self.txn.account_id = self.current_account_id();
                    self.refresh_txns().await.ok();
                }
                KeyCode::Char('n') => {
                    self.accounts.creating = true;
                    self.accounts.editing_id = None; 
                    self.accounts.form = AccountForm {
                        name: String::new(),
                        currency: "CAD".into(),
                        opening: "0".into(),
                        r#type: AccountType::Cash,
                        error: None,
                        editing: Some(AccField::Name),
                    };
                }
                
                KeyCode::Char('e') => {
                    if let Some(idx) = self.accounts.sel.selected() {
                        if let Some(acc) = self.accounts.list.get(idx) {
                            self.accounts.creating = true;
                            self.accounts.editing_id = Some(acc.id);
                        
                            self.accounts.form = AccountForm {
                                name: acc.name.clone(),
                                currency: acc.currency.clone(),
                                opening: acc.opening_balance.0.to_string(),
                                r#type: acc.r#type,
                                error: None,
                                editing: Some(AccField::Name),
                            };
                        }
                    }
                }
              
                KeyCode::Char('d') | KeyCode::Delete => {
                    if self.accounts.sel.selected().is_some() {
                        self.accounts.show_delete_confirm = true;
                    }
                }

                KeyCode::Char('g') => {
                         self.tab = Tab::Dashboard;
                         self.refresh_dashboard().await.ok();
                     }
                KeyCode::Char('r') => { self.refresh_accounts().await.ok(); }
                KeyCode::Char('?') => { self.tab = Tab::Help; }
                KeyCode::Esc => { /* no-op */ }
                _ => {}
            }
        }
        Tab::Transactions => match k.code {
            KeyCode::Up   => self.move_txn(-1),
            KeyCode::Down => self.move_txn(1),
            KeyCode::Char('a') => {
                // use crate::cli::state::EditField;
                self.tab = Tab::AddTxn;
                self.add.account_id = self.current_account_id();
                self.add.editing = None; 
                self.add.cat_sel.select(None);
                self.add.just_entered = true;
                self.add.editing_txn_id = None; 
                self.add.amount.clear();
                self.add.memo.clear();
                self.add.payee.clear();
                self.add.date = chrono::Utc::now().format("%Y-%m-%d").to_string();
                self.load_categories().await;
                self.ensure_cat_selected();
            }

            KeyCode::Char('e') => {
                if let Some(idx) = self.txn.tsel.selected() {
                    if let Some(txn) = self.txn.table.get(idx).cloned() {
                        self.tab = Tab::AddTxn;
                        self.load_categories().await; 
                        self.add.account_id = Some(txn.account_id);
                        self.add.editing_txn_id = Some(txn.id); 
                        self.add.date = txn.txn_date.format("%Y-%m-%d").to_string();
                        self.add.memo = txn.memo.unwrap_or_default();
                        self.add.payee = txn.payee.unwrap_or_default();
                        self.add.amount = txn.amount.0.abs().to_string(); 
                        self.add.is_expense = txn.amount.0.is_sign_negative();
                        
                        if let Some(cat_id) = txn.category_id {
                            if let Some(pos) = self.add.categories.iter().position(|c| c.id == cat_id) {
                                self.add.cat_sel.select(Some(pos));
                            }
                        }
                        
                        self.add.just_entered = false;
                        self.add.editing = None;
                    }
                }
            }


            KeyCode::Char('r') => {
                self.refresh_txns().await.ok();
            }
            KeyCode::Char('d') | KeyCode::Delete => {
                if let Some(id) = self.current_txn_id() {
                    if let Err(e) = self.api.delete_transaction(id).await {
                        self.status = format!("Delete failed: {e}");
                    } else {
                        let old_idx = self.txn.tsel.selected().unwrap_or(0);
                        self.refresh_txns().await.ok();
                        self.refresh_accounts().await.ok();
                        let n = self.txn.table.len();
                        if n == 0 { self.txn.tsel.select(None); } 
                        else { self.txn.tsel.select(Some(old_idx.min(n - 1))); }
                        self.status = "Deleted.".into();
                    }
                }
            }
            KeyCode::Esc => self.tab = Tab::Accounts,
            KeyCode::Char('?') => self.tab = Tab::Help,
            _ => {}
        },

        Tab::AddTxn => {
            if k.kind != KeyEventKind::Press { return Ok(()); }
            use crate::cli::state::EditField;
            
           
            if k.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) && k.code == KeyCode::Char('s') {
                self.submit_txn().await.ok();
                return Ok(());
            }

            if k.code == KeyCode::Esc {
                self.tab = Tab::Transactions;
                self.add.editing = None;
                return Ok(());
            }

          
            let current_field = self.add.editing.unwrap_or(EditField::Date); 

            match k.code {

                KeyCode::Enter => {
                    if self.add.editing.is_some() {
                        self.add.editing = None; 
                    } else {
                        self.add.editing = Some(EditField::Date);
                    }
                }
            
                KeyCode::Char(c) if current_field == EditField::Amount => {
                    if c == 't' {
                    
                        self.add.is_expense = !self.add.is_expense;
                    } else if c.is_ascii_digit() || c == '.' {
                       
                        self.add.amount.push(c);
                    }
                }
                KeyCode::Up if current_field == EditField::Amount => {
                    
                    self.move_cat(-1);
                }
                KeyCode::Down if current_field == EditField::Amount => {
                    
                    self.move_cat(1);
                }
                

              
                KeyCode::Char(c) => {
                    if let Some(field) = self.add.editing {
                        match field {
                            EditField::Payee  => self.add.payee.push(c),
                            EditField::Memo   => self.add.memo.push(c),
                            EditField::Date   => self.add.date.push(c),
                            EditField::Amount => {}, 
                            EditField::Category => {} 
                        }
                    } else {
                        if c == 't' { self.add.is_expense = !self.add.is_expense; }
                    }
                }
                KeyCode::Backspace => {
                    if let Some(field) = self.add.editing {
                        let target: Option<&mut String> = match field {
                            EditField::Date   => Some(&mut self.add.date),
                            EditField::Payee  => Some(&mut self.add.payee),
                            EditField::Memo   => Some(&mut self.add.memo),
                            EditField::Amount => Some(&mut self.add.amount),
                            EditField::Category => None,
                        };
                        if let Some(s) = target { s.pop(); }
                    }
                }

               
                KeyCode::Tab => {
                
                    let next = match self.add.editing {
                        Some(EditField::Date)   => EditField::Payee,
                        Some(EditField::Payee)  => EditField::Memo,
                        Some(EditField::Memo)   => EditField::Amount,
                        Some(EditField::Amount) => EditField::Date, 
                        _ => EditField::Date,
                    };
                    self.add.editing = Some(next);
                }

                KeyCode::Up => self.move_cat(-1),
                KeyCode::Down => self.move_cat(1),
                _ => {}
            }
            return Ok(());
        }
        

        Tab::Dashboard => {
                if self.dashboard.creating {
                  
                    use crate::cli::state::GoalField;
                    match k.code {
                        KeyCode::Esc => { self.dashboard.creating = false; }
                        KeyCode::Tab => {
                            let cur = self.dashboard.form.editing.unwrap_or(GoalField::Name);
                            self.dashboard.form.editing = Some(self.next_goal_field(cur));
                        }
                        KeyCode::Char(c) => {
                            if let Some(f) = self.dashboard.form.editing {
                                match f {
                                    GoalField::Name => self.dashboard.form.name.push(c),
                                    GoalField::Target => { if c.is_ascii_digit() || c == '.' { self.dashboard.form.target.push(c); } },
                                    GoalField::Current => { if c.is_ascii_digit() || c == '.' { self.dashboard.form.current.push(c); } },
                                    GoalField::Deadline => self.dashboard.form.deadline.push(c),
                                }
                            }
                        }
                        KeyCode::Backspace => {
                            if let Some(f) = self.dashboard.form.editing {
                                match f {
                                    GoalField::Name => { self.dashboard.form.name.pop(); },
                                    GoalField::Target => { self.dashboard.form.target.pop(); },
                                    GoalField::Current => { self.dashboard.form.current.pop(); },
                                    GoalField::Deadline => { self.dashboard.form.deadline.pop(); },
                                }
                            }
                        }
                        KeyCode::Enter => { self.submit_goal().await.ok(); }
                        _ => {}
                    }
                } else {
                   
                    match k.code {
                        KeyCode::Esc => self.tab = Tab::Accounts,
                        KeyCode::Char('r') => { self.refresh_dashboard().await.ok(); }
                        KeyCode::Char('?') => self.tab = Tab::Help,
                        
                       
                        KeyCode::Down => {
                            let len = self.dashboard.goals.len();
                            if len > 0 {
                                self.dashboard.selected_index = (self.dashboard.selected_index + 1) % len;
                            }
                        }
                        KeyCode::Up => {
                            let len = self.dashboard.goals.len();
                            if len > 0 {
                                if self.dashboard.selected_index == 0 {
                                    self.dashboard.selected_index = len - 1;
                                } else {
                                    self.dashboard.selected_index -= 1;
                                }
                            }
                        }

                      
                        KeyCode::Char('n') => {
                            self.dashboard.creating = true;
                            self.dashboard.editing_id = None; 
                            self.dashboard.form = GoalForm::default();
                            self.dashboard.form.editing = Some(GoalField::Name);
                        }

                      
                        KeyCode::Char('e') => {
                            if let Some(goal) = self.dashboard.goals.get(self.dashboard.selected_index) { 
                                self.dashboard.creating = true;
                                self.dashboard.editing_id = Some(goal.id); 
                                let date_clean = goal.deadline.clone().unwrap_or_default().chars().take(10).collect();

                                self.dashboard.form = GoalForm {
                                    name: goal.name.clone(),
                                    target: goal.target_amount.0.to_string(),
                                    current: goal.current_amount.0.to_string(),
                                    deadline: date_clean,
                                    error: None,
                                    editing: Some(GoalField::Current),
                                };
                            }
                        }

               
                        KeyCode::Char('d') | KeyCode::Delete => {
                            if let Some(goal) = self.dashboard.goals.get(self.dashboard.selected_index) {
                                self.api.delete_goal(goal.id).await.ok();
                                self.refresh_dashboard().await.ok();
                                if self.dashboard.selected_index >= self.dashboard.goals.len() && !self.dashboard.goals.is_empty() {
                                    self.dashboard.selected_index = self.dashboard.goals.len() - 1;
                                }
                            }
                        }
                        _ => {}
                    }
                }
            },
            
            Tab::Help => match k.code {
                KeyCode::Esc => self.tab = Tab::Accounts, 
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
        let acc = if let Some(id) = self.add.account_id { id } else {
            self.add.error = Some("Please choose account".into());
            return Ok(());
        };

        let amt = if self.add.amount.trim().is_empty() {
            self.add.error = Some("Amount cannot be empty".into());
            return Ok(());
        } else {
            match Decimal::from_str_exact(self.add.amount.trim()) {
                Ok(d) => d,
                Err(_) => {
                    self.add.error = Some("Invalid amount format".into());
                    return Ok(());
                }
            }
        };
        let mut decimal_amt = amt.abs();
        if self.add.is_expense {
            decimal_amt = -decimal_amt;
        }
        let final_amt = Money(decimal_amt);
        let date_str = self.add.date.trim();
        let date = if date_str.is_empty() {
            chrono::Utc::now().date_naive()
        } else {
            match NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
                Ok(d) => d,
                Err(_) => {
                    self.add.error = Some("Date must be YYYY-MM-DD".into());
                    return Ok(());
                }
            }
        };

        let cat_id = self.add.cat_sel.selected()
            .and_then(|i| self.add.categories.get(i))
            .map(|c| c.id);

        if cat_id.is_none() {
             self.add.error = Some("Category is required!".into());
             return Ok(());
        }

        let req = CreateTxnReq {
            account_id: acc,
            category_id: cat_id.unwrap(), 
            amount: final_amt,
            base_amount: final_amt, 
            is_expense: self.add.is_expense, 
            description: if self.add.memo.trim().is_empty() { 
                None 
            } else { 
                Some(self.add.memo.trim().to_string()) 
            },
            payee: if self.add.payee.trim().is_empty() { 
                None 
            } else { 
                Some(self.add.payee.trim().to_string()) 
            },
            currency: self.current_account()
                .map(|a| a.currency.clone())
                .unwrap_or_else(|| "CAD".into()),
            
            transacted_at: date.and_hms_opt(0, 0, 0).unwrap(), 
        };
        let res = if let Some(edit_id) = self.add.editing_txn_id {
           
            self.api.update_transaction(edit_id, &req).await
        } else {
            
            self.api.create_transaction(&req).await.map(|_| ()) 
        };

       match res {
            Ok(_) => {
                self.add.success = Some("Saved!".into());
                self.add.error = None;
                self.add.amount.clear();
                self.add.memo.clear();
                self.add.payee.clear();
                self.add.editing_txn_id = None;
                
                self.refresh_txns().await.ok();
                self.refresh_accounts().await.ok();
                
                self.tab = Tab::Transactions; 
                self.status = "Transaction saved.".into();
            }
            Err(e) => {
                eprintln!("DEBUG: API Error: {:?}", e);
                self.add.error = Some(format!("Error: {}", e));
            }
        }
        Ok(())
    } 

    pub async fn submit_goal(&mut self) -> anyhow::Result<()> {
        let name = self.dashboard.form.name.trim();
        if name.is_empty() {
            self.dashboard.form.error = Some("Name cannot be empty".into());
            return Ok(());
        }

        let target_str = self.dashboard.form.target.trim();
       
        let target_dec = if target_str.is_empty() { Decimal::ZERO } else { Decimal::from_str_exact(target_str).unwrap_or(Decimal::ZERO) };

        let current_str = self.dashboard.form.current.trim();
        
        let current_dec = if current_str.is_empty() { Decimal::ZERO } else { Decimal::from_str_exact(current_str).unwrap_or(Decimal::ZERO) };

      
        let date_str = self.dashboard.form.deadline.trim();
        let deadline_opt = if date_str.is_empty() {
            None
        } else {
            match NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
                Ok(d) => Some(d.and_hms_opt(0,0,0).unwrap()),
                Err(_) => {
                    self.dashboard.form.error = Some("Date must be YYYY-MM-DD".into());
                    return Ok(());
                }
            }
        };

       
        let req = crate::cli::api::CreateGoalReq {
            account_id: 1, 
            name: name.to_string(),
            target_amount: Money(target_dec),
            current_amount: Money(current_dec),
            deadline: deadline_opt,
        };

      
        if let Some(id) = self.dashboard.editing_id {
          
            self.api.update_goal(id, &req).await?; 
        } else {
           
            self.api.create_goal(&req).await?; 
        }
       
        self.dashboard.creating = false;
        self.dashboard.editing_id = None; 
        self.refresh_goals().await.ok();
        self.status = "Goal saved.".into();

        Ok(())
    }
} 
    