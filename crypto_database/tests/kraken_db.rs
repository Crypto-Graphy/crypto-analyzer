mod common;

mod kraken_db_should {
    use chrono::{DateTime, Utc};
    use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
    use uuid::Uuid;

    use crate::common::create_test_context;
    use crypto_database::kraken_db;
    use models_db::{
        schema::kraken_transactions, KrakenTransaction, NewKrakenTransaction, Pagination,
    };
    use rand::{self, Rng};
    use rust_decimal::Decimal;

    pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("./migrations");
    const KRAKEN_DB_NAME: &'static str = "kraken_test_database";

    #[test]
    fn insert_kraken() {
        let test_context = create_test_context(Some(KRAKEN_DB_NAME.to_owned()));
        let mut db_connection = test_context.create_connection();
        db_connection.run_pending_migrations(MIGRATIONS).unwrap();

        let paginations = Pagination::default();
        let results = kraken_db::get_kraken_transactions(&paginations, &mut db_connection).unwrap();
        assert_eq!(results.len(), 0);

        let kraken_transaction = create_random_kraken();

        let result =
            kraken_db::insert_kraken_transaction(kraken_transaction.clone(), &mut db_connection)
                .unwrap();
        let expected = create_kraken_transaction_from_new(kraken_transaction.clone(), result.id);
        assert_eq!(result, expected); // Tests the return is the same as the input + id assigned by the database.

        let kraken_transaction =
            kraken_db::get_kraken_transaction(result.id, &mut db_connection).unwrap();
        assert_eq!(kraken_transaction, expected);
    }

    fn create_random_kraken() -> NewKrakenTransaction {
        let assets = vec!["ADA", "BTC", "SOL", "ETH"];
        let mut rng = rand::thread_rng();

        let asset = assets
            .get(rng.gen_range(0..assets.len()))
            .unwrap()
            .to_string();
        let amount = Decimal::new(rng.gen_range(0..100000), rng.gen_range(0..6));
        let fee = Decimal::new(rng.gen_range(0..10), 0);

        NewKrakenTransaction {
            txid: Some(Uuid::new_v4().to_string()),
            refid: Uuid::new_v4().to_string(),
            transaction_time: DateTime::default(),
            record_type: "Buy".to_string(),
            subtype: None,
            a_class: "".to_string(),
            asset: asset,
            amount,
            fee,
            balance: Some(amount.clone()),
        }
    }

    fn create_kraken_transaction_from_new(
        new_coinbase_transaction: NewKrakenTransaction,
        id: i32,
    ) -> KrakenTransaction {
        KrakenTransaction {
            id,
            txid: new_coinbase_transaction.txid,
            refid: new_coinbase_transaction.refid,
            transaction_time: new_coinbase_transaction.transaction_time,
            record_type: new_coinbase_transaction.record_type,
            subtype: new_coinbase_transaction.subtype,
            a_class: new_coinbase_transaction.a_class,
            asset: new_coinbase_transaction.asset,
            amount: new_coinbase_transaction.amount,
            fee: new_coinbase_transaction.fee,
            balance: new_coinbase_transaction.balance,
        }
    }
}
