use std::{collections::HashMap, slice::Iter, str::FromStr};

use models::InputTransaction;
use models_db::CoinbaseTransaction;
use rust_decimal::Decimal;

pub use models::{
    coinbase::{
        CoinbaseTransactionRecord, CSV_HEADERS, INCLUDE_TRANSACTIONS, INPUT_TRANSACTIONS,
        OUTPUT_TRANSACTIONS,
    },
    ActiveAssetValues, InputTransactions, StakingRewards,
};

pub struct CoinbaseParser<T> {
    data: Vec<T>,
}

impl<T> CoinbaseParser<T> {
    pub fn new(data: Vec<T>) -> Self {
        Self { data }
    }

    pub fn create_iter(&self) -> Iter<T> {
        self.data.iter()
    }
}

impl StakingRewards for CoinbaseParser<CoinbaseTransactionRecord> {
    ///
    /// Generates rewards based on the vector of CoinbaseTransactionRecords contained within the struct.
    /// ```
    /// # use rust_decimal::Decimal;
    /// # use std::collections::HashMap;
    /// # use chrono::{DateTime, Utc};
    /// # use models::coinbase::CoinbaseTransactionRecord;
    /// # use coinbase_parser::{CoinbaseParser, StakingRewards};
    /// let coinbase_parser = CoinbaseParser::new(
    ///     vec![
    ///         CoinbaseTransactionRecord {
    ///             time_of_transaction: "2021-04-01T21:38:01Z".parse::<DateTime<Utc>>().unwrap(),
    ///             transaction_type: "Rewards Income".to_string(),
    ///             asset: "DOT".to_string(),
    ///             quantity_transacted: Decimal::new(22028, 6),
    ///             spot_price_currency: "USD".to_string(),
    ///             spot_price_at_transaction: Some(Decimal::new(5894398, 2)),
    ///             subtotal: Some(Decimal::new(9701, 2)),
    ///             total: Some(Decimal::new(100, 0)),
    ///             fees: None,
    ///             notes: "Bought 0.0016458 BTC for $100.00 USD".to_string(),
    ///         },
    ///     ]
    /// );
    /// let rewards = coinbase_parser.staking_rewards();
    /// let expected = Decimal::new(22028, 6);
    /// assert_eq!(rewards.get("DOT"), Some(&expected));
    /// ```
    fn staking_rewards(&self) -> HashMap<String, Decimal> {
        self.data
            .iter()
            .filter(|transaction| transaction.transaction_type.eq("Rewards Income"))
            .fold(HashMap::new(), |mut reward_map, record| {
                if let Some(value) = reward_map.get(&record.asset) {
                    reward_map.insert(record.asset.to_string(), value + record.quantity_transacted);
                } else {
                    reward_map.insert(record.asset.to_string(), record.quantity_transacted);
                }

                reward_map
            })
    }
}

impl InputTransactions<CoinbaseTransactionRecord> for CoinbaseParser<CoinbaseTransactionRecord> {
    ///
    /// Parses all transactions to match and return those that are known to be positive transactions into a wallet.
    /// ```
    /// # use rust_decimal::Decimal;
    /// # use std::collections::HashMap;
    /// # use chrono::{DateTime, Utc};
    /// # use models::coinbase::CoinbaseTransactionRecord;
    /// # use coinbase_parser::{CoinbaseParser, InputTransactions};
    /// let coinbase_parser = CoinbaseParser::new(
    ///     vec![
    ///         CoinbaseTransactionRecord {
    ///             time_of_transaction: "2022-04-01T21:38:01Z".parse::<DateTime<Utc>>().unwrap(),
    ///             transaction_type: "Buy".to_string(),
    ///             asset: "DOT".to_string(),
    ///             quantity_transacted: Decimal::new(22028, 6),
    ///             spot_price_currency: "USD".to_string(),
    ///             spot_price_at_transaction: Some(Decimal::new(800, 2)),
    ///             subtotal: Some(Decimal::new(800, 2)),
    ///             total: Some(Decimal::new(100, 0)),
    ///             fees: None,
    ///             notes: "Bought 0.022028 DOT for $100.00 USD".to_string(),
    ///         },
    ///         CoinbaseTransactionRecord {
    ///             time_of_transaction: "2022-04-01T21:38:01Z".parse::<DateTime<Utc>>().unwrap(),
    ///             transaction_type: "Sell".to_string(),
    ///             asset: "DOT".to_string(),
    ///             quantity_transacted: Decimal::new(22028, 6),
    ///             spot_price_currency: "USD".to_string(),
    ///             spot_price_at_transaction: Some(Decimal::new(800, 2)),
    ///             subtotal: Some(Decimal::new(800, 2)),
    ///             total: Some(Decimal::new(100, 0)),
    ///             fees: None,
    ///             notes: "Bought 0.022028 DOT for $100.00 USD".to_string(),
    ///         },
    ///     ]
    /// );
    ///
    /// let expected = CoinbaseTransactionRecord {
    ///     time_of_transaction: "2022-04-01T21:38:01Z".parse::<DateTime<Utc>>().unwrap(),
    ///     transaction_type: "Buy".to_string(),
    ///     asset: "DOT".to_string(),
    ///     quantity_transacted: Decimal::new(22028, 6),
    ///     spot_price_currency: "USD".to_string(),
    ///     spot_price_at_transaction: Some(Decimal::new(800, 2)),
    ///     subtotal: Some(Decimal::new(800, 2)),
    ///     total: Some(Decimal::new(100, 0)),
    ///     fees: None,
    ///     notes: "Bought 0.022028 DOT for $100.00 USD".to_string(),
    /// };

    /// let input_transactions = coinbase_parser.input_transactions();
    /// assert_eq!(input_transactions.len(), 1);
    /// assert_eq!(input_transactions.first(), Some(&&expected));
    /// ```
    fn input_transactions(&self) -> Vec<&CoinbaseTransactionRecord> {
        self.data
            .iter()
            .filter(|transaction| transaction.is_input_transaction())
            .collect()
    }
}

