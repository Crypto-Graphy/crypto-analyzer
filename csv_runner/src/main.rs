use std::time::SystemTime;

use coinbase_transactions::CoinbaseTransactionRecord;
use crypto_database::coinbase_db::NewCoinbaseTransaction;
use csv_parser::{Csv, CsvParser};

fn main() {
    let data = std::fs::read_to_string("./data/very-large-dataset.csv").unwrap();
    let start = SystemTime::now();
    let _data: Vec<NewCoinbaseTransaction> = Csv::parse_csv(&data)
        .into_iter()
        .map(|ctr: CoinbaseTransactionRecord| NewCoinbaseTransaction {
            time_of_transaction: ctr.time_of_transaction,
            transaction_type: ctr.transaction_type,
            asset: ctr.asset,
            quantity_transacted: ctr.quantity_transacted,
            spot_price_currency: ctr.spot_price_currency,
            spot_price_at_transaction: ctr.spot_price_at_transaction,
            subtotal: ctr.subtotal,
            total: ctr.total,
            fees: ctr.fees,
            notes: ctr.notes,
        })
        .collect();

    // let mut connection = crypto_database::establish_connection();

    // let result = crypto_database::bulk_insert_coinbase_transaction(data.collect(), &mut connection)
    //     .expect("Didn't write records");

    if let Ok(elapsed) = start.elapsed() {
        println!("Elapsed millis is: {}", elapsed.as_millis());
    }
}
