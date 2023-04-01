mod coinbase_db_should {
    use diesel::prelude::*;

    use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
    pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("./migrations");
    pub const TEST_MIGRATIONS: EmbeddedMigrations = embed_migrations!("./tests/test_migrations");

    struct Config {
        pub host: String,
        pub port: String,
        pub user: String,
        pub password: String,
        pub db_name: String,
    }

    impl Config {
        fn create_db_url(&self) -> String {
            format!(
                "postgres://{}:{}@{}:{}/{}",
                &self.host, &self.port, &self.user, &self.password, &self.db_name
            )
        }

        fn db_information(&self) -> (String, &String, &String, &String) {
            (self.create_db_url(), &self.host, &self.port, &self.db_name)
        }
    }

    impl Default for Config {
        fn default() -> Self {
            Self {
                host: "0.0.0.0".to_string(),
                port: "5433".to_string(),
                user: "super_user".to_string(),
                password: "password".to_string(),
                db_name: "crypto_database".to_string(),
            }
        }
    }

    // fn set_connection_env_vars(
    //     host: Option<String>,
    //     port: Option<String>,
    //     user: Option<String>,
    //     password: Option<String>,
    //     db_name: Option<String>,
    // ) {
    //     std::env::set_var("DB_HOST", host.unwrap_or("0.0.0.0".to_string()));
    //     std::env::set_var("DB_PORT", port.unwrap_or("5431".to_string()));
    //     std::env::set_var("DB_USER", user.unwrap_or("super_user".to_string()));
    //     std::env::set_var("DB_PASSWORD", password.unwrap_or("password".to_string()));
    //     std::env::set_var("DB_NAME", db_name.unwrap_or("crypto_database".to_string()));
    // }

    fn set_env_from_config(config: &Config) {
        std::env::set_var("DB_HOST", &config.host);
        std::env::set_var("DB_PORT", &config.port);
        std::env::set_var("DB_USER", &config.user);
        std::env::set_var("DB_PASSWORD", &config.password);
        std::env::set_var("DB_NAME", &config.db_name);
    }

    #[test]
    fn insert_coinbase_data() {
        let config = Config {
            port: "5431".to_string(),
            ..Default::default()
        };

        set_env_from_config(&config);

        let mut connection = PgConnection::establish(&crypto_database::get_connection_string().0)
            .expect("Failed to get db connection");
        connection.run_pending_migrations(MIGRATIONS).unwrap();
        connection.run_pending_migrations(TEST_MIGRATIONS).unwrap();
    }
}
