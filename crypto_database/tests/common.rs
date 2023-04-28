use std::sync::atomic::{AtomicU8, Ordering};

use diesel::{sql_query, Connection, PgConnection, RunQueryDsl};
use models_db::{DBConfig, DBConfigOptions};

pub static TEST_DB_COUNTER: AtomicU8 = AtomicU8::new(0);

pub struct TestContext {
    pub config: DBConfig,
    connection: PgConnection,
}

pub fn create_test_context(base_db_name: Option<String>) -> TestContext {
    let config = DBConfig::new(Some(DBConfigOptions {
        database_name: Some(format!(
            "{}_{}",
            base_db_name.unwrap_or("test_database".to_string()),
            TEST_DB_COUNTER.fetch_add(1, Ordering::SeqCst)
        )),
        ..Default::default()
    }));

    TestContext::new(
        config,
        crypto_database::establish_connection(None)
            .expect("Failed to establish a connection to the database"),
    )
}

impl TestContext {
    pub fn new(config: DBConfig, connection: PgConnection) -> Self {
        // TODO: Establish connection and create database.
        let mut ctx = Self { config, connection };
        ctx.setup();

        ctx
    }

    pub fn setup(&mut self) {
        sql_query(format!(
            "CREATE DATABASE {};",
            self.config.get_database_name()
        ))
        .execute(&mut self.connection)
        .unwrap();
    }

    pub fn create_connection(&self) -> PgConnection {
        PgConnection::establish(&self.config.connection_string())
            .expect("Failed to establish a connection with the database")
    }
}

impl Drop for TestContext {
    fn drop(&mut self) {
        sql_query(format!("DROP DATABASE {}", self.config.get_database_name()))
            .execute(&mut self.connection)
            .unwrap();
    }
}