impl ActiveAssetValues for CoinbaseParser<CoinbaseTransactionRecord> {
    fn active_assets(&self) -> HashMap<String, Decimal> {
        self.data
            .iter()
            .filter(|transaction| {
                INCLUDE_TRANSACTIONS.iter().any(|included_transaction| {
                    included_transaction.eq(&transaction.transaction_type)
                })
            })
            .fold(HashMap::new(), |mut map, transaction| {
                if is_gain_record(transaction) {
                    process_transaction(
                        &mut map,
                        &transaction.asset,
                        &transaction.quantity_transacted,
                    );
                } else if is_loss_record(transaction) {
                    process_transaction(
                        &mut map,
                        &transaction.asset,
                        &(transaction.quantity_transacted * Decimal::new(-1, 0)),
                    );
                } else if transaction.transaction_type.eq("Convert") {
                    if let Some(value) = transaction.notes.split("to").last() {
                        let vec: Vec<&str> = value.trim().split(' ').collect();

                        let amount =
                            Decimal::from_str(&vec.first().unwrap().trim().replace(',', ""))
                                .unwrap();
                        let asset = vec.last().unwrap().to_string();

                        process_transaction(
                            &mut map,
                            &transaction.asset,
                            &(transaction.quantity_transacted * Decimal::new(-1, 0)),
                        );

                        process_transaction(&mut map, &asset, &amount);
                    }
                };

                map
            })
    }
}

impl StakingRewards for CoinbaseParser<CoinbaseTransaction> {
    ///
    /// Generates rewards based on the vector of CoinbaseTransaction contained within the struct.
    /// ```
    /// # use rust_decimal::Decimal;
    /// # use std::collections::HashMap;
    /// # use chrono::{DateTime, Utc};
    /// # use models_db::CoinbaseTransaction;
    /// # use coinbase_parser::{CoinbaseParser, StakingRewards};
    /// let coinbase_parser = CoinbaseParser::new(
    ///     vec![
    ///         CoinbaseTransaction {
    ///             id: 3,
    ///             time_of_transaction: "2021-04-01T21:38:01Z".parse::<DateTime<Utc>>().unwrap(),
    ///             transaction_type: "Rewards Income".to_string(),
    ///             asset: "DOT".to_string(),
    ///             quantity_transacted: Decimal::new(22028, 6),
    ///             spot_price_currency: "USD".to_string(),
    ///             spot_price_at_transaction: Some(Decimal::new(5894398, 2)),
    ///             subtotal: Some(Decimal::new(9701, 2)),
    ///             total: Some(Decimal::new(100, 0)),
    ///             fees: None,
    ///             notes: "Bought 0.0016458 BTC for $100.00 USD".to_string(),
    ///         },
    ///     ]
    /// );
    /// let rewards = coinbase_parser.staking_rewards();
    /// let expected = Decimal::new(22028, 6);
    /// assert_eq!(rewards.get("DOT"), Some(&expected));
    /// ```
    fn staking_rewards(&self) -> HashMap<String, Decimal> {
        self.data
            .iter()
            .filter(|transaction| transaction.transaction_type.eq("Rewards Income"))
            .fold(HashMap::new(), |mut reward_map, record| {
                if let Some(value) = reward_map.get(&record.asset) {
                    reward_map.insert(record.asset.to_string(), value + record.quantity_transacted);
                } else {
                    reward_map.insert(record.asset.to_string(), record.quantity_transacted);
                }

                reward_map
            })
    }
}

impl InputTransactions<CoinbaseTransaction> for CoinbaseParser<CoinbaseTransaction> {
    ///
    /// Parses all transactions to match and return those that are known to be positive transactions into a wallet.
    /// ```
    /// # use rust_decimal::Decimal;
    /// # use std::collections::HashMap;
    /// # use chrono::{DateTime, Utc};
    /// # use models_db::CoinbaseTransaction;
    /// # use coinbase_parser::{CoinbaseParser, InputTransactions};
    /// let coinbase_parser = CoinbaseParser::new(
    ///     vec![
    ///         CoinbaseTransaction {
    ///             id: 1022734,
    ///             time_of_transaction: "2022-04-01T21:38:01Z".parse::<DateTime<Utc>>().unwrap(),
    ///             transaction_type: "Buy".to_string(),
    ///             asset: "DOT".to_string(),
    ///             quantity_transacted: Decimal::new(22028, 6),
    ///             spot_price_currency: "USD".to_string(),
    ///             spot_price_at_transaction: Some(Decimal::new(800, 2)),
    ///             subtotal: Some(Decimal::new(800, 2)),
    ///             total: Some(Decimal::new(100, 0)),
    ///             fees: None,
    ///             notes: "Bought 0.022028 DOT for $100.00 USD".to_string(),
    ///         },
    ///         CoinbaseTransaction {
    ///             id: 1022735,
    ///             time_of_transaction: "2022-04-01T21:38:01Z".parse::<DateTime<Utc>>().unwrap(),
    ///             transaction_type: "Sell".to_string(),
    ///             asset: "DOT".to_string(),
    ///             quantity_transacted: Decimal::new(22028, 6),
    ///             spot_price_currency: "USD".to_string(),
    ///             spot_price_at_transaction: Some(Decimal::new(800, 2)),
    ///             subtotal: Some(Decimal::new(800, 2)),
    ///             total: Some(Decimal::new(100, 0)),
    ///             fees: None,
    ///             notes: "Bought 0.022028 DOT for $100.00 USD".to_string(),
    ///         },
    ///     ]
    /// );
    ///
    /// let expected = CoinbaseTransaction {
    ///     id: 1022734,
    ///     time_of_transaction: "2022-04-01T21:38:01Z".parse::<DateTime<Utc>>().unwrap(),
    ///     transaction_type: "Buy".to_string(),
    ///     asset: "DOT".to_string(),
    ///     quantity_transacted: Decimal::new(22028, 6),
    ///     spot_price_currency: "USD".to_string(),
    ///     spot_price_at_transaction: Some(Decimal::new(800, 2)),
    ///     subtotal: Some(Decimal::new(800, 2)),
    ///     total: Some(Decimal::new(100, 0)),
    ///     fees: None,
    ///     notes: "Bought 0.022028 DOT for $100.00 USD".to_string(),
    /// };

