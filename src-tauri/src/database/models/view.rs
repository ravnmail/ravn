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
        #[serde(default)]
        filters: ListViewFilters,
    },
    Calendar {
        /// The date field to use for positioning emails on the calendar
        #[serde(default = "default_calendar_date_field")]
        date_field: CalendarDateField,
        /// Optional folder IDs to filter emails shown in this calendar view
        #[serde(default)]
        folder_ids: Vec<Uuid>,
        /// Display mode: month grid or week columns
        #[serde(default = "default_calendar_mode")]
        mode: CalendarMode,
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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ListViewFilters {
    #[serde(default)]
    pub groups: Vec<ListFilterGroup>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListFilterGroup {
    pub id: Uuid,
    #[serde(default = "default_list_filter_group_operator")]
    pub operator: ListFilterOperator,
    #[serde(default)]
    pub negated: bool,
    #[serde(default)]
    pub rules: Vec<ListFilterRule>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListFilterRule {
    pub id: Uuid,
    pub source: ListFilterRuleSource,
    #[serde(default)]
    pub values: Vec<Uuid>,
    #[serde(default = "default_list_filter_rule_operator")]
    pub operator: ListFilterOperator,
    #[serde(default)]
    pub negated: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ListFilterOperator {
    And,
    Or,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ListFilterRuleSource {
    Folders,
    Labels,
}

fn default_list_filter_group_operator() -> ListFilterOperator {
    ListFilterOperator::And
}

fn default_list_filter_rule_operator() -> ListFilterOperator {
    ListFilterOperator::Or
}

/// Which date field to use for positioning emails on the calendar
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CalendarDateField {
    ReceivedAt,
    SentAt,
    RemindAt,
}

impl Default for CalendarDateField {
    fn default() -> Self {
        CalendarDateField::RemindAt
    }
}

fn default_calendar_date_field() -> CalendarDateField {
    CalendarDateField::default()
}

/// Display mode for the calendar view
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CalendarMode {
    Month,
    Week,
}

impl Default for CalendarMode {
    fn default() -> Self {
        CalendarMode::Month
    }
}

fn default_calendar_mode() -> CalendarMode {
    CalendarMode::default()
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
