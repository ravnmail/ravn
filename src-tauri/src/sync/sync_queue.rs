use chrono::{DateTime, Utc};
use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use uuid::Uuid;

use super::error::SyncResult;
use super::sync_manager::SyncManager;
use super::types::SyncFolder;
use crate::database::models::account::Account;

#[derive(Clone, Debug)]
pub struct SyncQueueItem {
    pub account_id: Uuid,
    pub folder_id: Uuid,
    pub folder: SyncFolder,
    pub account: Account,
    pub priority: SyncPriority,
    pub last_synced_at: Option<DateTime<Utc>>,
    pub enqueued_at: DateTime<Utc>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum SyncPriority {
    High,
    Normal,
    Low,
}

impl PartialEq for SyncQueueItem {
    fn eq(&self, other: &Self) -> bool {
        self.folder_id == other.folder_id && self.account_id == other.account_id
    }
}

impl Eq for SyncQueueItem {}

impl PartialOrd for SyncQueueItem {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SyncQueueItem {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self.priority, other.priority) {
            (SyncPriority::High, SyncPriority::High) => self
                .last_synced_at
                .cmp(&other.last_synced_at)
                .then_with(|| other.enqueued_at.cmp(&self.enqueued_at)),
            (SyncPriority::High, _) => Ordering::Less,
            (_, SyncPriority::High) => Ordering::Greater,
            (SyncPriority::Normal, SyncPriority::Normal) => self
                .last_synced_at
                .cmp(&other.last_synced_at)
                .then_with(|| other.enqueued_at.cmp(&self.enqueued_at)),
            (SyncPriority::Normal, SyncPriority::Low) => Ordering::Less,
            (SyncPriority::Low, SyncPriority::Normal) => Ordering::Greater,
            (SyncPriority::Low, SyncPriority::Low) => self
                .last_synced_at
                .cmp(&other.last_synced_at)
                .then_with(|| other.enqueued_at.cmp(&self.enqueued_at)),
        }
    }
}

pub struct SyncQueue {
    queue: Arc<Mutex<BinaryHeap<SyncQueueItem>>>,
    active_syncs: Arc<RwLock<std::collections::HashSet<Uuid>>>,
    workers_limit: usize,
}

impl SyncQueue {
    pub fn new(workers_limit: usize) -> Self {
        Self {
            queue: Arc::new(Mutex::new(BinaryHeap::new())),
            active_syncs: Arc::new(RwLock::new(std::collections::HashSet::new())),
            workers_limit: workers_limit.min(100).max(1),
        }
    }

    pub async fn enqueue(&self, mut item: SyncQueueItem) -> SyncResult<()> {
        let priority = item.priority;
        item.enqueued_at = Utc::now();

        let mut queue = self.queue.lock().await;
        queue.push(item.clone());

        log::debug!(
            "Enqueued folder sync: account={}, folder={}, priority={:?}",
            item.account_id,
            item.folder.name,
            priority
        );

        Ok(())
    }

    pub async fn dequeue(&self) -> Option<SyncQueueItem> {
        let mut queue = self.queue.lock().await;
        queue.pop()
    }

    pub async fn is_processing(&self, folder_id: Uuid) -> bool {
        let active = self.active_syncs.read().await;
        active.contains(&folder_id)
    }

    pub async fn mark_processing(&self, folder_id: Uuid) {
        let mut active = self.active_syncs.write().await;
        active.insert(folder_id);
    }

    pub async fn mark_done(&self, folder_id: Uuid) {
        let mut active = self.active_syncs.write().await;
        active.remove(&folder_id);
    }

    pub async fn size(&self) -> usize {
        let queue = self.queue.lock().await;
        queue.len()
    }

    pub async fn active_count(&self) -> usize {
        let active = self.active_syncs.read().await;
        active.len()
    }

    pub fn workers_limit(&self) -> usize {
        self.workers_limit
    }

    pub async fn clear(&self) {
        let mut queue = self.queue.lock().await;
        queue.clear();
    }
}

pub struct SyncQueueWorker {
    queue: Arc<SyncQueue>,
    sync_manager: Arc<SyncManager>,
}

impl SyncQueueWorker {
    pub fn new(queue: Arc<SyncQueue>, sync_manager: Arc<SyncManager>) -> Self {
        Self {
            queue,
            sync_manager,
        }
    }

    pub async fn run(&self, worker_id: usize) -> SyncResult<()> {
        log::info!("Sync queue worker {} started", worker_id);

        loop {
            if let Some(item) = self.queue.dequeue().await {
                let folder_id = item.folder_id;
                let folder_name = item.folder.name.clone();
                let account_email = item.account.email.clone();

                self.queue.mark_processing(folder_id).await;

                log::debug!(
                    "[Worker {}] Processing folder sync: account={}, folder={}",
                    worker_id,
                    account_email,
                    folder_name
                );

                let result = self
                    .sync_manager
                    .sync_folder(&item.account, &item.folder, false)
                    .await;

                match result {
                    Ok(count) => {
                        log::info!(
                            "[Worker {}] Synced {} emails from folder {} (account {})",
                            worker_id,
                            count,
                            folder_name,
                            account_email
                        );
                    }
                    Err(e) => {
                        log::error!(
                            "[Worker {}] Failed to sync folder {} (account {}): {}",
                            worker_id,
                            folder_name,
                            account_email,
                            e
                        );
                    }
                }

                self.queue.mark_done(folder_id).await;
            } else {
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            }
        }
    }
}
