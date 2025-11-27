use crate::database::{error::DatabaseError, models::label::Label};
use async_trait::async_trait;
use sqlx::{Row, SqlitePool};
use uuid::Uuid;

#[async_trait]
pub trait LabelRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Label>, DatabaseError>;
    async fn get_all(&self) -> Result<Vec<Label>, DatabaseError>;
    async fn find_by_email(&self, email_id: Uuid) -> Result<Vec<Label>, DatabaseError>;
    async fn find_by_emails(
        &self,
        email_ids: &[Uuid],
    ) -> Result<std::collections::HashMap<Uuid, Vec<Label>>, DatabaseError>;
    async fn create(&self, label: &Label) -> Result<Uuid, DatabaseError>;
    async fn update(&self, label: &Label) -> Result<(), DatabaseError>;
    async fn delete(&self, id: Uuid) -> Result<(), DatabaseError>;
    async fn add_to_email(&self, email_id: Uuid, label_id: Uuid) -> Result<(), DatabaseError>;
    async fn remove_from_email(&self, email_id: Uuid, label_id: Uuid) -> Result<(), DatabaseError>;
}

pub struct SqliteLabelRepository {
    pool: SqlitePool,
}

impl SqliteLabelRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl LabelRepository for SqliteLabelRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Label>, DatabaseError> {
        sqlx::query_as::<_, Label>("SELECT * FROM labels WHERE id = ?")
            .bind(id.to_string())
            .fetch_optional(&self.pool)
            .await
            .map_err(DatabaseError::ConnectionError)
    }

    async fn get_all(&self) -> Result<Vec<Label>, DatabaseError> {
        sqlx::query_as::<_, Label>("SELECT * FROM labels ORDER BY name")
            .fetch_all(&self.pool)
            .await
            .map_err(DatabaseError::ConnectionError)
    }
    async fn find_by_email(&self, email_id: Uuid) -> Result<Vec<Label>, DatabaseError> {
        sqlx::query_as::<_, Label>(
            r#"
            SELECT l.*
            FROM labels l
            JOIN email_labels el ON el.label_id = l.id
            WHERE el.email_id = ?
            ORDER BY l.name
            "#,
        )
        .bind(email_id.to_string())
        .fetch_all(&self.pool)
        .await
        .map_err(DatabaseError::ConnectionError)
    }

    async fn find_by_emails(
        &self,
        email_ids: &[Uuid],
    ) -> Result<std::collections::HashMap<Uuid, Vec<Label>>, DatabaseError> {
        use std::collections::HashMap;

        if email_ids.is_empty() {
            return Ok(HashMap::new());
        }

        // Convert UUIDs to strings for binding
        let email_id_strings: Vec<String> = email_ids.iter().map(|id| id.to_string()).collect();

        // Build dynamic query with proper placeholders
        let placeholders = email_id_strings
            .iter()
            .map(|_| "?")
            .collect::<Vec<_>>()
            .join(", ");

        let query = format!(
            r#"
            SELECT el.email_id, l.*
            FROM labels l
            JOIN email_labels el ON el.label_id = l.id
            WHERE el.email_id IN ({})
            ORDER BY el.email_id, l.name
            "#,
            placeholders
        );

        // Build query with all bindings
        let mut sqlx_query = sqlx::query(&query);
        for email_id_str in &email_id_strings {
            sqlx_query = sqlx_query.bind(email_id_str);
        }

        let rows = sqlx_query
            .fetch_all(&self.pool)
            .await
            .map_err(DatabaseError::ConnectionError)?;

        // Group labels by email_id
        let mut result: HashMap<Uuid, Vec<Label>> = HashMap::new();

        for row in rows {
            let email_id_str: String = row
                .try_get("email_id")
                .map_err(|e| DatabaseError::QueryError(e.to_string()))?;
            let email_id = Uuid::parse_str(&email_id_str)
                .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

            let label = Label {
                id: {
                    let id_str: String = row
                        .try_get("id")
                        .map_err(|e| DatabaseError::QueryError(e.to_string()))?;
                    Uuid::parse_str(&id_str)
                        .map_err(|e| DatabaseError::QueryError(e.to_string()))?
                },
                name: row
                    .try_get("name")
                    .map_err(|e| DatabaseError::QueryError(e.to_string()))?,
                color: row
                    .try_get("color")
                    .map_err(|e| DatabaseError::QueryError(e.to_string()))?,
                icon: row
                    .try_get("icon")
                    .map_err(|e| DatabaseError::QueryError(e.to_string()))?,
                created_at: row
                    .try_get("created_at")
                    .map_err(|e| DatabaseError::QueryError(e.to_string()))?,
                updated_at: row
                    .try_get("updated_at")
                    .map_err(|e| DatabaseError::QueryError(e.to_string()))?,
            };

            result.entry(email_id).or_insert_with(Vec::new).push(label);
        }

        // Ensure all email_ids are in the result (even if they have no labels)
        for email_id in email_ids {
            result.entry(*email_id).or_insert_with(Vec::new);
        }

        Ok(result)
    }

    async fn create(&self, label: &Label) -> Result<Uuid, DatabaseError> {
        let id = label.id.to_string();

        sqlx::query!(
            r#"
            INSERT INTO labels (id, name, color, icon)
            VALUES (?, ?, ?, ?)
            "#,
            id,
            label.name,
            label.color,
            label.icon
        )
        .execute(&self.pool)
        .await
        .map_err(DatabaseError::ConnectionError)?;

        Ok(label.id)
    }

    async fn update(&self, label: &Label) -> Result<(), DatabaseError> {
        let id = label.id.to_string();

        sqlx::query!(
            r#"
            UPDATE labels
            SET name = ?, color = ?, icon = ?
            WHERE id = ?
            "#,
            label.name,
            label.color,
            label.icon,
            id
        )
        .execute(&self.pool)
        .await
        .map_err(DatabaseError::ConnectionError)?;

        Ok(())
    }

    async fn delete(&self, id: Uuid) -> Result<(), DatabaseError> {
        let id = id.to_string();

        // Start a transaction to ensure both operations complete or neither does
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(DatabaseError::ConnectionError)?;

        // First delete from the junction table
        sqlx::query!("DELETE FROM email_labels WHERE label_id = ?", id)
            .execute(&mut *tx)
            .await
            .map_err(DatabaseError::ConnectionError)?;

        // Then delete the label itself
        sqlx::query!("DELETE FROM labels WHERE id = ?", id)
            .execute(&mut *tx)
            .await
            .map_err(DatabaseError::ConnectionError)?;

        tx.commit().await.map_err(DatabaseError::ConnectionError)?;

        Ok(())
    }

    async fn add_to_email(&self, email_id: Uuid, label_id: Uuid) -> Result<(), DatabaseError> {
        let email_id = email_id.to_string();
        let label_id = label_id.to_string();

        sqlx::query!(
            r#"
            INSERT OR IGNORE INTO email_labels (email_id, label_id)
            VALUES (?, ?)
            "#,
            email_id,
            label_id
        )
        .execute(&self.pool)
        .await
        .map_err(DatabaseError::ConnectionError)?;

        Ok(())
    }

    async fn remove_from_email(&self, email_id: Uuid, label_id: Uuid) -> Result<(), DatabaseError> {
        let email_id = email_id.to_string();
        let label_id = label_id.to_string();

        sqlx::query!(
            r#"
            DELETE FROM email_labels
            WHERE email_id = ? AND label_id = ?
            "#,
            email_id,
            label_id
        )
        .execute(&self.pool)
        .await
        .map_err(DatabaseError::ConnectionError)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use sqlx::sqlite::SqlitePoolOptions;

    /// Helper function to create a test database pool
    async fn create_test_pool() -> SqlitePool {
        SqlitePoolOptions::new()
            .max_connections(5)
            .connect("sqlite::memory:")
            .await
            .expect("Failed to create test database pool")
    }

    /// Helper function to create test schema
    async fn setup_test_schema(pool: &SqlitePool) {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS labels (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                account_id INTEGER NOT NULL,
                name TEXT NOT NULL,
                color TEXT,
                created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
            );

            CREATE TABLE IF NOT EXISTS email_labels (
                email_id INTEGER NOT NULL,
                label_id INTEGER NOT NULL,
                PRIMARY KEY (email_id, label_id)
            );
            "#,
        )
        .execute(pool)
        .await
        .expect("Failed to create test schema");
    }

    /// Helper function to create a test label
    fn create_test_label() -> Label {
        Label {
            id: Uuid::now_v7(),
            name: "Test Label".to_string(),
            icon: Some * ("tag".to_string()),
            color: Some("#FF0000".to_string()),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[tokio::test]
    async fn test_create_label() {
        let pool = create_test_pool().await;
        setup_test_schema(&pool).await;

        let repository = SqliteLabelRepository::new(pool);
        let test_label = create_test_label();

        let result = repository.create(&test_label).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_find_by_id() {
        let pool = create_test_pool().await;
        setup_test_schema(&pool).await;

        let repository = SqliteLabelRepository::new(pool);
        let test_label = create_test_label();
        let id = test_label.id;

        // Create test label
        repository.create(&test_label).await.unwrap();

        // Test finding the label
        let result = repository.find_by_id(id).await.unwrap();
        assert!(result.is_some());

        let found_label = result.unwrap();
        assert_eq!(found_label.name, test_label.name);
        assert_eq!(found_label.color, test_label.color);
    }

    #[tokio::test]
    async fn test_get_all() {
        let pool = create_test_pool().await;
        setup_test_schema(&pool).await;

        let repository = SqliteLabelRepository::new(pool);

        // Create multiple test labels
        for i in 1..=3 {
            let mut label = create_test_label();
            label.name = format!("Test Label {}", i);
            repository.create(&label).await.unwrap();
        }

        // Test finding all labels for account
        let result = repository.get_all().await.unwrap();
        assert_eq!(result.len(), 3);

        // Verify labels are returned in alphabetical order
        for (i, label) in result.iter().enumerate() {
            assert_eq!(label.name, format!("Test Label {}", i + 1));
        }
    }

    #[tokio::test]
    async fn test_update_label() {
        let pool = create_test_pool().await;
        setup_test_schema(&pool).await;

        let repository = SqliteLabelRepository::new(pool);
        let mut test_label = create_test_label();
        let id = test_label.id;

        // Create test label
        repository.create(&test_label).await.unwrap();

        // Update label
        test_label.name = "Updated Label".to_string();
        test_label.color = Some("#00FF00".to_string());

        let update_result = repository.update(&test_label).await;
        assert!(update_result.is_ok());

        // Verify update
        let updated = repository.find_by_id(id).await.unwrap().unwrap();
        assert_eq!(updated.name, "Updated Label");
        assert_eq!(updated.color, Some("#00FF00".to_string()));
    }

    #[tokio::test]
    async fn test_delete_label() {
        let pool = create_test_pool().await;
        setup_test_schema(&pool).await;

        let repository = SqliteLabelRepository::new(pool);
        let test_label = create_test_label();
        let id = test_label.id;

        // Create test label
        repository.create(&test_label).await.unwrap();

        // Delete label
        let delete_result = repository.delete(id).await;
        assert!(delete_result.is_ok());

        // Verify deletion
        let find_result = repository.find_by_id(id).await.unwrap();
        assert!(find_result.is_none());
    }

    #[tokio::test]
    async fn test_add_label_to_email() {
        let pool = create_test_pool().await;
        setup_test_schema(&pool).await;

        let repository = SqliteLabelRepository::new(pool);
        let test_label = create_test_label();
        let label_id = test_label.id;

        // Create test label
        repository.create(&test_label).await.unwrap();
        let email_id = Uuid::now_v7();

        // Add label to email
        let result = repository.add_to_email(email_id, label_id).await;
        assert!(result.is_ok());

        // Verify label was added
        let labels = repository.find_by_email(email_id).await.unwrap();
        assert_eq!(labels.len(), 1);
        assert_eq!(labels[0].id, label_id);
    }

    #[tokio::test]
    async fn test_remove_label_from_email() {
        let pool = create_test_pool().await;
        setup_test_schema(&pool).await;

        let repository = SqliteLabelRepository::new(pool);
        let test_label = create_test_label();
        let label_id = test_label.id;

        // Create test label and add to email
        repository.create(&test_label).await.unwrap();
        let email_id = Uuid::now_v7();

        repository.add_to_email(email_id, label_id).await.unwrap();

        // Remove label from email
        let result = repository.remove_from_email(email_id, label_id).await;
        assert!(result.is_ok());

        // Verify label was removed
        let labels = repository.find_by_email(email_id).await.unwrap();
        assert!(labels.is_empty());
    }

    #[tokio::test]
    async fn test_multiple_labels_per_email() {
        let pool = create_test_pool().await;
        setup_test_schema(&pool).await;

        let repository = SqliteLabelRepository::new(pool);
        let email_id = Uuid::now_v7();

        // Create and add multiple labels
        let mut label_ids = vec![];
        for i in 1..=3 {
            let mut label = create_test_label();
            label.name = format!("Label {}", i);
            label.color = Some(format!("#{:06X}", i * 111111));

            let label_id = label.id;
            repository.create(&label).await.unwrap();
            repository.add_to_email(email_id, label_id).await.unwrap();
            label_ids.push(label_id);
        }

        // Verify all labels were added
        let labels = repository.find_by_email(email_id).await.unwrap();
        assert_eq!(labels.len(), 3);

        // Verify labels are in correct order
        for (i, label) in labels.iter().enumerate() {
            assert_eq!(label.name, format!("Label {}", i + 1));
        }
    }

    #[tokio::test]
    async fn test_duplicate_label_prevention() {
        let pool = create_test_pool().await;
        setup_test_schema(&pool).await;

        let repository = SqliteLabelRepository::new(pool);
        let test_label = create_test_label();
        let label_id = test_label.id;
        repository.create(&test_label).await.unwrap();
        let email_id = Uuid::now_v7();

        // Add label to email
        repository.add_to_email(email_id, label_id).await.unwrap();

        // Try to add same label again
        let result = repository.add_to_email(email_id, label_id).await;
        assert!(result.is_ok()); // Should not error due to IGNORE in SQL

        // Verify label exists only once
        let labels = repository.find_by_email(email_id).await.unwrap();
        assert_eq!(labels.len(), 1);
    }

    #[tokio::test]
    async fn test_error_handling() {
        let pool = create_test_pool().await;
        setup_test_schema(&pool).await;

        let repository = SqliteLabelRepository::new(pool);

        // Test finding non-existent label
        let result = repository.find_by_id(Uuid::now_v7()).await.unwrap();
        assert!(result.is_none());

        // Test finding labels for non-existent email
        let result = repository.find_by_email(Uuid::now_v7()).await.unwrap();
        assert!(result.is_empty());

        // Test removing non-existent label from email
        let result = repository
            .remove_from_email(Uuid::now_v7(), Uuid::now_v7())
            .await;
        assert!(result.is_ok()); // SQLite doesn't error on non-existent rows
    }
}
