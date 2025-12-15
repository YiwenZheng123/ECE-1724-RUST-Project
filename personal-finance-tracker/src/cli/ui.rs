use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Row, Table, Tabs, Cell, Wrap,Gauge, BarChart},
    Frame,
};

use ratatui::prelude::Alignment;
use crate::cli::state::{self, App, AccField, EditField}; 
use rust_decimal::Decimal;
use crate::cli::state::Tab;

pub fn draw(f: &mut Frame, app: &mut App) {
    let size = f.size();

    let root = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), 
            Constraint::Min(10), 
            Constraint::Length(3) 
        ])
        .split(size);

    // Tabs
    let titles = ["Accounts", "Transactions", "AddTxn", "Goals", "Help"] 
        .iter()
        .map(|t| Line::from(Span::raw(*t)))
        .collect::<Vec<_>>();
    
    let tabs = Tabs::new(titles)
        .select(match app.tab { 
            state::Tab::Accounts => 0, 
            state::Tab::Transactions => 1, 
            state::Tab::AddTxn => 2, 
            Tab::Dashboard => 3,
            state::Tab::Help => 4 
        })
        .block(Block::default().borders(Borders::ALL).title(" Finance Tracker "))
        .highlight_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)); 
    f.render_widget(tabs, root[0]);

    // Main Content
    match app.tab {
        state::Tab::Accounts => draw_accounts(f, root[1], app),
        state::Tab::Transactions => draw_txns(f, root[1], app),
        state::Tab::AddTxn => draw_add_txn(f, root[1], app),
        Tab::Dashboard => { ui_dashboard(f, &app.dashboard, root[1]); }
        state::Tab::Help => draw_help(f, root[1]),
    }

    let status_text = match app.tab {
        state::Tab::AddTxn => "",
        _ => " Press ? for help | q to quit ", 
    };
    
    let status = Paragraph::new(status_text)
        .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::DarkGray)));
    f.render_widget(status, root[2]);

  
    if app.accounts.creating {
        let area = center_rect(root[1], 60, 14); 
        f.render_widget(Clear, area);
        draw_new_account_modal(f, area, app);
    }

    if app.accounts.show_delete_confirm {
        let area = center_rect(root[1], 40, 10);
        
        f.render_widget(Clear, area); 
        
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Red))
            .title(Span::styled(" WARNING ", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD).add_modifier(Modifier::SLOW_BLINK)))
            .title_alignment(ratatui::layout::Alignment::Center)
            .style(Style::default().bg(Color::Black)); 

        let text = vec![
            Line::from(""),
            Line::from(Span::styled("Are you sure you want to", Style::default().fg(Color::White))),
            Line::from(Span::styled("DELETE this account?", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))),
            Line::from(""),
            Line::from(Span::raw("All transactions will be lost!")),
            Line::from(""),
            Line::from(""),
            Line::from(vec![
                Span::styled("[Enter/y] ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                Span::raw("Yes, Delete"),
            ]),
            Line::from(vec![
                Span::styled("[Esc/n]   ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::raw("Cancel"),
            ]),
        ];
        

        let p = Paragraph::new(text)
            .block(block)
            .alignment(ratatui::layout::Alignment::Center); 
            
        f.render_widget(p, area);
    }

    if app.tab == Tab::Dashboard && app.dashboard.creating {
        let area = center_rect(root[1], 60, 12);
        f.render_widget(Clear, area); 
        draw_new_goal_modal(f, area, app);
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
       
        let balance_color = if a.opening_balance.0.is_sign_negative() { Color::Red } else { Color::Green };
        
        let type_str = format!("{:?}", a.r#type); 
        let type_len = type_str.len();

        // Calculate the number of spaces that need to be filled
        const TYPE_WIDTH: usize = 15;
        let padding_len = TYPE_WIDTH.saturating_sub(type_len+3);
        let padding = " ".repeat(padding_len);

        let line = Line::from(vec![
            Span::styled(format!("{:<18}", a.name), Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(" ["), 
            Span::styled(type_str, Style::default().fg(Color::Cyan)), 
            Span::raw("] "),
            Span::raw(padding), 
            Span::raw(format!("{:<8}", a.currency)),
            Span::styled(format!("{:>13}",fmt_money(a.opening_balance.0)), Style::default().fg(balance_color)),
        ]);
        ListItem::new(line)
    }).collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(" Accounts (n:New e:Edit d:Del g: Goal Enter:Txns) "))
        .highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD));

    f.render_stateful_widget(list, cols[0], &mut app.accounts.sel);

    // Details
    let right = if let Some(acc) = app.current_account() {
        let balance_val = acc.opening_balance.0;
        let b_color = if balance_val.is_sign_negative() { Color::Red } else { Color::Green };

      
        let text = vec![
            Line::from(vec![Span::raw("ID:       "), Span::raw(acc.id.to_string())]),
            Line::from(vec![Span::raw("Name:     "), Span::styled(&acc.name, Style::default().add_modifier(Modifier::BOLD))]),
            Line::from(vec![Span::raw("Type:     "), Span::styled(format!("{:?}", acc.r#type), Style::default().fg(Color::Cyan))]),
            Line::from(vec![Span::raw("Currency: "), Span::raw(&acc.currency)]),
            Line::from(vec![Span::raw("Balance:  "), Span::styled(fmt_money(balance_val), Style::default().fg(b_color))]),
            Line::from(""),
            Line::from(vec![Span::raw("Created:  "), Span::raw(&acc.created_at)]),
        ];
        Paragraph::new(text)
    } else {
        Paragraph::new("No account selected")
    }.block(Block::default().borders(Borders::ALL).title(" Details "));
    
    f.render_widget(right, cols[1]);
}


fn draw_new_goal_modal(f: &mut Frame, area: Rect, app: &mut App) {
    let form = &app.dashboard.form;
    use crate::cli::state::GoalField;

    let style_line = |target: GoalField, label: &str, value: &str| -> Line {
        if Some(target) == form.editing {
            Line::from(vec![
                Span::styled(" > ", Style::default().fg(Color::Yellow)),
                Span::styled(format!("{:<10}: {}", label, value), Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            ])
        } else {
             Line::from(vec![
                Span::raw("   "),
                Span::raw(format!("{:<10}: {}", label, value)),
            ])
        }
    };

    let mut lines = vec![
        Line::from(""),
        style_line(GoalField::Name, "Goal Name", &form.name),
        style_line(GoalField::Target, "Target $", &form.target),
        style_line(GoalField::Current, "Current $", &form.current),
        style_line(GoalField::Deadline, "Deadline", &form.deadline),
        Line::from(""),
        Line::from(Span::styled(" [Enter] Save   [Esc] Cancel ", Style::default().add_modifier(Modifier::DIM))),
    ];

    if let Some(err) = &form.error {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(format!(" Error: {}", err), Style::default().fg(Color::Red))));
    }

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Create New Saving Goal ")
        .style(Style::default().bg(Color::Black));

    let p = Paragraph::new(lines).block(block);
    f.render_widget(p, area);
}

fn draw_new_account_modal(f: &mut Frame, area: Rect, app: &mut App) {
    let form = &app.accounts.form;

    let (help_text, key_hint) = match form.editing {
        Some(AccField::Name) => (
            "Enter name of the account", 
            "[Enter] Next Field    [Esc] Cancel"
        ),
        Some(AccField::Type) => (
            "Use ↑/↓ keys to change Account Type", 
            "[Enter] Next Field    [Esc] Cancel"
        ),
        Some(AccField::Currency) => (
            "Currency code (e.g. CAD, USD, CNY)", 
            "[Enter] Next Field    [Esc] Cancel"
        ),
        Some(AccField::Opening) => (
            "Initial Balance (Positive=Asset, Negative=Debt)", 
            "[Enter] Save/Create   [Esc] Cancel" 
        ),
        None => ("", "[Esc] Cancel"),
    };

  
    let style_line = |target: AccField, label: &str, value: &str| -> Line {
        if Some(target) == form.editing {
        
            Line::from(vec![
                Span::styled(" > ", Style::default().fg(Color::Yellow)),
                Span::styled(
                    format!("{:<9}: {}", label, value), 
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
                ),
            ])
        } else {
            
            Line::from(vec![
                Span::raw("    "),
                Span::raw(format!("{:<9}: {}", label, value)),
            ])
        }
    };

    let type_str = if Some(AccField::Type) == form.editing {
        format!("{:?} (Use ↑/↓)", form.r#type)
    } else {
        format!("{:?}", form.r#type)
    };


    let mut lines = vec![
        Line::from(""), 
        style_line(AccField::Name, "Name", &form.name),
        style_line(AccField::Type, "Type", &type_str),
        style_line(AccField::Currency, "Currency", &form.currency),
        style_line(AccField::Opening, "Opening", &form.opening),
        
        Line::from(""),

        Line::from(Span::styled(" -------------------------------------------------- ", Style::default().add_modifier(Modifier::DIM))),
    
        Line::from(Span::styled(format!(" Tip: {}", help_text), Style::default().fg(Color::Cyan))),
        
        Line::from(""),

        Line::from(Span::styled(format!(" {}", key_hint), Style::default().add_modifier(Modifier::DIM))),
    ];

    if let Some(err) = &form.error {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(format!(" Error: {}", err), Style::default().fg(Color::Red))));
    }

   let p = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL).title(" New Account "))
        .wrap(Wrap { trim: true }); 

    f.render_widget(p, area);
}
// Transactions Page
fn draw_txns(f: &mut Frame, area: Rect, app: &mut App) {
    let header = Row::new(vec!["Date", "Category", "Memo", "Amount"])
        .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)) 
        .height(1);

    let body: Vec<Row> = app.txn.table.iter().map(|t| {
        let amt_style = if t.amount.0.is_sign_negative() { 
            Style::default().fg(Color::Red) 
        } else { 
            Style::default().fg(Color::Green) 
        };
       let cat_str = t.category_id
            .and_then(|id| app.add.categories.iter().find(|c| c.id == id)) 
            .map(|c| format!("{} {}", c.icon, c.name)) 
            .unwrap_or_else(|| t.category_id.map(|id| format!("#{}", id)).unwrap_or("-".into())); 

        Row::new(vec![
            Cell::from(t.txn_date.to_string()),
            Cell::from(cat_str),
            Cell::from(t.memo.clone().unwrap_or_default()),
            Cell::from(Span::styled(fmt_money(t.amount.0), amt_style)),
        ])
    }).collect();

    let widths = [
        Constraint::Length(15),
        Constraint::Length(20),
        Constraint::Percentage(50),
        Constraint::Length(15),
    ];

    let mut tsel = app.txn.tsel.clone();
    let table = Table::new(body, widths)
        .header(header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(if app.txn.loading { " Transactions (Loading...) " } else { " Transactions (a:Add e:Edit d:Del Esc:Back) " }),
        )
        .highlight_style(Style::default().bg(Color::DarkGray));

    f.render_stateful_widget(table, area, &mut tsel);
    app.txn.tsel = tsel; 
}

