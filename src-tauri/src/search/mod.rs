mod error;
mod search_manager;

pub use error::{SearchError, SearchResult};
pub use search_manager::SearchManager;

// Re-export search-related types
pub use search_manager::{SearchQuery, SearchResultItem};
