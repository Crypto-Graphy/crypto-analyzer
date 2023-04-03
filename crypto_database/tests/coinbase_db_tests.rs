mod common;

mod coinbase_db_should {
    use std::sync::atomic::Ordering;

    use chrono::{DateTime, Utc};
    use crypto_database::{
        get_coinbase_transactions, insert_coinbase_transaction,
        models::{CoinbaseTransaction, NewCoinbaseTransaction, Pagination},
        schema::coinbase_transactions,
    };
    use diesel::{prelude::*, sql_query};
    use rust_decimal::{prelude::Zero, Decimal};
    use uuid::Uuid;

    use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

    use crate::common::{Config, TestContext, TEST_DB_COUNTER};

    // use crate::common;
    pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("./migrations");

    fn create_test_context(db_id: u8) -> TestContext {
        let config = Config {
            db_name: format!("test_database_{}", db_id),
            ..Default::default()
        };

        TestContext::new(
            config,
            crypto_database::establish_connection()
                .expect("Failed to establish a connection to the database"),
        )
    }

    fn create_coinbase_transaction_from_new(
        new_coinbase_transaction: NewCoinbaseTransaction,
        id: i32,
    ) -> CoinbaseTransaction {
        CoinbaseTransaction {
            id: id,
            time_of_transaction: new_coinbase_transaction.time_of_transaction,
            transaction_type: new_coinbase_transaction.transaction_type,
            asset: new_coinbase_transaction.asset,
            quantity_transacted: new_coinbase_transaction.quantity_transacted,
            spot_price_currency: new_coinbase_transaction.spot_price_currency,
            spot_price_at_transaction: new_coinbase_transaction.spot_price_at_transaction,
            subtotal: new_coinbase_transaction.subtotal,
            total: new_coinbase_transaction.total,
            fees: new_coinbase_transaction.fees,
            notes: new_coinbase_transaction.notes,
        }
    }

    #[test]
    fn insert_coinbase_data() {
        let ctx = create_test_context(TEST_DB_COUNTER.fetch_add(1, Ordering::SeqCst));

        let mut write_connection = ctx.create_connection();
        write_connection.run_pending_migrations(MIGRATIONS).unwrap();

        let pagination = Default::default();

        // Assure database is empty so we don't get a false positive match.
        let transactions = get_coinbase_transactions(&pagination, &mut write_connection).unwrap();
        assert_eq!(transactions.len(), 0);

        let time: DateTime<Utc> = DateTime::default();
        let new_coinbase_transaction = NewCoinbaseTransaction {
            time_of_transaction: time.clone(),
            transaction_type: "BUY".to_string(),
            asset: "ADA".to_string(),
            quantity_transacted: Decimal::new(2, 0),
            spot_price_currency: "USD".to_string(),
            spot_price_at_transaction: Some(Decimal::new(43, 2)),
            subtotal: Some(Decimal::new(86, 2)),
            total: Some(Decimal::new(86, 2)),
            fees: Some(Decimal::zero()),
            notes: "Bought 2 ADA at $0.43".to_string(),
        };

        let inserted =
            insert_coinbase_transaction(new_coinbase_transaction.clone(), &mut write_connection)
                .expect("Failed to insert coinbase transaction during test");

        let expected_coinbase_transaction =
            create_coinbase_transaction_from_new(new_coinbase_transaction, inserted.id);
        assert_eq!(&inserted, &expected_coinbase_transaction);

        let transactions = get_coinbase_transactions(&pagination, &mut write_connection).unwrap();
        assert_eq!(transactions.len(), 1);
        assert_eq!(
            transactions.first().unwrap(),
            &expected_coinbase_transaction
        );

        // Needs to be dropped before ctx. Since ctx deletes the database created here in this test.
        drop(write_connection);
    }
}
