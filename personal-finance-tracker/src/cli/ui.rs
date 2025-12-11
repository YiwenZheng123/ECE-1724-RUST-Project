use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Row, Table, TableState, Tabs, Cell},
    Frame,
};

use crate::cli::state::{self, App};
use rust_decimal::Decimal;

pub fn draw(f: &mut Frame, app: &mut App) {
    let size = f.size();

    // top tabs | main content | Bottom status bar
    let root = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(10)])
        .split(size);

    // Tabs
    let titles = ["Accounts", "Transactions", "AddTransaction", "Help"]
        .into_iter()
        .map(|t| Line::from(Span::raw(t)))
        .collect::<Vec<_>>();
    let tabs = Tabs::new(titles)
        .select(match app.tab { state::Tab::Accounts => 0, state::Tab::Transactions => 1, state::Tab::AddTxn => 2, state::Tab::Help => 3 })
        .block(Block::default().borders(Borders::ALL).title("Finance Tracker"))
        .highlight_style(Style::default().add_modifier(Modifier::BOLD));
    f.render_widget(tabs, root[0]);

    match app.tab {
        state::Tab::Accounts => draw_accounts(f, root[1], app),
        state::Tab::Transactions => draw_txns(f, root[1], app),
        state::Tab::AddTxn => draw_add_txn(f, root[1], app),
        state::Tab::Help => draw_help(f, root[1]),
    }

    if app.accounts.creating {
        let area = center_rect(root[1], 54, 12);
        f.render_widget(Clear, area);
        draw_new_account_modal(f, area, app);
    }
}

// Accounts Page

fn draw_accounts(f: &mut Frame, area: Rect, app: &mut App) {
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(area);

    // Account List
    let items: Vec<ListItem> = app.accounts.list.iter().map(|a| {
        let line = Line::from(vec![
            Span::raw(format!("{}  ", a.name)),
            Span::raw("["), Span::raw(format!("{:?}", a.r#type)), Span::raw("]  "),
            Span::raw(a.currency.clone()), Span::raw("  "),
            Span::raw(fmt_money(a.opening_balance.0)),
        ]);
        ListItem::new(line)
    }).collect();

    let len = app.accounts.list.len();
    if let Some(i) = app.accounts.sel.selected() {
        if i >= len {
            app.accounts.sel.select(if len == 0 { None } else { Some(len - 1) });
        }
    } else if len > 0 {
        app.accounts.sel.select(Some(0));
    }

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Accounts  (Up/Down, Enter→Txns, n=new, r=refresh)"))
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED));

    f.render_stateful_widget(list, cols[0], &mut app.accounts.sel);

    // Details
    let right = if let Some(acc) = app.current_account() {
        Paragraph::new(format!(
            "ID: {}\nName: {}\nType: {:?}\nCurrency: {}\nBalance: {}\nCreated: {}",
            acc.id, acc.name, acc.r#type, acc.currency, fmt_money(acc.opening_balance.0), acc.created_at
        ))
    } else {
        Paragraph::new("No account selected")
    }.block(Block::default().borders(Borders::ALL).title("Details"));
    f.render_widget(right, cols[1]);
}

fn draw_new_account_modal(f: &mut Frame, area: Rect, app: &mut App) {
    use crate::cli::state::{AccField, AccountType};

    let form = &app.accounts.form;

    let (m_name, m_type, m_currency, m_opening) = match form.editing {
        Some(AccField::Name)     => ("  <editing>", "", "", ""),
        Some(AccField::Type)     => ("", "  <editing> (use  ↑ / ↓)", "", ""),
        Some(AccField::Currency) => ("", "", "  <editing>", ""),
        Some(AccField::Opening)  => ("", "", "", "  <editing>"),
        None                     => ("", "", "", ""),
    };

    let lines = vec![
        format!("Name     : {}{}", form.name, m_name),
        format!("Type     : {:?}{}", form.r#type, m_type),
        format!("Currency : {}{}", form.currency, m_currency),
        format!("Opening  : {}{}", form.opening, m_opening),
        "".into(),
        "TAB: switch field | ↑/↓: change Type | Enter: create | Esc: cancel".into(),
        form.error.clone().unwrap_or_default(),
    ].join("\n");

    let p = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL).title("New Account"));
    f.render_widget(p, area);
}
// Transactions Page

fn draw_txns(f: &mut Frame, area: Rect, app: &mut App) {
    // Build table rows
    let header = Row::new(vec!["Date", "Category", "Memo", "Amount"]).height(1);

    let body: Vec<Row> = app.txn.table.iter().map(|t| {
        Row::new(vec![
            Cell::from(t.txn_date.to_string()),
            Cell::from(t.category_id.map(|id| format!("#{id}")).unwrap_or_else(|| "-".into())),
            Cell::from(t.memo.clone().unwrap_or_default()),
            Cell::from(fmt_money(t.amount.0)),
        ])
    }).collect();

    let widths = [
        Constraint::Length(12),
        Constraint::Length(12),
        Constraint::Percentage(60),
        Constraint::Length(14),
    ];

    let mut tsel = app.txn.tsel.clone();
    let table = Table::new(body, widths)
        .header(header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(if app.txn.loading { "Transactions (loading…)" } else { "Transactions" }),
        )
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED));

    f.render_stateful_widget(table, area, &mut tsel);
    app.txn.tsel = tsel;
}

