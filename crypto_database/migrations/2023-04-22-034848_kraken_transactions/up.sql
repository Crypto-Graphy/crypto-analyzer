-- Your SQL goes here

CREATE TABLE kraken_transactions (
    id SERIAL PRIMARY KEY,
    txid TEXT,
    refid TEXT NOT NULL,
    transaction_time TIMESTAMPTZ NOT NULL,
    record_type TEXT NOT NULL,
    subtype TEXT,
    a_class TEXT NOT NULL,
    asset TEXT NOT NULL,
    amount NUMERIC NOT NULL,
    fee NUMERIC NOT NULL,
    balance NUMERIC
)