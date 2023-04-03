use std::sync::atomic::AtomicU8;

use diesel::{sql_query, Connection, PgConnection, RunQueryDsl};

pub static TEST_DB_COUNTER: AtomicU8 = AtomicU8::new(0);

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Config {
    pub host: String,
    pub port: String,
    pub user: String,
    pub password: String,
    pub db_name: String,
}

impl Config {
    pub fn create_db_url(&self) -> String {
        format!("{}/{}", self.create_db_url_no_db_name(), &self.db_name)
    }

    pub fn create_db_url_no_db_name(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}",
            &self.user, &self.password, &self.host, &self.port,
        )
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".to_string(),
            port: "5432".to_string(),
            user: "super_user".to_string(),
            password: "password".to_string(),
            db_name: "crypto_data".to_string(),
        }
    }
}

pub struct TestContext {
    pub config: Config,
    connection: PgConnection,
}

impl TestContext {
    pub fn new(config: Config, connection: PgConnection) -> Self {
        // TODO: Establish connection and create database.
        let mut ctx = Self { config, connection };
        ctx.setup();

        ctx
    }

    pub fn setup(&mut self) {
        sql_query(format!("CREATE DATABASE {};", &self.config.db_name))
            .execute(&mut self.connection)
            .unwrap();
    }

    // pub fn tear_down(&self, connection: &mut PgConnection) {
    //     sql_query(format!("DROP DATABASE {}", &self.config.db_name))
    //         .execute(connection)
    //         .unwrap();
    // }

    pub fn create_connection(&self) -> PgConnection {
        PgConnection::establish(&self.config.create_db_url())
            .expect("Failed to establish a connection with the database")
    }

    // pub fn get_pure_connection(&self) -> PgConnection {
    //     PgConnection::establish(&self.config.create_db_url())
    //         .expect("Failed to establish a connection with the database")
    // }

    pub fn set_env_from_config(&self) {
        std::env::set_var("DB_HOST", &self.config.host);
        std::env::set_var("DB_PORT", &self.config.port);
        std::env::set_var("DB_USER", &self.config.user);
        std::env::set_var("DB_PASSWORD", &self.config.password);
        std::env::set_var("DB_NAME", &self.config.db_name);
    }
}

impl Drop for TestContext {
    fn drop(&mut self) {
        sql_query(format!("DROP DATABASE {}", &self.config.db_name))
            .execute(&mut self.connection)
            .unwrap();
    }
}

// Tests for Config (a test only struct)
// Easier to debug against tests than to create a runnable program to debug against.
mod config_should {
    use super::Config;

    #[test]
    fn create_postgres_url_with_no_name() {
        let host = "test-host".to_string();
        let port = "test-port".to_string();
        let user = "test-user".to_string();
        let password = "test-password".to_string();
        let db_name = "test-db-name".to_string();

        let config = Config {
            host: host.clone(),
            port: port.clone(),
            user: user.clone(),
            password: password.clone(),
            db_name: db_name.clone(),
        };
        let expected = format!("postgres://{}:{}@{}:{}", user, password, host, port);

        let url = config.create_db_url_no_db_name();

        assert_eq!(url, expected);
    }

    #[test]
    fn create_postgres_url() {
        let host = "test-host".to_string();
        let port = "test-port".to_string();
        let user = "test-user".to_string();
        let password = "test-password".to_string();
        let db_name = "test-db-name".to_string();

        let config = Config {
            host: host.clone(),
            port: port.clone(),
            user: user.clone(),
            password: password.clone(),
            db_name: db_name.clone(),
        };
        let expected = format!(
            "postgres://{}:{}@{}:{}/{}",
            user, password, host, port, db_name
        );

        let url = config.create_db_url();

        assert_eq!(url, expected);
    }
}
