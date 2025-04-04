use sqlx::{sqlite::SqliteConnectOptions, SqlitePool, Row};
use std::path::Path;

use crate::{ItemInfo, Message};




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
                "CREATE TABLE Item (item_id INTEGER PRIMARY KEY AUTOINCREMENT, name TEXT, rack_id UNSIGNED BIG INT, shelf_id UNSIGNED BIG INT, basket_id UNSIGNED BIG INT, FOREIGN KEY (shelf_id) REFERENCES Shelf(shelf_id) ON DELETE CASCADE, FOREIGN KEY (basket_id) REFERENCES Basket(basket_id) ON DELETE CASCADE, FOREIGN KEY (rack_id) REFERENCES Rack(rack_id) ON DELETE CASCADE)",
                "CREATE INDEX index_item_name ON Item (name)"
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
            let result = sqlx::query("INSERT OR IGNORE INTO Rack (rack_id) VALUES ($1)")
                .bind(&rack)
                .execute(&mut *connection)
                .await;

            match result {
                Err(err) => {
                    return Message::DatabaseTransactionFailure(pool, err.to_string());
                }
                Ok(_) => {}
            }

            let result = sqlx::query("INSERT OR IGNORE INTO Shelf (shelf_id) VALUES ($1)")
                .bind(&shelf)
                .execute(&mut *connection)
                .await;

            match result {
                Err(err) => {
                    return Message::DatabaseTransactionFailure(pool, err.to_string());
                }
                Ok(_) => {}
            }

            let result = sqlx::query("INSERT OR IGNORE INTO Basket (basket_id) VALUES ($1)")
                .bind(&basket)
                .execute(&mut *connection)
                .await;

            match result {
                Err(err) => {
                    return Message::DatabaseTransactionFailure(pool, err.to_string());
                }
                Ok(_) => {}
            }

            let result = sqlx::query("INSERT INTO Item (rack_id, shelf_id, basket_id, name) VALUES ($1, $2, $3, $4)")
                .bind(rack)
                .bind(shelf)
                .bind(basket)
                .bind(name)
                .execute(&mut *connection)
                .await;

            match result {
                Err(err) => {
                    return Message::DatabaseTransactionFailure(pool, err.to_string());
                }
                Ok(_) => {}
            }

            let result = connection.commit().await;

            match result {
                Err(err) => {
                    return Message::DatabaseTransactionFailure(pool, err.to_string());
                }
                Ok(_) => {}
            }

            Message::DatabaseTransactionSuccess(pool)
        }
    }
}

pub async fn search(
    pool: SqlitePool,
    name: String
) -> Message {
    match pool.begin().await {
        Err(err) => {
            Message::DatabaseTransactionFailure(pool, err.to_string())
        }
        Ok(mut connection) => {
            let result = sqlx::query("SELECT * FROM Item INDEXED BY index_item_name WHERE name = $1")
                .bind(name)
                .fetch_all(&mut *connection)
                .await;

            let result = match result {
                Err(err) => {
                    return Message::DatabaseTransactionFailure(pool, err.to_string());
                }
                Ok(result) => result,
            };

            if result.len() == 0 {
                Message::DatabaseSearchFailure(pool)
            } else {

                let result = result.into_iter()
                    .map(|row| {
                        ItemInfo {
                            rack_number: row.get::<i64,_>("rack_id").to_string(),
                            shelf_number: row.get::<u64,_>("shelf_id").to_string(),
                            basket_number: row.get::<u64,_>("basket_id").to_string(),
                            item_name: row.get("name"),
                        }
                    })
                    .collect::<Vec<_>>();
                Message::DatabaseSearchSuccess(pool, result)
            }
        }
    }
}
