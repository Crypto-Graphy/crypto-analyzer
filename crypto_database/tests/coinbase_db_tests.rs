mod coinbase_db_should {
    use diesel::{migration::MigrationConnection, prelude::*};

    use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
    pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("./migrations");
    pub const TEST_MIGRATIONS: EmbeddedMigrations = embed_migrations!("./tests/test_migrations");

    #[test]
    fn insert_coinbase_data() {
        let db = format!("coinbase_insert_test");
        std::env::set_var("DB_NAME", &db);

        let mut connection = PgConnection::establish(&crypto_database::get_connection_string().0)
            .expect("Failed to get db connection");
        connection.run_pending_migrations(MIGRATIONS).unwrap();
        connection.run_pending_migrations(TEST_MIGRATIONS).unwrap();
    }
}
