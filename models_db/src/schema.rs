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

diesel::table! {
    kraken_transactions (id) {
        id -> Int4,
        txid -> Nullable<Text>,
        refid -> Text,
        transaction_time -> Timestamptz,
        record_type -> Text,
        subtype -> Nullable<Text>,
        a_class -> Text,
        asset -> Text,
        amount -> Numeric,
        fee -> Numeric,
        balance -> Nullable<Numeric>,
    }
}

diesel::allow_tables_to_appear_in_same_query!(coinbase_transactions, kraken_transactions,);
