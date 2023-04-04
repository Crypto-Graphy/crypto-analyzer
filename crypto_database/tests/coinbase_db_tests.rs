mod common;

mod coinbase_db_should {
    use rand::{self, Rng};
    use std::sync::atomic::Ordering;

    use chrono::{DateTime, Utc};
    use crypto_database::{
        get_coinbase_transactions, insert_coinbase_transaction,
        models::{CoinbaseTransaction, NewCoinbaseTransaction, Pagination},
        schema::coinbase_transactions,
    };
    use diesel::prelude::*;
    use rust_decimal::Decimal;

    use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

    use crate::common::{Config, TestContext, TEST_DB_COUNTER};

    // use crate::common;
    pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("./migrations");

    fn create_random_new_coinbase_transaction() -> NewCoinbaseTransaction {
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

        NewCoinbaseTransaction {
            time_of_transaction,
            transaction_type: "BUY".to_string(),
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
            id,
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

        let pagination = Pagination::default();

        // Assure database is empty so we don't get a false positive match.
        let transactions = get_coinbase_transactions(&pagination, &mut write_connection).unwrap();
        assert_eq!(transactions.len(), 0);

        let new_coinbase_transaction = create_random_new_coinbase_transaction();

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

        // Needs to be dropped before ctx. Since ctx deletes the database connected here in this test.
        drop(write_connection);
    }

    #[test]
    fn retrieve_existing_coinbase_transactions() {
        let ctx = create_test_context(TEST_DB_COUNTER.fetch_add(1, Ordering::SeqCst));

        let mut test_connection = ctx.create_connection();
        test_connection.run_pending_migrations(MIGRATIONS).unwrap();

        // Create transactions to be found.
        let transactions_to_add = vec![
            create_random_new_coinbase_transaction(),
            create_random_new_coinbase_transaction(),
            create_random_new_coinbase_transaction(),
            create_random_new_coinbase_transaction(),
            create_random_new_coinbase_transaction(),
        ];

        // Add the transactions above directly to the database, not using crypto_database.
        let inserted_transactions = diesel::insert_into(coinbase_transactions::table)
            .values(&transactions_to_add)
            .get_results::<CoinbaseTransaction>(&mut test_connection)
            .unwrap();

        // Get the transactions from the database using crypto_database
        let pagination = Pagination::default();
        let coinbase_transactions =
            crypto_database::get_coinbase_transactions(&pagination, &mut test_connection).unwrap();

        // Loop though inserted transactions and transactions to add to contruct the expected coinbase_transaction object
        // The id isn't known until it is inserted into the database
        let mut expected_transactions = Vec::new();
        for i in 0..transactions_to_add.len() {
            let (transaction_to_add, inserted) = (
                transactions_to_add.get(i).unwrap(),
                inserted_transactions.get(i).unwrap(),
            );

            let expected =
                create_coinbase_transaction_from_new(transaction_to_add.clone(), inserted.id);
            expected_transactions.push(expected);
        }

        for i in 0..coinbase_transactions.len() {
            assert_eq!(coinbase_transactions.get(i), expected_transactions.get(i));
        }
    }

    #[test]
    fn page_retrieved_transactions() {
        let ctx = create_test_context(TEST_DB_COUNTER.fetch_add(1, Ordering::SeqCst));

        let mut test_connection = ctx.create_connection();
        test_connection.run_pending_migrations(MIGRATIONS).unwrap();

        // Create transactions to be found.
        let transactions_to_add: Vec<NewCoinbaseTransaction> = (0..15)
            .map(|_| create_random_new_coinbase_transaction())
            .collect();

        // Add the transactions above directly to the database, not using crypto_database.
        let inserted_transactions = diesel::insert_into(coinbase_transactions::table)
            .values(&transactions_to_add)
            .get_results::<CoinbaseTransaction>(&mut test_connection)
            .unwrap();
        assert_eq!(transactions_to_add.len(), inserted_transactions.len());

        // Loop though inserted transactions and transactions to add to contruct the expected coinbase_transaction object
        // The id isn't known until it is inserted into the database

        // First page
        {
            // Get the transactions from the database using crypto_database
            let pagination = Pagination::default();
            let coinbase_transactions =
                crypto_database::get_coinbase_transactions(&pagination, &mut test_connection)
                    .unwrap();

            assert_eq!(
                coinbase_transactions.len(),
                pagination.items_per_page as usize
            );
            assert_ne!(coinbase_transactions.len(), 0);

            for i in 0..coinbase_transactions.len() {
                let transaction_to_add = transactions_to_add.get(i).unwrap();
                let inserted_transaction = inserted_transactions.get(i).unwrap();

                let actual = coinbase_transactions.get(i).unwrap();
                let expected = create_coinbase_transaction_from_new(
                    transaction_to_add.clone(),
                    inserted_transaction.id,
                );
                assert_eq!(actual, &expected);
            }
        }

        // Second page
        {
            // Paginate to the next page of the list.
            let pagination = Pagination {
                page: 1,
                items_per_page: 10,
            };
            let coinbase_transactions =
                crypto_database::get_coinbase_transactions(&pagination, &mut test_connection)
                    .unwrap();

            assert_eq!(coinbase_transactions.len(), 5);

            for i in 0..coinbase_transactions.len() {
                let inserted = inserted_transactions
                    .get(i + (pagination.page * pagination.items_per_page) as usize)
                    .unwrap();
                let transaction_to_add = transactions_to_add
                    .get(i + (pagination.page * pagination.items_per_page) as usize)
                    .unwrap();
                let expected =
                    create_coinbase_transaction_from_new(transaction_to_add.clone(), inserted.id);

                let actual = coinbase_transactions.get(i).unwrap();
                assert_eq!(actual, &expected);
            }
        }
    }
}
