use std::collections::HashMap;

use chrono::{DateTime, Utc};
use coinbase_transactions::transaction_parser::{
    self, get_book_of_record, get_total_staking_rewards_map, CoinbaseTransactionRecord,
};
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rand::Rng;
use rust_decimal::Decimal;

fn create_random_new_coinbase_transaction(
    transaction_type: Option<String>,
) -> CoinbaseTransactionRecord {
    let assets = vec!["ADA", "BTC", "SOL", "ETH"];
    let mut rng = rand::thread_rng();

    let time_of_transaction: DateTime<Utc> = DateTime::default();
    let asset = assets
        .get(rng.gen_range(0..assets.len()))
        .unwrap()
        .to_string();
    let quantity_transacted = Decimal::new(rng.gen_range(0..100000), rng.gen_range(0..6));
    let spot_price = Some(Decimal::new(rng.gen_range(0..40000), rng.gen_range(0..=2)));
    let fees = Some(Decimal::new(rng.gen_range(0..10), 0));
    let subtotal = spot_price.map(|price| price * quantity_transacted);
    let total = subtotal.zip(fees).map(|(subtotal, fees)| subtotal + fees);
    let notes = format!(
        "Bought {} {} at {} USD",
        quantity_transacted,
        asset,
        spot_price.unwrap()
    );
    let transaction_type = transaction_type.unwrap_or("Buy".to_string());

    CoinbaseTransactionRecord {
        time_of_transaction,
        transaction_type,
        asset,
        quantity_transacted,
        spot_price_currency: "USD".to_string(),
        spot_price_at_transaction: spot_price,
        subtotal,
        total,
        fees,
        notes,
    }
}

fn criterion_benchmark(c: &mut Criterion) {
    let coinbase_records: Vec<CoinbaseTransactionRecord> = (0..350)
        .into_iter()
        .map(|_| create_random_new_coinbase_transaction(None))
        .collect();
    let rewards_map: Vec<CoinbaseTransactionRecord> = (0..400)
        .into_iter()
        .map(|_| create_random_new_coinbase_transaction(Some("Reward".to_string())))
        .collect();
    c.bench_function("Book of size 350", |b| {
        b.iter(|| get_book_of_record(black_box(coinbase_records.iter())))
    });
    c.bench_function("Book of size 400", |b| {
        b.iter(|| get_total_staking_rewards_map(black_box(rewards_map.iter())))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
