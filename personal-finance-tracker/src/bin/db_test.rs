
use dotenvy::dotenv;
use personal_finance_tracker::database::db::connection::get_db_pool;
use personal_finance_tracker::database::db::queries;
use personal_finance_tracker::database::models::{Budget, SavingsGoal};

use rust_decimal::Decimal;
use chrono::NaiveDateTime;
use std::str::FromStr;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();
    
    /* ==========Testing========== */
    let pool = get_db_pool().await?;
    
    sqlx::migrate!("./migrations").run(&pool).await?;
    println!("Migrations ran successfully!");

    let initial_acc_name = "test-account-cash";
    let initial_acc_type = "cash";
    let initial_currency = "CAD";

      // ----------------------------------------------------
    // TEST：CURRENCY RATE CONVERSION
    // ----------------------------------------------------
    println!("\n--- Testing: upsert_rate ---");
    queries::insert_rate(&pool, "CAD", 1.0).await?;
    queries::insert_rate(&pool, "USD", 0.75).await?;

    println!("Fetching CAD rate...");
    let rate = queries::get_rate(&pool, "USD").await?;
    println!("CAD/USD = {}", rate);


    // ----------------------------------------------------
    // TEST：CREATE ACCOUNT
    // ----------------------------------------------------
    println!("\n--- Testing: create_account ---");
    let account_id = queries::create_account(&pool, initial_acc_name, initial_acc_type, initial_currency).await?;
    println!("   > Account created successfully {}", account_id);
    assert!(account_id > 0, "Failed to create Account, ID invalid.");

    tokio::time::sleep(std::time::Duration::from_millis(50)).await;     //delay
    println!("\n--- Testing: get_account_by_id ---");
    let account_id_to_check = account_id;
    println!("--- searching account_id: {} ---", account_id_to_check);
    let fetched_account = queries::get_account_by_id(&pool, account_id_to_check).await?;
    println!("   > Acquired account: {:?}", fetched_account);
    assert_eq!(fetched_account.account_name, initial_acc_name, "account id not matched");


    println!("\n--- Testing: get_all_accounts ---");
    let account_num  = queries::get_all_accounts(&pool).await?;
    println!("   > Number of account: {:?}", account_num.len());
    // assert_eq!(account_num.len(), 2, "number of accounts unmatched!");
    
    // Check if the initial balance is 0.00.
    let zero_decimal = Decimal::from_str("0.00").unwrap();
    assert_eq!(fetched_account.balance, zero_decimal, "Initial balance is not zero!");

    // ----------------------------------------------------
    // TEST：UPDATE ACCOUNT
    // ----------------------------------------------------
    let new_acc_name = "Name changed - Savings Account";
    let new_acc_type = "Savings Account";
    println!("\n--- Testing: update_account ---");
    let update_success = queries::update_account(&pool, account_id, new_acc_name.to_string(), new_acc_type.to_string()).await?;
    println!("   > Updated successfully: {}", update_success);
    assert!(update_success, "Failed to update account!");

    // Update verification
    let updated_account = queries::get_account_by_id(&pool, account_id).await?;
    assert_eq!(updated_account.account_name, new_acc_name, "Updated account name does not match");

    // ----------------------------------------------------
    // TEST：CREATE CATEGORY
    // ----------------------------------------------------
    let cat_name = "Grocery";
    let cat_type = "Expense";
    let cat_icon = "🛒";

    /*Note!!! did not check for duplicate when adding new category */

    println!("\n--- Testing: create_category ---");
    let category_id = queries::create_category(&pool, cat_name, cat_type, cat_icon).await?;
    println!("   > Category created successfully, ID: {}", category_id);
    assert!(category_id > 0, "Failed to create Category, ID invalid!");

    println!("\n--- Testing: get_all_categories ---");

    let category_num  = queries::get_all_categories(&pool).await?;
    println!("   > Number of category: {:?}", category_num.len());
    // assert_eq!(category_num.len(), 1, "number of category unmatched!");

    // ----------------------------------------------------
    // Test: CREATE TRANSACTION (depends on Account and Category)
    // ----------------------------------------------------
    let amount_val = Decimal::from_str("-45.45").unwrap();
    let trans_time = NaiveDateTime::parse_from_str("2025-11-22 10:00:00", "%Y-%m-%d %H:%M:%S").unwrap();
    let currency = "CAD".to_string(); 
    let rate = queries::get_rate(&pool, &currency).await?;
    // let amount_f64 = t.amount.to_f64().unwrap_or(0.0);
    let base_amount_f64 = amount_val * rate;

    println!("\n--- Testing: create_transaction ---");
    let transaction_id = queries::create_transaction(
        &pool, 
        account_id, 
        category_id,
        amount_val, 
        base_amount_f64,
        currency,
        true, // is_expense
        Some("Metro purchasing"), 
        trans_time
    ).await?;
    println!("   > Transaction created successfully, ID: {}", transaction_id);
    assert!(transaction_id > 0, "Failed to create Transaction!");
    
    // Verification for account balance update
    let final_account = queries::get_account_by_id(&pool, account_id).await?;
    let expected_balance = zero_decimal - amount_val; // 0.00 - 45.45 = -45.45
    assert_eq!(final_account.balance, expected_balance, "Post-transaction balance update error");

    println!("\n--- Testing: get_transactions_by_account ---");
    let transac_by_acc  = queries::get_transactions_by_account(&pool, 1).await?;
    println!("   > Number of transaction: {:?}", transac_by_acc.len());
    // assert_eq!(category_num.len(), 1, "number of transaction unmatched!");

    // ----------------------------------------------------
    // Test: RECURRING TRANSACTIONS
    // ----------------------------------------------------
    println!("\n--- Testing: create_recurring ---");
    let set_amount = Decimal::from_str("100000.0").unwrap();
    let recurrence_rule = "monthly";
    let next_date = NaiveDateTime::parse_from_str("2025-11-22 10:00:00", "%Y-%m-%d %H:%M:%S").unwrap();
    let recurring_success = queries::create_recurring(
        &pool, account_id,set_amount,"CAD", Some(category_id), Some("Dream car"),recurrence_rule.to_string(), next_date).await?;
    println!("   > Recurring set successfully: {}", recurring_success);

    println!("\n--- Testing: get_due ---");
    let today = "2025-12-10 10:00:00";

    println!("Fetching due recurring transactions...");
    let due = queries::get_due(&pool, today).await?;

    println!("   > Due recurring transactions: {:?}", due);
    for r in &due {
        println!(
            "ID: {}, amount: {}, desc: {}, next_run_date: {}",
            r.recurring_id, r.amount, r.description.clone().unwrap_or("None".to_string()), r.next_run_date
        );
    }

    if let Some(first) = due.first() {
        let new_date = first.next_run_date + chrono::Duration::days(30);

        println!(
            "Updating next_run_date for id {} to {}",
            first.recurring_id, new_date
        );
        queries::update_next_date(&pool, first.recurring_id, &new_date.to_string()).await?;
    }

    // ----------------------------------------------------
    // Test: BUDGETS
    // ----------------------------------------------------
    println!("\n--- Testing: create_budget ---");
    let budget = Budget {
        budget_id: 0, // DB autoincrement
        account_id: 1,
        category_id: Some(1),
        period: "monthly".to_string(),
        amount: rust_decimal::Decimal::new(500, 0),
        currency: "CAD".to_string(),
        start_date: NaiveDateTime::parse_from_str("2025-11-22 10:00:00", "%Y-%m-%d %H:%M:%S").unwrap(),
    };

    let new_id = queries::create_budget(&pool, &budget).await?;
    println!("New budget inserted with id {}", new_id);

    println!("Listing budgets for account 1...");
    let items = queries::list_by_account(&pool, 1).await?;
    for item in items {
        println!("{:?}", item);
    }
    // let set_amount = Decimal::from_str("100000.0").unwrap();
    println!("   > create_budget successfully");

    // ----------------------------------------------------
    // Test: SAVING GOAL
    // ----------------------------------------------------
    println!("\n--- Testing: create_saving ---");
    let saving = SavingsGoal {
        goal_id: 0,
        account_id: 1,
        goal_name: "Buy a new laptop".into(),
        target_amount: rust_decimal::Decimal::new(1500, 0),
        current_amount: rust_decimal::Decimal::new(200, 0),
        deadline: NaiveDateTime::parse_from_str("2025-12-31 23:59:59", "%Y-%m-%d %H:%M:%S").unwrap(),
    };

    let new_goal_id = queries::create_saving_goal(&pool, &saving).await?;
    println!("New savings goal id = {}", new_goal_id);

    println!("Updating current amount...");
    queries::update_goal_amount(&pool, new_goal_id, rust_decimal::Decimal::new(300, 0)).await?;
    println!("   > update_goal_amount successfully");

       // ----------------------------------------------------
    // TEST：REPORTS
    // ----------------------------------------------------
    let start_dt = NaiveDateTime::parse_from_str("2025-01-01 00:00:00", "%Y-%m-%d %H:%M:%S").unwrap();
    let end_dt   = NaiveDateTime::parse_from_str("2025-12-31 23:59:59", "%Y-%m-%d %H:%M:%S").unwrap();

    println!("\n--- Testing: monthly_summary ---");
    let summary = queries::monthly_summary(&pool, &start_dt, &end_dt).await?;
    // println!("monthly summary = {:?}", summary);
    for (category_id, total_amount) in summary {
        println!("Category {} spent {}", category_id, total_amount);
    }

    // ========= Test Net Savings ============
    println!("\n--- Testing: net_savings ---");
    let net = queries::net_savings(&pool).await?;
    println!("Net savings = {}", net);


    // ----------------------------------------------------
    // TEST：DELETE ACCOUNT
    // ----------------------------------------------------
    // println!("\n--- Testing: delete_account ---");
    // let delete_success = queries::delete_account(&pool, account_id).await?;
    // println!("   > Account deleted successfully: {}", delete_success);
    // assert!(delete_success, "Failed to delete account!");

    // // Verification for delete
    // let get_result = queries::get_account_by_id(&pool, account_id).await;
    // assert!(matches!(get_result, Err(sqlx::Error::RowNotFound)), "Account not deleted or queries unmatched");


    println!("\n--- All tests passed!---");
    Ok(())
}


/* commands for manipulate database */
//cargo sqlx migrate run
//cargo sqlx database reset

