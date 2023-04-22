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
