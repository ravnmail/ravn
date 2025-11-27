use crate::database::{error::DatabaseError, models::account::Account};
use async_trait::async_trait;
use sqlx::SqlitePool;
use uuid::Uuid;

#[async_trait]
pub trait AccountRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Account>, DatabaseError>;
    async fn find_all(&self) -> Result<Vec<Account>, DatabaseError>;
    async fn find_by_sync_enabled(&self) -> Result<Vec<Account>, DatabaseError>;
    async fn create(&self, account: &Account) -> Result<Uuid, DatabaseError>;
    async fn update(&self, account: &Account) -> Result<(), DatabaseError>;
    async fn delete(&self, id: Uuid) -> Result<(), DatabaseError>;
}

pub struct SqliteAccountRepository {
    pool: SqlitePool,
}

impl SqliteAccountRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AccountRepository for SqliteAccountRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Account>, DatabaseError> {
        sqlx::query_as::<_, Account>("SELECT * FROM accounts WHERE id = ?")
            .bind(id.to_string())
            .fetch_optional(&self.pool)
            .await
            .map_err(DatabaseError::ConnectionError)
    }

    async fn find_all(&self) -> Result<Vec<Account>, DatabaseError> {
        sqlx::query_as::<_, Account>("SELECT * FROM accounts ORDER BY name")
            .fetch_all(&self.pool)
            .await
            .map_err(DatabaseError::ConnectionError)
    }

    async fn find_by_sync_enabled(&self) -> Result<Vec<Account>, DatabaseError> {
        sqlx::query_as::<_, Account>(
            r#"
            SELECT * FROM accounts
            WHERE json_extract(settings, '$.sync_enabled') = 1
            ORDER BY name
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(DatabaseError::ConnectionError)
    }

    async fn create(&self, account: &Account) -> Result<Uuid, DatabaseError> {
        let account_type = account.account_type;
        let id = account.id.to_string();
        sqlx::query!(
            r#"
            INSERT INTO accounts (id, name, email, account_type, settings)
            VALUES (?, ?, ?, ?, ?)
            "#,
            id,
            account.name,
            account.email,
            account_type,
            account.settings
        )
        .execute(&self.pool)
        .await
        .map_err(DatabaseError::ConnectionError)?;

        Ok(account.id)
    }

    async fn update(&self, account: &Account) -> Result<(), DatabaseError> {
        let account_type = account.account_type;
        let id = account.id.to_string();
        sqlx::query!(
            r#"
            UPDATE accounts
            SET name = ?, email = ?, account_type = ?, settings = ?, updated_at = CURRENT_TIMESTAMP
            WHERE id = ?
            "#,
            account.name,
            account.email,
            account_type,
            account.settings,
            id
        )
        .execute(&self.pool)
        .await
        .map_err(DatabaseError::ConnectionError)?;

        Ok(())
    }

    async fn delete(&self, id: Uuid) -> Result<(), DatabaseError> {
        let id_str = id.to_string();
        sqlx::query!("DELETE FROM accounts WHERE id = ?", id_str)
            .execute(&self.pool)
            .await
            .map_err(DatabaseError::ConnectionError)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::models::account::AccountType;
    use serde_json::json;
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
            CREATE TABLE IF NOT EXISTS accounts (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                email TEXT NOT NULL,
                account_type TEXT NOT NULL CHECK (account_type IN ('gmail', 'office365', 'apple', 'imap')),
                settings TEXT NOT NULL,
                created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
            );
            "#,
        )
        .execute(pool)
        .await
        .expect("Failed to create test schema");
    }

    /// Helper function to create a test account
    fn create_test_account() -> Account {
        Account {
            id: Uuid::now_v7(),
            name: "Test Account".to_string(),
            email: "test@example.com".to_string(),
            account_type: AccountType::Gmail,
            settings: json!({
                "oauth_token": "test_token",
                "refresh_token": "test_refresh_token"
            }),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }
    }

    #[tokio::test]
    async fn test_create_account() {
        let pool = create_test_pool().await;
        setup_test_schema(&pool).await;

        let repository = SqliteAccountRepository::new(pool);
        let test_account = create_test_account();

        let result = repository.create(&test_account).await;
        assert!(result.is_ok());

        let id = result.unwrap();
        assert_eq!(id, test_account.id);
    }

    #[tokio::test]
    async fn test_find_by_id() {
        let pool = create_test_pool().await;
        setup_test_schema(&pool).await;

        let repository = SqliteAccountRepository::new(pool);
        let test_account = create_test_account();

        // Create test account
        let id = repository.create(&test_account).await.unwrap();

        // Test finding the account
        let result = repository.find_by_id(id).await.unwrap();
        assert!(result.is_some());

        let found_account = result.unwrap();
        assert_eq!(found_account.name, test_account.name);
        assert_eq!(found_account.email, test_account.email);
        assert_eq!(found_account.account_type, test_account.account_type);
    }

    #[tokio::test]
    async fn test_find_all() {
        let pool = create_test_pool().await;
        setup_test_schema(&pool).await;

        let repository = SqliteAccountRepository::new(pool);

        // Create multiple test accounts
        let mut accounts = vec![];
        for i in 1..=3 {
            let mut account = create_test_account();
            account.name = format!("Test Account {}", i);
            account.email = format!("test{}@example.com", i);
            repository.create(&account).await.unwrap();
            accounts.push(account);
        }

        // Test finding all accounts
        let result = repository.find_all().await.unwrap();
        assert_eq!(result.len(), 3);

        // Verify accounts are returned in correct order (by name)
        for (i, account) in result.iter().enumerate() {
            assert_eq!(account.name, format!("Test Account {}", i + 1));
            assert_eq!(account.email, format!("test{}@example.com", i + 1));
        }
    }

    #[tokio::test]
    async fn test_update_account() {
        let pool = create_test_pool().await;
        setup_test_schema(&pool).await;

        let repository = SqliteAccountRepository::new(pool);
        let mut test_account = create_test_account();

        // Create test account
        let id = repository.create(&test_account).await.unwrap();

        // Update account
        test_account.name = "Updated Name".to_string();
        test_account.email = "updated@example.com".to_string();
        test_account.account_type = AccountType::Office365;
        test_account.settings = json!({
            "new_setting": "new_value"
        });

        let update_result = repository.update(&test_account).await;
        assert!(update_result.is_ok());

        // Verify update
        let updated = repository.find_by_id(id).await.unwrap().unwrap();
        assert_eq!(updated.name, "Updated Name");
        assert_eq!(updated.email, "updated@example.com");
        assert_eq!(updated.account_type, AccountType::Office365);
        assert_eq!(updated.settings, test_account.settings);
    }

    #[tokio::test]
    async fn test_delete_account() {
        let pool = create_test_pool().await;
        setup_test_schema(&pool).await;

        let repository = SqliteAccountRepository::new(pool);
        let test_account = create_test_account();

        // Create test account
        let id = repository.create(&test_account).await.unwrap();

        // Delete account
        let delete_result = repository.delete(id).await;
        assert!(delete_result.is_ok());

        // Verify deletion
        let find_result = repository.find_by_id(id).await.unwrap();
        assert!(find_result.is_none());
    }
}
