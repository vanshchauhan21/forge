use anyhow::{Context, Result};
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::sqlite::SqliteConnection;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use tracing::debug;

use super::conn::ConnectionOptions;

pub(crate) type SQLConnection = Pool<ConnectionManager<SqliteConnection>>;

const DB_NAME: &str = ".forge.db";
const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

/// SQLite driver that manages database connections and migrations
#[derive(Debug)]
pub(crate) struct Driver {
    pool: SQLConnection,
}

impl Driver {
    pub fn new(db_path: &str, timeout: Option<std::time::Duration>) -> Result<Self> {
        let db_path = format!("{}/{}", db_path, DB_NAME);

        // Run migrations first
        let mut conn = SqliteConnection::establish(&db_path)?;
        let migrations = conn
            .run_pending_migrations(MIGRATIONS)
            .map_err(|e| anyhow::anyhow!(e))
            .with_context(|| "Failed to run database migrations")?;

        debug!(
            "Running {} migrations for database: {}",
            migrations.len(),
            db_path
        );

        drop(conn);

        // Create connection pool
        let manager = ConnectionManager::<SqliteConnection>::new(db_path);
        let options = match timeout {
            Some(timeout) => ConnectionOptions::new(timeout),
            None => ConnectionOptions::default(),
        };

        let pool = Pool::builder()
            .connection_customizer(Box::new(options))
            .max_size(1) // SQLite works better with a single connection
            .build(manager)?;

        Ok(Driver { pool })
    }

    pub fn pool(&self) -> SQLConnection {
        self.pool.clone()
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use tempfile::TempDir;

    use super::*;

    pub struct TestDriver {
        driver: Driver,
        // Keep TempDir alive for the duration of the test
        _temp_dir: TempDir,
    }

    impl TestDriver {
        pub fn new() -> Result<Self> {
            let temp_dir = TempDir::new().unwrap();
            let db_path = temp_dir.path().to_str().unwrap().to_string();

            Ok(Self { driver: Driver::new(&db_path, None)?, _temp_dir: temp_dir })
        }

        pub fn with_timeout(timeout: std::time::Duration) -> Result<Self> {
            let temp_dir = TempDir::new().unwrap();
            let db_path = temp_dir.path().to_str().unwrap().to_string();

            Ok(Self {
                driver: Driver::new(&db_path, Some(timeout))?,
                _temp_dir: temp_dir,
            })
        }

        pub fn pool(&self) -> SQLConnection {
            self.driver.pool()
        }
    }

    #[tokio::test]
    async fn test_custom_timeout() -> Result<()> {
        let driver = TestDriver::with_timeout(std::time::Duration::from_secs(60))?;
        let pool = driver.pool();
        assert!(pool.get().is_ok());
        Ok(())
    }
}
