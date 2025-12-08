// src/main.rs
use personal_finance_tracker::cli; // crate 名= Cargo.toml 里的 name 把 - 换 _

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    cli::run().await
}
