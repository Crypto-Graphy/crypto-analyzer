pub mod transaction_parser {
    extern crate models;
    extern crate rust_decimal;

    use std::{collections::HashMap, str::FromStr};

    pub use self::models::coinbase::CoinbaseTransactionRecord;
    pub use self::models::coinbase::CSV_HEADERS;
    use self::rust_decimal::Decimal;

    const INCLUDE_TRANSACTIONS: &[&str] = &[
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

    const INPUT_TRANSACTIONS: &[&str] = &[
        "Buy",
        "Receive",
        "Rewards Income",
        "CardBuyBack",
        "Learning Reward",
        "Advanced Trade Buy",
    ];
    const OUTPUT_TRANSACTIONS: &[&str] = &["Sell", "Send", "CardSpend"];

    pub fn get_total_staking_rewards_map<'a>(
        transactions: impl Iterator<Item = &'a CoinbaseTransactionRecord>,
    ) -> HashMap<String, Decimal> {
        transactions
            .filter(|transcation| transcation.transaction_type.eq("Rewards Income"))
            .fold(HashMap::new(), |mut reward_map, record| {
                if let Some(value) = reward_map.get(&record.asset) {
                    reward_map.insert(record.asset.to_string(), value + record.quantity_transacted);
                } else {
                    reward_map.insert(record.asset.to_string(), record.quantity_transacted);
                }

                reward_map
            })
    }

    pub fn get_input_transactions<'a>(
        transactions: impl Iterator<Item = &'a CoinbaseTransactionRecord>,
    ) -> impl Iterator<Item = &'a CoinbaseTransactionRecord> {
        transactions.filter(|transaction| {
            INPUT_TRANSACTIONS.iter().any(|received_transaction_type| {
                received_transaction_type.eq(&transaction.transaction_type)
            })
        })
    }

    pub fn get_book_of_record<'a>(
        transactions: impl Iterator<Item = &'a CoinbaseTransactionRecord>,
    ) -> HashMap<String, Decimal> {
        transactions
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
}

#[cfg(test)]
mod test {
    extern crate chrono;
    extern crate models;
    extern crate rand;
    extern crate rust_decimal;

    use self::rand::Rng;

    use self::models::coinbase::CoinbaseTransactionRecord;

    use super::transaction_parser::*;

    use self::rust_decimal::Decimal;

    use self::chrono::prelude::*;

    #[test]
    fn should_get_total_staking_rewards_returns_with_multiple_asset_types() {
        let sample_vec = vec![
            CoinbaseTransactionRecord {
                id: None,
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
                id: None,
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

        let actual = get_total_staking_rewards_map(sample_vec.iter());

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
    fn get_total_staking_rewards_sums_transacted() {
        let mut rng = rand::thread_rng();

        let range = 0.010448745..2000.00022123;
        let given_transaction_1 =
            Decimal::from_str_radix(&rng.gen_range(range.clone()).to_string(), 10).unwrap();
        let given_transaction_2 =
            Decimal::from_str_radix(&rng.gen_range(range).to_string(), 10).unwrap();

        let sample_vec = vec![
            CoinbaseTransactionRecord {
                id: None,
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
                id: None,
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

        let actual = get_total_staking_rewards_map(sample_vec.iter());

        assert_eq!(*actual.get("DOT").unwrap(), expected_transacted);
    }

    #[test]
    fn get_total_staking_rewards_when_given_empty_vec() {
        let sample_vec: Vec<CoinbaseTransactionRecord> = Vec::new();
        let actual = get_total_staking_rewards_map(sample_vec.iter());

        assert!(actual.is_empty());
    }

    #[test]
    fn should_return_input_transactions_with_expected_content() {
        let sample_vec = vec![
            CoinbaseTransactionRecord {
                id: None,
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
                id: None,
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

        let actual: Vec<&CoinbaseTransactionRecord> =
            get_input_transactions(sample_vec.iter()).collect();

        assert_eq!(actual.len(), 2);
        assert_eq!(**actual.get(0).unwrap(), *sample_vec.get(0).unwrap());
        assert_eq!(**actual.get(1).unwrap(), *sample_vec.get(1).unwrap());
    }

    #[test]
    fn should_filter_out_non_input_transactions() {
        let sample_vec = vec![
            CoinbaseTransactionRecord {
                id: None,
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
                id: None,
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

        let actual: Vec<&CoinbaseTransactionRecord> =
            get_input_transactions(sample_vec.iter()).collect();

        assert_eq!(actual.len(), 1);
        assert_eq!(**actual.get(0).unwrap(), *sample_vec.get(0).unwrap());
    }
}

#[cfg(test)]
mod book_of_record {
    extern crate chrono;
    extern crate models;
    extern crate rust_decimal;

    use super::transaction_parser::get_book_of_record;

    use self::models::coinbase::CoinbaseTransactionRecord;
    use self::rust_decimal::{prelude::Zero, Decimal};

    use self::chrono::prelude::*;

    #[test]
    fn performs_addition_and_subtraction() {
        let sample_vec = vec![
            CoinbaseTransactionRecord {
                id: None,
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
                id: None,
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
                id: None,
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

        let book_of_record = get_book_of_record(sample_vec.iter());

        let expected_sum = Decimal::new(2499324, 5);
        assert_eq!(book_of_record.len(), 1);
        assert!(book_of_record.contains_key("DOT"));
        assert_eq!(*book_of_record.get("DOT").unwrap(), expected_sum);
    }

    #[test]
    fn stores_multiple_assets() {
        let sample_vec = vec![
            CoinbaseTransactionRecord {
                id: None,
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
                id: None,
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

        let book_of_record = get_book_of_record(sample_vec.iter());

        assert_eq!(book_of_record.len(), 2);
        assert!(
            book_of_record.contains_key("DOT"),
            "book did not contain DOT"
        );
        assert!(
            book_of_record.contains_key("BTC"),
            "book did not contain BTC"
        );
        assert_eq!(
            *book_of_record.get("DOT").unwrap(),
            Decimal::new(2200024, 5)
        );
        assert_eq!(
            *book_of_record.get("BTC").unwrap(),
            Decimal::new(1802442, 5)
        );
    }

    #[test]
    fn empty_map() {
        let book_of_record = get_book_of_record(Vec::new().iter());

        assert!(book_of_record.is_empty());
    }

    #[test]
    fn convert_asset() {
        let sample_vec = vec![
            CoinbaseTransactionRecord {
                id: None,
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
                id: None,
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

        let book_of_record = get_book_of_record(sample_vec.iter());

        assert_eq!(book_of_record.len(), 2);
        assert!(
            book_of_record.contains_key("BTC"),
            "book did not contain BTC"
        );
        assert!(
            book_of_record.contains_key("DOT"),
            "book did not contain DOT"
        );
        assert_eq!(*book_of_record.get("BTC").unwrap(), Decimal::zero());
        assert_eq!(
            *book_of_record.get("DOT").unwrap(),
            Decimal::new(3370245, 4)
        );
    }
}
