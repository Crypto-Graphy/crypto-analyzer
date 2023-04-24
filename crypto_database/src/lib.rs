use diesel::*;

pub struct DBConfig {
    host: String,
    port: String,
    username: String,
    password: String,
    database_name: String,
}

impl Default for DBConfig {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".to_string(),
            port: "5432".to_string(),
            username: "super_user".to_string(),
            password: "password".to_string(),
            database_name: "crypto_data".to_string(),
        }
    }
}

impl DBConfig {
    pub fn connection_string(&self) -> String {
        "postgres://{self.user_name}:{self.password}@{self.host}:{self.port}/{self.database_name}"
            .into()
    }

    pub fn init_from_env() -> Self {
        let default = Self::default();

        Self {
            host: std::env::var("DB_HOST").unwrap_or(default.host),
            port: std::env::var("DB_PORT").unwrap_or(default.port),
            username: std::env::var("DB_USER").unwrap_or(default.username),
            password: std::env::var("DB_PASSWORD").unwrap_or(default.password),
            database_name: std::env::var("DB_NAME").unwrap_or(default.database_name),
        }
    }
}

pub fn get_connection_string() -> (String, String, String, String) {
    let host = std::env::var("DB_HOST").unwrap_or("0.0.0.0".to_string());
    let port = std::env::var("DB_PORT").unwrap_or("5432".to_string());
    let user_name = std::env::var("DB_USER").unwrap_or("super_user".to_string());
    let password = std::env::var("DB_PASSWORD").unwrap_or("password".to_string());
    let database_name = std::env::var("DB_NAME").unwrap_or("crypto_data".to_string());

    (
        format!("postgres://{user_name}:{password}@{host}:{port}/{database_name}"),
        host,
        port,
        database_name,
    )
}

pub fn establish_connection() -> Result<PgConnection, ConnectionError> {
    let (db_url, host, port, db_name) = get_connection_string();
    println!("Attempting to connect to {host}:{port}/{db_name}");

    PgConnection::establish(&db_url)
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
