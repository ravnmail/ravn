// File: /src/database/models/contact.rs
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contact {
    pub id: Uuid,
    pub display_name: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub company: Option<String>,
    pub email: String,
    pub source: String,      // 'observed', 'imported', 'manual'
    pub avatar_type: String, // 'gravatar', 'unavatar', 'favicon', 'none'
    pub avatar_path: Option<String>,
    pub send_count: i64,
    pub receive_count: i64,
    pub last_used_at: Option<DateTime<Utc>>,
    pub first_seen_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl sqlx::FromRow<'_, sqlx::sqlite::SqliteRow> for Contact {
    fn from_row(row: &sqlx::sqlite::SqliteRow) -> Result<Self, sqlx::Error> {
        use sqlx::Row;

        let id_str: String = row.try_get("id")?;
        let id = Uuid::parse_str(&id_str).map_err(|e| sqlx::Error::Decode(Box::new(e)))?;

        Ok(Contact {
            id,
            display_name: row.try_get("display_name")?,
            first_name: row.try_get("first_name")?,
            last_name: row.try_get("last_name")?,
            company: row.try_get("company")?,
            email: row.try_get("email")?,
            source: row.try_get("source")?,
            avatar_type: row.try_get("avatar_type")?,
            avatar_path: row.try_get("avatar_path")?,
            send_count: row.try_get("send_count")?,
            receive_count: row.try_get("receive_count")?,
            last_used_at: row.try_get("last_used_at")?,
            first_seen_at: row.try_get("first_seen_at")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        })
    }
}

// Contact summary for UI display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContactSummary {
    pub id: Uuid,
    pub email: String,
    pub display_name: Option<String>,
    pub avatar_path: Option<String>,
    pub send_count: i64,
    pub receive_count: i64,
    pub last_used_at: Option<DateTime<Utc>>,
    pub usage_score: i64,
}

impl Contact {
    pub fn full_name(&self) -> String {
        match (&self.display_name, &self.first_name, &self.last_name) {
            (Some(display), _, _) if !display.is_empty() => display.clone(),
            (_, Some(first), Some(last)) => format!("{} {}", first, last),
            (_, Some(first), None) => first.clone(),
            (_, None, Some(last)) => last.clone(),
            _ => self.email.clone(),
        }
    }

    pub fn usage_score(&self) -> i64 {
        let base_score = self.send_count * 2 + self.receive_count;

        if let Some(last_used) = self.last_used_at {
            let days_since = (Utc::now() - last_used).num_days();
            if days_since < 30 {
                return base_score + (30 - days_since as i64) * 5;
            }
        }

        base_score
    }
}
