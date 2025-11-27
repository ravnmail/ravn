use thiserror::Error;

#[derive(Error, Debug)]
pub enum SearchError {
    #[error("Tantivy error: {0}")]
    TantivyError(#[from] tantivy::TantivyError),

    #[error("Query parser error: {0}")]
    QueryParserError(#[from] tantivy::query::QueryParserError),

    #[error("Directory error: {0}")]
    DirectoryError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Invalid query: {0}")]
    InvalidQuery(String),

    #[error("Index not found or not initialized")]
    IndexNotFound,

    #[error("Failed to parse UUID: {0}")]
    UuidError(#[from] uuid::Error),

    #[error("Failed to parse date: {0}")]
    DateParseError(String),

    #[error("Search error: {0}")]
    Other(String),
}

impl From<tantivy::directory::error::OpenDirectoryError> for SearchError {
    fn from(err: tantivy::directory::error::OpenDirectoryError) -> Self {
        SearchError::DirectoryError(err.to_string())
    }
}

impl From<tantivy::directory::error::OpenReadError> for SearchError {
    fn from(err: tantivy::directory::error::OpenReadError) -> Self {
        SearchError::DirectoryError(err.to_string())
    }
}

pub type SearchResult<T> = Result<T, SearchError>;
