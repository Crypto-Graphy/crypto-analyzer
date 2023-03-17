use std::time::SystemTime;

use coinbase_transactions::transaction_parser::CoinbaseTransactionRecord;
use csv_parser::{Csv, CsvParser};

fn main() {
    let data = std::fs::read_to_string("./data/very-large-dataset.csv").unwrap();
    let start = SystemTime::now();
    let _data: Vec<CoinbaseTransactionRecord> = Csv::parse_csv(&data);

    if let Ok(elapsed) = start.elapsed() {
        println!("Elapsed millis is: {}", elapsed.as_millis());
    }
}