    /// let input_transactions = coinbase_parser.input_transactions();
    /// assert_eq!(input_transactions.len(), 1);
    /// assert_eq!(input_transactions.first(), Some(&&expected));
    /// ```
    fn input_transactions(&self) -> Vec<&CoinbaseTransaction> {
        self.data
            .iter()
            .filter(|transaction| {
                INPUT_TRANSACTIONS.iter().any(|received_transaction_type| {
                    received_transaction_type.eq(&transaction.transaction_type)
                })
            })
            .collect()
    }
}

impl ActiveAssetValues for CoinbaseParser<CoinbaseTransaction> {
    fn active_assets(&self) -> HashMap<String, Decimal> {
        self.data
            .iter()
            .filter(|transaction| {
                INCLUDE_TRANSACTIONS.iter().any(|included_transaction| {
                    included_transaction.eq(&transaction.transaction_type)
                })
            })
            .fold(HashMap::new(), |mut map, transaction| {
                if is_gain(transaction) {
                    process_transaction(
                        &mut map,
                        &transaction.asset,
                        &transaction.quantity_transacted,
                    );
                } else if is_loss(transaction) {
                    process_transaction(
                        &mut map,
                        &transaction.asset,
                        &(transaction.quantity_transacted * Decimal::new(-1, 0)),
                    );
                } else if transaction.transaction_type.eq("Convert") {
                    if let Some(value) = transaction.notes.split("to").last() {
                        let vec: Vec<&str> = value.trim().split(' ').collect();

                        let amount =
                            Decimal::from_str(&vec.first().unwrap().trim().replace(',', ""))
                                .unwrap();
                        let asset = vec.last().unwrap().to_string();

                        process_transaction(
                            &mut map,
                            &transaction.asset,
                            &(transaction.quantity_transacted * Decimal::new(-1, 0)),
                        );

                        process_transaction(&mut map, &asset, &amount);
                    }
                };

                map
            })
    }
}

fn process_transaction(map: &mut HashMap<String, Decimal>, asset: &String, amount: &Decimal) {
    if let Some(value) = map.get(asset) {
        Decimal::from_str(&value.to_string()).unwrap();
        map.insert(asset.to_string(), value + *amount);
    } else {
        map.insert(asset.to_string(), *amount);
    }
}

fn is_gain_record(transaction: &CoinbaseTransactionRecord) -> bool {
    INPUT_TRANSACTIONS
        .iter()
        .any(|receive_type| receive_type.eq(&transaction.transaction_type))
}

fn is_loss_record(transaction: &CoinbaseTransactionRecord) -> bool {
    OUTPUT_TRANSACTIONS
        .iter()
        .any(|send_transaction| send_transaction.eq(&transaction.transaction_type))
}

// TODO: Combined these together?
fn is_gain(transaction: &CoinbaseTransaction) -> bool {
    INPUT_TRANSACTIONS
        .iter()
        .any(|receive_type| receive_type.eq(&transaction.transaction_type))
}

fn is_loss(transaction: &CoinbaseTransaction) -> bool {
    OUTPUT_TRANSACTIONS
        .iter()
        .any(|send_transaction| send_transaction.eq(&transaction.transaction_type))
}

#[cfg(test)]
mod staking_reward_for {
    mod coinbase_transaction_record {
        use crate::{CoinbaseParser, StakingRewards};

        use crate::CoinbaseTransactionRecord;
        use chrono::{DateTime, Utc};
        use rand::Rng;
        use rust_decimal::Decimal;

        #[test]
        fn total_staking_rewards_returns_with_multiple_asset_types() {
            let sample_vec = vec![
                CoinbaseTransactionRecord {
                    time_of_transaction: "2021-04-01T21:38:01Z".parse::<DateTime<Utc>>().unwrap(),
                    transaction_type: "Rewards Income".to_string(),
                    asset: "DOT".to_string(),
                    quantity_transacted: Decimal::new(22028, 6),
                    spot_price_currency: "USD".to_string(),
                    spot_price_at_transaction: Some(Decimal::new(5894398, 2)),
                    subtotal: Some(Decimal::new(9701, 2)),
                    total: Some(Decimal::new(100, 0)),
                    fees: Some(Decimal::new(299, 2)),
                    notes: "Bought 0.0016458 BTC for $100.00 USD".to_string(),
                },
                CoinbaseTransactionRecord {
                    time_of_transaction: "2021-04-01T21:38:02Z".parse::<DateTime<Utc>>().unwrap(),
                    transaction_type: "Rewards Income".to_string(),
                    asset: "ALGO".to_string(),
                    quantity_transacted: Decimal::new(16458, 7),
                    spot_price_currency: "USD".to_string(),
                    spot_price_at_transaction: Some(Decimal::new(5894398, 2)),
                    subtotal: Some(Decimal::new(9701, 2)),
                    total: Some(Decimal::new(100, 0)),
                    fees: Some(Decimal::new(299, 2)),
                    notes: "Bought 0.0016458 BTC for $100.00 USD".to_string(),
                },
            ];

            let coinbase_parser = CoinbaseParser::new(sample_vec);
            let actual = coinbase_parser.staking_rewards();

            // Keys
            let expected_keys = ["DOT", "ALGO"];
            assert_eq!(actual.len(), 2);
            expected_keys
                .iter()
                .for_each(|key| assert!(actual.contains_key(&key.to_string())));

            // Values
            let mut values = actual.values().cloned().collect::<Vec<Decimal>>();
            values.sort_by(|a, b| a.partial_cmp(b).unwrap());
            assert_eq!(values[0], Decimal::new(16458, 7));
            assert_eq!(values[1], Decimal::new(22028, 6));
        }

