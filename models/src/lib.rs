use rust_decimal::Decimal;
use std::collections::HashMap;

// TODO: This is not the right place for this trait.
pub trait StakingRewards {
    fn staking_rewards(&self) -> HashMap<String, Decimal>;
}

pub trait InputTransaction {
    fn is_input_transaction(&self) -> bool;
}

pub trait InputTransactions<T> {
    fn input_transactions(&self) -> Vec<&T>;
}

pub trait ActiveAssetValues {
    fn active_assets(&self) -> HashMap<String, Decimal>;
}

pub trait RecordsByAsset<T> {
    fn by_asset(&self) -> HashMap<String, Vec<&T>>;
}

pub mod coinbase {
    pub use chrono::{DateTime, Utc};
    use rust_decimal::Decimal;
    use serde::{Deserialize, Serialize};

    use crate::InputTransaction;

    pub const INCLUDE_TRANSACTIONS: &[&str] = &[
        "Buy",
        "Send",
        "Receive",
        "Convert",
        "Rewards Income",
        "CardSpend",
        "CardBuyBack",
        "Learning Reward",
        "Sell",
        "Advanced Trade Buy",
    ];

    pub const INPUT_TRANSACTIONS: &[&str] = &[
        "Buy",
        "Receive",
        "Rewards Income",
        "CardBuyBack",
        "Learning Reward",
        "Advanced Trade Buy",
    ];

    pub const OUTPUT_TRANSACTIONS: &[&str] = &["Sell", "Send", "CardSpend"];

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

    impl InputTransaction for CoinbaseTransactionRecord {
        fn is_input_transaction(&self) -> bool {
            INPUT_TRANSACTIONS.iter().any(|received_transaction_type| {
                received_transaction_type.eq(&self.transaction_type)
            })
        }
    }
}

pub mod kraken {
    pub const DATE_FORMAT: &str = "%Y-%m-%d %H:%M:%S";
    use chrono::TimeZone;
    pub use chrono::{DateTime, Utc};
    use rust_decimal::{prelude::Zero, Decimal};
    use serde::{Deserialize, Deserializer, Serialize};

    use crate::InputTransaction;

    pub const CSV_HEADERS: &[&str] = &[
        "txid", "refid", "time", "type", "subtype", "aclass", "asset", "amount", "fee", "balance",
    ];

    #[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
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

    impl InputTransaction for KrakenLedgerRecord {
        fn is_input_transaction(&self) -> bool {
            self.amount >= Decimal::zero()
        }
    }

    fn parse_date_time<'de, D: Deserializer<'de>>(d: D) -> Result<DateTime<Utc>, D::Error> {
        // 2021-09-29 15:18:30
        let s: Option<String> = Deserialize::deserialize(d)?;

        Utc.datetime_from_str(&s.unwrap(), DATE_FORMAT)
            .map_err(serde::de::Error::custom)
    }

    #[cfg(test)]
    mod input_transaction_should {
        use chrono::{TimeZone, Utc};
        use rust_decimal::{prelude::Zero, Decimal};

        use crate::InputTransaction;

        use super::{KrakenLedgerRecord, DATE_FORMAT as KRAKEN_DATE_FORMAT};

        #[test]
        fn find_positive_amount_as_input() {
            let record = KrakenLedgerRecord {
                txid: Some("L7RLII-OFGWB-JTUO7J".to_string()),
                refid: "RKB7ODD-ILZGC5-LCRRBL".to_string(),
                time: Utc
                    .datetime_from_str("2021-09-29 15:18:30", KRAKEN_DATE_FORMAT)
                    .unwrap(),
                record_type: "buy".to_string(),
                subtype: None,
                a_class: "currency".to_string(),
                asset: "BTC".to_string(),
                amount: Decimal::new(51002, 4),
                fee: Decimal::zero(),
                balance: Some(Decimal::new(5, 0)),
            };

            assert!(
                record.is_input_transaction(),
                "Input transaction was not seen as an input transaction"
            );
        }

        #[test]
        fn find_negative_amount_as_not_input() {
            let record = KrakenLedgerRecord {
                txid: Some("L7RLII-OFGWB-JTUO7J".to_string()),
                refid: "RKB7ODD-ILZGC5-LCRRBL".to_string(),
                time: Utc
                    .datetime_from_str("2021-09-29 15:18:30", KRAKEN_DATE_FORMAT)
                    .unwrap(),
                record_type: "sell".to_string(),
                subtype: None,
                a_class: "currency".to_string(),
                asset: "BTC".to_string(),
                amount: Decimal::new(-51002, 4),
                fee: Decimal::zero(),
                balance: Some(Decimal::new(5, 0)),
            };

            assert!(
                !record.is_input_transaction(),
                "Output transaction was seen as an input transaction"
            );
        }

        #[test]
        fn find_zero_amount_as_input() {
            let record = KrakenLedgerRecord {
                txid: Some("L7RLII-OFGWB-JTUO7J".to_string()),
                refid: "RKB7ODD-ILZGC5-LCRRBL".to_string(),
                time: Utc
                    .datetime_from_str("2021-09-29 15:18:30", KRAKEN_DATE_FORMAT)
                    .unwrap(),
                record_type: "sell".to_string(),
                subtype: None,
                a_class: "currency".to_string(),
                asset: "BTC".to_string(),
                amount: Decimal::new(0, 0),
                fee: Decimal::zero(),
                balance: Some(Decimal::new(5, 0)),
            };

            assert!(
                record.is_input_transaction(),
                "Zero amount transaction was not seen as an input transaction"
            );
        }
    }
}
