use std::time::Duration;

use diesel::r2d2;
use diesel::sqlite::SqliteConnection;

/// Options for customizing SQLite connections
#[derive(Debug)]
pub(crate) struct ConnectionOptions {
    busy_timeout: Duration,
}

impl ConnectionOptions {
    pub fn new(busy_timeout: Duration) -> Self {
        Self { busy_timeout }
    }

    pub fn default() -> Self {
        Self::new(Duration::from_secs(30))
    }
}

impl r2d2::CustomizeConnection<SqliteConnection, diesel::r2d2::Error> for ConnectionOptions {
    fn on_acquire(&self, conn: &mut SqliteConnection) -> Result<(), diesel::r2d2::Error> {
        use diesel::connection::SimpleConnection;

        conn.batch_execute(&format!(
            "PRAGMA busy_timeout = {}; 
             PRAGMA journal_mode = WAL;
             PRAGMA synchronous = NORMAL;",
            self.busy_timeout.as_millis()
        ))
        .map_err(diesel::r2d2::Error::QueryError)?;

        Ok(())
    }
}
