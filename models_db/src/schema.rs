// @generated automatically by Diesel CLI.

diesel::table! {
    coinbase_transactions (id) {
        id -> Int4,
        time_of_transaction -> Timestamptz,
        transaction_type -> Text,
        asset -> Text,
        quantity_transacted -> Numeric,
        spot_price_currency -> Text,
        spot_price_at_transaction -> Nullable<Numeric>,
        subtotal -> Nullable<Numeric>,
        total -> Nullable<Numeric>,
        fees -> Nullable<Numeric>,
        notes -> Text,
    }
}
