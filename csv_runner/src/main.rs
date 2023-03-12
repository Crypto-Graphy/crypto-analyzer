use std::time::SystemTime;

use coinbase_transactions::transaction_parser::parse_csv_str;

fn main() {
    let data = std::fs::read_to_string("./data/very-large-dataset.csv").unwrap();
    let start = SystemTime::now();
    let _data = parse_csv_str(data);

    if let Ok(elapsed) = start.elapsed() {
        println!("{}", elapsed.as_millis());
    }
}
