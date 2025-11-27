use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncState {
    pub id: Uuid,
    pub account_id: Uuid,
    pub folder_id: Option<Uuid>,
    pub last_sync_at: Option<DateTime<Utc>>,
    pub next_sync_at: Option<DateTime<Utc>>,
    pub last_uid: Option<i64>,
    pub sync_token: Option<String>,
    pub sync_status: String,
    pub error_message: Option<String>,
    pub error_count: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl SyncState {
    pub fn new(account_id: Uuid, folder_id: Option<Uuid>) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::now_v7(),
            account_id,
            folder_id,
            last_sync_at: None,
            next_sync_at: None,
            last_uid: None,
            sync_token: None,
            sync_status: "idle".to_string(),
            error_message: None,
            error_count: 0,
            created_at: now,
            updated_at: now,
        }
    }
}

impl sqlx::FromRow<'_, sqlx::sqlite::SqliteRow> for SyncState {
    fn from_row(row: &sqlx::sqlite::SqliteRow) -> Result<Self, sqlx::Error> {
        use sqlx::Row;

        let id_str: String = row.try_get("id")?;
        let id = Uuid::parse_str(&id_str).map_err(|e| sqlx::Error::ColumnDecode {
            index: "id".into(),
            source: Box::new(e),
        })?;

        let account_id_str: String = row.try_get("account_id")?;
        let account_id =
            Uuid::parse_str(&account_id_str).map_err(|e| sqlx::Error::ColumnDecode {
                index: "account_id".into(),
                source: Box::new(e),
            })?;

        let folder_id_opt: Option<String> = row.try_get("folder_id")?;
        let folder_id = match folder_id_opt {
            Some(fid) => Some(
                Uuid::parse_str(&fid).map_err(|e| sqlx::Error::ColumnDecode {
                    index: "folder_id".into(),
                    source: Box::new(e),
                })?,
            ),
            None => None,
        };

        Ok(SyncState {
            id,
            account_id,
            folder_id,
            last_sync_at: row.try_get("last_sync_at")?,
            next_sync_at: row.try_get("next_sync_at")?,
            last_uid: row.try_get("last_uid")?,
            sync_token: row.try_get("sync_token")?,
            sync_status: row.try_get("sync_status")?,
            error_message: row.try_get("error_message")?,
            error_count: row.try_get("error_count")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        })
    }
}
