use crate::database::{error::DatabaseError, models::folder::Folder};
use async_trait::async_trait;
use sqlx::SqlitePool;
use uuid::Uuid;

#[async_trait]
pub trait FolderRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Folder>, DatabaseError>;
    async fn get_all(&self) -> Result<Vec<Folder>, DatabaseError>;
    async fn find_by_account(&self, account_id: Uuid) -> Result<Vec<Folder>, DatabaseError>;
    async fn find_by_parent(&self, parent_id: Uuid) -> Result<Vec<Folder>, DatabaseError>;
    async fn find_by_type(
        &self,
        account_id: Uuid,
        folder_type: &str,
    ) -> Result<Option<Folder>, DatabaseError>;
    async fn create(&self, folder: &Folder) -> Result<Uuid, DatabaseError>;
    async fn update(&self, folder: &Folder) -> Result<(), DatabaseError>;
    async fn delete(&self, id: Uuid) -> Result<(), DatabaseError>;
}

pub struct SqliteFolderRepository {
    pool: SqlitePool,
}

impl SqliteFolderRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl FolderRepository for SqliteFolderRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Folder>, DatabaseError> {
        sqlx::query_as::<_, Folder>("SELECT * FROM folders WHERE id = ?")
            .bind(id.to_string())
            .fetch_optional(&self.pool)
            .await
            .map_err(DatabaseError::ConnectionError)
    }

    async fn find_by_account(&self, account_id: Uuid) -> Result<Vec<Folder>, DatabaseError> {
        sqlx::query_as::<_, Folder>(
            "SELECT * FROM folders WHERE account_id = ? ORDER BY sort_order",
        )
        .bind(account_id.to_string())
        .fetch_all(&self.pool)
        .await
        .map_err(DatabaseError::ConnectionError)
    }

    async fn find_by_parent(&self, parent_id: Uuid) -> Result<Vec<Folder>, DatabaseError> {
        sqlx::query_as::<_, Folder>("SELECT * FROM folders WHERE parent_id = ? ORDER BY sort_order")
            .bind(parent_id.to_string())
            .fetch_all(&self.pool)
            .await
            .map_err(DatabaseError::ConnectionError)
    }
    async fn get_all(&self) -> Result<Vec<Folder>, DatabaseError> {
        sqlx::query_as::<_, Folder>("SELECT * FROM folders ORDER BY folders.account_id, sort_order")
            .fetch_all(&self.pool)
            .await
            .map_err(DatabaseError::ConnectionError)
    }

    async fn find_by_type(
        &self,
        account_id: Uuid,
        folder_type: &str,
    ) -> Result<Option<Folder>, DatabaseError> {
        sqlx::query_as::<_, Folder>(
            "SELECT * FROM folders WHERE account_id = ? AND folder_type = ? LIMIT 1",
        )
        .bind(account_id.to_string())
        .bind(folder_type)
        .fetch_optional(&self.pool)
        .await
        .map_err(DatabaseError::ConnectionError)
    }

    async fn create(&self, folder: &Folder) -> Result<Uuid, DatabaseError> {
        let id = folder.id.to_string();
        let account_id = folder.account_id.to_string();
        let parent_id = folder.parent_id.map(|pid| pid.to_string());
        let folder_type = folder.folder_type;
        let settings_json = serde_json::to_string(&folder.settings)?;

        sqlx::query!(
            r#"
            INSERT INTO folders (id, account_id, name, folder_type, remote_id, color, icon, sort_order, parent_id, settings)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            id,
            account_id,
            folder.name,
            folder_type,
            folder.remote_id,
            folder.color,
            folder.icon,
            folder.sort_order,
            parent_id,
            settings_json
        )
        .execute(&self.pool)
        .await
        .map_err(DatabaseError::ConnectionError)?;

        Ok(folder.id)
    }

    async fn update(&self, folder: &Folder) -> Result<(), DatabaseError> {
        let id = folder.id.to_string();
        let parent_id = folder.parent_id.map(|pid| pid.to_string());
        let folder_type = folder.folder_type;
        let settings_json = serde_json::to_string(&folder.settings)?;

        sqlx::query!(
            r#"
            UPDATE folders
            SET name = ?, folder_type = ?, remote_id = ?, color = ?,
                icon = ?, sort_order = ?, parent_id = ?, settings = ?,
                expanded = ?, hidden = ?
            WHERE id = ?
            "#,
            folder.name,
            folder_type,
            folder.remote_id,
            folder.color,
            folder.icon,
            folder.sort_order,
            parent_id,
            settings_json,
            folder.expanded,
            folder.hidden,
            id
        )
        .execute(&self.pool)
        .await
        .map_err(DatabaseError::ConnectionError)?;

        Ok(())
    }

    async fn delete(&self, id: Uuid) -> Result<(), DatabaseError> {
        let id = id.to_string();
        sqlx::query!("DELETE FROM folders WHERE id = ?", id)
            .execute(&self.pool)
            .await
            .map_err(DatabaseError::ConnectionError)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::models::folder::FolderType;
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
            CREATE TABLE IF NOT EXISTS folders (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                account_id INTEGER NOT NULL,
                name TEXT NOT NULL,
                folder_type TEXT NOT NULL CHECK (folder_type IN ('archive', 'inbox', 'sent', 'draft', 'trash', 'spam', 'custom')),
                remote_id TEXT,
                color TEXT,
                icon TEXT,
                sort_order INTEGER NOT NULL DEFAULT 0,
                parent_id INTEGER REFERENCES folders(id)
            );
            "#,
        )
        .execute(pool)
        .await
        .expect("Failed to create test schema");
    }

    /// Helper function to create a test folder
    fn create_test_folder(account_id: Uuid) -> Folder {
        use crate::database::models::folder::FolderSettings;
        use chrono::Utc;

        Folder {
            id: Uuid::now_v7(),
            account_id,
            name: "Test Folder".to_string(),
            folder_type: FolderType::Custom,
            remote_id: Some("remote_123".to_string()),
            color: Some("#FF0000".to_string()),
            icon: Some("📁".to_string()),
            sort_order: 0,
            expanded: false,
            hidden: false,
            parent_id: None,
            settings: FolderSettings::default(),
            synced_at: Utc::now(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[tokio::test]
    async fn test_create_folder() {
        let pool = create_test_pool().await;
        setup_test_schema(&pool).await;

        let repository = SqliteFolderRepository::new(pool);
        let test_folder = create_test_folder(Uuid::now_v7());

        let result = repository.create(&test_folder).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_find_by_id() {
        let pool = create_test_pool().await;
        setup_test_schema(&pool).await;

        let repository = SqliteFolderRepository::new(pool);
        let test_folder = create_test_folder(Uuid::now_v7());
        let id = test_folder.id;

        // Create test folder
        repository.create(&test_folder).await.unwrap();

        // Test finding the folder
        let result = repository.find_by_id(id).await.unwrap();
        assert!(result.is_some());

        let found_folder = result.unwrap();
        assert_eq!(found_folder.name, test_folder.name);
        assert_eq!(found_folder.folder_type, test_folder.folder_type);
        assert_eq!(found_folder.color, test_folder.color);
    }

    #[tokio::test]
    async fn test_find_by_account() {
        let pool = create_test_pool().await;
        setup_test_schema(&pool).await;

        let repository = SqliteFolderRepository::new(pool);
        let account_id = Uuid::now_v7();

        // Create multiple test folders
        for i in 1..=3 {
            let mut folder = create_test_folder(account_id);
            folder.name = format!("Test Folder {}", i);
            folder.sort_order = i;
            repository.create(&folder).await.unwrap();
        }

        // Create a folder for a different account
        let other_folder = create_test_folder(Uuid::now_v7());
        repository.create(&other_folder).await.unwrap();

        // Test finding all folders for account
        let result = repository.find_by_account(account_id).await.unwrap();
        assert_eq!(result.len(), 3);

        // Verify folders are returned in correct order
        for (i, folder) in result.iter().enumerate() {
            assert_eq!(folder.name, format!("Test Folder {}", i + 1));
            assert_eq!(folder.sort_order, (i + 1) as i32);
        }
    }

    #[tokio::test]
    async fn test_find_by_parent() {
        let pool = create_test_pool().await;
        setup_test_schema(&pool).await;

        let repository = SqliteFolderRepository::new(pool);
        let account_id = Uuid::now_v7();

        // Create parent folder
        let mut parent_folder = create_test_folder(account_id);
        parent_folder.name = "Parent Folder".to_string();
        let parent_id = parent_folder.id;
        repository.create(&parent_folder).await.unwrap();

        // Create child folders
        for i in 1..=3 {
            let mut folder = create_test_folder(account_id);
            folder.name = format!("Child Folder {}", i);
            folder.parent_id = Some(parent_id);
            folder.sort_order = i;
            repository.create(&folder).await.unwrap();
        }

        // Test finding child folders
        let result = repository.find_by_parent(parent_id).await.unwrap();
        assert_eq!(result.len(), 3);

        // Verify child folders
        for (i, folder) in result.iter().enumerate() {
            assert_eq!(folder.name, format!("Child Folder {}", i + 1));
            assert_eq!(folder.parent_id, Some(parent_id));
        }
    }

    #[tokio::test]
    async fn test_update_folder() {
        let pool = create_test_pool().await;
        setup_test_schema(&pool).await;

        let repository = SqliteFolderRepository::new(pool);
        let mut test_folder = create_test_folder(Uuid::now_v7());
        let id = test_folder.id;

        // Create test folder
        repository.create(&test_folder).await.unwrap();

        // Update folder
        test_folder.name = "Updated Folder".to_string();
        test_folder.color = Some("#00FF00".to_string());
        test_folder.icon = Some("📂".to_string());
        test_folder.sort_order = 5;

        let update_result = repository.update(&test_folder).await;
        assert!(update_result.is_ok());

        // Verify update
        let updated = repository.find_by_id(id).await.unwrap().unwrap();
        assert_eq!(updated.name, "Updated Folder");
        assert_eq!(updated.color, Some("#00FF00".to_string()));
        assert_eq!(updated.icon, Some("📂".to_string()));
        assert_eq!(updated.sort_order, 5);
    }

    #[tokio::test]
    async fn test_delete_folder() {
        let pool = create_test_pool().await;
        setup_test_schema(&pool).await;

        let repository = SqliteFolderRepository::new(pool);
        let test_folder = create_test_folder(Uuid::now_v7());
        let id = test_folder.id;

        // Create test folder
        repository.create(&test_folder).await.unwrap();

        // Delete folder
        let delete_result = repository.delete(id).await;
        assert!(delete_result.is_ok());

        // Verify deletion
        let find_result = repository.find_by_id(id).await.unwrap();
        assert!(find_result.is_none());
    }

    #[tokio::test]
    async fn test_system_folders() {
        let pool = create_test_pool().await;
        setup_test_schema(&pool).await;

        let repository = SqliteFolderRepository::new(pool);
        let account_id = Uuid::now_v7();

        // Create one of each system folder type
        let folder_types = vec![
            FolderType::Archive,
            FolderType::Inbox,
            FolderType::Sent,
            FolderType::Draft,
            FolderType::Trash,
            FolderType::Spam,
        ];

        for folder_type in folder_types {
            let mut folder = create_test_folder(account_id);
            folder.folder_type = folder_type;
            folder.name = format!("{:?}", folder_type);

            let result = repository.create(&folder).await;
            assert!(result.is_ok());
        }

        // Verify all system folders were created
        let folders = repository.find_by_account(account_id).await.unwrap();
        assert_eq!(folders.len(), 6);

        // Check each system folder type exists
        for folder in folders {
            match folder.folder_type {
                FolderType::Custom => panic!("Unexpected custom folder"),
                _ => assert_eq!(folder.name, format!("{:?}", folder.folder_type)),
            }
        }
    }

    #[tokio::test]
    async fn test_nested_folders() {
        let pool = create_test_pool().await;
        setup_test_schema(&pool).await;

        let repository = SqliteFolderRepository::new(pool);
        let account_id = Uuid::now_v7();

        // Create root folder
        let mut root = create_test_folder(account_id);
        root.name = "Root".to_string();
        let root_id = root.id;
        repository.create(&root).await.unwrap();

        // Create parent folder
        let mut parent = create_test_folder(account_id);
        parent.name = "Parent".to_string();
        parent.parent_id = Some(root_id);
        let parent_id = parent.id;
        repository.create(&parent).await.unwrap();

        // Create child folder
        let mut child = create_test_folder(account_id);
        child.name = "Child".to_string();
        child.parent_id = Some(parent_id);
        let child_id = child.id;
        repository.create(&child).await.unwrap();

        // Verify folder hierarchy
        let root_children = repository.find_by_parent(root_id).await.unwrap();
        assert_eq!(root_children.len(), 1);
        assert_eq!(root_children[0].name, "Parent");

        let parent_children = repository.find_by_parent(parent_id).await.unwrap();
        assert_eq!(parent_children.len(), 1);
        assert_eq!(parent_children[0].name, "Child");

        let child_children = repository.find_by_parent(child_id).await.unwrap();
        assert_eq!(child_children.len(), 0);
    }

    #[tokio::test]
    async fn test_error_handling() {
        let pool = create_test_pool().await;
        setup_test_schema(&pool).await;

        let repository = SqliteFolderRepository::new(pool);

        // Test finding non-existent folder
        let result = repository.find_by_id(Uuid::now_v7()).await.unwrap();
        assert!(result.is_none());

        // Test finding children of non-existent parent
        let result = repository.find_by_parent(Uuid::now_v7()).await.unwrap();
        assert!(result.is_empty());

        // Test invalid folder type
        let result = sqlx::query(
            "INSERT INTO folders (account_id, name, folder_type) VALUES (?, 'Test', 'invalid')",
        )
        .bind(Uuid::now_v7().to_string())
        .execute(&repository.pool)
        .await;
        assert!(result.is_err());
    }
}
