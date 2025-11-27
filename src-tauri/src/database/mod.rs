use crate::database::{error::DatabaseError, repositories::RepositoryFactory};
use sqlx::{migrate::MigrateDatabase, sqlite::SqlitePool, Connection, SqliteConnection};
use std::path::Path;

pub mod error;
pub mod models;
pub mod repositories;
pub mod utils;

pub struct Database {
    pool: SqlitePool,
}

impl Database {
    pub async fn new(data_dir: &Path) -> Result<Self, DatabaseError> {
        let db_path = data_dir.join("ravn.db");
        let database_url = format!("sqlite:{}", db_path.display());

        if !sqlx::Sqlite::database_exists(&database_url)
            .await
            .map_err(DatabaseError::ConnectionError)?
        {
            println!("Creating database at {}", database_url);
            sqlx::Sqlite::create_database(&database_url)
                .await
                .map_err(DatabaseError::ConnectionError)?;
        }

        let mut conn = SqliteConnection::connect(&database_url)
            .await
            .map_err(DatabaseError::ConnectionError)?;

        sqlx::migrate!("./migrations")
            .run(&mut conn)
            .await
            .map_err(DatabaseError::MigrationError)?;

        let pool = SqlitePool::connect(&database_url)
            .await
            .map_err(DatabaseError::ConnectionError)?;

        Ok(Self { pool })
    }

    pub fn get_pool(&self) -> &SqlitePool {
        &self.pool
    }

    pub fn repositories(&self) -> RepositoryFactory {
        RepositoryFactory::new(self.pool.clone())
    }
}
