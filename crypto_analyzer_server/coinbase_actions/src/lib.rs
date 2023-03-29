use crypto_database::models::{CoinbaseTransaction, NewCoinbaseTransaction, Pagination};
use server_response::ServerResponse;
use uuid::Uuid;

pub fn get_coinbase_transaction(id: i32) -> ServerResponse<CoinbaseTransaction> {
    let mut connection = crypto_database::establish_connection();
    let result = crypto_database::get_coinbase_transaction(id, &mut connection);

    let messages = result.as_ref().map_or(None, |transaction| {
        Some(vec![format!(
            "Found coinbase transaction with id: {}",
            &transaction.id
        )])
    });
    let errors = match result.as_ref() {
        Ok(_) => None,
        Err(e) => Some(vec![format!("{}", e)]),
    };

    ServerResponse::new(
        Default::default(),
        result.is_ok(),
        result.ok(),
        messages,
        errors,
    )
}

pub fn get_coinbase_transactions(
    pagination: Pagination,
) -> ServerResponse<Vec<CoinbaseTransaction>> {
    // TODO: Probably shouldn't create a connection on every request. ... This may be handled behind the scenes in diesel. review
    let mut connection = crypto_database::establish_connection();
    let coinbase_transactions =
        crypto_database::get_coinbase_transactions(&pagination, &mut connection);

    let messages = coinbase_transactions.as_ref().map_or(None, |cts| {
        Some(vec![format!(
            "Retrieved {} records from page {}",
            cts.len(),
            &pagination.page
        )])
    });
    let errors = match coinbase_transactions.as_ref() {
        Ok(_) => None,
        Err(e) => Some(vec![format!("{}", e)]),
    };

    ServerResponse::new(
        Default::default(),
        coinbase_transactions.is_ok(),
        coinbase_transactions.ok(),
        messages,
        errors,
    )
}

pub fn insert_coinbase_transaction(
    new_coinbase_transaction: NewCoinbaseTransaction,
) -> ServerResponse<CoinbaseTransaction> {
    // TODO: Probably shouldn't create a connection on every request. ... This may be handled behind the scenes in diesel. review
    let mut connection = crypto_database::establish_connection();
    let coinbase_transaction =
        crypto_database::insert_coinbase_transaction(new_coinbase_transaction, &mut connection);

    let messages = coinbase_transaction.as_ref().map_or(None, |ct| {
        Some(vec![format!(
            "Inserted new coinbase transaction with id: {}",
            &ct.id
        )])
    });
    let errors = match coinbase_transaction.as_ref() {
        Ok(_) => None,
        Err(e) => Some(vec![format!("{}", e)]),
    };

    ServerResponse::new(
        Some(Uuid::new_v4()),
        coinbase_transaction.is_ok(),
        coinbase_transaction.ok(),
        messages,
        errors,
    )
}
