use crate::database::{
    error::DatabaseError,
    models::view::{View, ViewConfig, ViewType},
};
use async_trait::async_trait;
use sqlx::{Row, SqlitePool};
use uuid::Uuid;

#[async_trait]
pub trait ViewRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<View>, DatabaseError>;
    async fn get_all(&self) -> Result<Vec<View>, DatabaseError>;
    async fn create(&self, view: &View) -> Result<Uuid, DatabaseError>;
    async fn update(&self, view: &View) -> Result<(), DatabaseError>;
    async fn delete(&self, id: Uuid) -> Result<(), DatabaseError>;
    async fn set_default(&self, id: Uuid) -> Result<(), DatabaseError>;
}

pub struct SqliteViewRepository {
    pool: SqlitePool,
}

impl SqliteViewRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    fn map_row_to_view(row: &sqlx::sqlite::SqliteRow) -> Result<View, DatabaseError> {
        let view_type_str: String = row.get("view_type");
        let view_type = view_type_str
            .parse::<ViewType>()
            .map_err(|e| DatabaseError::RepositoryError(e))?;

        let config_json: String = row.get("config");
        let config: ViewConfig =
            serde_json::from_str(&config_json).map_err(DatabaseError::JsonError)?;

        let folders_json: String = row.get("folders");
        let folders: Vec<Uuid> =
            serde_json::from_str(&folders_json).map_err(DatabaseError::JsonError)?;

        Ok(View {
            id: Uuid::parse_str(&row.get::<String, _>("id"))
                .map_err(|e| DatabaseError::RepositoryError(e.to_string()))?,
            name: row.get("name"),
            icon: row.get("icon"),
            color: row.get("color"),
            view_type,
            config,
            folders,
            sort_order: row.get("sort_order"),
            is_default: row.get("is_default"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        })
    }
}

#[async_trait]
impl ViewRepository for SqliteViewRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<View>, DatabaseError> {
        let row = sqlx::query("SELECT * FROM views WHERE id = ?")
            .bind(id.to_string())
            .fetch_optional(&self.pool)
            .await
            .map_err(DatabaseError::ConnectionError)?;

        match row {
            Some(r) => Ok(Some(Self::map_row_to_view(&r)?)),
            None => Ok(None),
        }
    }

    async fn get_all(&self) -> Result<Vec<View>, DatabaseError> {
        let rows = sqlx::query("SELECT * FROM views ORDER BY sort_order, name")
            .fetch_all(&self.pool)
            .await
            .map_err(DatabaseError::ConnectionError)?;

        rows.iter().map(Self::map_row_to_view).collect()
    }

    async fn create(&self, view: &View) -> Result<Uuid, DatabaseError> {
        let id = view.id.to_string();
        let view_type = view.view_type.to_string();
        let config_json = serde_json::to_string(&view.config).map_err(DatabaseError::JsonError)?;
        let folders_json =
            serde_json::to_string(&view.folders).map_err(DatabaseError::JsonError)?;

        sqlx::query!(
            r#"
            INSERT INTO views (id, name, view_type, icon, color, config, folders, sort_order, is_default)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            id,
            view.name,
            view_type,
            view.icon,
            view.color,
            config_json,
            folders_json,
            view.sort_order,
            view.is_default
        )
        .execute(&self.pool)
        .await
        .map_err(DatabaseError::ConnectionError)?;

        Ok(view.id)
    }

    async fn update(&self, view: &View) -> Result<(), DatabaseError> {
        let id = view.id.to_string();
        let view_type = view.view_type.to_string();
        let config_json = serde_json::to_string(&view.config).map_err(DatabaseError::JsonError)?;
        let folders_json =
            serde_json::to_string(&view.folders).map_err(DatabaseError::JsonError)?;

        sqlx::query!(
            r#"
            UPDATE views
            SET name = ?, view_type = ?, config = ?, folders = ?, sort_order = ?, is_default = ?,
                color = ?, icon = ?
            WHERE id = ?
            "#,
            view.name,
            view_type,
            config_json,
            folders_json,
            view.sort_order,
            view.is_default,
            view.color,
            view.icon,
            id
        )
        .execute(&self.pool)
        .await
        .map_err(DatabaseError::ConnectionError)?;

        Ok(())
    }

    async fn delete(&self, id: Uuid) -> Result<(), DatabaseError> {
        let id = id.to_string();

        sqlx::query!("DELETE FROM views WHERE id = ?", id)
            .execute(&self.pool)
            .await
            .map_err(DatabaseError::ConnectionError)?;

        Ok(())
    }

    async fn set_default(&self, id: Uuid) -> Result<(), DatabaseError> {
        let id_str = id.to_string();

        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(DatabaseError::ConnectionError)?;

        // Clear all existing defaults
        sqlx::query!("UPDATE views SET is_default = 0")
            .execute(&mut *tx)
            .await
            .map_err(DatabaseError::ConnectionError)?;

        // Set the new default
        sqlx::query!("UPDATE views SET is_default = 1 WHERE id = ?", id_str)
            .execute(&mut *tx)
            .await
            .map_err(DatabaseError::ConnectionError)?;

        tx.commit().await.map_err(DatabaseError::ConnectionError)?;

        Ok(())
    }
}
