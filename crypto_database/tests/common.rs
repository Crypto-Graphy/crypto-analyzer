use std::sync::atomic::AtomicU8;

use diesel::{sql_query, Connection, PgConnection, RunQueryDsl};
use models_db::DBConfig;

pub static TEST_DB_COUNTER: AtomicU8 = AtomicU8::new(0);

pub struct TestContext {
    pub config: DBConfig,
    connection: PgConnection,
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
