use crate::database::models::pending_operation::PendingOperationType;
use crate::database::repositories::{
    FolderRepository, SqliteFolderRepository, SqlitePendingOperationRepository,
};
use crate::sync::error::{SyncError, SyncResult};
use crate::sync::types::{SyncDiff, SyncEmail, SyncFolder};
use chrono::Utc;
use sqlx::SqlitePool;
use tauri::Emitter;
use uuid::Uuid;

/// Result of a reconciliation pass
#[derive(Debug, Default)]
pub struct ReconciliationResult {
    pub added: usize,
    pub modified: usize,
    pub deleted: usize,
    pub conflicts_resolved: usize,
}

/// Reconciles provider state with local state, handling pending operations and conflicts.
///
/// Design principles:
/// - Provider is authoritative: provider changes always supersede local changes
/// - Pending operations that conflict with provider state are cancelled
/// - Deletions are tombstoned with timestamps rather than immediately removed
pub struct Reconciler {
    pool: SqlitePool,
}

impl Reconciler {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Reconcile a provider diff against local state
    ///
    /// For each item in the diff:
    /// - Added: Check for conflicting pending ops, upsert email
    /// - Modified: Check for conflicting pending ops, update email
    /// - Deleted: Tombstone the email, cancel conflicting pending ops
    pub async fn reconcile_diff(
        &self,
        account_id: Uuid,
        folder: &SyncFolder,
        diff: &SyncDiff,
        email_sync: &super::email_sync::EmailSync,
    ) -> SyncResult<ReconciliationResult> {
        let mut result = ReconciliationResult::default();
        let pending_repo = SqlitePendingOperationRepository::new(self.pool.clone());

        let folder_repo = SqliteFolderRepository::new(self.pool.clone());

        // Process added emails
        for email in &diff.added {
            let conflicts = self.resolve_conflicts_for_email(email, &pending_repo).await;
            result.conflicts_resolved += conflicts;

            match email_sync.upsert_email(email, account_id, "synced").await {
                Ok((_email_id, _inline_attachment_ids, is_new, db_email)) => {
                    result.added += 1;

                    if is_new {
                        if let Err(e) = self
                            .notify_for_new_email(&folder_repo, email_sync, email, &db_email)
                            .await
                        {
                            log::warn!(
                                "[Reconciler] Failed to notify for new email {}: {}",
                                email.remote_id,
                                e
                            );
                        }
                    }
                }
                Err(e) => {
                    log::error!(
                        "[Reconciler] Failed to upsert added email {}: {}",
                        email.remote_id,
                        e
                    );
                }
            }
        }

        // Process modified emails
        for email in &diff.modified {
            let conflicts = self.resolve_conflicts_for_email(email, &pending_repo).await;
            result.conflicts_resolved += conflicts;

            match email_sync.upsert_email(email, account_id, "synced").await {
                Ok(_) => result.modified += 1,
                Err(e) => {
                    log::error!(
                        "[Reconciler] Failed to upsert modified email {}: {}",
                        email.remote_id,
                        e
                    );
                }
            }
        }

        // Process deleted emails — tombstone with timestamp
        let folder_id_str = folder.id.unwrap().to_string();
        let now = Utc::now();

        for remote_id in &diff.deleted {
            // Cancel any pending ops for this email
            if let Ok(Some(email_id)) = self
                .find_email_id_by_remote_id(remote_id, &folder_id_str)
                .await
            {
                let pending_ops = pending_repo
                    .find_pending_for_email(email_id)
                    .await
                    .unwrap_or_default();

                for op in &pending_ops {
                    log::info!(
                        "[Reconciler] Cancelling conflicting pending op {} ({}) for deleted email {}",
                        op.id,
                        op.operation_type,
                        remote_id
                    );
                    let _ = pending_repo.cancel(op.id).await;
                    result.conflicts_resolved += 1;
                }
            }

            // Tombstone: set is_deleted, deleted_at, deletion_source
            sqlx::query!(
                r#"
                UPDATE emails
                SET is_deleted = 1,
                    deleted_at = ?,
                    deletion_source = 'provider',
                    updated_at = CURRENT_TIMESTAMP
                WHERE folder_id = ? AND remote_id = ? AND is_deleted = 0
                "#,
                now,
                folder_id_str,
                remote_id
            )
            .execute(&self.pool)
            .await
            .map_err(|e| SyncError::DatabaseError(e.to_string()))?;

            result.deleted += 1;
        }

        log::info!(
            "[Reconciler] Reconciliation complete: +{} ~{} -{} (conflicts: {})",
            result.added,
            result.modified,
            result.deleted,
            result.conflicts_resolved
        );

        Ok(result)
    }