        #[test]
        fn total_staking_rewards_sums_transacted() {
            let mut rng = rand::thread_rng();

            let range = 0.010448745..2000.00022123;
            let given_transaction_1 =
                Decimal::from_str_radix(&rng.gen_range(range.clone()).to_string(), 10).unwrap();
            let given_transaction_2 =
                Decimal::from_str_radix(&rng.gen_range(range).to_string(), 10).unwrap();

            let sample_vec = vec![
                CoinbaseTransactionRecord {
                    time_of_transaction: "2021-04-01T21:38:01Z".parse::<DateTime<Utc>>().unwrap(),
                    transaction_type: "Rewards Income".to_string(),
                    asset: "DOT".to_string(),
                    quantity_transacted: given_transaction_1,
                    spot_price_currency: "USD".to_string(),
                    spot_price_at_transaction: Some(Decimal::new(5894398, 2)),
                    subtotal: Some(Decimal::new(9701, 2)),
                    total: Some(Decimal::new(100, 0)),
                    fees: Some(Decimal::new(299, 2)),
                    notes: "Bought 0.0016458 BTC for $100.00 USD".to_string(),
                },
                CoinbaseTransactionRecord {
                    time_of_transaction: "2021-04-01T21:38:02Z".parse::<DateTime<Utc>>().unwrap(),
                    transaction_type: "Rewards Income".to_string(),
                    asset: "DOT".to_string(),
                    quantity_transacted: given_transaction_2,
                    spot_price_currency: "USD".to_string(),
                    spot_price_at_transaction: Some(Decimal::new(5894398, 2)),
                    subtotal: Some(Decimal::new(9701, 2)),
                    total: Some(Decimal::new(100, 0)),
                    fees: Some(Decimal::new(299, 2)),
                    notes: "Bought 0.0016458 BTC for $100.00 USD".to_string(),
                },
            ];
            let expected_transacted = given_transaction_1 + given_transaction_2;

            let coinbase_parser = CoinbaseParser::new(sample_vec);
            let actual = coinbase_parser.staking_rewards();

            assert_eq!(*actual.get("DOT").unwrap(), expected_transacted);
        }

        #[test]
        fn total_staking_rewards_when_given_empty_vec() {
            let sample_vec: Vec<CoinbaseTransactionRecord> = Vec::new();

            let coinbase_parser = CoinbaseParser::new(sample_vec);
            let actual = coinbase_parser.staking_rewards();

            assert!(actual.is_empty());
        }
    }

    #[cfg(test)]
    mod coinbase_transaction {
        use chrono::{DateTime, Utc};
        use models_db::CoinbaseTransaction;
        use rand::Rng;
        use rust_decimal::Decimal;

        use crate::{CoinbaseParser, StakingRewards};

        #[test]
        fn total_staking_rewards_returns_with_multiple_asset_types() {
            let sample_vec = vec![
                CoinbaseTransaction {
                    id: 202227,
                    time_of_transaction: "2021-04-01T21:38:01Z".parse::<DateTime<Utc>>().unwrap(),
                    transaction_type: "Rewards Income".to_string(),
                    asset: "DOT".to_string(),
                    quantity_transacted: Decimal::new(22028, 6),
                    spot_price_currency: "USD".to_string(),
                    spot_price_at_transaction: Some(Decimal::new(5894398, 2)),
                    subtotal: Some(Decimal::new(9701, 2)),
                    total: Some(Decimal::new(100, 0)),
                    fees: Some(Decimal::new(299, 2)),
                    notes: "Bought 0.0016458 BTC for $100.00 USD".to_string(),
                },
                CoinbaseTransaction {
                    id: 37222,
                    time_of_transaction: "2021-04-01T21:38:02Z".parse::<DateTime<Utc>>().unwrap(),
                    transaction_type: "Rewards Income".to_string(),
                    asset: "ALGO".to_string(),
                    quantity_transacted: Decimal::new(16458, 7),
                    spot_price_currency: "USD".to_string(),
                    spot_price_at_transaction: Some(Decimal::new(5894398, 2)),
                    subtotal: Some(Decimal::new(9701, 2)),
                    total: Some(Decimal::new(100, 0)),
                    fees: Some(Decimal::new(299, 2)),
                    notes: "Bought 0.0016458 BTC for $100.00 USD".to_string(),
                },
            ];

            let coinbase_parser = CoinbaseParser::new(sample_vec);
            let actual = coinbase_parser.staking_rewards();

            // Keys
            let expected_keys = ["DOT", "ALGO"];
            assert_eq!(actual.len(), 2);
            expected_keys
                .iter()
                .for_each(|key| assert!(actual.contains_key(&key.to_string())));

            // Values
            let mut values = actual.values().cloned().collect::<Vec<Decimal>>();
            values.sort_by(|a, b| a.partial_cmp(b).unwrap());
            assert_eq!(values[0], Decimal::new(16458, 7));
            assert_eq!(values[1], Decimal::new(22028, 6));
        }

        #[test]
        fn total_staking_rewards_sums_transacted() {
            let mut rng = rand::thread_rng();

            let range = 0.010448745..2000.00022123;
            let given_transaction_1 =
                Decimal::from_str_radix(&rng.gen_range(range.clone()).to_string(), 10).unwrap();
            let given_transaction_2 =
                Decimal::from_str_radix(&rng.gen_range(range).to_string(), 10).unwrap();

            let sample_vec = vec![
                CoinbaseTransaction {
                    id: 20684,
                    time_of_transaction: "2021-04-01T21:38:01Z".parse::<DateTime<Utc>>().unwrap(),
                    transaction_type: "Rewards Income".to_string(),
                    asset: "DOT".to_string(),
                    quantity_transacted: given_transaction_1,
                    spot_price_currency: "USD".to_string(),
                    spot_price_at_transaction: Some(Decimal::new(5894398, 2)),
                    subtotal: Some(Decimal::new(9701, 2)),
                    total: Some(Decimal::new(100, 0)),
                    fees: Some(Decimal::new(299, 2)),
                    notes: "Bought 0.0016458 BTC for $100.00 USD".to_string(),
                },
                CoinbaseTransaction {
                    id: 101,
                    time_of_transaction: "2021-04-01T21:38:02Z".parse::<DateTime<Utc>>().unwrap(),
                    transaction_type: "Rewards Income".to_string(),
                    asset: "DOT".to_string(),
                    quantity_transacted: given_transaction_2,
                    spot_price_currency: "USD".to_string(),
                    spot_price_at_transaction: Some(Decimal::new(5894398, 2)),
                    subtotal: Some(Decimal::new(9701, 2)),
                    total: Some(Decimal::new(100, 0)),
                    fees: Some(Decimal::new(299, 2)),
                    notes: "Bought 0.0016458 BTC for $100.00 USD".to_string(),
                },
            ];
            let expected_transacted = given_transaction_1 + given_transaction_2;

            let coinbase_parser = CoinbaseParser::new(sample_vec);
            let actual = coinbase_parser.staking_rewards();

            assert_eq!(*actual.get("DOT").unwrap(), expected_transacted);
        }

