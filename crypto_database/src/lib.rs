pub mod models;
mod schema;

use self::models::*;
use crate::schema::coinbase_transactions::dsl::coinbase_transactions;
use diesel::{prelude::*, result::Error};
use dotenvy::{self, dotenv};

pub fn get_connection_string() -> (String, String, String) {
    let host = std::env::var("DB_HOST").unwrap_or("0.0.0.0".to_string());
    let port = std::env::var("DB_PORT").unwrap_or("5432".to_string());
    let user_name = std::env::var("DB_USER").unwrap_or("super_user".to_string());
    let password = std::env::var("DB_PASSWORD").unwrap_or("password".to_string());
    let database_name = std::env::var("DB_NAME").unwrap_or("crypto_database".to_string());

    (
        format!("postgres://{user_name}:{password}@{host}:{port}/{database_name}"),
        host,
        port,
    )
}

pub fn establish_connection() -> PgConnection {
    let (db_url, host, port) = get_connection_string();

    PgConnection::establish(&db_url)
        .unwrap_or_else(|_| panic!("Error connecting with connection: {}:{}", host, port))
}

pub fn insert_coinbase_transaction(
    new_coinbase_transaction: NewCoinbaseTransaction,
    connection: &mut PgConnection,
) -> Result<CoinbaseTransaction, Error> {
    diesel::insert_into(coinbase_transactions)
        .values(&new_coinbase_transaction)
        .get_result::<CoinbaseTransaction>(connection)
}

pub fn bulk_insert_coinbase_transaction(
    new_coinbase_transactions: Vec<NewCoinbaseTransaction>,
    connection: &mut PgConnection,
) -> Result<Vec<CoinbaseTransaction>, Error> {
    diesel::insert_into(coinbase_transactions)
        .values(&new_coinbase_transactions)
        .get_results::<CoinbaseTransaction>(connection)
}

pub fn get_coinbase_transactions(
    pagination: &Pagination,
    connection: &mut PgConnection,
) -> Result<Vec<CoinbaseTransaction>, Error> {
    coinbase_transactions
        .offset(pagination.items_per_page * pagination.page)
        .limit(pagination.items_per_page)
        .get_results::<CoinbaseTransaction>(connection)
}

pub fn get_coinbase_transaction(
    id: i32,
    connection: &mut PgConnection,
) -> Result<CoinbaseTransaction, Error> {
    coinbase_transactions
        .find(id)
        .get_result::<CoinbaseTransaction>(connection)
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn it_works() {
//         let result = add(2, 2);
//         assert_eq!(result, 4);
//     }
// }
