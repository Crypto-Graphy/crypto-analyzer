use coinbase_transactions::transaction_parser::{
    self, is_transactions_csv, CoinbaseTransactionRecord,
};
use kraken_ledgers::ledger_parser::{self, is_ledgers_csv, KrakenLedgerRecord};
use serde::Serialize;

#[derive(Serialize)]
#[serde(untagged)]
pub enum CsvType {
    CoinbaseTransactions(Vec<CoinbaseTransactionRecord>),
    KrakenLedgers(Vec<KrakenLedgerRecord>),
    NotRecognized(&'static str),
}

pub fn parse_csv(csv: String) -> CsvType {
    if is_transactions_csv(&csv) {
        let coinbase_transactions = transaction_parser::parse_csv_str(csv);
        CsvType::CoinbaseTransactions(coinbase_transactions)
    } else if is_ledgers_csv(&csv) {
        let kraken_ledgers = ledger_parser::parse_csv_str(csv);
        CsvType::KrakenLedgers(kraken_ledgers)
    } else {
        CsvType::NotRecognized("Failed to match csv to known types. This may happen when headers are changed from coinbase or kraken")
    }
}