        #[test]
        fn total_staking_rewards_when_given_empty_vec() {
            let sample_vec: Vec<CoinbaseTransaction> = Vec::new();

            let coinbase_parser = CoinbaseParser::new(sample_vec);
            let actual = coinbase_parser.staking_rewards();

            assert!(actual.is_empty());
        }
    }
}

#[cfg(test)]
mod input_transactions_for {
    #[cfg(test)]
    mod coinbase_transaction_record {
        use chrono::{DateTime, Utc};
        use models::{coinbase::CoinbaseTransactionRecord, InputTransactions};
        use rust_decimal::Decimal;

        use crate::CoinbaseParser;

        #[test]
        fn should_return_input_transactions_with_expected_content() {
            let sample_vec = vec![
                CoinbaseTransactionRecord {
                    time_of_transaction: "2021-04-01T21:38:02Z".parse::<DateTime<Utc>>().unwrap(),
                    transaction_type: "Buy".to_string(),
                    asset: "DOT".to_string(),
                    quantity_transacted: Decimal::new(2200024, 5),
                    spot_price_currency: "USD".to_string(),
                    spot_price_at_transaction: Some(Decimal::new(5894398, 2)),
                    subtotal: Some(Decimal::new(9701, 2)),
                    total: Some(Decimal::new(100, 0)),
                    fees: Some(Decimal::new(299, 2)),
                    notes: "Bought 0.0016458 BTC for $100.00 USD".to_string(),
                },
                CoinbaseTransactionRecord {
                    time_of_transaction: "2021-04-01T21:38:02Z".parse::<DateTime<Utc>>().unwrap(),
                    transaction_type: "Receive".to_string(),
                    asset: "DOT".to_string(),
                    quantity_transacted: Decimal::new(2200024, 5),
                    spot_price_currency: "USD".to_string(),
                    spot_price_at_transaction: Some(Decimal::new(5894398, 2)),
                    subtotal: Some(Decimal::new(9701, 2)),
                    total: Some(Decimal::new(100, 0)),
                    fees: Some(Decimal::new(299, 2)),
                    notes: "Bought 0.0016458 BTC for $100.00 USD".to_string(),
                },
            ];

            let coinbase_parser = CoinbaseParser::new(sample_vec.clone());
            let actual = coinbase_parser.input_transactions();

            assert_eq!(actual.len(), 2);
            assert_eq!(**actual.get(0).unwrap(), *sample_vec.get(0).unwrap());
            assert_eq!(**actual.get(1).unwrap(), *sample_vec.get(1).unwrap());
        }

        #[test]
        fn should_filter_out_non_input_transactions() {
            let sample_vec = vec![
                CoinbaseTransactionRecord {
                    time_of_transaction: "2021-04-01T21:38:02Z".parse::<DateTime<Utc>>().unwrap(),
                    transaction_type: "Buy".to_string(),
                    asset: "DOT".to_string(),
                    quantity_transacted: Decimal::new(2200024, 5),
                    spot_price_currency: "USD".to_string(),
                    spot_price_at_transaction: Some(Decimal::new(5894398, 2)),
                    subtotal: Some(Decimal::new(9701, 2)),
                    total: Some(Decimal::new(100, 0)),
                    fees: Some(Decimal::new(299, 2)),
                    notes: "Bought 0.0016458 BTC for $100.00 USD".to_string(),
                },
                CoinbaseTransactionRecord {
                    time_of_transaction: "2021-04-01T21:38:02Z".parse::<DateTime<Utc>>().unwrap(),
                    transaction_type: "Send".to_string(),
                    asset: "DOT".to_string(),
                    quantity_transacted: Decimal::new(2200024, 5),
                    spot_price_currency: "USD".to_string(),
                    spot_price_at_transaction: Some(Decimal::new(5894398, 2)),
                    subtotal: Some(Decimal::new(9701, 2)),
                    total: Some(Decimal::new(100, 0)),
                    fees: Some(Decimal::new(299, 2)),
                    notes: "Bought 0.0016458 BTC for $100.00 USD".to_string(),
                },
            ];

            let coinbase_parser = CoinbaseParser::new(sample_vec.clone());
            let actual = coinbase_parser.input_transactions();

            assert_eq!(actual.len(), 1);
            assert_eq!(**actual.get(0).unwrap(), *sample_vec.get(0).unwrap());
        }
    }

    #[cfg(test)]
    mod coinbase_transaction {
        use chrono::{DateTime, Utc};
        use models::InputTransactions;
        use models_db::CoinbaseTransaction;
        use rust_decimal::Decimal;

        use crate::CoinbaseParser;

