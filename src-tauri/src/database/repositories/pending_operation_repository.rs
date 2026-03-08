use crate::database::error::DatabaseError;
use crate::database::models::pending_operation::PendingOperation;
use chrono::{Duration, Utc};
use sqlx::SqlitePool;
use uuid::Uuid;

pub struct SqlitePendingOperationRepository {
    pool: SqlitePool,
}

impl SqlitePendingOperationRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Create a new pending operation
    pub async fn create(&self, op: &PendingOperation) -> Result<Uuid, DatabaseError> {
        let id = op.id.to_string();
        let account_id = op.account_id.to_string();
        let email_id = op.email_id.map(|id| id.to_string());
        let folder_id = op.folder_id.map(|id| id.to_string());

        sqlx::query!(
            r#"
            INSERT INTO pending_operations (
                id, account_id, email_id, folder_id, operation_type,
                payload, status, retry_count, max_retries, error_message,
                created_at, completed_at, expires_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            id,
            account_id,
            email_id,
            folder_id,
            op.operation_type,
            op.payload,
            op.status,
            op.retry_count,
            op.max_retries,
            op.error_message,
            op.created_at,
            op.completed_at,
            op.expires_at,
        )
        .execute(&self.pool)
        .await
        .map_err(DatabaseError::ConnectionError)?;

        Ok(op.id)
    }

    /// Find all pending operations for an account, ordered by creation time (FIFO)
    pub async fn find_pending_by_account(
        &self,
        account_id: Uuid,
    ) -> Result<Vec<PendingOperation>, DatabaseError> {
        let account_id_str = account_id.to_string();
        let now = Utc::now();

        sqlx::query_as::<_, PendingOperation>(
            r#"
            SELECT id, account_id, email_id, folder_id, operation_type,
                   payload, status, retry_count, max_retries, error_message,
                   created_at, completed_at, expires_at
            FROM pending_operations
            WHERE account_id = ? AND status = 'pending'
              AND (expires_at IS NULL OR expires_at > ?)
            ORDER BY created_at ASC
            "#,
        )
        .bind(account_id_str)
        .bind(now)
        .fetch_all(&self.pool)
        .await
        .map_err(DatabaseError::ConnectionError)
    }

    /// Find all pending operations for a specific email
    pub async fn find_pending_for_email(
        &self,
        email_id: Uuid,
    ) -> Result<Vec<PendingOperation>, DatabaseError> {
        let email_id_str = email_id.to_string();

        sqlx::query_as::<_, PendingOperation>(
            r#"
            SELECT id, account_id, email_id, folder_id, operation_type,
                   payload, status, retry_count, max_retries, error_message,
                   created_at, completed_at, expires_at
            FROM pending_operations
            WHERE email_id = ? AND status IN ('pending', 'in_progress')
            ORDER BY created_at ASC
            "#,
        )
        .bind(email_id_str)
        .fetch_all(&self.pool)
        .await
        .map_err(DatabaseError::ConnectionError)
    }

    /// Mark an operation as in progress
    pub async fn mark_in_progress(&self, id: Uuid) -> Result<(), DatabaseError> {
        let id_str = id.to_string();

        sqlx::query!(
            "UPDATE pending_operations SET status = 'in_progress' WHERE id = ?",
            id_str,
        )
        .execute(&self.pool)
        .await
        .map_err(DatabaseError::ConnectionError)?;

        Ok(())
    }

    /// Mark an operation as completed
    pub async fn mark_completed(&self, id: Uuid) -> Result<(), DatabaseError> {
        let id_str = id.to_string();
        let now = Utc::now();

        sqlx::query!(
            "UPDATE pending_operations SET status = 'completed', completed_at = ? WHERE id = ?",
            now,
            id_str,
        )
        .execute(&self.pool)
        .await
        .map_err(DatabaseError::ConnectionError)?;

        Ok(())
    }

    /// Mark an operation as failed with error message
    pub async fn mark_failed(&self, id: Uuid, error: &str) -> Result<(), DatabaseError> {
        let id_str = id.to_string();

        sqlx::query!(
            r#"
            UPDATE pending_operations
            SET status = 'failed', error_message = ?, retry_count = retry_count + 1
            WHERE id = ?
            "#,
            error,
            id_str,
        )
        .execute(&self.pool)
        .await
        .map_err(DatabaseError::ConnectionError)?;

        Ok(())
    }

    /// Cancel an operation (for undo support)
    pub async fn cancel(&self, id: Uuid) -> Result<bool, DatabaseError> {
        let id_str = id.to_string();

        let result = sqlx::query!(
            r#"
            UPDATE pending_operations
            SET status = 'cancelled'
            WHERE id = ? AND status = 'pending'
            "#,
            id_str,
        )
        .execute(&self.pool)
        .await
        .map_err(DatabaseError::ConnectionError)?;

        Ok(result.rows_affected() > 0)
    }

    /// Cancel all pending operations for an email of a specific type
    pub async fn cancel_by_email_and_type(
        &self,
        email_id: Uuid,
        operation_type: &str,
    ) -> Result<u64, DatabaseError> {
        let email_id_str = email_id.to_string();

        let result = sqlx::query!(
            r#"
            UPDATE pending_operations
            SET status = 'cancelled'
            WHERE email_id = ? AND operation_type = ? AND status IN ('pending', 'in_progress')
            "#,
            email_id_str,
            operation_type,
        )
        .execute(&self.pool)
        .await
        .map_err(DatabaseError::ConnectionError)?;

        Ok(result.rows_affected())
    }

    /// Reset a failed operation back to pending for retry
    pub async fn reset_for_retry(&self, id: Uuid) -> Result<(), DatabaseError> {
        let id_str = id.to_string();

        sqlx::query!(
            r#"
            UPDATE pending_operations
            SET status = 'pending', error_message = NULL
            WHERE id = ? AND status = 'failed' AND retry_count < max_retries
            "#,
            id_str,
        )
        .execute(&self.pool)
        .await
        .map_err(DatabaseError::ConnectionError)?;

        Ok(())
    }

    /// Count pending operations for an account
    pub async fn count_pending(&self, account_id: Uuid) -> Result<i64, DatabaseError> {
        let account_id_str = account_id.to_string();

        let count = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM pending_operations WHERE account_id = ? AND status IN ('pending', 'in_progress')",
            account_id_str,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(DatabaseError::ConnectionError)?;

        Ok(count)
    }

    /// Delete completed operations older than the given duration
    pub async fn delete_completed_older_than(&self, days: i64) -> Result<u64, DatabaseError> {
        let cutoff = Utc::now() - Duration::days(days);

        let result = sqlx::query!(
            r#"
            DELETE FROM pending_operations
            WHERE status IN ('completed', 'cancelled') AND created_at < ?
            "#,
            cutoff,
        )
        .execute(&self.pool)
        .await
        .map_err(DatabaseError::ConnectionError)?;

        Ok(result.rows_affected())
    }

    /// Find all accounts that have pending operations
    pub async fn find_accounts_with_pending_ops(&self) -> Result<Vec<Uuid>, DatabaseError> {
        let now = Utc::now();

        let records = sqlx::query_scalar!(
            r#"
            SELECT DISTINCT account_id
            FROM pending_operations
            WHERE status = 'pending'
              AND (expires_at IS NULL OR expires_at > ?)
            "#,
            now,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(DatabaseError::ConnectionError)?;

        Ok(records
            .into_iter()
            .filter_map(|s| Uuid::parse_str(&s).ok())
            .collect())
    }
}
