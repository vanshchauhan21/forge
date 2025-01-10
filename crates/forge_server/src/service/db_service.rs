use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::sqlite::SqliteConnection;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use tracing::debug;

use super::Service;
use crate::Result;

type SQLConnection = Pool<ConnectionManager<SqliteConnection>>;

const DB_NAME: &str = "conversations.db";
const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

#[async_trait::async_trait]
pub trait DBService: Send + Sync {
    async fn pool(&self) -> Result<SQLConnection>;
}

impl Service {
    pub fn db_pool_service(db_path: &str) -> Result<impl DBService> {
        Live::new(db_path)
    }
}

struct Live {
    pool: SQLConnection,
}

impl Live {
    fn new(db_path: &str) -> Result<Self> {
        let db_path = format!("{}/{}", db_path, DB_NAME);

        // Run migrations first

        let mut conn = SqliteConnection::establish(&db_path)?;
        let migrations = conn.run_pending_migrations(MIGRATIONS)?;

        debug!(
            "Running {} migrations for database: {}",
            migrations.len(),
            db_path
        );

        drop(conn);

        // Create connection pool
        let manager = ConnectionManager::<SqliteConnection>::new(db_path);
        let pool = Pool::builder().build(manager)?;

        Ok(Live { pool })
    }
}

#[async_trait::async_trait]
impl DBService for Live {
    async fn pool(&self) -> Result<SQLConnection> {
        Ok(self.pool.clone())
    }
}

#[cfg(test)]
pub mod tests {
    use tempfile::TempDir;

    use super::*;

    pub struct TestDbPool {
        live: Live,
        // Keep TempDir alive for the duration of TestDbPool
        _temp_dir: TempDir,
    }

    impl TestDbPool {
        pub fn new() -> Result<Self> {
            let temp_dir = TempDir::new().unwrap();
            let db_path = temp_dir.path().to_str().unwrap().to_string();

            Ok(Self { live: Live::new(&db_path)?, _temp_dir: temp_dir })
        }
    }

    #[async_trait::async_trait]
    impl DBService for TestDbPool {
        async fn pool(&self) -> Result<SQLConnection> {
            Ok(self.live.pool.clone())
        }
    }
}
