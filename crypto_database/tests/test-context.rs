pub struct TestContext {
    pub database_name: String,
}

impl TestContext {
    pub fn new(database_name: String) -> Self {
        Self { database_name }
    }
}

impl Drop for TestContext {
    fn drop(&mut self) {}
}
