use crate::schema::coinbase_transactions;
use chrono::prelude::*;
use diesel::prelude::*;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Queryable, Serialize, Deserialize)]
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

#[derive(Insertable, Deserialize)]
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
