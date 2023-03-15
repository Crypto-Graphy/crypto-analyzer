extern crate csv;

use csv::ReaderBuilder;

pub trait CsvParser {
    fn parse_csv<C: for<'a> serde::Deserialize<'a>>(csv: &str) -> Vec<C>;
}

pub trait CsvIdentifier {
    fn is_valid_csv<'a>(csv: &str, expected_csv_headers: impl IntoIterator<Item = &'a str>)
        -> bool;
}

pub struct Csv;

impl CsvParser for Csv {
    fn parse_csv<C: for<'a> serde::Deserialize<'a>>(csv: &str) -> Vec<C> {
        ReaderBuilder::new()
            .flexible(true)
            .from_reader(csv.as_bytes())
            .into_deserialize()
            .flatten()
            .collect()
    }
}

impl CsvIdentifier for Csv {
    fn is_valid_csv<'a>(
        csv: &str,
        expected_csv_headers: impl IntoIterator<Item = &'a str>,
    ) -> bool {
        let mut binding = ReaderBuilder::new().from_reader(csv.as_bytes());
        let header_row_headers: Vec<&str> = match binding.headers() {
            Ok(csv_headers) => csv_headers.into_iter().collect(),
            Err(_) => return false,
        };

        return expected_csv_headers
            .into_iter()
            .all(|expected_header| header_row_headers.contains(&expected_header));
    }
}

#[cfg(test)]
mod parse_csv_should {
    extern crate models;
    extern crate rust_decimal;
    use self::rust_decimal::Decimal;
    use crate::{Csv, CsvIdentifier, CsvParser};

    use models::{
        coinbase::CSV_HEADERS as COINBASE_HEADERS,
        kraken::{KrakenLedgerRecord, CSV_HEADERS as KRAKEN_HEADERS},
    };

    #[test]
    fn return_true_when_valid_csv() {
        let csv = "Timestamp,Transaction Type,Asset,Quantity Transacted,Spot Price Currency,Spot Price at Transaction,Subtotal,Total (inclusive of fees and/or spread),Fees and/or Spread,Notes\n".to_string() 
        + "2021-01-22T21:38:01Z,Buy,BTC,0.0016458,USD,1617.57,97.01,2.66,2.99,Bought 0.0016458 BTC for $2.66 USD";

        let valid_csv = Csv::is_valid_csv(
            csv.as_str(),
            COINBASE_HEADERS
                .clone()
                .into_iter()
                .map(|header| *header)
                .collect::<Vec<&str>>(),
        );
        assert!(valid_csv);
    }

    #[test]
    fn return_false_when_empty() {
        let csv = "";

        let valid_csv = Csv::is_valid_csv(
            csv,
            COINBASE_HEADERS
                .clone()
                .into_iter()
                .map(|header| *header)
                .collect::<Vec<&str>>(),
        );
        assert!(!valid_csv);
    }

    #[test]
    fn return_false_when_contains_some_headers() {
        let csv = "Asset,Quantity Transacted\n".to_string() + "BTC,0.01";
        let headers = COINBASE_HEADERS
            .clone()
            .into_iter()
            .map(|header| *header)
            .collect::<Vec<&str>>();

        let valid_csv = Csv::is_valid_csv(csv.as_str(), headers);
        assert!(!valid_csv);
    }

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
            fee: Decimal::ZERO,
            balance: Some(Decimal::new(5, 0)),
        };

        let actual: Vec<KrakenLedgerRecord> = Csv::parse_csv(&given_csv);
        assert_eq!(*actual.first().unwrap(), expected_ledger);
    }
}

#[cfg(test)]
mod is_valid_csv {
    extern crate models;
    use self::models::kraken::CSV_HEADERS as KRAKEN_HEADERS;
    use crate::CsvIdentifier;

    use super::Csv;

    fn get_kraken_headers() -> Vec<&'static str> {
        KRAKEN_HEADERS.into_iter().map(|header| *header).collect()
    }

    #[test]
    fn return_true_when_valid_csv() {
        let csv = "txid,refid,time,type,subtype,aclass,asset,amount,fee,balance\n".to_string() 
            + "QWERTY-FOGWB-JOTO7J,QWERTY-ILZGGG-LCBLBL,2021-07-29 1:19:30,deposit,,currency,ADA,5.00000000,0.00000000,5.00000000";

        let valid_csv = Csv::is_valid_csv(&csv, get_kraken_headers());
        assert!(valid_csv);
    }

    #[test]
    fn return_false_when_empty() {
        let csv = "";

        let valid_csv = Csv::is_valid_csv(&csv, get_kraken_headers());
        assert!(!valid_csv);
    }

    #[test]
    fn return_false_when_contains_some_headers() {
        let csv = "type,subtype\n".to_string() + "deposit,ADA";

        let valid_csv = Csv::is_valid_csv(&csv, get_kraken_headers());
        assert!(!valid_csv);
    }
}
