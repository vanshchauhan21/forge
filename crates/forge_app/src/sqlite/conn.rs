use std::time::Duration;

use diesel::r2d2;
use diesel::sqlite::SqliteConnection;

/// Options for customizing SQLite connections
#[derive(Debug, Clone)]
pub(crate) struct ConnectionOptions {
    pub(crate) busy_timeout: Duration,
    pub(crate) max_connections: u32,
    pub(crate) connection_timeout: Duration,
}

impl ConnectionOptions {
    pub fn new(busy_timeout: Duration, max_connections: u32, connection_timeout: Duration) -> Self {
        Self { busy_timeout, max_connections, connection_timeout }
    }

    pub fn default() -> Self {
        Self::new(
            Duration::from_secs(30), // busy timeout
            5,                       // max connections
            Duration::from_secs(30), // connection timeout
        )
    }
}

impl r2d2::CustomizeConnection<SqliteConnection, diesel::r2d2::Error> for ConnectionOptions {
    fn on_acquire(&self, conn: &mut SqliteConnection) -> Result<(), diesel::r2d2::Error> {
        use diesel::connection::SimpleConnection;

        conn.batch_execute(&format!(
            "PRAGMA busy_timeout = {}; 
             PRAGMA journal_mode = WAL;
             PRAGMA synchronous = NORMAL;
             PRAGMA foreign_keys = ON;
             PRAGMA temp_store = MEMORY;
             PRAGMA cache_size = -2000;", // 2MB cache
            self.busy_timeout.as_millis()
        ))
        .map_err(diesel::r2d2::Error::QueryError)?;

        Ok(())
    }
}
