mod common;

mod coinbase_db_should {
    use diesel::prelude::*;
    use uuid::Uuid;

    use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

    use crate::common::Config;

    // use crate::common;
    pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("./migrations");

    fn set_env_from_config(config: &Config) {
        std::env::set_var("DB_HOST", &config.host);
        std::env::set_var("DB_PORT", &config.port);
        std::env::set_var("DB_USER", &config.user);
        std::env::set_var("DB_PASSWORD", &config.password);
        std::env::set_var("DB_NAME", &config.db_name);
    }

    #[test]
    fn insert_coinbase_data() {
        let db_name = Uuid::new_v4(); // Use AtomicU8?
        let config = Config {
            db_name: format!("test_database_{}", db_name),
            ..Default::default()
        };

        set_env_from_config(&config);

        let mut connection = PgConnection::establish(&crypto_database::get_connection_string().0)
            .expect("Failed to get db connection");
        connection.run_pending_migrations(MIGRATIONS).unwrap();
    }
}
