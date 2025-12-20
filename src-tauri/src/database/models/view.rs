use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct View {
    pub id: Uuid,
    pub name: String,
    pub icon: Option<String>,
    pub color: Option<String>,
    pub view_type: ViewType,
    #[serde(default)]
    pub config: ViewConfig,
    #[serde(default)]
    pub folders: Vec<Uuid>,
    pub sort_order: i32,
    pub is_default: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ViewType {
    List,
    Kanban,
    Calendar,
    Smart,
    Unified,
}

impl std::fmt::Display for ViewType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ViewType::List => write!(f, "list"),
            ViewType::Kanban => write!(f, "kanban"),
            ViewType::Calendar => write!(f, "calendar"),
            ViewType::Smart => write!(f, "smart"),
            ViewType::Unified => write!(f, "unified"),
        }
    }
}

impl std::str::FromStr for ViewType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "list" => Ok(ViewType::List),
            "kanban" => Ok(ViewType::Kanban),
            "calendar" => Ok(ViewType::Calendar),
            "smart" => Ok(ViewType::Smart),
            "unified" => Ok(ViewType::Unified),
            _ => Err(format!("Invalid view type: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum ViewConfig {
    Kanban {
        #[serde(default)]
        swimlanes: Vec<KanbanSwimlane>,
    },
    List {
        // Future: list-specific config
    },
    Calendar {
        // Future: calendar-specific config
    },
    Smart {
        // Future: smart filters config
    },
    Unified {
        // Future: unified inbox config
    },
}

impl Default for ViewConfig {
    fn default() -> Self {
        ViewConfig::Kanban {
            swimlanes: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KanbanSwimlane {
    pub id: Uuid,
    pub title: String,
    pub icon: Option<String>,
    pub color: Option<String>,
    #[serde(default)]
    pub label_ids: Vec<Uuid>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub folder_ids: Option<Vec<Uuid>>,
    pub state: SwimlaneState,
    pub sort_order: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SwimlaneState {
    Open,
    Closed,
}

impl Default for SwimlaneState {
    fn default() -> Self {
        SwimlaneState::Open
    }
}