        #[test]
        fn should_return_input_transactions_with_expected_content() {
            let sample_vec = vec![
                CoinbaseTransaction {
                    id: 32313,
                    time_of_transaction: "2021-04-01T21:38:02Z".parse::<DateTime<Utc>>().unwrap(),
                    transaction_type: "Buy".to_string(),
                    asset: "DOT".to_string(),
                    quantity_transacted: Decimal::new(2200024, 5),
                    spot_price_currency: "USD".to_string(),
                    spot_price_at_transaction: Some(Decimal::new(5894398, 2)),
                    subtotal: Some(Decimal::new(9701, 2)),
                    total: Some(Decimal::new(100, 0)),
                    fees: Some(Decimal::new(299, 2)),
                    notes: "Bought 0.0016458 BTC for $100.00 USD".to_string(),
                },
                CoinbaseTransaction {
                    id: 32313,
                    time_of_transaction: "2021-04-01T21:38:02Z".parse::<DateTime<Utc>>().unwrap(),
                    transaction_type: "Receive".to_string(),
                    asset: "DOT".to_string(),
                    quantity_transacted: Decimal::new(2200024, 5),
                    spot_price_currency: "USD".to_string(),
                    spot_price_at_transaction: Some(Decimal::new(5894398, 2)),
                    subtotal: Some(Decimal::new(9701, 2)),
                    total: Some(Decimal::new(100, 0)),
                    fees: Some(Decimal::new(299, 2)),
                    notes: "Bought 0.0016458 BTC for $100.00 USD".to_string(),
                },
            ];

            let coinbase_parser = CoinbaseParser::new(sample_vec.clone());
            let actual = coinbase_parser.input_transactions();

            assert_eq!(actual.len(), 2);
            assert_eq!(**actual.get(0).unwrap(), *sample_vec.get(0).unwrap());
            assert_eq!(**actual.get(1).unwrap(), *sample_vec.get(1).unwrap());
        }

        #[test]
        fn should_filter_out_non_input_transactions() {
            let sample_vec = vec![
                CoinbaseTransaction {
                    id: 102224,
                    time_of_transaction: "2021-04-01T21:38:02Z".parse::<DateTime<Utc>>().unwrap(),
                    transaction_type: "Buy".to_string(),
                    asset: "DOT".to_string(),
                    quantity_transacted: Decimal::new(2200024, 5),
                    spot_price_currency: "USD".to_string(),
                    spot_price_at_transaction: Some(Decimal::new(5894398, 2)),
                    subtotal: Some(Decimal::new(9701, 2)),
                    total: Some(Decimal::new(100, 0)),
                    fees: Some(Decimal::new(299, 2)),
                    notes: "Bought 0.0016458 BTC for $100.00 USD".to_string(),
                },
                CoinbaseTransaction {
                    id: 3773,
                    time_of_transaction: "2021-04-01T21:38:02Z".parse::<DateTime<Utc>>().unwrap(),
                    transaction_type: "Send".to_string(),
                    asset: "DOT".to_string(),
                    quantity_transacted: Decimal::new(2200024, 5),
                    spot_price_currency: "USD".to_string(),
                    spot_price_at_transaction: Some(Decimal::new(5894398, 2)),
                    subtotal: Some(Decimal::new(9701, 2)),
                    total: Some(Decimal::new(100, 0)),
                    fees: Some(Decimal::new(299, 2)),
                    notes: "Bought 0.0016458 BTC for $100.00 USD".to_string(),
                },
            ];

            let coinbase_parser = CoinbaseParser::new(sample_vec.clone());
            let actual = coinbase_parser.input_transactions();

            assert_eq!(actual.len(), 1);
            assert_eq!(**actual.get(0).unwrap(), *sample_vec.get(0).unwrap());
        }
    }
}

#[cfg(test)]
mod active_assets_for {
    #[cfg(test)]
    mod coinbase_transaction_record {
        use chrono::{DateTime, Utc};
        use models::{coinbase::CoinbaseTransactionRecord, ActiveAssetValues};
        use rust_decimal::{prelude::Zero, Decimal};

        use crate::CoinbaseParser;

        #[test]
        fn performs_addition_and_subtraction() {
            let sample_vec = vec![
                CoinbaseTransactionRecord {
                    time_of_transaction: "2021-04-01T21:38:02Z".parse::<DateTime<Utc>>().unwrap(),
                    transaction_type: "Buy".to_string(),
                    asset: "DOT".to_string(),
                    quantity_transacted: Decimal::new(2200024, 5),
                    spot_price_currency: "USD".to_string(),
                    spot_price_at_transaction: Some(Decimal::new(605, 2)),
                    subtotal: Some(Decimal::new(2200024, 5) * Decimal::new(605, 2)),
                    total: Some(Decimal::new(2200024, 5) * Decimal::new(605, 2)),
                    fees: Some(Decimal::zero()),
                    notes: "Bought 22.00024 DOT for $122.00 USD".to_string(),
                },
                CoinbaseTransactionRecord {
                    time_of_transaction: "2021-04-04T21:38:02Z".parse::<DateTime<Utc>>().unwrap(),
                    transaction_type: "Buy".to_string(),
                    asset: "DOT".to_string(),
                    quantity_transacted: Decimal::new(602, 2),
                    spot_price_currency: "USD".to_string(),
                    spot_price_at_transaction: Some(Decimal::new(605, 2)),
                    subtotal: Some(Decimal::new(602, 2) * Decimal::new(605, 2)),
                    total: Some(Decimal::new(602, 2) * Decimal::new(605, 2)),
                    fees: Some(Decimal::zero()),
                    notes: "Bought 22.00024 DOT for $122.00 USD".to_string(),
                },
                CoinbaseTransactionRecord {
                    time_of_transaction: "2021-04-05T21:38:02Z".parse::<DateTime<Utc>>().unwrap(),
                    transaction_type: "Sell".to_string(),
                    asset: "DOT".to_string(),
                    quantity_transacted: Decimal::new(3027, 3),
                    spot_price_currency: "USD".to_string(),
                    spot_price_at_transaction: Some(Decimal::new(3027, 3)),
                    subtotal: Some(Decimal::new(3027, 3)),
                    total: Some(Decimal::new(3027, 3)),
                    fees: Some(Decimal::zero()),
                    notes: "Bought 22.00024 DOT for $122.00 USD".to_string(),
                },
            ];

            let coinbase_parser = CoinbaseParser::new(sample_vec);
            let active_assets = coinbase_parser.active_assets();

            let expected_sum = Decimal::new(2499324, 5);
            assert_eq!(active_assets.len(), 1);
            assert!(active_assets.contains_key("DOT"));
            assert_eq!(*active_assets.get("DOT").unwrap(), expected_sum);
        }

