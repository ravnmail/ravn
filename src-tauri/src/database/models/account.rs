use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::Type;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub account_type: AccountType,
    pub settings: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type, PartialEq, Eq)]
#[sqlx(type_name = "TEXT", rename_all = "lowercase")]
pub enum AccountType {
    Gmail,
    Office365,
    Apple,
    Imap,
}

impl AccountType {
    pub fn as_str(&self) -> &'static str {
        match self {
            AccountType::Gmail => "gmail",
            AccountType::Office365 => "office365",
            AccountType::Apple => "apple",
            AccountType::Imap => "imap",
        }
    }
}

impl std::fmt::Display for AccountType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

// Provide conversions from database string values into AccountType so sqlx's
// query_as! macro (which expects Into<AccountType> for TEXT) can map the column.
impl From<String> for AccountType {
    fn from(s: String) -> Self {
        match s.to_lowercase().as_str() {
            "gmail" => AccountType::Gmail,
            "office365" => AccountType::Office365,
            "apple" => AccountType::Apple,
            "imap" => AccountType::Imap,
            // Accept a few common variants and provide a safe default.
            "outlook" => AccountType::Office365,
            _ => AccountType::Imap,
        }
    }
}

impl From<&str> for AccountType {
    fn from(s: &str) -> Self {
        AccountType::from(s.to_string())
    }
}

impl sqlx::FromRow<'_, sqlx::sqlite::SqliteRow> for Account {
    fn from_row(row: &sqlx::sqlite::SqliteRow) -> Result<Self, sqlx::Error> {
        use sqlx::Row;

        let id_str: String = row.try_get("id")?;
        let id = Uuid::parse_str(&id_str).map_err(|e| sqlx::Error::Decode(Box::new(e)))?;

        Ok(Account {
            id,
            name: row.try_get("name")?,
            email: row.try_get("email")?,
            account_type: row.try_get("account_type")?,
            settings: row.try_get("settings")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        })
    }
}
