mod models;
mod schema;

use self::models::*;
use crate::schema::coinbase_transactions::dsl::*;
use diesel::{prelude::*, result::Error};

pub fn insert_coinbase_transaction(
    new_coinbase_transaction: NewCoinbaseTransaction,
    conn: &mut PgConnection,
) -> Result<CoinbaseTransaction, Error> {
    diesel::insert_into(coinbase_transactions)
        .values(&new_coinbase_transaction)
        .get_result::<CoinbaseTransaction>(conn)
}

pub fn get_coinbase_transactions(conn: &mut PgConnection) -> Vec<CoinbaseTransaction> {
    coinbase_transactions
        .limit(5i64)
        .load::<CoinbaseTransaction>(conn)
        .expect("Failed to get CoinbaseTransactions")
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
