// use sqlx::postgres::PgPoolOptions;
// use sqlx::{Pool, Postgres};
use sqlx::{Pool, Sqlite};
use sqlx::sqlite::SqlitePoolOptions;

use std::env;

pub async fn get_db_pool() -> Result<Pool<Sqlite>, sqlx::Error> {
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await
        // .expect("Failed to connect to database") //returns Pool<Sqlite> not Result
}


/* postgresql version */
// pub async fn get_db_pool() -> Result<Pool<Postgres>, sqlx::Error> {
//     let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

//     PgPoolOptions::new()
//         .max_connections(5)
//         .connect(&db_url)
//         .await
//         // .expect("Failed to connect to database") //returns Pool<Sqlite> not Result
// }