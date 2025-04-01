use sqlx::{sqlite::SqliteConnectOptions, SqlitePool};
use std::path::Path;

use crate::Message;




pub async fn create_database<P: AsRef<Path>>(path: P) -> Message {
    let options = SqliteConnectOptions::new()
        .filename(path)
        .create_if_missing(true);

    match SqlitePool::connect_with(options).await {
        Ok(pool) => {
            Message::CreateDatabaseSuccess(pool)
        }
        Err(err) => {
            Message::CreateDatabaseFailure(err.to_string())
        }
    }
}

pub async fn close_database(pool: SqlitePool) -> Message {
    pool.close().await;
    Message::ClosedDatabase
}
