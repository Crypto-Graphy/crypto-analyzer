use crate::schema::coinbase_transactions;
use chrono::prelude::*;
use diesel::prelude::*;
use rust_decimal::Decimal;

#[derive(Queryable)]
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

#[derive(Insertable)]
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
