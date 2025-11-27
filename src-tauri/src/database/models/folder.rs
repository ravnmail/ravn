use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{Row, Type};
use std::fmt;
use std::str::FromStr;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FolderSettings {
    #[serde(default = "default_cache_attachments")]
    pub cache_attachments: bool,

    // Mail list view preferences
    #[serde(default = "default_sort_by")]
    pub sort_by: String,

    #[serde(default = "default_sort_order")]
    pub sort_order: String,

    #[serde(default = "default_grouping_enabled")]
    pub grouping_enabled: bool,

    #[serde(default = "default_expanded_groups")]
    pub expanded_groups: Vec<String>,

    #[serde(default)]
    pub filter_read: Option<bool>,

    #[serde(default)]
    pub filter_has_attachments: Option<bool>,
}

fn default_sort_by() -> String {
    "received_at".to_string()
}

fn default_sort_order() -> String {
    "desc".to_string()
}

fn default_grouping_enabled() -> bool {
    true
}

fn default_cache_attachments() -> bool {
    true
}

fn default_expanded_groups() -> Vec<String> {
    vec![
        "today".to_string(),
        "yesterday".to_string(),
        "thisWeek".to_string(),
        "thisMonth".to_string(),
        "older".to_string(),
        "enormous".to_string(),
        "huge".to_string(),
        "veryLarge".to_string(),
        "large".to_string(),
        "medium".to_string(),
        "small".to_string(),
    ]
}

impl Default for FolderSettings {
    fn default() -> Self {
        Self {
            cache_attachments: default_cache_attachments(),
            sort_by: default_sort_by(),
            sort_order: default_sort_order(),
            grouping_enabled: default_grouping_enabled(),
            expanded_groups: default_expanded_groups(),
            filter_read: None,
            filter_has_attachments: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Folder {
    pub id: Uuid,
    pub account_id: Uuid,
    pub name: String,
    pub folder_type: FolderType,
    pub remote_id: Option<String>,
    pub color: Option<String>,
    pub icon: Option<String>,
    pub sort_order: i32,
    pub expanded: bool,
    pub hidden: bool,
    pub parent_id: Option<Uuid>,
    pub settings: FolderSettings,
    pub sync_interval: i64,
    pub unread_count: i64,
    pub total_count: i64,
    pub synced_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl sqlx::FromRow<'_, sqlx::sqlite::SqliteRow> for Folder {
    fn from_row(row: &sqlx::sqlite::SqliteRow) -> Result<Self, sqlx::Error> {
        let id_str: String = row.try_get("id")?;
        let id = Uuid::parse_str(&id_str).map_err(|e| sqlx::Error::Decode(Box::new(e)))?;

        let account_id_str: String = row.try_get("account_id")?;
        let account_id =
            Uuid::parse_str(&account_id_str).map_err(|e| sqlx::Error::Decode(Box::new(e)))?;

        let parent_id_str: Option<String> = row.try_get("parent_id")?;
        let parent_id = parent_id_str
            .as_deref()
            .map(Uuid::parse_str)
            .transpose()
            .map_err(|e| sqlx::Error::Decode(Box::new(e)))?;

        let settings_str: String = row.try_get("settings")?;
        let settings: FolderSettings = serde_json::from_str(&settings_str).unwrap_or_default();

        Ok(Folder {
            id,
            account_id,
            name: row.try_get("name")?,
            folder_type: row.try_get("folder_type")?,
            remote_id: row.try_get("remote_id")?,
            color: row.try_get("color")?,
            icon: row.try_get("icon")?,
            sort_order: row.try_get("sort_order")?,
            expanded: row.try_get("expanded")?,
            hidden: row.try_get("hidden")?,
            parent_id,
            settings,
            sync_interval: row.try_get("sync_interval")?,
            unread_count: row.try_get("unread_count")?,
            total_count: row.try_get("total_count")?,
            synced_at: row.try_get("synced_at")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        })
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type, PartialEq, Eq)]
#[sqlx(type_name = "TEXT", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum FolderType {
    Archive,
    Inbox,
    Sent,
    Draft,
    Trash,
    Spam,
    Starred,
    Custom,
}

impl FolderType {
    /// Get the default icon name for this folder type
    pub fn default_icon(&self) -> &'static str {
        match self {
            FolderType::Inbox => "inbox",
            FolderType::Sent => "send",
            FolderType::Draft => "file-edit",
            FolderType::Trash => "trash-2",
            FolderType::Spam => "shield-alert",
            FolderType::Archive => "archive",
            FolderType::Starred => "star",
            FolderType::Custom => "folder",
        }
    }

    /// Get the default sync interval in seconds for this folder type
    pub fn default_sync_interval(&self) -> u64 {
        match self {
            FolderType::Inbox => 60,     // 1 minute
            FolderType::Sent => 300,     // 5 minutes
            FolderType::Draft => 180,    // 3 minutes
            FolderType::Trash => 600,    // 10 minutes
            FolderType::Spam => 600,     // 10 minutes
            FolderType::Archive => 1800, // 30 minutes
            FolderType::Starred => 300,  // 5 minutes
            FolderType::Custom => 300,   // 5 minutes
        }
    }

    /// Convert enum to lowercase string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            FolderType::Archive => "archive",
            FolderType::Inbox => "inbox",
            FolderType::Sent => "sent",
            FolderType::Draft => "draft",
            FolderType::Trash => "trash",
            FolderType::Spam => "spam",
            FolderType::Starred => "starred",
            FolderType::Custom => "custom",
        }
    }
}

impl fmt::Display for FolderType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for FolderType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "archive" => Ok(FolderType::Archive),
            "inbox" => Ok(FolderType::Inbox),
            "sent" => Ok(FolderType::Sent),
            "draft" => Ok(FolderType::Draft),
            "trash" => Ok(FolderType::Trash),
            "spam" => Ok(FolderType::Spam),
            "starred" => Ok(FolderType::Starred),
            "custom" => Ok(FolderType::Custom),
            _ => Err(format!("Unknown folder type: {}", s)),
        }
    }
}

impl AsRef<str> for FolderType {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
