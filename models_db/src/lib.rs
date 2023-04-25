pub mod schema;

use crate::schema::{coinbase_transactions, kraken_transactions};
use chrono::prelude::*;
use diesel::prelude::*;
use models::{coinbase::INPUT_TRANSACTIONS, InputTransaction};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Queryable, Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct CoinbaseTransaction {
    pub id: i32,
    pub time_of_transaction: DateTime<Utc>,
    pub transaction_type: String,
    pub asset: String,
    pub quantity_transacted: Decimal,
    pub spot_price_currency: String,
    pub spot_price_at_transaction: Option<Decimal>,
    pub subtotal: Option<Decimal>,
    pub total: Option<Decimal>,
    pub fees: Option<Decimal>,
    pub notes: String,
}

impl InputTransaction for CoinbaseTransaction {
    fn is_input_transaction(&self) -> bool {
        INPUT_TRANSACTIONS
            .iter()
            .any(|received_transaction_type| received_transaction_type.eq(&self.transaction_type))
    }
}

#[derive(Insertable, Deserialize, PartialEq, Eq, Clone)]
#[diesel(table_name = coinbase_transactions)]
pub struct NewCoinbaseTransaction {
    pub time_of_transaction: DateTime<Utc>,
    pub transaction_type: String,
    pub asset: String,
    pub quantity_transacted: Decimal,
    pub spot_price_currency: String,
    pub spot_price_at_transaction: Option<Decimal>,
    pub subtotal: Option<Decimal>,
    pub total: Option<Decimal>,
    pub fees: Option<Decimal>,
    pub notes: String,
}

#[derive(Queryable, Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct KrakenTransaction {
    pub id: i32,
    pub txid: Option<String>,
    pub refid: String,
    pub transaction_time: DateTime<Utc>,
    pub record_type: String,
    pub subtype: Option<String>,
    pub a_class: String,
    pub asset: String,
    pub amount: Decimal,
    pub fee: Decimal,
    pub balance: Option<Decimal>,
}

#[derive(Insertable, Deserialize, PartialEq, Eq, Clone)]
#[diesel(table_name = kraken_transactions)]
pub struct NewKrakenTransaction {
    pub txid: Option<String>,
    pub refid: String,
    pub transaction_time: DateTime<Utc>,
    pub record_type: String,
    pub subtype: Option<String>,
    pub a_class: String,
    pub asset: String,
    pub amount: Decimal,
    pub fee: Decimal,
    pub balance: Option<Decimal>,
}

#[derive(Deserialize)]
pub struct Pagination {
    pub page: i64,
    #[serde(alias = "rows")]
    pub items_per_page: i64,
}

impl Default for Pagination {
    fn default() -> Self {
        Self {
            page: 0,
            items_per_page: 10,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct DBConfigOptions {
    pub host: Option<String>,
    pub port: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub database_name: Option<String>,
}

impl Default for DBConfigOptions {
    fn default() -> Self {
        Self {
            host: Default::default(),
            port: Default::default(),
            username: Default::default(),
            password: Default::default(),
            database_name: Default::default(),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
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
    pub fn new(db_config_options: Option<DBConfigOptions>) -> Self {
        let default_config = DBConfig::default();

        match db_config_options {
            Some(config_options) => Self {
                host: config_options.host.unwrap_or(default_config.host),
                port: config_options.port.unwrap_or(default_config.port),
                username: config_options.username.unwrap_or(default_config.username),
                password: config_options.password.unwrap_or(default_config.password),
                database_name: config_options
                    .database_name
                    .unwrap_or(default_config.database_name),
            },
            None => return default_config,
        }
    }

    pub fn connection_string(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.get_username(),
            self.get_password(),
            self.get_host(),
            self.get_port(),
            self.get_database_name()
        )
    }

    pub fn get_host(&self) -> &str {
        &self.host
    }

    pub fn get_port(&self) -> &str {
        &self.port
    }

    pub fn get_username(&self) -> &str {
        &self.username
    }

    pub fn get_password(&self) -> &str {
        &self.password
    }

    pub fn get_database_name(&self) -> &str {
        &self.database_name
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

#[cfg(test)]
mod db_config_should {
    use super::DBConfig;

    #[test]
    fn return_connection_string() {
        let host = "test_host".to_string();
        let port = "test_port".to_string();
        let password = "test_password".to_string();
        let username = "test_username".to_string();
        let database_name = "test_db".to_string();

        let config = DBConfig {
            host: host.clone(),
            port: port.clone(),
            password: password.clone(),
            username: username.clone(),
            database_name: database_name.clone(),
        };

        let expected = format!("postgres://{username}:{password}@{host}:{port}/{database_name}");
        assert_eq!(config.connection_string(), expected);
    }

    #[test]
    fn have_correct_assignments_from_new() {}
}
