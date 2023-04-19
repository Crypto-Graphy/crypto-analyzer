use std::collections::HashMap;

pub use models::{
    kraken::{KrakenLedgerRecord, CSV_HEADERS, DATE_FORMAT},
    StakingRewards,
};
use models::{ActiveAssetValues, InputTransactions, RecordsByAsset};
pub use rust_decimal::Decimal;

pub struct KrakenParser<T> {
    data: Vec<T>,
}

impl<T> KrakenParser<T> {
    pub fn new(data: Vec<T>) -> Self {
        Self { data }
    }
}

impl StakingRewards for KrakenParser<KrakenLedgerRecord> {
    fn staking_rewards(&self) -> HashMap<String, Decimal> {
        self.data
            .iter()
            .filter(|record| record.record_type.eq("staking"))
            .fold(HashMap::new(), |mut reward_map, record| {
                if let Some(value) = reward_map.get(&record.asset) {
                    reward_map.insert(record.asset.to_string(), value + record.amount);
                } else {
                    reward_map.insert(record.asset.to_string(), record.amount);
                }

                reward_map
            })
    }
}

impl ActiveAssetValues for KrakenParser<KrakenLedgerRecord> {
    fn active_assets(&self) -> HashMap<String, Decimal> {
        self.data
            .iter()
            .filter(|record| record.txid.is_some())
            .fold(HashMap::new(), |mut map, record| {
                if let Some(value) = map.get(&record.asset) {
                    map.insert(record.asset.to_string(), value + record.amount);
                } else {
                    map.insert(record.asset.to_string(), record.amount);
                }

                map
            })
    }
}

impl RecordsByAsset<KrakenLedgerRecord> for KrakenParser<KrakenLedgerRecord> {
    fn by_asset(&self) -> HashMap<String, Vec<&KrakenLedgerRecord>> {
        self.data
            .iter()
            .fold(HashMap::new(), |mut currency_map, record| {
                let mut vector = currency_map.remove(&record.asset).unwrap_or_default();

                vector.push(record);
                currency_map.insert(record.asset.to_string(), vector);

                currency_map
            })
    }
}

#[cfg(test)]
mod staking_rewards_for {
    #[cfg(test)]
    mod kraken_ledger_record {
        use chrono::{TimeZone, Utc};
        use models::{
            kraken::{KrakenLedgerRecord, DATE_FORMAT as KRAKEN_DATE_FORMAT},
            StakingRewards,
        };
        use rust_decimal::prelude::{Decimal, Zero};

        use crate::KrakenParser;

        #[test]
        fn should_get_staking_rewards_for_multiple() {
            let sample_ledger_1 = KrakenLedgerRecord {
                txid: Some("L7RLII-OFGWB-JTUO7J".to_string()),
                refid: "RKB7ODD-ILZGC5-LCRRBL".to_string(),
                time: Utc
                    .datetime_from_str("2021-09-29 15:18:30", KRAKEN_DATE_FORMAT)
                    .unwrap(),
                record_type: "staking".to_string(),
                subtype: None,
                a_class: "currency".to_string(),
                asset: "DOT".to_string(),
                amount: Decimal::new(51002, 4),
                fee: Decimal::zero(),
                balance: Some(Decimal::new(5, 0)),
            };

            let sample_ledger_2 = KrakenLedgerRecord {
                txid: Some("899OJA-OFGWB-JTUO7J".to_string()),
                refid: "RKB7ODD-ILZGC5-LCRRBL".to_string(),
                time: Utc
                    .datetime_from_str("2021-09-29 15:18:30", KRAKEN_DATE_FORMAT)
                    .unwrap(),
                record_type: "staking".to_string(),
                subtype: None,
                a_class: "currency".to_string(),
                asset: "ADA".to_string(),
                amount: Decimal::new(5, 0),
                fee: Decimal::zero(),
                balance: Some(Decimal::new(5, 0)),
            };

            let sample_vec = vec![sample_ledger_1, sample_ledger_2];

            let kraken_parser = KrakenParser::new(sample_vec);
            let reward_map = kraken_parser.staking_rewards();
            assert_eq!(reward_map.len(), 2);

            // Keys
            let mut keys: Vec<String> = reward_map.keys().cloned().collect();
            keys.sort();
            assert_eq!(keys[0], "ADA");
            assert_eq!(keys[1], "DOT");

            // Values
            let mut values = reward_map.values().cloned().collect::<Vec<Decimal>>();
            values.sort_by(|a, b| a.partial_cmp(b).unwrap());

            assert_eq!(values[0], Decimal::new(5, 0));
            assert_eq!(values[1], Decimal::new(51002, 4));
        }

        #[test]
        fn should_sum_rewards_of_the_same_currency() {
            let sample_ledger_1 = KrakenLedgerRecord {
                txid: Some("L7RLII-OFGWB-JTUO7J".to_string()),
                refid: "RKB7ODD-ILZGC5-LCRRBL".to_string(),
                time: Utc
                    .datetime_from_str("2021-09-29 15:18:30", KRAKEN_DATE_FORMAT)
                    .unwrap(),
                record_type: "staking".to_string(),
                subtype: None,
                a_class: "currency".to_string(),
                asset: "DOT".to_string(),
                amount: Decimal::new(510020000, 8),
                fee: Decimal::zero(),
                balance: Some(Decimal::new(5, 0)),
            };

            let sample_ledger_2 = KrakenLedgerRecord {
                txid: Some("L7RLII-OFGWB-JTUO7J".to_string()),
                refid: "RKB7ODD-ILZGC5-LCRRBL".to_string(),
                time: Utc
                    .datetime_from_str("2021-09-29 15:18:30", KRAKEN_DATE_FORMAT)
                    .unwrap(),
                record_type: "staking".to_string(),
                subtype: None,
                a_class: "currency".to_string(),
                asset: "DOT".to_string(),
                amount: Decimal::new(5, 0),
                fee: Decimal::zero(),
                balance: Some(Decimal::new(5, 0)),
            };

            let expected = Decimal::new(51002, 4) + Decimal::new(5, 0);

            let kraken_parser = KrakenParser::new(vec![sample_ledger_1, sample_ledger_2]);
            let reward_map = kraken_parser.staking_rewards();

            assert_eq!(*reward_map.get("DOT").unwrap(), expected);
        }

