use axum::{
    extract::{Path, Query},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use crypto_database::{
    coinbase_db::{CoinbaseTransaction, NewCoinbaseTransaction, Pagination},
    kraken_db::{KrakenTransaction, NewKrakenTransaction},
};
use parse_csv::{parse_csv, CsvType};
use server_response::ServerResponse;
use std::{env, net::SocketAddr, str::FromStr};
use tower_http::cors::{Any, CorsLayer};

const API_VERSION: &str = "v1";

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route(
            "/",
            get(|| async { "You may have left off /api/v1/ in your request" }),
        )
        .route(
            format!("/api/{}/parse-csv", API_VERSION).as_str(),
            post(parse_csver),
        )
        .route(
            format!("/api/{}/coinbase-transaction/:id", API_VERSION).as_str(),
            get(get_coinbase_transaction),
        )
        .route(
            format!("/api/{}/coinbase-transaction", API_VERSION).as_str(),
            get(get_coinbase_transactions),
        )
        .route(
            format!("/api/{}/coinbase-transaction", API_VERSION).as_str(),
            post(insert_coinbase_transaction),
        )
        .route(
            format!("/api/{}/kraken-transaction/:id", API_VERSION).as_str(),
            get(get_kraken_transaction),
        )
        .route(
            format!("/api/{}/kraken-transaction", API_VERSION).as_str(),
            get(get_kraken_transactions),
        )
        .route(
            format!("/api/{}/kraken-transaction", API_VERSION).as_str(),
            post(insert_kraken_transaction),
        );

    let coors_layer = CorsLayer::new().allow_origin(Any);
    axum::Server::bind(&get_socket_address())
        .serve(app.layer(coors_layer).into_make_service())
        .await
        .unwrap();
}

fn get_socket_address() -> SocketAddr {
    let ip = env::var("ip_address").unwrap_or("0.0.0.0".to_string());
    let port = env::var("port").unwrap_or("3000".to_string());
    let address = SocketAddr::from_str(format!("{ip}:{port}").as_str());

    match address {
        Ok(address) => {
            println!("Connecting to: {}", address);
            address
        }
        Err(error) => panic!("{}", error),
    }
}

async fn parse_csver(payload: String) -> (StatusCode, Json<CsvType>) {
    println!("parsing csv");
    match parse_csv(payload) {
        CsvType::CoinbaseTransactions(value) => {
            (StatusCode::OK, Json(CsvType::CoinbaseTransactions(value)))
        }
        CsvType::KrakenLedgers(value) => (StatusCode::OK, Json(CsvType::KrakenLedgers(value))),
        CsvType::NotRecognized(e) => (StatusCode::BAD_REQUEST, Json(CsvType::NotRecognized(e))),
    }
}

async fn get_coinbase_transaction(
    id: Path<i32>,
) -> (StatusCode, Json<ServerResponse<CoinbaseTransaction>>) {
    let server_response = coinbase_actions::get_coinbase_transaction(id.0);

    (StatusCode::OK, Json(server_response))
}

async fn get_coinbase_transactions(
    pagination: Query<Pagination>,
) -> (StatusCode, Json<ServerResponse<Vec<CoinbaseTransaction>>>) {
    let server_response = coinbase_actions::get_coinbase_transactions(pagination.0);

    (StatusCode::OK, Json(server_response))
}

async fn insert_coinbase_transaction(
    payload: Json<NewCoinbaseTransaction>,
) -> (StatusCode, Json<ServerResponse<CoinbaseTransaction>>) {
    let coinbase_transaction = coinbase_actions::insert_coinbase_transaction(payload.0);

    let status_code = match &coinbase_transaction.success {
        true => StatusCode::CREATED,
        false => StatusCode::INTERNAL_SERVER_ERROR,
    };

    (status_code, Json(coinbase_transaction))
}

async fn get_kraken_transaction(
    id: Path<i32>,
) -> (StatusCode, Json<ServerResponse<KrakenTransaction>>) {
    let kraken_transaction = kraken_actions::get_kraken_transaction(id.0);

    (StatusCode::OK, Json(kraken_transaction))
}

async fn get_kraken_transactions(
    pagination: Query<Pagination>,
) -> (StatusCode, Json<ServerResponse<Vec<KrakenTransaction>>>) {
    let kraken_trasnactions = kraken_actions::get_kraken_transactions(pagination.0);

    (StatusCode::OK, Json(kraken_trasnactions))
}

async fn insert_kraken_transaction(
    payload: Json<NewKrakenTransaction>,
) -> (StatusCode, Json<ServerResponse<KrakenTransaction>>) {
    let kraken_transaction = kraken_actions::insert_kraken_transaction(payload.0);

    let status_code = match &kraken_transaction.success {
        true => StatusCode::CREATED,
        false => StatusCode::INTERNAL_SERVER_ERROR,
    };

    (status_code, Json(kraken_transaction))
}

#[cfg(test)]
mod parse_csver_should {
    extern crate rust_decimal;
    use std::{str::FromStr, vec};

    use coinbase_parser::CoinbaseTransactionRecord;
    use kraken_parser::{KrakenLedgerRecord, DATE_FORMAT as KRAKEN_DATE_FORMAT};

    use axum::{http::StatusCode, Json};
    use parse_csv::CsvType;
    use rust_decimal::Decimal;

    use super::parse_csver;
    use chrono::prelude::*;

