use sqlx::SqlitePool;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tauri::Emitter;
use tokio::sync::RwLock;
use tokio::time::sleep;
use uuid::Uuid;

use super::error::{SyncError, SyncResult};
use crate::database::repositories::{EmailRepository, SqliteEmailRepository};
use crate::services::corvus::CorvusService;

const ANALYSIS_BATCH_SIZE: i64 = 5;
const ANALYSIS_INTERVAL_SECS: u64 = 10;

pub struct BackgroundAiAnalyzer {
    pool: SqlitePool,
    app_handle: tauri::AppHandle,
    ai_service: Arc<CorvusService>,
    active_analysis: Arc<RwLock<HashMap<Uuid, bool>>>,
    shutdown_tx: tokio::sync::broadcast::Sender<()>,
}

impl BackgroundAiAnalyzer {
    pub fn new(
        pool: SqlitePool,
        app_handle: tauri::AppHandle,
        ai_service: Arc<CorvusService>,
    ) -> Self {
        let (shutdown_tx, _) = tokio::sync::broadcast::channel(1);

        Self {
            pool,
            app_handle,
            ai_service,
            active_analysis: Arc::new(RwLock::new(HashMap::new())),
            shutdown_tx,
        }
    }

    pub async fn start(&self) -> SyncResult<()> {
        log::info!("[BackgroundAiAnalyzer] Starting background AI analyzer service");

        let pool = self.pool.clone();
        let app_handle = self.app_handle.clone();
        let ai_service = Arc::clone(&self.ai_service);
        let active_analysis = Arc::clone(&self.active_analysis);
        let mut shutdown_rx = self.shutdown_tx.subscribe();

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = shutdown_rx.recv() => {
                        log::info!("[BackgroundAiAnalyzer] Shutdown signal received");
                        break;
                    }
                    _ = sleep(Duration::from_secs(ANALYSIS_INTERVAL_SECS)) => {
                        if let Err(e) = Self::analyze_pending_emails(
                            &pool,
                            &app_handle,
                            &ai_service,
                            &active_analysis,
                        ).await {
                            log::error!("[BackgroundAiAnalyzer] Error analyzing emails: {}", e);
                        }
                    }
                }
            }
        });

        Ok(())
    }

    pub fn stop(&self) {
        log::info!("[BackgroundAiAnalyzer] Stopping background AI analyzer service");
        let _ = self.shutdown_tx.send(());
    }

    async fn analyze_pending_emails(
        pool: &SqlitePool,
        app_handle: &tauri::AppHandle,
        ai_service: &Arc<CorvusService>,
        active_analysis: &Arc<RwLock<HashMap<Uuid, bool>>>,
    ) -> SyncResult<()> {
        let email_repo = SqliteEmailRepository::new(pool.clone());
        let pending_emails = email_repo
            .find_pending_ai_analysis(ANALYSIS_BATCH_SIZE)
            .await
            .map_err(|e| SyncError::DatabaseError(e.to_string()))?;

        if pending_emails.is_empty() {
            return Ok(());
        }

        log::debug!(
            "[BackgroundAiAnalyzer] Found {} personal inbox emails pending AI analysis",
            pending_emails.len()
        );

        for (email_id, subject, body_plain, body_html) in pending_emails {
            {
                let analysis = active_analysis.read().await;
                if analysis.get(&email_id).copied().unwrap_or(false) {
                    continue;
                }
            }

            {
                let mut analysis = active_analysis.write().await;
                analysis.insert(email_id, true);
            }

            let pool_clone = pool.clone();
            let app_handle_clone = app_handle.clone();
            let ai_service_clone = Arc::clone(ai_service);
            let active_analysis_clone = Arc::clone(active_analysis);

            tokio::spawn(async move {
                match Self::analyze_email_background(
                    &pool_clone,
                    &app_handle_clone,
                    &ai_service_clone,
                    email_id,
                    subject,
                    body_plain,
                    body_html,
                )
                .await
                {
                    Ok(_) => {
                        log::info!(
                            "[BackgroundAiAnalyzer] Successfully analyzed personal email {}",
                            email_id
                        );
                    }
                    Err(e) => {
                        log::error!(
                            "[BackgroundAiAnalyzer] Failed to analyze personal email {}: {}",
                            email_id,
                            e
                        );
                    }
                }

                let mut analysis = active_analysis_clone.write().await;
                analysis.insert(email_id, false);
            });
        }

        Ok(())
    }

    async fn analyze_email_background(
        pool: &SqlitePool,
        app_handle: &tauri::AppHandle,
        ai_service: &Arc<CorvusService>,
        email_id: Uuid,
        subject: Option<String>,
        body_plain: Option<String>,
        body_html: Option<String>,
    ) -> SyncResult<()> {
        let email_content = body_plain
            .or(body_html)
            .ok_or_else(|| SyncError::Other("Email has no content".to_string()))?;

        let subject = subject.unwrap_or_else(|| "(No subject)".to_string());
        let analysis = ai_service
            .analyze_email(subject, email_content, None)
            .await
            .map_err(|e| SyncError::Other(e))?;

        let analysis_json = serde_json::to_string(&analysis)
            .map_err(|e| SyncError::Other(format!("Failed to serialize analysis: {}", e)))?;

        let email_repo = SqliteEmailRepository::new(pool.clone());
        email_repo
            .update_ai_cache(email_id, &analysis_json)
            .await
            .map_err(|e| SyncError::DatabaseError(e.to_string()))?;

        let _ = app_handle.emit("email:ai-analysis-complete", email_id.to_string());

        Ok(())
    }
}
