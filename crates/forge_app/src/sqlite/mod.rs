mod conn;
mod driver;

use anyhow::Result;

use crate::Service;

#[async_trait::async_trait]
pub trait Sqlite: Send + Sync {
    async fn pool(&self) -> Result<driver::SQLConnection>;
}

struct Live {
    driver: driver::Driver,
}

impl Live {
    fn new(db_path: &str, timeout: Option<std::time::Duration>) -> Result<Self> {
        Ok(Self { driver: driver::Driver::new(db_path, timeout)? })
    }
}

#[async_trait::async_trait]
impl Sqlite for Live {
    async fn pool(&self) -> Result<driver::SQLConnection> {
        Ok(self.driver.pool())
    }
}

impl Service {
    /// Create a new SQLite pool service with default timeout (30 seconds)
    pub fn db_pool_service(db_path: &str) -> Result<impl Sqlite> {
        Live::new(db_path, None)
    }

    /// Create a new SQLite pool service with custom timeout
    pub fn db_pool_service_with_timeout(
        db_path: &str,
        timeout: std::time::Duration,
    ) -> Result<impl Sqlite> {
        Live::new(db_path, Some(timeout))
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    pub struct TestSqlite {
        test_driver: driver::tests::TestDriver,
    }

    impl TestSqlite {
        pub fn new() -> Result<Self> {
            Ok(Self { test_driver: driver::tests::TestDriver::new()? })
        }

        pub fn with_timeout(timeout: std::time::Duration) -> Result<Self> {
            Ok(Self {
                test_driver: driver::tests::TestDriver::with_timeout(timeout)?,
            })
        }
    }

    #[async_trait::async_trait]
    impl Sqlite for TestSqlite {
        async fn pool(&self) -> Result<driver::SQLConnection> {
            Ok(self.test_driver.pool())
        }
    }

    #[tokio::test]
    async fn test_custom_timeout() -> Result<()> {
        let sqlite = TestSqlite::with_timeout(std::time::Duration::from_secs(60))?;
        let pool = sqlite.pool().await?;
        assert!(pool.get().is_ok());
        Ok(())
    }
}
