use crate::database::{error::DatabaseError, models::attachment::Attachment};
use async_trait::async_trait;
use sqlx::SqlitePool;
use uuid::Uuid;

#[async_trait]
pub trait AttachmentRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Attachment>, DatabaseError>;
    async fn find_by_email(&self, email_id: Uuid) -> Result<Vec<Attachment>, DatabaseError>;
    async fn find_by_hash(&self, hash: &str) -> Result<Option<Attachment>, DatabaseError>;
    async fn find_by_conversation_id(
        &self,
        conversation_id: Uuid,
    ) -> Result<Vec<Attachment>, DatabaseError>;
    async fn create(&self, attachment: &Attachment) -> Result<Uuid, DatabaseError>;
    async fn delete(&self, id: Uuid) -> Result<(), DatabaseError>;
    async fn find_all_cached(&self)
        -> Result<Vec<(String, Option<String>, String)>, DatabaseError>;
    async fn update_hash(&self, id: &str, hash: &str) -> Result<(), DatabaseError>;
}

pub struct SqliteAttachmentRepository {
    pool: SqlitePool,
}

impl SqliteAttachmentRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AttachmentRepository for SqliteAttachmentRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Attachment>, DatabaseError> {
        sqlx::query_as::<_, Attachment>("SELECT * FROM attachments WHERE id = ?")
            .bind(id.to_string())
            .fetch_optional(&self.pool)
            .await
            .map_err(DatabaseError::ConnectionError)
    }

    async fn find_by_email(&self, email_id: Uuid) -> Result<Vec<Attachment>, DatabaseError> {
        sqlx::query_as::<_, Attachment>("SELECT * FROM attachments WHERE email_id = ?")
            .bind(email_id.to_string())
            .fetch_all(&self.pool)
            .await
            .map_err(DatabaseError::ConnectionError)
    }

    async fn find_by_hash(&self, hash: &str) -> Result<Option<Attachment>, DatabaseError> {
        sqlx::query_as::<_, Attachment>("SELECT * FROM attachments WHERE hash = ?")
            .bind(hash)
            .fetch_optional(&self.pool)
            .await
            .map_err(DatabaseError::ConnectionError)
    }

    async fn find_by_conversation_id(
        &self,
        conversation_id: Uuid,
    ) -> Result<Vec<Attachment>, DatabaseError> {
        sqlx::query_as::<_, Attachment>(
            r#"
            SELECT a.* FROM attachments a
            INNER JOIN emails e ON a.email_id = e.id
            WHERE e.conversation_id = ?
            ORDER BY e.received_at ASC
            "#,
        )
        .bind(conversation_id.to_string())
        .fetch_all(&self.pool)
        .await
        .map_err(DatabaseError::ConnectionError)
    }

    async fn create(&self, attachment: &Attachment) -> Result<Uuid, DatabaseError> {
        let id = attachment.id.to_string();
        let email_id = attachment.email_id.to_string();

        sqlx::query!(
            r#"
            INSERT INTO attachments (
                id, email_id, filename, content_type, size, hash,
                cache_path, is_inline, content_id
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            id,
            email_id,
            attachment.filename,
            attachment.content_type,
            attachment.size,
            attachment.hash,
            attachment.cache_path,
            attachment.is_inline,
            attachment.content_id
        )
        .execute(&self.pool)
        .await
        .map_err(DatabaseError::ConnectionError)?;

        Ok(attachment.id)
    }

    async fn delete(&self, id: Uuid) -> Result<(), DatabaseError> {
        let id = id.to_string();
        sqlx::query!("DELETE FROM attachments WHERE id = ?", id)
            .execute(&self.pool)
            .await
            .map_err(DatabaseError::ConnectionError)?;

        Ok(())
    }

    async fn find_all_cached(
        &self,
    ) -> Result<Vec<(String, Option<String>, String)>, DatabaseError> {
        #[derive(sqlx::FromRow)]
        struct CachedAttachment {
            id: String,
            cache_path: Option<String>,
            hash: String,
        }

        let results = sqlx::query_as::<_, CachedAttachment>(
            "SELECT id, cache_path, hash FROM attachments WHERE is_cached = 1 AND cache_path IS NOT NULL"
        )
        .fetch_all(&self.pool)
        .await
        .map_err(DatabaseError::ConnectionError)?;

        Ok(results
            .into_iter()
            .map(|r| (r.id, r.cache_path, r.hash))
            .collect())
    }

    async fn update_hash(&self, id: &str, hash: &str) -> Result<(), DatabaseError> {
        sqlx::query!("UPDATE attachments SET hash = ? WHERE id = ?", hash, id)
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

    async fn create_test_pool() -> SqlitePool {
        SqlitePoolOptions::new()
            .max_connections(5)
            .connect("sqlite::memory:")
            .await
            .expect("Failed to create test database pool")
    }

    async fn setup_test_schema(pool: &SqlitePool) {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS attachments (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                email_id INTEGER NOT NULL,
                filename TEXT NOT NULL,
                content_type TEXT NOT NULL,
                size INTEGER NOT NULL,
                hash TEXT NOT NULL,
                cache_path TEXT,
                is_inline BOOLEAN NOT NULL DEFAULT 0,
                content_id TEXT,
                created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
            );
            CREATE INDEX IF NOT EXISTS idx_attachments_hash ON attachments(hash);
            "#,
        )
        .execute(pool)
        .await
        .expect("Failed to create test schema");
    }

    /// Helper function to create a test attachment
    fn create_test_attachment(email_id: Uuid) -> Attachment {
        Attachment {
            id: Uuid::now_v7(),
            email_id,
            filename: "test.pdf".to_string(),
            content_type: "application/pdf".to_string(),
            size: 1024,
            hash: "test_hash_123".to_string(),
            cache_path: None,
            is_inline: false,
            is_cached: false,
            content_id: None,
            created_at: Utc::now(),
        }
    }

    #[tokio::test]
    async fn test_create_attachment() {
        let pool = create_test_pool().await;
        setup_test_schema(&pool).await;

        let repository = SqliteAttachmentRepository::new(pool);
        let test_attachment = create_test_attachment(Uuid::now_v7());

        let result = repository.create(&test_attachment).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_find_by_id() {
        let pool = create_test_pool().await;
        setup_test_schema(&pool).await;

        let repository = SqliteAttachmentRepository::new(pool);
        let test_attachment = create_test_attachment(Uuid::now_v7());
        let id = test_attachment.id;

        // Create test attachment
        repository.create(&test_attachment).await.unwrap();

        // Test finding the attachment
        let result = repository.find_by_id(id).await.unwrap();
        assert!(result.is_some());

        let found_attachment = result.unwrap();
        assert_eq!(found_attachment.filename, test_attachment.filename);
        assert_eq!(found_attachment.content_type, test_attachment.content_type);
        assert_eq!(found_attachment.hash, test_attachment.hash);
    }

    #[tokio::test]
    async fn test_find_by_email() {
        let pool = create_test_pool().await;
        setup_test_schema(&pool).await;

        let repository = SqliteAttachmentRepository::new(pool);

        // Create multiple test attachments for the same email
        let email_id = Uuid::now_v7();
        let mut attachments = vec![];
        for i in 1..=3 {
            let mut attachment = create_test_attachment(email_id);
            attachment.filename = format!("test{}.pdf", i);
            attachment.hash = format!("hash_{}", i);
            repository.create(&attachment).await.unwrap();
            attachments.push(attachment);
        }

        // Test finding all attachments for the email
        let result = repository.find_by_email(email_id).await.unwrap();
        assert_eq!(result.len(), 3);

        // Verify attachments
        for (i, attachment) in result.iter().enumerate() {
            assert_eq!(attachment.filename, format!("test{}.pdf", i + 1));
            assert_eq!(attachment.hash, format!("hash_{}", i + 1));
        }
    }

    #[tokio::test]
    async fn test_find_by_hash() {
        let pool = create_test_pool().await;
        setup_test_schema(&pool).await;

        let repository = SqliteAttachmentRepository::new(pool);
        let test_attachment = create_test_attachment(Uuid::now_v7());
        let test_hash = test_attachment.hash.clone();

        // Create test attachment
        repository.create(&test_attachment).await.unwrap();

        // Test finding by hash
        let result = repository.find_by_hash(&test_hash).await.unwrap();
        assert!(result.is_some());

        let found_attachment = result.unwrap();
        assert_eq!(found_attachment.hash, test_hash);
    }

    #[tokio::test]
    async fn test_delete_attachment() {
        let pool = create_test_pool().await;
        setup_test_schema(&pool).await;

        let repository = SqliteAttachmentRepository::new(pool);
        let test_attachment = create_test_attachment(Uuid::now_v7());
        let id = test_attachment.id;

        // Create test attachment
        repository.create(&test_attachment).await.unwrap();

        // Delete attachment
        let delete_result = repository.delete(id).await;
        assert!(delete_result.is_ok());

        // Verify deletion
        let find_result = repository.find_by_id(id).await.unwrap();
        assert!(find_result.is_none());
    }

    #[tokio::test]
    async fn test_inline_attachment() {
        let pool = create_test_pool().await;
        setup_test_schema(&pool).await;

        let repository = SqliteAttachmentRepository::new(pool);
        let mut test_attachment = create_test_attachment(Uuid::now_v7());
        let id = test_attachment.id;

        // Set inline attachment properties
        test_attachment.is_inline = true;
        test_attachment.content_id = Some("test-content-id".to_string());

        // Create test attachment
        repository.create(&test_attachment).await.unwrap();

        // Verify inline properties
        let found = repository.find_by_id(id).await.unwrap().unwrap();
        assert!(found.is_inline);
        assert_eq!(found.content_id, Some("test-content-id".to_string()));
    }

    #[tokio::test]
    async fn test_attachment_deduplication() {
        let pool = create_test_pool().await;
        setup_test_schema(&pool).await;

        let repository = SqliteAttachmentRepository::new(pool);

        // Create two attachments with the same hash but different emails
        let mut attachment1 = create_test_attachment(Uuid::now_v7());
        let mut attachment2 = create_test_attachment(Uuid::now_v7());

        // Set same hash
        let shared_hash = "shared_hash_123".to_string();
        attachment1.hash = shared_hash.clone();
        attachment2.hash = shared_hash.clone();

        // Create both attachments
        repository.create(&attachment1).await.unwrap();
        repository.create(&attachment2).await.unwrap();

        // Verify we can find both by hash
        let found = repository.find_by_hash(&shared_hash).await.unwrap();
        assert!(found.is_some());
    }

    #[tokio::test]
    async fn test_error_handling() {
        let pool = create_test_pool().await;
        setup_test_schema(&pool).await;

        let repository = SqliteAttachmentRepository::new(pool);

        // Test finding non-existent attachment
        let result = repository.find_by_id(Uuid::now_v7()).await.unwrap();
        assert!(result.is_none());

        // Test finding by non-existent hash
        let result = repository.find_by_hash("non_existent_hash").await.unwrap();
        assert!(result.is_none());

        // Test finding attachments for non-existent email
        let result = repository.find_by_email(Uuid::now_v7()).await.unwrap();
        assert!(result.is_empty());

        // Test deleting non-existent attachment
        let result = repository.delete(Uuid::now_v7()).await;
        assert!(result.is_ok()); // SQLite doesn't error on deleting non-existent rows
    }
}
