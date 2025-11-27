#[derive(Debug, thiserror::Error)]
pub enum DatabaseError {
    #[error("Failed to connect to database: {0}")]
    ConnectionError(#[from] sqlx::Error),

    #[error("Failed to execute query: {0}")]
    QueryError(String),

    #[error("Invalid data found: {0}")]
    InvalidData(String),

    #[error("Migration failed: {0}")]
    MigrationError(#[from] sqlx::migrate::MigrateError),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Repository error: {0}")]
    RepositoryError(String),
}
