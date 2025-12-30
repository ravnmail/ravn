mod client;
mod manager;
mod types;

pub use client::ActivationClient;
pub use manager::LicenseManager;
pub use types::*;

use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::time::sleep;

pub struct LicenseRefreshRunner {
    manager: Arc<LicenseManager>,
    running: Arc<RwLock<bool>>,
}

impl LicenseRefreshRunner {
    pub fn new(manager: Arc<LicenseManager>) -> Self {
        Self {
            manager,
            running: Arc::new(RwLock::new(false)),
        }
    }

    pub async fn start(&self) {
        let mut running = self.running.write().await;
        if *running {
            log::warn!("License refresh runner already running");
            return;
        }
        *running = true;
        drop(running);

        let manager = Arc::clone(&self.manager);
        let running = Arc::clone(&self.running);

        tokio::spawn(async move {
            log::info!("License refresh runner started");

            loop {
                // Check if we should stop
                {
                    let is_running = running.read().await;
                    if !*is_running {
                        log::info!("License refresh runner stopped");
                        break;
                    }
                }

                // Sleep for 1 hour
                sleep(Duration::from_secs(3600)).await;

                // Refresh license
                match manager.refresh_license().await {
                    Ok(_) => log::debug!("License refresh cycle completed"),
                    Err(e) => log::error!("License refresh failed: {}", e),
                }
            }
        });
    }

    pub async fn stop(&self) {
        let mut running = self.running.write().await;
        *running = false;
        log::info!("License refresh runner stop requested");
    }
}