// Add Transaction Page
fn draw_add_txn(f: &mut Frame, area: Rect, app: &mut App) {
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
        .split(area);

    let left_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(10), Constraint::Length(10)]) 
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
    
    let items: Vec<ListItem> = app.add.categories.iter().map(|c| {
        let style = if c.r#type == state::CategoryType::Income { Style::default().fg(Color::Green) } else { Style::default().fg(Color::Red) };
        ListItem::new(Line::from(vec![
            Span::raw(format!("{:<15}", c.name)),
            Span::styled(format!("{:?}", c.r#type), style),
            Span::raw(format!("  {}", c.icon))
        ]))
    }).collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(" Select Category (↑/↓) "))
        .highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD));
    f.render_stateful_widget(list, cols[1], &mut app.add.cat_sel);


   
    let get_form_style = |target: EditField| -> (Style, &str) {
        if Some(target) == app.add.editing {
            (Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD), " > ")
        } else {
            (Style::default(), "   ")
        }
    };

    let (s_date, p_date) = get_form_style(EditField::Date);
    let (s_payee, p_payee) = get_form_style(EditField::Payee);
    let (s_memo, p_memo) = get_form_style(EditField::Memo);
    let (s_amt, p_amt) = get_form_style(EditField::Amount);

    let selected_cat_name = app.add.cat_sel.selected()
        .and_then(|i| app.add.categories.get(i))
        .map(|c| c.name.clone())
        .unwrap_or_else(|| "None".to_string());

    let account_name = app.accounts.list.iter()
        .find(|a| Some(a.id) == app.add.account_id) 
        .map(|a| format!("{} ({})", a.name, a.currency)) 
        .unwrap_or_else(|| "No Account Selected".to_string()); 

    let title_str = if app.add.editing_txn_id.is_some() {
            " Edit Txn (Tab:Next 't':Type Ctrl+s:Save Esc:Cancel) "
        } else {
            " Add Txn (Tab:Next 't':Type Ctrl+s:Save Esc:Cancel) "
        };
        
    let form_text = vec![
        Line::from(""),
        Line::from(vec![
                Span::raw("   Account : "), 
                Span::styled(account_name, Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
            ]), 
        Line::from(""),
        Line::from(""),
        Line::from(vec![
            Span::styled(p_date, s_date), Span::raw("Date    : "), Span::styled(&app.add.date, s_date) 
        ]),
        Line::from(vec![
            Span::styled(p_payee, s_payee), Span::raw("Payee   : "), Span::styled(&app.add.payee, s_payee)
        ]),
        Line::from(vec![
            Span::styled(p_memo, s_memo), Span::raw("Memo    : "), Span::styled(&app.add.memo, s_memo)
        ]),
        Line::from(vec![
            Span::styled(p_amt, s_amt), Span::raw("Amount  : "), Span::styled(&app.add.amount, s_amt),
            Span::raw("  "),
            if app.add.is_expense { 
                Span::styled("[Expense -]", Style::default().fg(Color::Red)) 
            } else { 
                Span::styled("[Income +]", Style::default().fg(Color::Green)) 
            }
        ]),
        Line::from(""),
        Line::from(vec![
            Span::raw("   Category: "), Span::styled(selected_cat_name, Style::default().fg(Color::Cyan))
        ]),
    ];

   f.render_widget(
        Paragraph::new(form_text).block(
            Block::default()
                .borders(Borders::ALL)
                .title(title_str) 
        ),
        left_chunks[0]
    );


    let help_lines = vec![
        Line::from(vec![Span::styled(" Controls: ", Style::default().fg(Color::Yellow))]),
        Line::from("  TAB: Switch Field | ENTER: Edit Mode"),
        Line::from("  t: Toggle Income/Expense"),
        Line::from("  Ctrl+s: Save | ESC: Back"),
        Line::from(""),
        if let Some(err) = &app.add.error {
            Line::from(Span::styled(format!(" Error: {}", err), Style::default().fg(Color::Red)))
        } else if let Some(succ) = &app.add.success {
            Line::from(Span::styled(format!(" Success: {}", succ), Style::default().fg(Color::Green)))
        } else {
            Line::from("")
        }
    ];
    
    f.render_widget(
        Paragraph::new(help_lines).block(Block::default().borders(Borders::ALL).title(" Status ")).wrap(Wrap{ trim: true }),
        left_chunks[1]
    );
}

