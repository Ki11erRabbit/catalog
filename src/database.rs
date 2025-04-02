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

pub async fn open_database<P: AsRef<Path>>(path: P) -> Message {
    let options = SqliteConnectOptions::new()
        .filename(path);

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

pub async fn initialize_database(pool: SqlitePool) -> Message {

    match pool.begin().await {
        Err(err) => {
            Message::DatabaseTransactionFailure(pool, err.to_string())
        }
        Ok(mut connection) => {
            let statements = [
                "CREATE TABLE Rack (rack_id UNSIGNED BIG INT, PRIMARY KEY (rack_id))",
                "CREATE TABLE Shelf (shelf_id UNSIGNED BIG INT, rack_id UNSIGNED BIG INT, PRIMARY KEY (shelf_id), FOREIGN KEY (rack_id) REFERENCES Rack(rack_id))",
                "CREATE TABLE Basket (shelf_id UNSIGNED BIG INT, basket_id UNSIGNED BIG INT, PRIMARY KEY (basket_id), FOREIGN KEY (shelf_id) REFERENCES Shelf(shelf_id))",
                "CREATE TABLE Item (item_id UNSIGNED BIG INT, name TEXT, rack_id UNSIGNED BIG INT, shelf_id UNSIGNED BIG INT, basket_id UNSIGNED BIG INT, PRIMARY KEY (item_id), FOREIGN KEY (shelf_id) REFERENCES Shelf(shelf_id) ON DELETE CASCADE, FOREIGN KEY (basket_id) REFERENCES Basket(basket_id) ON DELETE CASCADE, FOREIGN KEY (rack_id) REFERENCES Rack(rack_id) ON DELETE CASCADE)",
            ];

            for stmt in statements {
                let result = sqlx::query(stmt)
                    .execute(&mut *connection)
                    .await;
                match result {
                    Err(err) => return Message::DatabaseTransactionFailure(pool, err.to_string()),
                    _ => {}
                }
            }
            match connection.commit().await {
                Err(err) => return Message::DatabaseTransactionFailure(pool, err.to_string()),
                _ => {}
            }
            Message::DatabaseTransactionSuccess(pool)
        }
    }
}


pub async fn insert(
    pool: SqlitePool,
    rack: String,
    shelf: String,
    basket: String,
    name: String
) -> Message {
    match pool.begin().await {
        Err(err) => {
            Message::DatabaseTransactionFailure(pool, err.to_string())
        }
        Ok(mut connection) => {
            let result = sqlx::query("INSERT INTO Rack IF NOT EXISTS (rack_id) VALUES ($1)")
                .bind(&rack)
                .execute(&mut *connection)
                .await;

            let result = sqlx::query("INSERT INTO Shelf IF NOT EXISTS (shelf_id) VALUES ($1)")
                .bind(&shelf)
                .execute(&mut *connection)
                .await;

            let result = sqlx::query("INSERT INTO Basket IF NOT EXISTS (basket_id) VALUES ($1)")
                .bind(&basket)
                .execute(&mut *connection)
                .await;

            let result = sqlx::query("INSERT INTO Item (rack_id, shelf_id, basket_id, name) VALUES ($1, $2, $3, $4)")
                .bind(rack)
                .bind(shelf)
                .bind(basket)
                .bind(name)
                .execute(&mut *connection)
                .await;

            let result = connection.commit();

            Message::DatabaseTransactionSuccess(pool)
        }
    }
}