        #[test]
        fn get_total_staking_rewards_when_given_empty_vec() {
            let sample_vec: Vec<KrakenLedgerRecord> = Vec::new();
            let kraken_parser = KrakenParser::new(sample_vec);
            let reward_map = kraken_parser.staking_rewards();

            assert!(reward_map.is_empty(), "staking rewards is not empty");
        }
    }
}

#[cfg(test)]
mod active_asset_values_for {
    #[cfg(test)]
    mod kraken_ledger_record {
        use chrono::{TimeZone, Utc};
        use models::{
            kraken::{KrakenLedgerRecord, DATE_FORMAT as KRAKEN_DATE_FORMAT},
            ActiveAssetValues,
        };
        use rust_decimal::prelude::{Decimal, Zero};

        use crate::KrakenParser;

        #[test]
        fn should_sum_multiple_book() {
            let sample_ledger_1 = KrakenLedgerRecord {
                txid: Some("L7RLII-OFGWB-JTUO7J".to_string()),
                refid: "RKB7ODD-ILZGC5-LCRRBL".to_string(),
                time: Utc
                    .datetime_from_str("2021-09-29 15:18:30", KRAKEN_DATE_FORMAT)
                    .unwrap(),
                record_type: "staking".to_string(),
                subtype: None,
                a_class: "currency".to_string(),
                asset: "DOT".to_string(),
                amount: Decimal::new(51002, 4),
                fee: Decimal::zero(),
                balance: Some(Decimal::new(5, 0)),
            };

            let sample_ledger_2 = KrakenLedgerRecord {
                txid: Some("L7RLII-OFGWB-JTUO7J".to_string()),
                refid: "RKB7ODD-ILZGC5-LCRRBL".to_string(),
                time: Utc
                    .datetime_from_str("2021-09-29 15:18:30", KRAKEN_DATE_FORMAT)
                    .unwrap(),
                record_type: "staking".to_string(),
                subtype: None,
                a_class: "currency".to_string(),
                asset: "DOT".to_string(),
                amount: Decimal::new(5, 0),
                fee: Decimal::zero(),
                balance: Some(Decimal::new(5, 0)),
            };

            let sample_vec = vec![sample_ledger_1, sample_ledger_2];
            let expected_sum = Decimal::new(51002, 4) + Decimal::new(5, 0);

            let kraken_parser = KrakenParser::new(sample_vec);
            let active_assets = kraken_parser.active_assets();

            assert!(
                active_assets.contains_key("DOT"),
                "book did not contain DOT"
            );
            assert_eq!(*active_assets.get("DOT").unwrap(), expected_sum);
        }

        #[test]
        fn should_subtract_negative_values_book() {
            let sample_ledger_1 = KrakenLedgerRecord {
                txid: Some("L7RLII-OFGWB-JTUO7J".to_string()),
                refid: "RKB7ODD-ILZGC5-LCRRBL".to_string(),
                time: Utc
                    .datetime_from_str("2021-09-29 15:18:30", KRAKEN_DATE_FORMAT)
                    .unwrap(),
                record_type: "staking".to_string(),
                subtype: None,
                a_class: "currency".to_string(),
                asset: "DOT".to_string(),
                amount: Decimal::new(51002, 4),
                fee: Decimal::zero(),
                balance: Some(Decimal::new(5, 0)),
            };

            let sample_ledger_2 = KrakenLedgerRecord {
                txid: Some("L7RLII-OFGWB-JTUO7J".to_string()),
                refid: "RKB7ODD-ILZGC5-LCRRBL".to_string(),
                time: Utc
                    .datetime_from_str("2021-09-29 15:18:30", KRAKEN_DATE_FORMAT)
                    .unwrap(),
                record_type: "staking".to_string(),
                subtype: None,
                a_class: "currency".to_string(),
                asset: "DOT".to_string(),
                amount: Decimal::new(-51002, 4),
                fee: Decimal::zero(),
                balance: Some(Decimal::new(5, 0)),
            };

            let sample_vec = vec![sample_ledger_1, sample_ledger_2];
            let expected_sum = Decimal::zero();

            let kraken_parser = KrakenParser::new(sample_vec);
            let active_assets = kraken_parser.active_assets();

            assert!(
                active_assets.contains_key("DOT"),
                "book did not contain DOT"
            );
            assert_eq!(*active_assets.get("DOT").unwrap(), expected_sum);
        }

        #[test]
        fn returns_empty_map_when_empty_iter() {
            let sample_vec = Vec::new();

            let kraken_parser = KrakenParser::new(sample_vec);
            let active_assets = kraken_parser.active_assets();

            assert!(active_assets.is_empty(), "book was not empty");
        }
    }
}
