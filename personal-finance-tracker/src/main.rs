use std::env;
use dotenvy::dotenv;
use personal_finance_tracker::{backend, cli, database};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();
    
    let args: Vec<String> = env::args().collect();

    if args.len() > 1 && args[1] == "server" {
        println!("Starting Backend Server...");

        let pool = database::db::connection::get_db_pool().await?;
        
        backend::run_server(pool).await?;
    } else {
        println!("Starting CLI...");
        

        cli::run().await?;
    }
    Ok(())
}