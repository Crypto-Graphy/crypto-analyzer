-- Your SQL goes here
CREATE TABLE coinbase_transactions (
    id SERIAL PRIMARY KEY,
    time_of_transaction TIMESTAMPTZ NOT NULL,
    transaction_type TEXT NOT NULL,
    asset TEXT NOT NULL,
    quantity_transacted NUMERIC NOT NULL,
    spot_price_currency TEXT NOT NULL,
    spot_price_at_transaction NUMERIC,
    subtotal NUMERIC,
    total NUMERIC,
    fees NUMERIC,
    notes TEXT NOT NULL
)