    /// Check for and resolve conflicts between a provider email and pending local operations.
    /// Provider always wins — conflicting pending ops are cancelled.
    async fn resolve_conflicts_for_email(
        &self,
        email: &SyncEmail,
        pending_repo: &SqlitePendingOperationRepository,
    ) -> usize {
        let email_id = match email.id {
            Some(id) => id,
            None => {
                // Try to find existing email by remote_id
                let folder_id_str = email.folder_id.to_string();
                match self
                    .find_email_id_by_remote_id(&email.remote_id, &folder_id_str)
                    .await
                {
                    Ok(Some(id)) => id,
                    _ => return 0,
                }
            }
        };

        let pending_ops = match pending_repo.find_pending_for_email(email_id).await {
            Ok(ops) => ops,
            Err(_) => return 0,
        };

        let mut cancelled = 0;
        for op in &pending_ops {
            let op_type = PendingOperationType::from_str(&op.operation_type);

            // Check if the provider state already reflects this operation
            let is_superseded = match op_type {
                Some(PendingOperationType::MarkRead) => email.flags.contains(&"\\Seen".to_string()),
                Some(PendingOperationType::MarkUnread) => {
                    !email.flags.contains(&"\\Seen".to_string())
                }
                Some(PendingOperationType::Flag) => email.flags.contains(&"\\Flagged".to_string()),
                Some(PendingOperationType::Unflag) => {
                    !email.flags.contains(&"\\Flagged".to_string())
                }
                _ => false,
            };

            if is_superseded {
                log::debug!(
                    "[Reconciler] Pending op {} ({}) superseded by provider state, cancelling",
                    op.id,
                    op.operation_type
                );
                let _ = pending_repo.cancel(op.id).await;
                cancelled += 1;
            }
        }

        cancelled
    }

    async fn notify_for_new_email(
        &self,
        folder_repo: &SqliteFolderRepository,
        email_sync: &super::email_sync::EmailSync,
        email: &SyncEmail,
        db_email: &crate::database::models::email::Email,
    ) -> SyncResult<()> {
        let folder = folder_repo
            .find_by_id(email.folder_id)
            .await
            .map_err(|e| SyncError::DatabaseError(e.to_string()))?
            .ok_or_else(|| {
                SyncError::DatabaseError(format!("Folder not found for email: {}", email.folder_id))
            })?;

        let app_handle = email_sync
            .app_handle
            .as_ref()
            .ok_or_else(|| SyncError::DatabaseError("App handle not available".to_string()))?
            .clone();

        if let Some(notification_service) = &email_sync.notification_service {
            if let Err(e) = notification_service
                .notify_incoming_email(folder.id, folder.folder_type, db_email)
                .await
            {
                log::warn!(
                    "[Reconciler] Notification service failed for new email {}: {}",
                    email.remote_id,
                    e
                );
            }
        }

        if let Err(e) = app_handle.emit("email:created", db_email.clone()) {
            log::warn!(
                "[Reconciler] Failed to emit email:created for {}: {}",
                email.remote_id,
                e
            );
        }

        Ok(())
    }

    /// Find an email's UUID by its remote_id and folder_id
    async fn find_email_id_by_remote_id(
        &self,
        remote_id: &str,
        folder_id_str: &str,
    ) -> SyncResult<Option<Uuid>> {
        let record = sqlx::query!(
            "SELECT id FROM emails WHERE folder_id = ? AND remote_id = ? LIMIT 1",
            folder_id_str,
            remote_id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| SyncError::DatabaseError(e.to_string()))?;

        match record {
            Some(r) => Uuid::parse_str(&r.id)
                .map(Some)
                .map_err(|e| SyncError::DatabaseError(e.to_string())),
            None => Ok(None),
        }
    }
}
