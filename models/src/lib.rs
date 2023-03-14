extern crate rust_decimal;
extern crate serde;

pub mod coinbase {
    use rust_decimal::Decimal;
    use serde::{Deserialize, Serialize};

    pub const CSV_HEADERS: &[&str] = &[
        "Timestamp",
        "Transaction Type",
        "Asset",
        "Quantity Transacted",
        "Spot Price Currency",
        "Spot Price at Transaction",
        "Subtotal",
        "Total (inclusive of fees and/or spread)",
        "Fees and/or Spread",
        "Notes",
    ];

    #[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
    pub struct CoinbaseTransactionRecord {
        #[serde(rename = "Timestamp")]
        pub time_of_transaction: String,
        #[serde(rename = "Transaction Type")]
        pub transaction_type: String,
        #[serde(rename = "Asset")]
        pub asset: String,
        #[serde(rename = "Quantity Transacted")]
        pub quantity_transacted: Decimal,
        #[serde(rename = "Spot Price Currency")]
        pub spot_price_currency: String,
        #[serde(rename = "Spot Price at Transaction")]
        pub spot_price_at_transaction: Option<Decimal>,
        #[serde(rename = "Subtotal")]
        pub subtotal: Option<Decimal>,
        #[serde(rename = "Total (inclusive of fees and/or spread)")]
        pub total: Option<Decimal>,
        #[serde(rename = "Fees and/or Spread")]
        pub fees: Option<Decimal>,
        #[serde(rename = "Notes")]
        pub notes: String,
    }
}
