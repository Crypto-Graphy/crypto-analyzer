pub mod ledger_parser {
    extern crate csv;
    extern crate rust_decimal;
    extern crate serde;
    use std::collections::HashMap;

    extern crate models;
    use self::rust_decimal::Decimal;

    pub use self::models::kraken::{KrakenLedgerRecord, CSV_HEADERS, DATE_FORMAT};

    pub fn get_total_staking_rewards_map<'a, I>(records: I) -> HashMap<String, Decimal>
    where
        I: Iterator<Item = &'a KrakenLedgerRecord>,
    {
        records
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

    pub fn get_book_of_record<'a, I>(records: I) -> HashMap<String, Decimal>
    where
        I: Iterator<Item = &'a KrakenLedgerRecord>,
    {
        records
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

    pub fn get_record_of_transactions<'a>(
        records: impl Iterator<Item = &'a KrakenLedgerRecord>,
    ) -> HashMap<String, Vec<&'a KrakenLedgerRecord>> {
        records
            .into_iter()
            .fold(HashMap::new(), |mut currency_map, record| {
                let mut vector = currency_map.remove(&record.asset).unwrap_or_default();

                vector.push(record);
                currency_map.insert(record.asset.to_string(), vector);

                currency_map
            })
    }
}

#[cfg(test)]
mod test {
    extern crate chrono;
    extern crate rust_decimal;

    use self::rust_decimal::{prelude::Zero, Decimal};

    use self::chrono::prelude::*;
    use super::ledger_parser::{
        get_total_staking_rewards_map, KrakenLedgerRecord, DATE_FORMAT as KRAKEN_DATE_FORMAT,
    };

    #[test]
    fn should_get_staking_rewards_for_multiple() {
        let sample_ledger_1 = KrakenLedgerRecord {
            id: None,
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
            id: None,
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

        let actual = get_total_staking_rewards_map(sample_vec.iter());
        assert_eq!(actual.len(), 2);

        // Keys
        let mut keys: Vec<String> = actual.keys().cloned().collect();
        keys.sort();
        assert_eq!(keys[0], "ADA");
        assert_eq!(keys[1], "DOT");

        // Values
        let mut values = actual.values().cloned().collect::<Vec<Decimal>>();
        values.sort_by(|a, b| a.partial_cmp(b).unwrap());

        assert_eq!(values[0], Decimal::new(5, 0));
        assert_eq!(values[1], Decimal::new(51002, 4));
    }

    #[test]
    fn should_sum_rewards_of_the_same_currency() {
        let sample_ledger_1 = KrakenLedgerRecord {
            id: None,
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
            id: None,
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

        let expected = Decimal::from_str_radix("5.10020000", 10).unwrap() + Decimal::new(5, 0);

        let reward_map =
            get_total_staking_rewards_map(vec![sample_ledger_1, sample_ledger_2].iter());

        assert_eq!(*reward_map.get("DOT").unwrap(), expected);
    }

    #[test]
    fn get_total_staking_rewards_when_given_empty_vec() {
        let sample_vec: Vec<KrakenLedgerRecord> = Vec::new();
        let actual = get_total_staking_rewards_map(sample_vec.iter());

        assert!(actual.is_empty(), "staking rewards is not empty");
    }
}

#[cfg(test)]
mod book_of_record {
    extern crate chrono;
    extern crate rust_decimal;
    use super::ledger_parser::{
        get_book_of_record, KrakenLedgerRecord, DATE_FORMAT as KRAKEN_DATE_FORMAT,
    };

    use self::chrono::prelude::*;
    use self::rust_decimal::{prelude::Zero, Decimal};

    #[test]
    fn should_sum_multiple_book() {
        let sample_ledger_1 = KrakenLedgerRecord {
            id: None,
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
            id: None,
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
        let expected_sum = Decimal::from_str_radix("5.10020000", 10).unwrap() + Decimal::new(5, 0);

        let bor = get_book_of_record(sample_vec.iter());

        assert!(bor.contains_key("DOT"), "book did not contain DOT");
        assert_eq!(*bor.get("DOT").unwrap(), expected_sum);
    }

    #[test]
    fn should_subtract_negative_values_book() {
        let sample_ledger_1 = KrakenLedgerRecord {
            id: None,
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
            id: None,
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
        let book_of_record = get_book_of_record(sample_vec.iter());

        assert!(
            book_of_record.contains_key("DOT"),
            "book did not contain DOT"
        );
        assert_eq!(*book_of_record.get("DOT").unwrap(), expected_sum);
    }

    #[test]
    fn returns_empty_map_when_empty_iter() {
        let sample_vec = Vec::new();

        let book_of_record = get_book_of_record(sample_vec.iter());

        assert!(book_of_record.is_empty(), "book was not empty");
    }
}
