use anyhow::Context;
use chrono::{NaiveDateTime, Utc};
use diesel::dsl::max;
use diesel::prelude::*;
use diesel::sql_types::{Text, Timestamp};
use forge_domain::{Config, ConfigRepository};
use serde::{Deserialize, Serialize};

use crate::schema::configuration_table::{self};
use crate::service::Service;
use crate::sqlite::Sqlite;

#[derive(Debug, Serialize, Deserialize)]
struct ConfigId(String);

impl std::fmt::Display for ConfigId {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl ConfigId {
    pub fn generate() -> Self {
        Self(uuid::Uuid::new_v4().to_string())
    }
}

#[derive(Debug, Insertable, Queryable, QueryableByName)]
#[diesel(table_name = configuration_table)]
struct ConfigEntity {
    #[diesel(sql_type = Text)]
    id: String,
    #[diesel(sql_type = Text)]
    data: String,
    #[diesel(sql_type = Timestamp)]
    created_at: NaiveDateTime,
}

impl TryFrom<ConfigEntity> for Config {
    type Error = anyhow::Error;

    fn try_from(raw: ConfigEntity) -> Result<Self, Self::Error> {
        // TODO: currently we don't need id and created_at.
        serde_json::from_str(&raw.data).with_context(|| "failed to load configuration from store")
    }
}

pub struct Live<P> {
    pool_service: P,
}

impl<P: Sqlite> Live<P> {
    pub fn new(pool_service: P) -> Self {
        Self { pool_service }
    }
}

#[async_trait::async_trait]
impl<P: Sqlite + Send + Sync> ConfigRepository for Live<P> {
    async fn get(&self) -> anyhow::Result<Config> {
        let pool = self
            .pool_service
            .pool()
            .await
            .context("Failed to get database pool")?;
        let mut conn = pool
            .get()
            .context("Failed to get database connection from pool")?;

        // get the max timestamp.
        let max_ts: Option<NaiveDateTime> = configuration_table::table
            .select(max(configuration_table::created_at))
            .first(&mut conn)?;

        // use the max timestamp to get the latest config.
        let result: ConfigEntity = configuration_table::table
            .filter(configuration_table::created_at.eq_any(max_ts))
            .limit(1)
            .first(&mut conn)?;

        Ok(Config::try_from(result)?)
    }

    async fn set(&self, data: Config) -> anyhow::Result<Config> {
        let pool: r2d2::Pool<diesel::r2d2::ConnectionManager<SqliteConnection>> =
            self.pool_service.pool().await?;
        let mut conn = pool.get()?;
        let now = Utc::now().naive_utc();

        let raw = ConfigEntity {
            id: ConfigId::generate().to_string(),
            data: serde_json::to_string(&data)?,
            created_at: now,
        };

        diesel::insert_into(configuration_table::table)
            .values(&raw)
            .execute(&mut conn)
            .with_context(|| format!("Failed to save configuration with id: {}", raw.id))?;

        self.get().await
    }
}

impl Service {
    pub fn config_service(database_url: &str) -> anyhow::Result<impl ConfigRepository> {
        Ok(Live::new(Service::db_pool_service(database_url)?))
    }
}

#[cfg(test)]
pub mod tests {
    use forge_domain::{ApiKey, ModelConfig, ModelId, Permissions, ProviderId};

    use super::*;
    use crate::sqlite::tests::TestSqlite;

    pub struct TestConfigStorage;

    impl TestConfigStorage {
        pub fn in_memory() -> anyhow::Result<impl ConfigRepository> {
            let pool_service = TestSqlite::new()?;
            Ok(Live::new(pool_service))
        }
    }

    async fn setup_storage() -> anyhow::Result<impl ConfigRepository> {
        TestConfigStorage::in_memory()
    }

    fn test_config() -> Config {
        Config {
            primary_model: ModelConfig {
                provider_id: ProviderId::new("anthrophic"),
                model_id: ModelId::new("o4"),
                api_key: Some(ApiKey::new("abc-efg")),
            },
            secondary_model: ModelConfig {
                provider_id: ProviderId::new("open-ai"),
                model_id: ModelId::new("o4-mini"),
                api_key: Some(ApiKey::new("abc-efg")),
            },
            permissions: Permissions {
                read: true,
                edit: true,
                commands: true,
                browser: true,
                mcp: true,
            },
            max_requests: 12,
            notifications: true,
        }
    }

    #[tokio::test]
    async fn test_config_can_be_stored_and_retrieved() -> anyhow::Result<()> {
        let storage = setup_storage().await?;
        let config = test_config();

        let result = storage.set(config.clone()).await?;
        let latest_config = storage.get().await?;
        assert_eq!(result, latest_config);
        Ok(())
    }

    #[tokio::test]
    async fn test_always_get_latest_config() -> anyhow::Result<()> {
        let storage = setup_storage().await?;
        let mut config = test_config();

        let result = storage.set(config.clone()).await?;
        let latest_config = storage.get().await?;
        assert_eq!(result, latest_config);

        config.primary_model.model_id = ModelId::new("o4-mini");
        // should alaways get the latest config
        let result = storage.set(config.clone()).await?;
        let latest_config = storage.get().await?;
        assert_eq!(result, latest_config);
        Ok(())
    }
}
