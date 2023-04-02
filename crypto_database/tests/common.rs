use std::sync::atomic::AtomicU8;

pub struct Config {
    pub host: String,
    pub port: String,
    pub user: String,
    pub password: String,
    pub db_name: String,
}

impl Config {
    pub fn create_db_url(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            &self.host, &self.port, &self.user, &self.password, &self.db_name
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
            db_name: "crypto_database".to_string(),
        }
    }
}

pub struct TestContext {
    pub config: Config,
}

impl TestContext {
    pub fn new(config: Config) -> Self {
        // TODO: Establish connection and create database.
        Self { config }
    }
}

impl Drop for TestContext {
    fn drop(&mut self) {}
}