    #[actix_rt::test]
    async fn parse_coinbase_transaction() {
        let csv = "Timestamp,Transaction Type,Asset,Quantity Transacted,Spot Price Currency,Spot Price at Transaction,Subtotal,Total (inclusive of fees and/or spread),Fees and/or Spread,Notes\n".to_string()
            + "2021-01-22T21:38:01Z,Buy,BTC,0.0016458,USD,1617.57,97.01,100.00,2.99,Bought 0.0016458 BTC for $2.66 USD\n"
            + "2022-01-22T21:39:01Z,Sell,BTC,0.0016458,USD,1617.57,97.01,100.00,2.99,Sold 0.0016458 BTC for $2.66 USD";

        let expected_vec = vec![
            CoinbaseTransactionRecord {
                time_of_transaction: "2021-01-22T21:38:01Z".parse::<DateTime<Utc>>().unwrap(),
                transaction_type: "Buy".to_string(),
                asset: "BTC".to_string(),
                quantity_transacted: Decimal::from_str("0.0016458").unwrap(),
                spot_price_currency: "USD".to_string(),
                spot_price_at_transaction: Some(Decimal::from_str("1617.57").unwrap()),
                subtotal: Some(Decimal::from_str("97.01").unwrap()),
                total: Some(Decimal::from_str("100").unwrap()),
                fees: Some(Decimal::from_str("2.99").unwrap()),
                notes: "Bought 0.0016458 BTC for $2.66 USD".to_string(),
            },
            CoinbaseTransactionRecord {
                time_of_transaction: "2022-01-22T21:39:01Z".parse::<DateTime<Utc>>().unwrap(),
                transaction_type: "Sell".to_string(),
                asset: "BTC".to_string(),
                quantity_transacted: Decimal::from_str("0.0016458").unwrap(),
                spot_price_currency: "USD".to_string(),
                spot_price_at_transaction: Some(Decimal::from_str("1617.57").unwrap()),
                subtotal: Some(Decimal::from_str("97.01").unwrap()),
                total: Some(Decimal::from_str("100").unwrap()),
                fees: Some(Decimal::from_str("2.99").unwrap()),
                notes: "Sold 0.0016458 BTC for $2.66 USD".to_string(),
            },
        ];

        let (status_code, Json(parsed)) = parse_csver(csv).await;

        assert_eq!(status_code, StatusCode::OK);
        match parsed {
            CsvType::CoinbaseTransactions(transaction_list) => {
                assert_eq!(
                    transaction_list.get(0).unwrap(),
                    expected_vec.get(0).unwrap()
                );
                assert_eq!(
                    transaction_list.get(1).unwrap(),
                    expected_vec.get(1).unwrap()
                );
            }
            _ => panic!("Failed to parse as Coinbase transaction"),
        }
    }

    #[actix_rt::test]
    async fn parse_kraken_ledger() {
        let csv = "txid,refid,time,type,subtype,aclass,asset,amount,fee,balance\n".to_string() 
                + "QWERTY-FOGWB-JOTO7J,QWERTY-ILZGGG-LCBLBL,2021-07-29 1:19:30,Buy,,currency,ADA,5.00000000,0.00000000,5.00000000\n"
                + "YTREWQ-FOGWB-JOTO7J,YTREWQ-ILZGGG-LCBLBL,2022-07-29 1:19:30,Sell,,currency,ADA,5.00000000,0.00000000,0.00000000";

        let expected_vec = vec![
            KrakenLedgerRecord {
                txid: Some("QWERTY-FOGWB-JOTO7J".to_string()),
                refid: "QWERTY-ILZGGG-LCBLBL".to_string(),
                time: Utc
                    .datetime_from_str("2021-07-29 1:19:30", KRAKEN_DATE_FORMAT)
                    .unwrap(),
                record_type: "Buy".to_string(),
                subtype: None,
                a_class: "currency".to_string(),
                asset: "ADA".to_string(),
                amount: Decimal::from_str("5.00000000").unwrap(),
                fee: Decimal::from_str("0.00000000").unwrap(),
                balance: Some(Decimal::from_str("5.00000000").unwrap()),
            },
            KrakenLedgerRecord {
                txid: Some("YTREWQ-FOGWB-JOTO7J".to_string()),
                refid: "YTREWQ-ILZGGG-LCBLBL".to_string(),
                time: Utc
                    .datetime_from_str("2022-07-29 1:19:30", KRAKEN_DATE_FORMAT)
                    .unwrap(),
                record_type: "Sell".to_string(),
                subtype: None,
                a_class: "currency".to_string(),
                asset: "ADA".to_string(),
                amount: Decimal::from_str("5.00000000").unwrap(),
                fee: Decimal::from_str("0.00000000").unwrap(),
                balance: Some(Decimal::from_str("0.00000000").unwrap()),
            },
        ];

        let (status_code, Json(parsed)) = parse_csver(csv).await;

        assert_eq!(status_code, StatusCode::OK);

        match parsed {
            CsvType::KrakenLedgers(kraken_vec) => {
                assert_eq!(kraken_vec.get(0).unwrap(), expected_vec.get(0).unwrap());
                assert_eq!(kraken_vec.get(1).unwrap(), expected_vec.get(1).unwrap());
            }
            _ => panic!("Response was not parsed as a Kraken record"),
        }
    }

    #[actix_rt::test]
    async fn parse_not_not_recognized() {
        let csv = "Something Random,Another Random Column\n".to_string()
            + "some random data, some random column";

        let (status_code, Json(parsed)) = parse_csver(csv).await;

        assert_eq!(status_code, StatusCode::BAD_REQUEST);
        match parsed {
            CsvType::NotRecognized(message) => assert_eq!(message, "Failed to match csv to known types. This may happen when headers are changed from coinbase or kraken"),
            _ => panic!("Response was recognized but shouldn't have"),
        }
    }
}
