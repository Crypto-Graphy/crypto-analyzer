pub mod models;
mod schema;

use self::models::*;
use crate::schema::coinbase_transactions::dsl::coinbase_transactions;
use diesel::{prelude::*, result::Error};
use dotenvy::{self, dotenv};

pub fn establish_connection() -> PgConnection {
    dotenv().ok();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL is a required env var");
    PgConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
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
        .load::<CoinbaseTransaction>(connection)
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
