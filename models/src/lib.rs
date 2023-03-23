use chrono::{DateTime, TimeZone, Utc};
use serde::{Deserialize, Deserializer};

pub mod coinbase {
    pub use chrono::{DateTime, Utc};
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
        #[serde(rename(serialize = "timeOfTransaction", deserialize = "Timestamp"))]
        pub time_of_transaction: DateTime<Utc>,
        #[serde(rename(serialize = "transactionType", deserialize = "Transaction Type"))]
        pub transaction_type: String,
        #[serde(rename(serialize = "asset", deserialize = "Asset"))]
        pub asset: String,
        #[serde(rename(serialize = "quantityTransacted", deserialize = "Quantity Transacted"))]
        pub quantity_transacted: Decimal,
        #[serde(rename(serialize = "quantityTransacted", deserialize = "Spot Price Currency"))]
        pub spot_price_currency: String,
        #[serde(rename(
            serialize = "spotPriceAtTransaction",
            deserialize = "Spot Price at Transaction"
        ))]
        pub spot_price_at_transaction: Option<Decimal>,
        #[serde(rename(serialize = "subtotal", deserialize = "Subtotal"))]
        pub subtotal: Option<Decimal>,
        #[serde(rename(
            serialize = "total",
            deserialize = "Total (inclusive of fees and/or spread)"
        ))]
        pub total: Option<Decimal>,
        #[serde(rename(serialize = "fees", deserialize = "Fees and/or Spread"))]
        pub fees: Option<Decimal>,
        #[serde(rename(serialize = "notes", deserialize = "Notes"))]
        pub notes: String,
    }
}

pub mod kraken {
    pub const DATE_FORMAT: &'static str = "%Y-%m-%d %H:%M:%S";
    use chrono::TimeZone;
    pub use chrono::{DateTime, Utc};
    use rust_decimal::Decimal;
    use serde::{Deserialize, Deserializer, Serialize};

    pub const CSV_HEADERS: &[&str] = &[
        "txid", "refid", "time", "type", "subtype", "aclass", "asset", "amount", "fee", "balance",
    ];

    #[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
    #[serde(rename_all(serialize = "camelCase"))]
    pub struct KrakenLedgerRecord {
        pub txid: Option<String>,
        pub refid: String,
        #[serde(deserialize_with = "parse_date_time")]
        pub time: DateTime<Utc>,
        #[serde(rename(deserialize = "type"))]
        pub record_type: String,
        pub subtype: Option<String>,
        #[serde(rename(deserialize = "aclass"))]
        pub a_class: String,
        pub asset: String,
        #[serde(with = "rust_decimal::serde::str")]
        pub amount: Decimal,
        #[serde(with = "rust_decimal::serde::str")]
        pub fee: Decimal,
        #[serde(with = "rust_decimal::serde::str_option")]
        pub balance: Option<Decimal>,
    }

    fn parse_date_time<'de, D: Deserializer<'de>>(d: D) -> Result<DateTime<Utc>, D::Error> {
        // 2021-09-29 15:18:30
        let s: Option<String> = Deserialize::deserialize(d)?;

        Utc.datetime_from_str(&s.unwrap(), DATE_FORMAT)
            .map_err(serde::de::Error::custom)
    }
}
