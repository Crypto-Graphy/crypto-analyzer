use coinbase_parser::{CoinbaseTransactionRecord, CSV_HEADERS};
use csv_parser::{Csv, CsvIdentifier, CsvParser};
use kraken_parser::{KrakenLedgerRecord, CSV_HEADERS as KRAKEN_HEADERS};
use serde::Serialize;

#[derive(Serialize)]
#[serde(untagged)]
pub enum CsvType {
    CoinbaseTransactions(Vec<CoinbaseTransactionRecord>),
    KrakenLedgers(Vec<KrakenLedgerRecord>),
    NotRecognized(&'static str),
}

pub fn parse_csv(csv: String) -> CsvType {
    let coinbase_headers: Vec<&str> = CSV_HEADERS.to_vec();
    let kraken_headers: Vec<&str> = KRAKEN_HEADERS.to_vec();

    if Csv::is_valid_csv(&csv, coinbase_headers) {
        let coinbase_transactions: Vec<CoinbaseTransactionRecord> = Csv::parse_csv(&csv);
        CsvType::CoinbaseTransactions(coinbase_transactions)
    } else if Csv::is_valid_csv(&csv, kraken_headers) {
        let kraken_ledgers = Csv::parse_csv(&csv);
        CsvType::KrakenLedgers(kraken_ledgers)
    } else {
        CsvType::NotRecognized("Failed to match csv to known types. This may happen when headers are changed from coinbase or kraken")
    }
}