        #[test]
        fn stores_multiple_assets() {
            let sample_vec = vec![
                CoinbaseTransactionRecord {
                    time_of_transaction: "2021-04-01T21:38:02Z".parse::<DateTime<Utc>>().unwrap(),
                    transaction_type: "Buy".to_string(),
                    asset: "DOT".to_string(),
                    quantity_transacted: Decimal::new(2200024, 5),
                    spot_price_currency: "USD".to_string(),
                    spot_price_at_transaction: Some(Decimal::new(605, 2)),
                    subtotal: Some(Decimal::new(2200024, 5) * Decimal::new(605, 2)),
                    total: Some(Decimal::new(2200024, 5) * Decimal::new(605, 2)),
                    fees: Some(Decimal::zero()),
                    notes: "Bought 22.00024 DOT for $122.00 USD".to_string(),
                },
                CoinbaseTransactionRecord {
                    time_of_transaction: "2021-04-01T21:38:02Z".parse::<DateTime<Utc>>().unwrap(),
                    transaction_type: "Buy".to_string(),
                    asset: "BTC".to_string(),
                    quantity_transacted: Decimal::new(1802442, 5),
                    spot_price_currency: "USD".to_string(),
                    spot_price_at_transaction: Some(Decimal::new(48744, 0)),
                    subtotal: Some(Decimal::new(1802442, 5) * Decimal::new(48744, 0)),
                    total: Some(Decimal::new(1802442, 5) * Decimal::new(48744, 0)),
                    fees: Some(Decimal::zero()),
                    notes: "Bought 22.00024 BTC for $122.00 USD".to_string(),
                },
            ];

            let coinbase_parser = CoinbaseParser::new(sample_vec);
            let active_assets = coinbase_parser.active_assets();

            assert_eq!(active_assets.len(), 2);
            assert!(
                active_assets.contains_key("DOT"),
                "book did not contain DOT"
            );
            assert!(
                active_assets.contains_key("BTC"),
                "book did not contain BTC"
            );
            assert_eq!(*active_assets.get("DOT").unwrap(), Decimal::new(2200024, 5));
            assert_eq!(*active_assets.get("BTC").unwrap(), Decimal::new(1802442, 5));
        }

        #[test]
        fn empty_map() {
            let coinbase_parser: CoinbaseParser<CoinbaseTransactionRecord> =
                CoinbaseParser::new(Vec::new());
            let active_assets = coinbase_parser.active_assets();

            assert!(active_assets.is_empty());
        }

        #[test]
        fn convert_asset() {
            let sample_vec = vec![
                CoinbaseTransactionRecord {
                    time_of_transaction: "2021-04-01T21:38:02Z".parse::<DateTime<Utc>>().unwrap(),
                    transaction_type: "Buy".to_string(),
                    asset: "BTC".to_string(),
                    quantity_transacted: Decimal::new(1802442, 5),
                    spot_price_currency: "USD".to_string(),
                    spot_price_at_transaction: Some(Decimal::new(48744, 0)),
                    subtotal: Some(Decimal::new(1802442, 5) * Decimal::new(48744, 0)),
                    total: Some(Decimal::new(1802442, 5) * Decimal::new(48744, 0)),
                    fees: Some(Decimal::zero()),
                    notes: "Bought 22.00024 BTC for $122.00 USD".to_string(),
                },
                CoinbaseTransactionRecord {
                    time_of_transaction: "2021-04-01T21:38:02Z".parse::<DateTime<Utc>>().unwrap(),
                    transaction_type: "Convert".to_string(),
                    asset: "BTC".to_string(),
                    quantity_transacted: Decimal::new(1802442, 5),
                    spot_price_currency: "USD".to_string(),
                    spot_price_at_transaction: Some(Decimal::new(48744, 0)),
                    subtotal: Some(Decimal::new(1802442, 5) * Decimal::new(48744, 0)),
                    total: Some(Decimal::new(1802442, 5) * Decimal::new(48744, 0)),
                    fees: Some(Decimal::zero()),
                    notes: "Converted 18.02442 BTC to 337.0245 DOT".to_string(),
                },
            ];

            let coinbase_parser = CoinbaseParser::new(sample_vec);
            let active_assets = coinbase_parser.active_assets();

            assert_eq!(active_assets.len(), 2);
            assert!(
                active_assets.contains_key("BTC"),
                "book did not contain BTC"
            );
            assert!(
                active_assets.contains_key("DOT"),
                "book did not contain DOT"
            );
            assert_eq!(*active_assets.get("BTC").unwrap(), Decimal::zero());
            assert_eq!(*active_assets.get("DOT").unwrap(), Decimal::new(3370245, 4));
        }
    }

    #[cfg(test)]
    mod coinbase_transaction {
        use chrono::{DateTime, Utc};
        use models::ActiveAssetValues;
        use models_db::CoinbaseTransaction;
        use rust_decimal::{prelude::Zero, Decimal};

        use crate::CoinbaseParser;

