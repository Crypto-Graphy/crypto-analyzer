mod common;

mod kraken_db_should {
    use chrono::DateTime;
    use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
    use uuid::Uuid;

    use crate::common::create_test_context;
    use crypto_database::kraken_db;
    use models_db::{KrakenTransaction, NewKrakenTransaction, Pagination};
    use rand::{self, Rng};
    use rust_decimal::Decimal;

    pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("./migrations");
    const KRAKEN_DB_NAME: &str = "kraken_test_database";

    #[test]
    fn insert_kraken() {
        let test_context = create_test_context(Some(KRAKEN_DB_NAME.to_owned()));
        let mut db_connection = test_context.create_connection();
        db_connection.run_pending_migrations(MIGRATIONS).unwrap();

        let pagination = Pagination::default();
        let results = kraken_db::get_kraken_transactions(&pagination, &mut db_connection).unwrap();
        assert_eq!(results.len(), 0);

        let kraken_transaction = create_random_kraken();

        let result =
            kraken_db::insert_kraken_transaction(kraken_transaction.clone(), &mut db_connection)
                .unwrap();
        let expected = create_kraken_transaction_from_new(kraken_transaction, result.id);
        assert_eq!(result, expected); // Tests the return is the same as the input + id assigned by the database.

        let kraken_transaction =
            kraken_db::get_kraken_transaction(result.id, &mut db_connection).unwrap();
        assert_eq!(kraken_transaction, expected);
    }

    #[test]
    fn bulk_insert_kraken() {
        let test_context = create_test_context(Some(KRAKEN_DB_NAME.to_owned()));
        let mut db_connection = test_context.create_connection();
        db_connection.run_pending_migrations(MIGRATIONS).unwrap();

        let pagination = Pagination::default();
        let results = kraken_db::get_kraken_transactions(&pagination, &mut db_connection).unwrap();
        assert_eq!(results.len(), 0);

        let kraken_transactions: Vec<NewKrakenTransaction> = (0..10)
            .into_iter()
            .map(|_| create_random_kraken())
            .collect();
        let results = kraken_db::bulk_insert_kraken_transaction(
            kraken_transactions.clone(),
            &mut db_connection,
        )
        .unwrap();
        assert!(results.len() > 0, "Bulk insert did not return a vec.");
        assert_eq!(kraken_transactions.len(), results.len());

        for i in 0..kraken_transactions.len() {
            let new_transaction = kraken_transactions.iter().nth(i).unwrap().clone();
            let result = results.iter().nth(i).unwrap().clone();
            let expected = create_kraken_transaction_from_new(new_transaction, result.id);
            assert_eq!(result, expected);
        }
    }

    #[test]
    fn get_transaction_by_id() {
        let test_context = create_test_context(Some(KRAKEN_DB_NAME.to_owned()));
        let mut db_connection = test_context.create_connection();
        db_connection.run_pending_migrations(MIGRATIONS).unwrap();

        let new_kraken_transaction = create_random_kraken();
        let inserted = kraken_db::insert_kraken_transaction(
            new_kraken_transaction.clone(),
            &mut db_connection,
        )
        .unwrap();
        let expected =
            create_kraken_transaction_from_new(new_kraken_transaction.clone(), inserted.id);
        assert_eq!(inserted, expected);

        let result = kraken_db::get_kraken_transaction(inserted.id, &mut db_connection).unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn get_multiple_transactions() {
        let test_context = create_test_context(Some(KRAKEN_DB_NAME.to_owned()));
        let mut db_connection = test_context.create_connection();
        db_connection.run_pending_migrations(MIGRATIONS).unwrap();

        let kraken_transactions: Vec<NewKrakenTransaction> = (0..15)
            .into_iter()
            .map(|_| create_random_kraken())
            .collect();
        let inserted_transactions = kraken_db::bulk_insert_kraken_transaction(
            kraken_transactions.clone(),
            &mut db_connection,
        )
        .unwrap();
        assert_eq!(kraken_transactions.len(), inserted_transactions.len());

        let pagination = Pagination {
            page: 0,
            items_per_page: kraken_transactions.len() as i64,
        };
        let results = kraken_db::get_kraken_transactions(&pagination, &mut db_connection).unwrap();
        assert_eq!(results.len() as i64, pagination.items_per_page);
        assert!(results.len() > 0);

        for i in 0..kraken_transactions.len() {
            let new_kraken_transaction = kraken_transactions.iter().nth(i).unwrap().clone();
            let inserted = inserted_transactions.iter().nth(i).unwrap().clone();
            let expected = create_kraken_transaction_from_new(new_kraken_transaction, inserted.id);
            let result = results.iter().nth(i).unwrap().clone();
            assert_eq!(inserted, expected);
            assert_eq!(result, expected);
        }
    }

    #[test]
    fn will_paginate() {
        let test_context = create_test_context(Some(KRAKEN_DB_NAME.to_owned()));
        let mut db_connection = test_context.create_connection();
        db_connection.run_pending_migrations(MIGRATIONS).unwrap();
        let mut pagination = Pagination {
            page: 0,
            items_per_page: 5,
        };

        let kraken_transactions: Vec<NewKrakenTransaction> = (0..10)
            .into_iter()
            .map(|_| create_random_kraken())
            .collect();
        let inserted_transactions = kraken_db::bulk_insert_kraken_transaction(
            kraken_transactions.clone(),
            &mut db_connection,
        )
        .unwrap();
        assert_eq!(kraken_transactions.len(), inserted_transactions.len());

        let page = kraken_db::get_kraken_transactions(&pagination, &mut db_connection).unwrap();
        assert_eq!(page.len() as i64, pagination.items_per_page);

        for i in 0..pagination.items_per_page as usize {
            let new_kraken_transaction = kraken_transactions.iter().nth(i).unwrap().clone();
            let inserted = inserted_transactions.iter().nth(i).unwrap().clone();
            let expected = create_kraken_transaction_from_new(new_kraken_transaction, inserted.id);
            let result = page.iter().nth(i).unwrap().clone();
            assert_eq!(result, expected);
        }

        pagination.page = 1;

        let page = kraken_db::get_kraken_transactions(&pagination, &mut db_connection).unwrap();
        assert_eq!(page.len() as i64, pagination.items_per_page);

        for i in 5..(pagination.items_per_page + 5) as usize {
            let new_kraken_transaction = kraken_transactions.iter().nth(i).unwrap().clone();
            let inserted = inserted_transactions.iter().nth(i).unwrap().clone();
            let expected = create_kraken_transaction_from_new(new_kraken_transaction, inserted.id);
            let result = page.iter().nth(i - 5).unwrap().clone();
            assert_eq!(result, expected);
        }
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
            asset,
            amount,
            fee,
            balance: Some(amount),
        }
    }

    fn create_kraken_transaction_from_new(
        new_kraken_transaction: NewKrakenTransaction,
        id: i32,
    ) -> KrakenTransaction {
        KrakenTransaction {
            id,
            txid: new_kraken_transaction.txid,
            refid: new_kraken_transaction.refid,
            transaction_time: new_kraken_transaction.transaction_time,
            record_type: new_kraken_transaction.record_type,
            subtype: new_kraken_transaction.subtype,
            a_class: new_kraken_transaction.a_class,
            asset: new_kraken_transaction.asset,
            amount: new_kraken_transaction.amount,
            fee: new_kraken_transaction.fee,
            balance: new_kraken_transaction.balance,
        }
    }
}
