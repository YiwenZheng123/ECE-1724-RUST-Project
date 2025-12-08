
use dotenvy::dotenv;
use personal_finance_tracker::database::db::connection::get_db_pool;
use personal_finance_tracker::database::db::queries;
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
    // TEST：CREATE ACCOUNT
    // ----------------------------------------------------
    println!("\n--- Testing: create_account ---");
    let account_id = queries::create_account(&pool, initial_acc_name, initial_acc_type, initial_currency).await?;
    println!("   > Account created successfully {}", account_id);
    assert!(account_id > 0, "Failed to create Account, ID invalid.");

    println!("\n--- Testing: get_account_by_id ---");
    let fetched_account = queries::get_account_by_id(&pool, account_id).await?;
    println!("   > Acquired account: {:?}", fetched_account);
    assert_eq!(fetched_account.account_name, initial_acc_name, "account name not matched");

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

    println!("\n--- Testing: create_category ---");
    let category_id = queries::create_category(&pool, cat_name, cat_type, cat_icon).await?;
    println!("   > Category created successfully, ID: {}", category_id);
    assert!(category_id > 0, "Failed to create Category, ID invalid!");

    println!("\n--- Testing: get_all_accounts ---");
    let category_num  = queries::get_all_categories(&pool).await?;
    println!("   > Number of category: {:?}", category_num.len());
    // assert_eq!(category_num.len(), 1, "number of category unmatched!");

    // ----------------------------------------------------
    // Test: CREATE TRANSACTION (depends on Account and Category)
    // ----------------------------------------------------
    let amount_val = Decimal::from_str("45.45").unwrap();
    let trans_time = NaiveDateTime::parse_from_str("2025-11-22 10:00:00", "%Y-%m-%d %H:%M:%S").unwrap();
    
    println!("\n--- Testing: create_transaction ---");
    let transaction_id = queries::create_transaction(
        &pool, 
        account_id, 
        Some(category_id),
        amount_val, 
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
    // TEST：DELETE ACCOUNT
    // ----------------------------------------------------
    println!("\n--- Testing: delete_account ---");
    let delete_success = queries::delete_account(&pool, account_id).await?;
    println!("   > Account deleted successfully: {}", delete_success);
    assert!(delete_success, "Failed to delete account!");

    // Verification for delete
    let get_result = queries::get_account_by_id(&pool, account_id).await;
    assert!(matches!(get_result, Err(sqlx::Error::RowNotFound)), "Account not deleted or queries unmatched");

    println!("\n--- All tests passed!---");
    Ok(())
}


/* commands for manipulate database */
//cargo sqlx migrate run
//cargo sqlx database reset
