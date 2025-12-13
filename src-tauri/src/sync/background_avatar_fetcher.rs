use super::error::{SyncError, SyncResult};
use crate::database::repositories::{AccountRepository, ContactRepository, RepositoryFactory};
use crate::services::avatar_service::{AvatarProvider, AvatarService};
use sqlx::SqlitePool;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use uuid::Uuid;

const FETCH_BATCH_SIZE: i64 = 20;
const FETCH_INTERVAL_SECS: u64 = 30;

pub struct BackgroundAvatarFetcher {
    pool: SqlitePool,
    avatar_service: Arc<AvatarService>,
    shutdown_tx: tokio::sync::broadcast::Sender<()>,
}

impl BackgroundAvatarFetcher {
    pub fn new(pool: SqlitePool, app_data_dir: String, providers: Option<Vec<String>>) -> Self {
        let (shutdown_tx, _) = tokio::sync::broadcast::channel(1);

        let avatar_providers = providers
            .map(|provider_list| {
                provider_list
                    .iter()
                    .filter_map(|s| AvatarProvider::from_str(s))
                    .collect::<Vec<_>>()
            })
            .filter(|list| !list.is_empty());

        let cache_dir = std::path::PathBuf::from(&app_data_dir);
        let avatar_service = Arc::new(AvatarService::new(cache_dir, avatar_providers));

        log::info!(
            "[BackgroundAvatarFetcher] Initialized with providers: {:?}",
            avatar_service
                .providers
                .iter()
                .map(|p| p.as_str())
                .collect::<Vec<_>>()
        );

        Self {
            pool,
            avatar_service,
            shutdown_tx,
        }
    }

    pub async fn start(&self) -> SyncResult<()> {
        log::info!("[BackgroundAvatarFetcher] Starting background avatar fetcher service");

        let pool = self.pool.clone();
        let avatar_service = Arc::clone(&self.avatar_service);
        let mut shutdown_rx = self.shutdown_tx.subscribe();

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = shutdown_rx.recv() => {
                        log::info!("[BackgroundAvatarFetcher] Shutdown signal received");
                        break;
                    }
                    _ = sleep(Duration::from_secs(FETCH_INTERVAL_SECS)) => {
                        if let Err(e) = Self::fetch_missing_avatars(
                            &pool,
                            &avatar_service,
                        ).await {
                            log::error!("[BackgroundAvatarFetcher] Error fetching avatars: {}", e);
                        }
                    }
                }
            }
        });

        Ok(())
    }

    pub fn stop(&self) {
        log::info!("[BackgroundAvatarFetcher] Stopping background avatar fetcher service");
        let _ = self.shutdown_tx.send(());
    }

    async fn fetch_missing_avatars(
        pool: &SqlitePool,
        avatar_service: &Arc<AvatarService>,
    ) -> SyncResult<()> {
        let repo_factory = RepositoryFactory::new(pool.clone());
        let account_repo = repo_factory.account_repository();

        if let Err(e) = Self::fetch_avatars(pool, avatar_service).await {
            log::error!("[BackgroundAvatarFetcher] Failed to fetch avatars: {}", e);
        }

        Ok(())
    }

    async fn fetch_avatars(
        pool: &SqlitePool,
        avatar_service: &Arc<AvatarService>,
    ) -> SyncResult<()> {
        let repo_factory = RepositoryFactory::new(pool.clone());
        let contact_repo = repo_factory.contact_repository();

        let contacts = contact_repo
            .find_contacts_without_avatars(FETCH_BATCH_SIZE)
            .await
            .map_err(|e| SyncError::DatabaseError(format!("Failed to fetch contacts: {}", e)))?;

        log::info!(
            "[BackgroundAvatarFetcher] Found {} contacts without avatars for account",
            contacts.len()
        );

        for contact in contacts {
            match avatar_service
                .fetch_avatar(contact.id, &contact.email)
                .await
            {
                Ok((avatar_type, avatar_path)) => {
                    if let Err(e) = contact_repo
                        .update_avatar(contact.id, &avatar_type, Some(avatar_path))
                        .await
                    {
                        log::warn!(
                            "[BackgroundAvatarFetcher] Failed to update avatar for contact {}: {}",
                            contact.id,
                            e
                        );
                    } else {
                        log::debug!(
                            "[BackgroundAvatarFetcher] Successfully fetched avatar for {} via {}",
                            contact.email,
                            avatar_type
                        );
                    }
                }
                Err(e) => {
                    if let Err(e) = contact_repo.update_avatar(contact.id, "none", None).await {
                        log::warn!(
                            "[BackgroundAvatarFetcher] Failed to update avatar for contact {}: {}",
                            contact.id,
                            e
                        );
                    } else {
                        log::debug!(
                            "[BackgroundAvatarFetcher] Failed to fetch avatar for {}: {}",
                            contact.email,
                            e
                        );
                    }
                }
            }

            sleep(Duration::from_millis(100)).await;
        }

        Ok(())
    }
}
