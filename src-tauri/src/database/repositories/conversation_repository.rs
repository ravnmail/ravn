use crate::database::{error::DatabaseError, models::conversation::Conversation};
use async_trait::async_trait;
use sqlx::SqlitePool;
use uuid::Uuid;

#[async_trait]
pub trait ConversationRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Conversation>, DatabaseError>;
    async fn find_by_remote_id(
        &self,
        remote_id: &str,
    ) -> Result<Option<Conversation>, DatabaseError>;
    async fn find_by_ids(&self, ids: Vec<Uuid>) -> Result<Vec<Conversation>, DatabaseError>;
    async fn create(&self, conversation: &Conversation) -> Result<Uuid, DatabaseError>;
    async fn update(&self, conversation: &Conversation) -> Result<(), DatabaseError>;
    async fn delete(&self, id: Uuid) -> Result<(), DatabaseError>;
    async fn find_or_create_by_remote_id(
        &self,
        remote_id: &str,
    ) -> Result<Conversation, DatabaseError>;
}

pub struct SqliteConversationRepository {
    pool: SqlitePool,
}

impl SqliteConversationRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ConversationRepository for SqliteConversationRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Conversation>, DatabaseError> {
        sqlx::query_as::<_, Conversation>("SELECT * FROM conversations WHERE id = ?")
            .bind(id.to_string())
            .fetch_optional(&self.pool)
            .await
            .map_err(DatabaseError::ConnectionError)
    }

    async fn find_by_remote_id(
        &self,
        remote_id: &str,
    ) -> Result<Option<Conversation>, DatabaseError> {
        sqlx::query_as::<_, Conversation>("SELECT * FROM conversations WHERE remote_id = ?")
            .bind(remote_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(DatabaseError::ConnectionError)
    }

    async fn find_by_ids(&self, ids: Vec<Uuid>) -> Result<Vec<Conversation>, DatabaseError> {
        if ids.is_empty() {
            return Ok(Vec::new());
        }

        let placeholders = ids.iter().map(|_| "?").collect::<Vec<_>>().join(", ");
        let query = format!("SELECT * FROM conversations WHERE id IN ({})", placeholders);

        let mut query_builder = sqlx::query_as::<_, Conversation>(&query);
        for id in ids {
            query_builder = query_builder.bind(id.to_string());
        }

        query_builder
            .fetch_all(&self.pool)
            .await
            .map_err(DatabaseError::ConnectionError)
    }

    async fn create(&self, conversation: &Conversation) -> Result<Uuid, DatabaseError> {
        let id = conversation.id.to_string();

        sqlx::query!(
            r#"
            INSERT INTO conversations (
                id, remote_id, message_count, ai_cache
            ) VALUES (?, ?, ?, ?)
            "#,
            id,
            conversation.remote_id,
            conversation.message_count,
            conversation.ai_cache,
        )
        .execute(&self.pool)
        .await
        .map_err(DatabaseError::ConnectionError)?;

        Ok(conversation.id)
    }

    async fn update(&self, conversation: &Conversation) -> Result<(), DatabaseError> {
        let id = conversation.id.to_string();

        sqlx::query!(
            r#"
            UPDATE conversations SET
                remote_id = ?, message_count = ?, ai_cache = ?,
                updated_at = CURRENT_TIMESTAMP
            WHERE id = ?
            "#,
            conversation.remote_id,
            conversation.message_count,
            conversation.ai_cache,
            id,
        )
        .execute(&self.pool)
        .await
        .map_err(DatabaseError::ConnectionError)?;

        Ok(())
    }

    async fn delete(&self, id: Uuid) -> Result<(), DatabaseError> {
        let id_str = id.to_string();
        sqlx::query!("DELETE FROM conversations WHERE id = ?", id_str)
            .execute(&self.pool)
            .await
            .map_err(DatabaseError::ConnectionError)?;
        Ok(())
    }

    async fn find_or_create_by_remote_id(
        &self,
        remote_id: &str,
    ) -> Result<Conversation, DatabaseError> {
        // Try to find existing conversation
        if let Some(conversation) = self.find_by_remote_id(remote_id).await? {
            return Ok(conversation);
        }

        // Create new conversation if not found
        let conversation = Conversation {
            id: Uuid::now_v7(),
            remote_id: remote_id.to_string(),
            message_count: 0,
            ai_cache: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        self.create(&conversation).await?;
        Ok(conversation)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::Database;

    async fn setup_test_db() -> SqlitePool {
        let db = Database::new_in_memory().await.unwrap();
        db.get_pool().clone()
    }

    #[tokio::test]
    async fn test_create_and_find_conversation() {
        let pool = setup_test_db().await;
        let repo = SqliteConversationRepository::new(pool);

        let conversation = Conversation {
            id: Uuid::now_v7(),
            remote_id: "test-remote-id".to_string(),
            message_count: 0,
            ai_cache: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let created_id = repo.create(&conversation).await.unwrap();
        assert_eq!(created_id, conversation.id);

        let found = repo.find_by_id(conversation.id).await.unwrap().unwrap();
        assert_eq!(found.id, conversation.id);
        assert_eq!(found.remote_id, conversation.remote_id);
    }

    #[tokio::test]
    async fn test_find_by_remote_id() {
        let pool = setup_test_db().await;
        let repo = SqliteConversationRepository::new(pool);

        let conversation = Conversation {
            id: Uuid::now_v7(),
            remote_id: "test-remote-123".to_string(),
            message_count: 0,
            ai_cache: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        repo.create(&conversation).await.unwrap();

        let found = repo
            .find_by_remote_id("test-remote-123")
            .await
            .unwrap()
            .unwrap();
        assert_eq!(found.id, conversation.id);
    }

    #[tokio::test]
    async fn test_find_or_create_by_remote_id() {
        let pool = setup_test_db().await;
        let repo = SqliteConversationRepository::new(pool);

        // First call should create
        let conv1 = repo
            .find_or_create_by_remote_id("auto-create-test")
            .await
            .unwrap();
        assert_eq!(conv1.remote_id, "auto-create-test");

        // Second call should find existing
        let conv2 = repo
            .find_or_create_by_remote_id("auto-create-test")
            .await
            .unwrap();
        assert_eq!(conv1.id, conv2.id);
    }

    #[tokio::test]
    async fn test_update_conversation() {
        let pool = setup_test_db().await;
        let repo = SqliteConversationRepository::new(pool);

        let mut conversation = Conversation {
            id: Uuid::now_v7(),
            remote_id: "update-test".to_string(),
            message_count: 0,
            ai_cache: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        repo.create(&conversation).await.unwrap();

        conversation.message_count = 5;
        conversation.ai_cache = Some("test cache".to_string());
        repo.update(&conversation).await.unwrap();

        let found = repo.find_by_id(conversation.id).await.unwrap().unwrap();
        assert_eq!(found.message_count, 5);
        assert_eq!(found.ai_cache, Some("test cache".to_string()));
    }

    #[tokio::test]
    async fn test_find_by_ids() {
        let pool = setup_test_db().await;
        let repo = SqliteConversationRepository::new(pool);

        let conv1 = Conversation {
            id: Uuid::now_v7(),
            remote_id: "multi-1".to_string(),
            message_count: 0,
            ai_cache: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let conv2 = Conversation {
            id: Uuid::now_v7(),
            remote_id: "multi-2".to_string(),
            message_count: 0,
            ai_cache: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        repo.create(&conv1).await.unwrap();
        repo.create(&conv2).await.unwrap();

        let found = repo.find_by_ids(vec![conv1.id, conv2.id]).await.unwrap();
        assert_eq!(found.len(), 2);
    }

    #[tokio::test]
    async fn test_delete_conversation() {
        let pool = setup_test_db().await;
        let repo = SqliteConversationRepository::new(pool);

        let conversation = Conversation {
            id: Uuid::now_v7(),
            remote_id: "delete-test".to_string(),
            message_count: 0,
            ai_cache: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        repo.create(&conversation).await.unwrap();
        repo.delete(conversation.id).await.unwrap();

        let found = repo.find_by_id(conversation.id).await.unwrap();
        assert!(found.is_none());
    }
}
