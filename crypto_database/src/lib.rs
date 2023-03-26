pub mod models;
mod schema;

use self::models::*;
use crate::schema::coinbase_transactions::{self, dsl::*};
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
    conn: &mut PgConnection,
) -> Result<CoinbaseTransaction, Error> {
    diesel::insert_into(coinbase_transactions)
        .values(&new_coinbase_transaction)
        .get_result::<CoinbaseTransaction>(conn)
}

pub fn get_coinbase_transactions(
    conn: &mut PgConnection,
) -> Result<Vec<CoinbaseTransaction>, Error> {
    coinbase_transactions
        .limit(10i64)
        .load::<CoinbaseTransaction>(conn)
}

pub fn get_coinbase_transaction(
    id: coinbase_transactions::columns::id,
    conn: &mut PgConnection,
) -> Result<Vec<CoinbaseTransaction>, Error> {
    coinbase_transactions
        .find(id)
        .load::<CoinbaseTransaction>(conn)
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
