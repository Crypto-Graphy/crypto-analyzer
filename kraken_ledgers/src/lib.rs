pub mod ledger_parser {
    extern crate csv;
    extern crate rust_decimal;
    extern crate serde;
    use std::collections::HashMap;

    use self::csv::{Reader, ReaderBuilder};
    use self::rust_decimal::Decimal;
    use self::serde::{Deserialize, Serialize};

    const CSV_HEADERS: &[&str] = &[
        "txid", "refid", "time", "type", "subtype", "aclass", "asset", "amount", "fee", "balance",
    ];

    #[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
    pub struct KrakenLedgerRecord {
        pub txid: Option<String>,
        pub refid: String,
        pub time: String,
        #[serde(rename = "type")]
        pub record_type: String,
        pub subtype: Option<String>,
        #[serde(rename = "aclass")]
        pub a_class: String,
        pub asset: String,
        #[serde(with = "rust_decimal::serde::str")]
        pub amount: Decimal,
        #[serde(with = "rust_decimal::serde::str")]
        pub fee: Decimal,
        #[serde(with = "rust_decimal::serde::str_option")]
        pub balance: Option<Decimal>,
    }

    pub fn is_ledgers_csv(csv: &str) -> bool {
        let mut binding = ReaderBuilder::new().from_reader(csv.as_bytes());
        let header_row_headers: Vec<&str> = match binding.headers() {
            Ok(headers) => headers.into_iter().collect(),
            Err(_) => return false,
        };

        return CSV_HEADERS
            .iter()
            .all(|header| header_row_headers.contains(header));
    }

    pub fn parse_csv_str(csv: String) -> Vec<KrakenLedgerRecord> {
        Reader::from_reader(csv.as_bytes())
            .into_deserialize()
            .flatten()
            .collect()
    }

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

    #[cfg(test)]
    mod is_ledgers_csv_should {
        use super::is_ledgers_csv;

        #[test]
        fn return_true_when_valid_csv() {
            let csv = "txid,refid,time,type,subtype,aclass,asset,amount,fee,balance\n".to_string() 
            + "QWERTY-FOGWB-JOTO7J,QWERTY-ILZGGG-LCBLBL,2021-07-29 1:19:30,deposit,,currency,ADA,5.00000000,0.00000000,5.00000000";

            let valid_csv = is_ledgers_csv(csv.as_str());
            assert!(valid_csv);
        }

        #[test]
        fn return_false_when_empty() {
            let csv = "";

            let valid_csv = is_ledgers_csv(csv);
            assert!(!valid_csv);
        }

        #[test]
        fn return_false_when_contains_some_headers() {
            let csv = "type,subtype\n".to_string() + "deposit,ADA";

            let valid_csv = is_ledgers_csv(csv.as_str());
            assert!(!valid_csv);
        }
    }
}

#[cfg(test)]
mod test {
    extern crate rust_decimal;

    use self::rust_decimal::{prelude::Zero, Decimal};

    use super::ledger_parser::{get_total_staking_rewards_map, parse_csv_str, KrakenLedgerRecord};

    #[test]
    fn should_parse_csv() {
        let given_csv: String =
            "txid,refid,time,type,subtype,aclass,asset,amount,fee,balance\n".to_string() 
                + "L7RLII-OFGWB-JTUO7J,RKB7ODD-ILZGC5-LCRRBL,2021-09-29 15:18:30,deposit,,currency,ADA,5.00000000,0.00000000,5.00000000";

        let expected_ledger = KrakenLedgerRecord {
            txid: Some("L7RLII-OFGWB-JTUO7J".to_string()),
            refid: "RKB7ODD-ILZGC5-LCRRBL".to_string(),
            time: "2021-09-29 15:18:30".to_string(),
            record_type: "deposit".to_string(),
            subtype: None,
            a_class: "currency".to_string(),
            asset: "ADA".to_string(),
            amount: Decimal::new(5, 0),
            fee: Decimal::zero(),
            balance: Some(Decimal::new(5, 0)),
        };

        let actual = parse_csv_str(given_csv);
        assert_eq!(*actual.first().unwrap(), expected_ledger);
    }

    #[test]
    fn should_get_staking_rewards_for_multiple() {
        let sample_ledger_1 = KrakenLedgerRecord {
            txid: Some("L7RLII-OFGWB-JTUO7J".to_string()),
            refid: "RKB7ODD-ILZGC5-LCRRBL".to_string(),
            time: "2021-09-29 15:18:30".to_string(),
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
            time: "2021-09-29 15:18:30".to_string(),
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
            txid: Some("L7RLII-OFGWB-JTUO7J".to_string()),
            refid: "RKB7ODD-ILZGC5-LCRRBL".to_string(),
            time: "2021-09-29 15:18:30".to_string(),
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
            time: "2021-09-29 15:18:30".to_string(),
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
    extern crate rust_decimal;
    use super::ledger_parser::{get_book_of_record, KrakenLedgerRecord};

    use self::rust_decimal::{prelude::Zero, Decimal};

    #[test]
    fn should_sum_multiple_book() {
        let sample_ledger_1 = KrakenLedgerRecord {
            txid: Some("L7RLII-OFGWB-JTUO7J".to_string()),
            refid: "RKB7ODD-ILZGC5-LCRRBL".to_string(),
            time: "2021-09-29 15:18:30".to_string(),
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
            time: "2021-09-29 15:18:30".to_string(),
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
            txid: Some("L7RLII-OFGWB-JTUO7J".to_string()),
            refid: "RKB7ODD-ILZGC5-LCRRBL".to_string(),
            time: "2021-09-29 15:18:30".to_string(),
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
            time: "2021-09-29 15:18:30".to_string(),
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
