use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attachment {
    pub id: Uuid,
    pub email_id: Uuid,
    pub filename: String,
    pub content_type: String,
    pub size: i64,
    pub hash: String,
    pub cache_path: Option<String>,
    pub is_inline: bool,
    pub is_cached: bool,
    pub content_id: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl sqlx::FromRow<'_, sqlx::sqlite::SqliteRow> for Attachment {
    fn from_row(row: &sqlx::sqlite::SqliteRow) -> Result<Self, sqlx::Error> {
        use sqlx::Row;

        let id_str: String = row.try_get("id")?;
        let id = Uuid::parse_str(&id_str).map_err(|e| sqlx::Error::Decode(Box::new(e)))?;

        let email_id_str: String = row.try_get("email_id")?;
        let email_id =
            Uuid::parse_str(&email_id_str).map_err(|e| sqlx::Error::Decode(Box::new(e)))?;

        Ok(Attachment {
            id,
            email_id,
            filename: row.try_get("filename")?,
            content_type: row.try_get("content_type")?,
            size: row.try_get("size")?,
            hash: row.try_get("hash")?,
            cache_path: row.try_get("cache_path")?,
            is_inline: row.try_get("is_inline")?,
            is_cached: row.try_get("is_cached")?,
            content_id: row.try_get("content_id")?,
            created_at: row.try_get("created_at")?,
        })
    }
}
