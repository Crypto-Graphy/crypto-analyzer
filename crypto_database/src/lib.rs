use diesel::*;
use models_db::DBConfig;

pub fn establish_connection(config: Option<DBConfig>) -> Result<PgConnection, ConnectionError> {
    let config = config.unwrap_or_default();
    println!(
        "Attempting to connect to {}:{}/{}",
        config.get_host(),
        config.get_port(),
        config.get_database_name()
    );

    PgConnection::establish(&config.connection_string())
}

pub mod coinbase_db {
    use diesel::{prelude::*, result::Error};
    pub use models_db::{
        self,
        schema::{self, coinbase_transactions::dsl::coinbase_transactions},
        CoinbaseTransaction, NewCoinbaseTransaction, Pagination,
    };

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
}

pub mod kraken_db {
    use diesel::{prelude::*, result::Error};
    pub use models_db::{
        self,
        schema::{self, kraken_transactions::dsl::kraken_transactions},
        KrakenTransaction, NewKrakenTransaction, Pagination,
    };

    pub fn insert_kraken_transaction(
        new_kraken_transaction: NewKrakenTransaction,
        connection: &mut PgConnection,
    ) -> Result<KrakenTransaction, Error> {
        diesel::insert_into(kraken_transactions)
            .values(&new_kraken_transaction)
            .get_result::<KrakenTransaction>(connection)
    }

    pub fn bulk_insert_kraken_transaction(
        new_kraken_transactions: Vec<NewKrakenTransaction>,
        connection: &mut PgConnection,
    ) -> Result<Vec<KrakenTransaction>, Error> {
        diesel::insert_into(kraken_transactions)
            .values(&new_kraken_transactions)
            .get_results::<KrakenTransaction>(connection)
    }

    pub fn get_kraken_transactions(
        pagination: &Pagination,
        connection: &mut PgConnection,
    ) -> Result<Vec<KrakenTransaction>, Error> {
        kraken_transactions
            .offset(pagination.items_per_page * pagination.page)
            .limit(pagination.items_per_page)
            .get_results::<KrakenTransaction>(connection)
    }

    pub fn get_kraken_transaction(
        id: i32,
        connection: &mut PgConnection,
    ) -> Result<KrakenTransaction, Error> {
        kraken_transactions
            .find(id)
            .get_result::<KrakenTransaction>(connection)
    }
}
