use super::error::{SyncError, SyncResult};
use super::storage::{FileStorage, LocalFileStorage, PathGenerator};
use sqlx::SqlitePool;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::time::sleep;
use uuid::Uuid;

const CLEANUP_BATCH_SIZE: i64 = 50;
const CLEANUP_INTERVAL_SECS: u64 = 60;

pub struct BackgroundCleanup {
    pool: SqlitePool,
    storage: Arc<LocalFileStorage>,
    active_cleanup: Arc<RwLock<bool>>,
    shutdown_tx: tokio::sync::broadcast::Sender<()>,
}

impl BackgroundCleanup {
    pub fn new(pool: SqlitePool, app_data_dir: String) -> Self {
        let (shutdown_tx, _) = tokio::sync::broadcast::channel(1);
        let cache_dir = std::path::PathBuf::from(&app_data_dir).join("attachments");
        let storage = Arc::new(LocalFileStorage::new(cache_dir));

        Self {
            pool,
            storage,
            active_cleanup: Arc::new(RwLock::new(false)),
            shutdown_tx,
        }
    }

    /// Start the background cleanup service
    pub async fn start(&self) -> SyncResult<()> {
        log::info!("[BackgroundCleanup] Starting background cleanup service");

        let pool = self.pool.clone();
        let storage = Arc::clone(&self.storage);
        let active_cleanup = Arc::clone(&self.active_cleanup);
        let mut shutdown_rx = self.shutdown_tx.subscribe();

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = shutdown_rx.recv() => {
                        log::info!("[BackgroundCleanup] Shutdown signal received");
                        break;
                    }
                    _ = sleep(Duration::from_secs(CLEANUP_INTERVAL_SECS)) => {
                        {
                            let is_active = active_cleanup.read().await;
                            if *is_active {
                                log::debug!("[BackgroundCleanup] Cleanup already running, skipping");
                                continue;
                            }
                        }

                        {
                            let mut is_active = active_cleanup.write().await;
                            *is_active = true;
                        }

                        if let Err(e) = Self::cleanup_deleted_emails(&pool, &storage).await {
                            log::error!("[BackgroundCleanup] Error during cleanup: {}", e);
                        }

                        {
                            let mut is_active = active_cleanup.write().await;
                            *is_active = false;
                        }
                    }
                }
            }
        });

        Ok(())
    }

    /// Stop the background cleanup service
    pub fn stop(&self) {
        log::info!("[BackgroundCleanup] Stopping background cleanup service");
        let _ = self.shutdown_tx.send(());
    }

    /// Clean up emails marked as deleted
    async fn cleanup_deleted_emails(
        pool: &SqlitePool,
        storage: &Arc<LocalFileStorage>,
    ) -> SyncResult<()> {
        let emails = sqlx::query!(
            r#"
            SELECT id, account_id, has_attachments
            FROM emails
            WHERE is_deleted = 1
            ORDER BY updated_at ASC
            LIMIT ?
            "#,
            CLEANUP_BATCH_SIZE
        )
        .fetch_all(pool)
        .await
        .map_err(|e| SyncError::DatabaseError(e.to_string()))?;

        if emails.is_empty() {
            return Ok(());
        }

        log::info!(
            "[BackgroundCleanup] Found {} deleted emails to clean up",
            emails.len()
        );

        let mut cleaned_count = 0;

        for email_record in emails {
            let email_id = Uuid::parse_str(&email_record.id)
                .map_err(|e| SyncError::DatabaseError(format!("Invalid email ID: {}", e)))?;

            log::debug!("[BackgroundCleanup] Cleaning up email {}", email_id);

            if email_record.has_attachments {
                if let Err(e) = Self::delete_email_attachments(pool, storage, email_id).await {
                    log::error!(
                        "[BackgroundCleanup] Failed to delete attachments for email {}: {}",
                        email_id,
                        e
                    );
                }
            }

            let email_id_str = email_id.to_string();
            sqlx::query!("DELETE FROM email_labels WHERE email_id = ?", email_id_str)
                .execute(pool)
                .await
                .map_err(|e| SyncError::DatabaseError(e.to_string()))?;

            sqlx::query!("DELETE FROM emails WHERE id = ?", email_id_str)
                .execute(pool)
                .await
                .map_err(|e| SyncError::DatabaseError(e.to_string()))?;

            cleaned_count += 1;
            log::debug!(
                "[BackgroundCleanup] Successfully cleaned up email {}",
                email_id
            );
        }

        if cleaned_count > 0 {
            log::info!(
                "[BackgroundCleanup] Successfully cleaned up {} deleted emails",
                cleaned_count
            );
        }

        Ok(())
    }

    /// Delete all attachment files for an email
    async fn delete_email_attachments(
        pool: &SqlitePool,
        storage: &Arc<LocalFileStorage>,
        email_id: Uuid,
    ) -> SyncResult<()> {
        let email_id_str = email_id.to_string();
        let attachments = sqlx::query!(
            r#"
            SELECT id, filename, cache_path, is_cached
            FROM attachments
            WHERE email_id = ?
            "#,
            email_id_str
        )
        .fetch_all(pool)
        .await
        .map_err(|e| SyncError::DatabaseError(e.to_string()))?;

        log::debug!(
            "[BackgroundCleanup] Deleting {} attachments for email {}",
            attachments.len(),
            email_id
        );

        for attachment in attachments {
            if attachment.is_cached {
                if let Some(cache_path) = &attachment.cache_path {
                    let path_buf = PathGenerator::cache_path_to_pathbuf(cache_path);

                    match storage.delete(&path_buf).await {
                        Ok(_) => {
                            log::debug!(
                                "[BackgroundCleanup] Deleted attachment file: {}",
                                cache_path
                            );
                        }
                        Err(e) => {
                            log::warn!(
                                "[BackgroundCleanup] Failed to delete attachment file {}: {}",
                                cache_path,
                                e
                            );
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Manually trigger cleanup (for testing or admin tools)
    pub async fn trigger_cleanup(&self) -> SyncResult<()> {
        log::info!("[BackgroundCleanup] Manual cleanup triggered");

        {
            let is_active = self.active_cleanup.read().await;
            if *is_active {
                return Err(SyncError::InvalidConfiguration(
                    "Cleanup is already running".to_string(),
                ));
            }
        }

        {
            let mut is_active = self.active_cleanup.write().await;
            *is_active = true;
        }

        let result = Self::cleanup_deleted_emails(&self.pool, &self.storage).await;

        {
            let mut is_active = self.active_cleanup.write().await;
            *is_active = false;
        }

        result
    }
}