// Add Transaction Page

fn draw_add_txn(f: &mut Frame, area: Rect, app: &mut App) {
    use crate::cli::state::EditField;
    use ratatui::widgets::Wrap; 

    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
        .split(area);

    let left_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(8),     
            Constraint::Length(8),  
        ])
        .split(cols[0]);

    {
        let len = app.add.categories.len();
        let new_sel = match (len, app.add.cat_sel.selected()) {
            (0, _) => None,
            (n, Some(i)) if i >= n => Some(n - 1),
            (_, x) => x,
        };
        app.add.cat_sel.select(new_sel);
    }
    let selected_name = app
            .add
            .cat_sel
            .selected()
            .and_then(|i| app.add.categories.get(i))
            .map(|c| format!("{} ({:?}) {}", c.name, c.r#type, c.icon))
            .unwrap_or_else(|| "<none>".into());

    let (m_date, m_payee, m_memo, m_category, m_amount) = match app.add.editing {
        Some(EditField::Date)     => ("  <editing>", "", "", "", ""),
        Some(EditField::Payee)    => ("", "  <editing>", "", "", ""),
        Some(EditField::Memo)     => ("", "", "  <editing>", "", ""),
        Some(EditField::Category) => ("", "", "", "  <editing> (↑/↓ to choose)", ""),
        Some(EditField::Amount)   => ("", "", "", "", "  <editing>"),
        None                      => ("", "", "", "", ""),
    };

    // List input
    let form_lines = vec![
        format!("Account : {:?}", app.add.account_id),
        format!("Date    : {}{}",   app.add.date,   m_date),
        format!("Payee   : {}{}",   app.add.payee,  m_payee),
        format!("Desc    : {}{}",   app.add.memo,   m_memo), 
        format!("Category: {}{}",   selected_name,  m_category), 
        format!(
            "Amount  : {}{}  [{}]",
            app.add.amount,
            m_amount,
            if app.add.is_expense { "Expense (-)" } else { "Income (+)" }
        ),
    ].join("\n");

    let form_p = Paragraph::new(form_lines)
        .block(Block::default().borders(Borders::ALL).title("Add Transaction"));
    f.render_widget(form_p, left_chunks[0]); 

    let help_lines = vec![
        "Controls:".into(),
        "  Tab/Shift+Tab: Switch field".into(),
        "  Enter: Edit mode (or toggle)".into(),
        "  p/a/m/d: Quick jump".into(),
        "Actions:".into(),
        "  Ctrl+s: Save | Esc: Back | t: Type".into(),
        String::new(), 
        if let Some(err) = &app.add.error {
            format!("Error: {}", err)
        } else if let Some(succ) = &app.add.success {
            format!("Success: {}", succ)
        } else {
            String::new()
        }
    ].join("\n");

    let help_p = Paragraph::new(help_lines)
        .block(Block::default().borders(Borders::ALL).title("Help & Status"))
        .wrap(Wrap { trim: true }); 
    f.render_widget(help_p, left_chunks[1]); 

    let items: Vec<ListItem> = app
        .add
        .categories
        .iter()
        .map(|c| ListItem::new(Line::from(format!("{}  {:?}  {}", c.name, c.r#type, c.icon))))
        .collect();
    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Categories"))
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED));

    f.render_stateful_widget(list, cols[1], &mut app.add.cat_sel);
}
fn draw_help(f: &mut Frame, area: Rect) {
    let help_text = vec![
        "Global Keys:",
        "  q        : Quit App",
        "  ?        : Toggle this Help tab",
        "  Tab      : Switch tabs (Accounts <-> Transactions <-> Add)",
        "",
        "Accounts Tab:",
        "  Up/Down  : Navigate list",
        "  Enter    : View transactions for selected account",
        "  n        : Create new account",
        "  r        : Refresh data",
        "",
        "Transactions Tab:",
        "  Up/Down  : Navigate list",
        "  a        : Add new transaction",
        "  x/Del    : Delete selected transaction",
        "  r        : Refresh list",
        "  b        : Back to Accounts",
        "",
        "Add Transaction Tab:",
        "  Tab      : Cycle through fields (Date -> Payee -> ...)",
        "  Enter    : Enter/Exit Edit Mode (Toggle)",
        "  Ctrl + s : Save Transaction (Submit)",
        "  Esc      : Cancel / Back to list",
        "  t        : Toggle Expense/Income",
        "  p/m/a/d  : Quick jump to Payee/Memo/Amount/Date",
        "  Up/Down  : Select Category (when Category field is active)",
    ].join("\n");

    let p = Paragraph::new(help_text)
        .block(Block::default().borders(Borders::ALL).title("Help & Keybindings"));
    
    f.render_widget(p, area);
}


fn fmt_money(d: Decimal) -> String {
    d.round_dp(2).to_string()
}

fn center_rect(rect: Rect, w: u16, h: u16) -> Rect {
    let x = rect.x + rect.width.saturating_sub(w) / 2;
    let y = rect.y + rect.height.saturating_sub(h) / 2;
    Rect { x, y, width: w.min(rect.width), height: h.min(rect.height) }
}