        #[test]
        fn performs_addition_and_subtraction() {
            let sample_vec = vec![
                CoinbaseTransaction {
                    id: 2021,
                    time_of_transaction: "2021-04-01T21:38:02Z".parse::<DateTime<Utc>>().unwrap(),
                    transaction_type: "Buy".to_string(),
                    asset: "DOT".to_string(),
                    quantity_transacted: Decimal::new(2200024, 5),
                    spot_price_currency: "USD".to_string(),
                    spot_price_at_transaction: Some(Decimal::new(605, 2)),
                    subtotal: Some(Decimal::new(2200024, 5) * Decimal::new(605, 2)),
                    total: Some(Decimal::new(2200024, 5) * Decimal::new(605, 2)),
                    fees: Some(Decimal::zero()),
                    notes: "Bought 22.00024 DOT for $122.00 USD".to_string(),
                },
                CoinbaseTransaction {
                    id: 2029,
                    time_of_transaction: "2021-04-04T21:38:02Z".parse::<DateTime<Utc>>().unwrap(),
                    transaction_type: "Buy".to_string(),
                    asset: "DOT".to_string(),
                    quantity_transacted: Decimal::new(602, 2),
                    spot_price_currency: "USD".to_string(),
                    spot_price_at_transaction: Some(Decimal::new(605, 2)),
                    subtotal: Some(Decimal::new(602, 2) * Decimal::new(605, 2)),
                    total: Some(Decimal::new(602, 2) * Decimal::new(605, 2)),
                    fees: Some(Decimal::zero()),
                    notes: "Bought 22.00024 DOT for $122.00 USD".to_string(),
                },
                CoinbaseTransaction {
                    id: 222,
                    time_of_transaction: "2021-04-05T21:38:02Z".parse::<DateTime<Utc>>().unwrap(),
                    transaction_type: "Sell".to_string(),
                    asset: "DOT".to_string(),
                    quantity_transacted: Decimal::new(3027, 3),
                    spot_price_currency: "USD".to_string(),
                    spot_price_at_transaction: Some(Decimal::new(3027, 3)),
                    subtotal: Some(Decimal::new(3027, 3)),
                    total: Some(Decimal::new(3027, 3)),
                    fees: Some(Decimal::zero()),
                    notes: "Bought 22.00024 DOT for $122.00 USD".to_string(),
                },
            ];

            let coinbase_parser = CoinbaseParser::new(sample_vec);
            let active_assets = coinbase_parser.active_assets();

            let expected_sum = Decimal::new(2499324, 5);
            assert_eq!(active_assets.len(), 1);
            assert!(active_assets.contains_key("DOT"));
            assert_eq!(*active_assets.get("DOT").unwrap(), expected_sum);
        }

        #[test]
        fn stores_multiple_assets() {
            let sample_vec = vec![
                CoinbaseTransaction {
                    id: 103,
                    time_of_transaction: "2021-04-01T21:38:02Z".parse::<DateTime<Utc>>().unwrap(),
                    transaction_type: "Buy".to_string(),
                    asset: "DOT".to_string(),
                    quantity_transacted: Decimal::new(2200024, 5),
                    spot_price_currency: "USD".to_string(),
                    spot_price_at_transaction: Some(Decimal::new(605, 2)),
                    subtotal: Some(Decimal::new(2200024, 5) * Decimal::new(605, 2)),
                    total: Some(Decimal::new(2200024, 5) * Decimal::new(605, 2)),
                    fees: Some(Decimal::zero()),
                    notes: "Bought 22.00024 DOT for $122.00 USD".to_string(),
                },
                CoinbaseTransaction {
                    id: 301,
                    time_of_transaction: "2021-04-01T21:38:02Z".parse::<DateTime<Utc>>().unwrap(),
                    transaction_type: "Buy".to_string(),
                    asset: "BTC".to_string(),
                    quantity_transacted: Decimal::new(1802442, 5),
                    spot_price_currency: "USD".to_string(),
                    spot_price_at_transaction: Some(Decimal::new(48744, 0)),
                    subtotal: Some(Decimal::new(1802442, 5) * Decimal::new(48744, 0)),
                    total: Some(Decimal::new(1802442, 5) * Decimal::new(48744, 0)),
                    fees: Some(Decimal::zero()),
                    notes: "Bought 22.00024 BTC for $122.00 USD".to_string(),
                },
            ];

            let coinbase_parser = CoinbaseParser::new(sample_vec);
            let active_assets = coinbase_parser.active_assets();

            assert_eq!(active_assets.len(), 2);
            assert!(
                active_assets.contains_key("DOT"),
                "book did not contain DOT"
            );
            assert!(
                active_assets.contains_key("BTC"),
                "book did not contain BTC"
            );
            assert_eq!(*active_assets.get("DOT").unwrap(), Decimal::new(2200024, 5));
            assert_eq!(*active_assets.get("BTC").unwrap(), Decimal::new(1802442, 5));
        }

        #[test]
        fn empty_map() {
            let coinbase_parser: CoinbaseParser<CoinbaseTransaction> =
                CoinbaseParser::new(Vec::new());
            let active_assets = coinbase_parser.active_assets();

            assert!(active_assets.is_empty());
        }

        #[test]
        fn convert_asset() {
            let sample_vec = vec![
                CoinbaseTransaction {
                    id: 9999,
                    time_of_transaction: "2021-04-01T21:38:02Z".parse::<DateTime<Utc>>().unwrap(),
                    transaction_type: "Buy".to_string(),
                    asset: "BTC".to_string(),
                    quantity_transacted: Decimal::new(1802442, 5),
                    spot_price_currency: "USD".to_string(),
                    spot_price_at_transaction: Some(Decimal::new(48744, 0)),
                    subtotal: Some(Decimal::new(1802442, 5) * Decimal::new(48744, 0)),
                    total: Some(Decimal::new(1802442, 5) * Decimal::new(48744, 0)),
                    fees: Some(Decimal::zero()),
                    notes: "Bought 22.00024 BTC for $122.00 USD".to_string(),
                },
                CoinbaseTransaction {
                    id: 2912,
                    time_of_transaction: "2021-04-01T21:38:02Z".parse::<DateTime<Utc>>().unwrap(),
                    transaction_type: "Convert".to_string(),
                    asset: "BTC".to_string(),
                    quantity_transacted: Decimal::new(1802442, 5),
                    spot_price_currency: "USD".to_string(),
                    spot_price_at_transaction: Some(Decimal::new(48744, 0)),
                    subtotal: Some(Decimal::new(1802442, 5) * Decimal::new(48744, 0)),
                    total: Some(Decimal::new(1802442, 5) * Decimal::new(48744, 0)),
                    fees: Some(Decimal::zero()),
                    notes: "Converted 18.02442 BTC to 337.0245 DOT".to_string(),
                },
            ];

            let coinbase_parser = CoinbaseParser::new(sample_vec);
            let active_assets = coinbase_parser.active_assets();

            assert_eq!(active_assets.len(), 2);
            assert!(
                active_assets.contains_key("BTC"),
                "book did not contain BTC"
            );
            assert!(
                active_assets.contains_key("DOT"),
                "book did not contain DOT"
            );
            assert_eq!(*active_assets.get("BTC").unwrap(), Decimal::zero());
            assert_eq!(*active_assets.get("DOT").unwrap(), Decimal::new(3370245, 4));
        }
    }
}
