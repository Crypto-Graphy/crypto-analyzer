use crypto_database::kraken_db::{
    self, models_db::DBConfig, KrakenTransaction, NewKrakenTransaction, Pagination,
};
use server_response::ServerResponse;
use uuid::Uuid;

pub fn get_kraken_transaction(id: i32) -> ServerResponse<KrakenTransaction> {
    let mut connection = crypto_database::establish_connection(Some(DBConfig::init_from_env()))
        .expect("Failed to establish db connection");
    let kraken_transacton = kraken_db::get_kraken_transaction(id, &mut connection);

    let messages = kraken_transacton.as_ref().map_or(None, |transaction| {
        Some(vec![format!(
            "Found kraken transaction with id: {}",
            &transaction.id
        )])
    });
    let errors = match kraken_transacton.as_ref() {
        Ok(_) => None,
        Err(e) => Some(vec![format!("{}", e)]),
    };

    ServerResponse::new(
        Some(Uuid::new_v4()),
        kraken_transacton.is_ok(),
        kraken_transacton.ok(),
        messages,
        errors,
    )
}

pub fn get_kraken_transactions(pagination: Pagination) -> ServerResponse<Vec<KrakenTransaction>> {
    let mut connection = crypto_database::establish_connection(Some(DBConfig::init_from_env()))
        .expect("Failed to establish db connection");
    let kraken_transactions = kraken_db::get_kraken_transactions(&pagination, &mut connection);

    let messages = kraken_transactions.as_ref().map_or(None, |transactions| {
        Some(vec![format!(
            "Retrieved {} records from page {}",
            transactions.len(),
            &pagination.page
        )])
    });
    let errors = match kraken_transactions.as_ref() {
        Ok(_) => None,
        Err(e) => Some(vec![format!("{}", e)]),
    };

    ServerResponse::new(
        Some(Uuid::new_v4()),
        kraken_transactions.is_ok(),
        kraken_transactions.ok(),
        messages,
        errors,
    )
}

pub fn insert_kraken_transaction(
    new_kraken_transaction: NewKrakenTransaction,
) -> ServerResponse<KrakenTransaction> {
    let mut connection = crypto_database::establish_connection(Some(DBConfig::init_from_env()))
        .expect("Failed to establish db connection");
    let kraken_transaction =
        kraken_db::insert_kraken_transaction(new_kraken_transaction, &mut connection);

    let messages = kraken_transaction.as_ref().map_or(None, |kt| {
        Some(vec![format!(
            "Inserted new kraken transaction with id: {}",
            &kt.id
        )])
    });
    let errors = match kraken_transaction.as_ref() {
        Ok(_) => None,
        Err(e) => Some(vec![format!("{}", e)]),
    };

    ServerResponse::new(
        Some(Uuid::new_v4()),
        kraken_transaction.is_ok(),
        kraken_transaction.ok(),
        messages,
        errors,
    )
}