fn draw_help(f: &mut Frame, area: Rect) {
   let help_text = vec![
        "Global Keys:",
        "  q        : Quit App",
        "  ?        : Toggle this Help",
        "  Tab      : Switch Tabs",
        "",
        "Accounts Tab:",
        "  n        : Create New Account",
        "  e        : Edit Selected Account",
        "  d        : Delete Selected Account",
        "  Enter    : View Transactions",
        "  r        : Refresh",
        "",
        "Transactions Tab:",
        "  a        : Add Transaction",
        "  x/Del    : Delete Transaction",
        "  Esc      : Back to Accounts",
        "",
        
        "Dashboard / Goals Tab:",
        "  n        : New Saving Goal",
        "  e        : Edit Selected Goal (Update Amount)",
        "  d        : Delete Selected Goal",
        "  ↑ / ↓    : Select Goal",
        "  Esc      : Back to Accounts",
        "",
    
        "Add Transaction Tab:",
        "  Ctrl+s   : Save",
        "  t        : Toggle Expense/Income",
        "  Enter    : Toggle Edit Mode",
    ].join("\n");

    let p = Paragraph::new(help_text)
        .block(Block::default().borders(Borders::ALL).title(" Help & Keys ").style(Style::default().fg(Color::Cyan)));
    
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

use crate::cli::state::DashboardPage;
use rust_decimal::prelude::ToPrimitive; 

pub fn ui_dashboard(f: &mut Frame, page: &DashboardPage, area: Rect) { 
    if page.loading {
        let p = Paragraph::new("Loading...").alignment(Alignment::Center);
        f.render_widget(p, area); 
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area); 

    let left_block = Block::default().title(" Savings Goals (n:New e:Edit d:Del ↑/↓:Select) ").borders(Borders::ALL);
    let left_area = left_block.inner(chunks[0]);
    f.render_widget(left_block, chunks[0]);

    if page.goals.is_empty() {
        f.render_widget(Paragraph::new("No goals set (Press 'n' to add)").alignment(Alignment::Center), left_area);
    } else {
        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Length(3); page.goals.len()])
            .split(left_area);

        for (i, goal) in page.goals.iter().enumerate() {
            if i >= rows.len() { break; }
            let current = goal.current_amount.0.to_f64().unwrap_or(0.0);
            let target = goal.target_amount.0.to_f64().unwrap_or(1.0);
            let ratio = (current / target).clamp(0.0, 1.0);
            let percent = (ratio * 100.0) as u16;

            let is_selected = i == page.selected_index;
            let label_color = if is_selected { Color::Yellow } else { Color::White };
            let prefix = if is_selected { "> " } else { "  " };

            let label = format!("{}{}: {}/{} ({:.0}%)", prefix, goal.name, goal.current_amount.0, goal.target_amount.0, ratio * 100.0);

           
            let gauge = Gauge::default()
                .block(Block::default().borders(Borders::NONE))
                .gauge_style(Style::default().fg(Color::Green)) 
                .style(Style::default().fg(label_color))       
                .percent(percent)
                .label(Span::styled(label, Style::default().fg(label_color).add_modifier(Modifier::BOLD)));

            f.render_widget(gauge, rows[i]);
        }
    }

    let right_block = Block::default().title(" Monthly Spending ").borders(Borders::ALL);
    let right_area = right_block.inner(chunks[1]);
    f.render_widget(right_block, chunks[1]);

    // let bar_data: Vec<(&str, u64)> = page.report.iter()
    //     .map(|r| (r.category.as_str(), r.total_amount.0.to_u64().unwrap_or(0)))
    //     .collect();

    let labels: Vec<String> = page.report.iter()
        .map(|r| format!("{} ${}", r.category, r.total_amount.0)) 
        .collect();

    let bar_data: Vec<(&str, u64)> = page.report.iter()
        .zip(labels.iter()) 
        .map(|(r, label)| (
            label.as_str(), 
            r.total_amount.0.to_u64().unwrap_or(0) 
        ))
        .collect();

    let barchart = BarChart::default()
        .data(&bar_data)
        .bar_width(15)
        .bar_gap(2)
        .style(Style::default().fg(Color::Yellow))
        .value_style(Style::default().bg(Color::Yellow).fg(Color::Black));
    
    f.render_widget(barchart, right_area);
}