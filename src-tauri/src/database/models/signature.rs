use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Signature {
    pub id: Uuid,
    pub account_id: Uuid,
    pub name: String,
    pub signature: String,
    pub is_default: bool,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
