use async_trait::async_trait;
use chrono::Utc;
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::database::{error::DatabaseError, models::sync_state::SyncState};

#[async_trait]
pub trait SyncStateRepository {
    async fn find_by_account_and_folder(
        &self,
        account_id: Uuid,
        folder_id: Uuid,
    ) -> Result<Option<SyncState>, DatabaseError>;
    async fn upsert(
        &self,
        account_id: Uuid,
        folder_id: Uuid,
        status: &str,
        error_message: Option<&str>,
    ) -> Result<(), DatabaseError>;
}

pub struct SqliteSyncStateRepository {
    pool: SqlitePool,
}

impl SqliteSyncStateRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl SyncStateRepository for SqliteSyncStateRepository {
    async fn find_by_account_and_folder(
        &self,
        account_id: Uuid,
        folder_id: Uuid,
    ) -> Result<Option<SyncState>, DatabaseError> {
        let account_id_str = account_id.to_string();
        let folder_id_str = folder_id.to_string();

        sqlx::query_as::<_, SyncState>(
            "SELECT * FROM sync_state WHERE account_id = ? AND folder_id = ?",
        )
        .bind(account_id_str)
        .bind(folder_id_str)
        .fetch_optional(&self.pool)
        .await
        .map_err(DatabaseError::ConnectionError)
    }

    async fn upsert(
        &self,
        account_id: Uuid,
        folder_id: Uuid,
        status: &str,
        error_message: Option<&str>,
    ) -> Result<(), DatabaseError> {
        let id = Uuid::now_v7().to_string();
        let now = Utc::now();
        let account_id_str = account_id.to_string();
        let folder_id_str = folder_id.to_string();

        sqlx::query!(
            r#"
            INSERT INTO sync_state (id, account_id, folder_id, sync_status, error_message, last_sync_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(account_id, folder_id) DO UPDATE SET
                sync_status = excluded.sync_status,
                error_message = excluded.error_message,
                last_sync_at = excluded.last_sync_at,
                updated_at = excluded.updated_at,
                error_count = CASE WHEN excluded.sync_status = 'error' THEN error_count + 1 ELSE 0 END
            "#,
            id,
            account_id_str,
            folder_id_str,
            status,
            error_message,
            now,
            now
        )
        .execute(&self.pool)
        .await
        .map_err(DatabaseError::ConnectionError)?;

        Ok(())
    }
}
