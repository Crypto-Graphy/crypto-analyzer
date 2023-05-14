// use std::time::SystemTime;

// use coinbase_parser::CoinbaseTransactionRecord;
// use crypto_database::coinbase_db::NewCoinbaseTransaction;
// use csv_parser::{Csv, CsvParser};
// use kraken_parser::KrakenLedgerRecord;
// use models_db::{DBConfig, NewKrakenTransaction};

fn main() {}

// fn coinbase_data() {
//     let path = std::env::var("CSV_PATH").unwrap_or("../data/coinbase-data.csv".to_string());
//     let data = std::fs::read_to_string(path).unwrap();
//     let start = SystemTime::now();
//     let data: Vec<NewCoinbaseTransaction> = Csv::parse_csv(&data)
//         .into_iter()
//         .map(|ctr: CoinbaseTransactionRecord| NewCoinbaseTransaction {
//             time_of_transaction: ctr.time_of_transaction,
//             transaction_type: ctr.transaction_type,
//             asset: ctr.asset,
//             quantity_transacted: ctr.quantity_transacted,
//             spot_price_currency: ctr.spot_price_currency,
//             spot_price_at_transaction: ctr.spot_price_at_transaction,
//             subtotal: ctr.subtotal,
//             total: ctr.total,
//             fees: ctr.fees,
//             notes: ctr.notes,
//         })
//         .collect();

//     println!("length: {}", data.iter().len());

//     let mut connection = crypto_database::establish_connection(Some(DBConfig::default()))
//         .expect("Failed to get connection");

//     let results =
//         crypto_database::coinbase_db::bulk_insert_coinbase_transaction(data, &mut connection)
//             .expect("Failed to insert data");
// }

// fn kraken_data() {
//     let path = "../data/ledgers.csv".to_string();
//     let data = std::fs::read_to_string(path).unwrap();
//     let data: Vec<NewKrakenTransaction> = Csv::parse_csv(&data)
//         .into_iter()
//         .map(|ctr: KrakenLedgerRecord| NewKrakenTransaction {
//             txid: ctr.txid,
//             refid: ctr.refid,
//             transaction_time: ctr.time,
//             record_type: ctr.record_type,
//             subtype: ctr.subtype,
//             a_class: ctr.a_class,
//             asset: ctr.asset,
//             amount: ctr.amount,
//             fee: ctr.fee,
//             balance: ctr.balance,
//         })
//         .collect();

//     println!("{}", data.iter().len());
//     let mut connection = crypto_database::establish_connection(Some(DBConfig::default()))
//         .expect("Failed to get connection");

//     let results = crypto_database::kraken_db::bulk_insert_kraken_transaction(data, &mut connection)
//         .expect("Failed to insert kraken");
//     println!("{}", results.iter().len());
// }
