use chrono::{DateTime, Utc};
use sqlx::{Row, SqlitePool};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use uuid::Uuid;

use crate::database::models::email::{Email, EmailAddress};
use crate::services::notification_service::NotificationService;

const DEFAULT_POLL_INTERVAL_SECS: u64 = 30;
const LOOKAHEAD_WINDOW_SECS: i64 = 15;
const REMINDER_DEDUP_TTL_SECS: i64 = 60 * 60 * 24 * 14;

pub struct BackgroundReminderNotifier {
    pool: SqlitePool,
    notification_service: Arc<NotificationService>,
    shutdown_tx: tokio::sync::broadcast::Sender<()>,
    poll_interval: Duration,
}

impl BackgroundReminderNotifier {
    pub fn new(pool: SqlitePool, notification_service: Arc<NotificationService>) -> Self {
        let (shutdown_tx, _) = tokio::sync::broadcast::channel(1);

        Self {
            pool,
            notification_service,
            shutdown_tx,
            poll_interval: Duration::from_secs(DEFAULT_POLL_INTERVAL_SECS),
        }
    }

    pub fn with_poll_interval(mut self, poll_interval: Duration) -> Self {
        self.poll_interval = poll_interval;
        self
    }

    pub async fn start(&self) -> Result<(), String> {
        log::info!("[BackgroundReminderNotifier] Starting background reminder notifier");

        let pool = self.pool.clone();
        let notification_service = Arc::clone(&self.notification_service);
        let poll_interval = self.poll_interval;
        let mut shutdown_rx = self.shutdown_tx.subscribe();

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = shutdown_rx.recv() => {
                        log::info!("[BackgroundReminderNotifier] Shutdown signal received");
                        break;
                    }
                    _ = sleep(poll_interval) => {
                        if let Err(error) = Self::process_due_reminders(&pool, &notification_service).await {
                            log::error!(
                                "[BackgroundReminderNotifier] Failed to process reminder notifications: {}",
                                error
                            );
                        }
                    }
                }
            }
        });

        if let Err(error) =
            Self::process_due_reminders(&self.pool, &self.notification_service).await
        {
            log::error!(
                "[BackgroundReminderNotifier] Initial reminder scan failed: {}",
                error
            );
        }

        Ok(())
    }

    pub fn shutdown(&self) {
        if let Err(error) = self.shutdown_tx.send(()) {
            log::debug!(
                "[BackgroundReminderNotifier] Shutdown signal could not be delivered: {}",
                error
            );
        }
    }

    async fn process_due_reminders(
        pool: &SqlitePool,
        notification_service: &Arc<NotificationService>,
    ) -> Result<(), String> {
        let now = Utc::now();
        let upper_bound = now + chrono::Duration::seconds(LOOKAHEAD_WINDOW_SECS);

        let rows = sqlx::query(
            r#"
            SELECT
                id,
                remind_at
            FROM emails
            WHERE remind_at IS NOT NULL
              AND is_deleted = 0
              AND is_draft = 0
              AND remind_at <= ?
            ORDER BY remind_at ASC
            "#,
        )
        .bind(upper_bound.to_rfc3339())
        .fetch_all(pool)
        .await
        .map_err(|error| format!("Failed to query due reminders: {error}"))?;

        if rows.is_empty() {
            return Ok(());
        }

        for row in rows {
            let email_id_raw: String = row
                .try_get("id")
                .map_err(|error| format!("Failed to read reminder email id: {error}"))?;

            let remind_at_raw = row
                .try_get::<Option<String>, _>("remind_at")
                .map_err(|error| format!("Failed to read reminder timestamp: {error}"))?;

            let Some(remind_at_raw) = remind_at_raw else {
                continue;
            };

            let email_id = Uuid::parse_str(&email_id_raw).map_err(|error| {
                format!("Failed to parse reminder email id '{email_id_raw}': {error}")
            })?;

            let remind_at = DateTime::parse_from_rfc3339(&remind_at_raw)
                .map_err(|error| {
                    format!("Failed to parse remind_at '{remind_at_raw}' for email {email_id_raw}: {error}")
                })?
                .with_timezone(&Utc);

            if Self::has_recent_notification_record(pool, email_id, remind_at).await? {
                continue;
            }

            let email = Self::load_email(pool, &email_id_raw).await?;
            let Some(email) = email else {
                continue;
            };

            notification_service
                .notify_reminder_email(&email)
                .await
                .map_err(|error| {
                    format!("Failed to send reminder notification for {email_id_raw}: {error}")
                })?;

            Self::store_notification_record(pool, email_id, remind_at).await?;
        }

        Self::prune_notification_records(pool).await?;
        Ok(())
    }

    async fn load_email(pool: &SqlitePool, email_id: &str) -> Result<Option<Email>, String> {
        let row = sqlx::query(
            r#"
            SELECT *
            FROM emails
            WHERE id = ?
            LIMIT 1
            "#,
        )
        .bind(email_id)
        .fetch_optional(pool)
        .await
        .map_err(|error| format!("Failed to load reminder email {email_id}: {error}"))?;

        let Some(row) = row else {
            return Ok(None);
        };

        Ok(Some(Self::map_email_row(&row)?))
    }

    fn map_email_row(row: &sqlx::sqlite::SqliteRow) -> Result<Email, String> {
        let from_json: String = row
            .try_get("from")
            .map_err(|error| format!("Failed to read email.from: {error}"))?;
        let to_json: String = row
            .try_get("to")
            .map_err(|error| format!("Failed to read email.to: {error}"))?;
        let cc_json: String = row
            .try_get("cc")
            .map_err(|error| format!("Failed to read email.cc: {error}"))?;
        let bcc_json: String = row
            .try_get("bcc")
            .map_err(|error| format!("Failed to read email.bcc: {error}"))?;
        let reply_to_json: Option<String> = row
            .try_get("reply_to")
            .map_err(|error| format!("Failed to read email.reply_to: {error}"))?;

        let from: EmailAddress = serde_json::from_str(&from_json)
            .map_err(|error| format!("Failed to decode email.from JSON: {error}"))?;
        let to: Vec<EmailAddress> = serde_json::from_str(&to_json)
            .map_err(|error| format!("Failed to decode email.to JSON: {error}"))?;
        let cc: Vec<EmailAddress> = serde_json::from_str(&cc_json)
            .map_err(|error| format!("Failed to decode email.cc JSON: {error}"))?;
        let bcc: Vec<EmailAddress> = serde_json::from_str(&bcc_json)
            .map_err(|error| format!("Failed to decode email.bcc JSON: {error}"))?;
        let reply_to = match reply_to_json {
            Some(value) => Some(
                serde_json::from_str(&value)
                    .map_err(|error| format!("Failed to decode email.reply_to JSON: {error}"))?,
            ),
            None => None,
        };

        let id_raw: String = row
            .try_get("id")
            .map_err(|error| format!("Failed to read email.id: {error}"))?;
        let account_id_raw: String = row
            .try_get("account_id")
            .map_err(|error| format!("Failed to read email.account_id: {error}"))?;
        let folder_id_raw: String = row
            .try_get("folder_id")
            .map_err(|error| format!("Failed to read email.folder_id: {error}"))?;

        let parse_uuid = |value: String, field: &str| {
            Uuid::parse_str(&value)
                .map_err(|error| format!("Failed to parse {field} '{value}': {error}"))
        };

        let parse_dt = |value: String, field: &str| {
            DateTime::parse_from_rfc3339(&value)
                .or_else(|_| {
                    chrono::NaiveDateTime::parse_from_str(&value, "%Y-%m-%d %H:%M:%S")
                        .map(|dt| dt.and_utc().fixed_offset())
                })
                .or_else(|_| {
                    chrono::NaiveDateTime::parse_from_str(&value, "%Y-%m-%d %H:%M:%S%.f")
                        .map(|dt| dt.and_utc().fixed_offset())
                })
                .map(|dt| dt.with_timezone(&Utc))
                .map_err(|error| format!("Failed to parse {field} '{value}': {error}"))
        };

        let parse_opt_dt =
            |value: Option<String>, field: &str| -> Result<Option<DateTime<Utc>>, String> {
                value.map(|v| parse_dt(v, field)).transpose()
            };

        Ok(Email {
            id: parse_uuid(id_raw, "email.id")?,
            account_id: parse_uuid(account_id_raw, "email.account_id")?,
            folder_id: parse_uuid(folder_id_raw, "email.folder_id")?,
            message_id: row
                .try_get("message_id")
                .map_err(|error| format!("Failed to read email.message_id: {error}"))?,
            conversation_id: row
                .try_get("conversation_id")
                .map_err(|error| format!("Failed to read email.conversation_id: {error}"))?,
            remote_id: row
                .try_get("remote_id")
                .map_err(|error| format!("Failed to read email.remote_id: {error}"))?,
            from: sqlx::types::Json(from),
            to: sqlx::types::Json(to),
            cc: sqlx::types::Json(cc),
            bcc: sqlx::types::Json(bcc),
            reply_to: reply_to.map(sqlx::types::Json),
            subject: row
                .try_get("subject")
                .map_err(|error| format!("Failed to read email.subject: {error}"))?,
            snippet: row
                .try_get("snippet")
                .map_err(|error| format!("Failed to read email.snippet: {error}"))?,
            body_plain: row
                .try_get("body_plain")
                .map_err(|error| format!("Failed to read email.body_plain: {error}"))?,
            body_html: row
                .try_get("body_html")
                .map_err(|error| format!("Failed to read email.body_html: {error}"))?,
            other_mails: row
                .try_get("other_mails")
                .map_err(|error| format!("Failed to read email.other_mails: {error}"))?,
            category: row
                .try_get("category")
                .map_err(|error| format!("Failed to read email.category: {error}"))?,
            ai_cache: row
                .try_get("ai_cache")
                .map_err(|error| format!("Failed to read email.ai_cache: {error}"))?,
            received_at: parse_dt(
                row.try_get("received_at")
                    .map_err(|error| format!("Failed to read email.received_at: {error}"))?,
                "email.received_at",
            )?,
            sent_at: parse_opt_dt(
                row.try_get("sent_at")
                    .map_err(|error| format!("Failed to read email.sent_at: {error}"))?,
                "email.sent_at",
            )?,
            scheduled_send_at: parse_opt_dt(
                row.try_get("scheduled_send_at")
                    .map_err(|error| format!("Failed to read email.scheduled_send_at: {error}"))?,
                "email.scheduled_send_at",
            )?,
            remind_at: parse_opt_dt(
                row.try_get("remind_at")
                    .map_err(|error| format!("Failed to read email.remind_at: {error}"))?,
                "email.remind_at",
            )?,
            is_read: row
                .try_get("is_read")
                .map_err(|error| format!("Failed to read email.is_read: {error}"))?,
            is_flagged: row
                .try_get("is_flagged")
                .map_err(|error| format!("Failed to read email.is_flagged: {error}"))?,
            has_attachments: row
                .try_get("has_attachments")
                .map_err(|error| format!("Failed to read email.has_attachments: {error}"))?,
            is_draft: row
                .try_get("is_draft")
                .map_err(|error| format!("Failed to read email.is_draft: {error}"))?,
            is_deleted: row
                .try_get("is_deleted")
                .map_err(|error| format!("Failed to read email.is_deleted: {error}"))?,
            headers: row
                .try_get("headers")
                .map_err(|error| format!("Failed to read email.headers: {error}"))?,
            sync_status: row
                .try_get("sync_status")
                .map_err(|error| format!("Failed to read email.sync_status: {error}"))?,
            tracking_blocked: row
                .try_get("tracking_blocked")
                .map_err(|error| format!("Failed to read email.tracking_blocked: {error}"))?,
            images_blocked: row
                .try_get("images_blocked")
                .map_err(|error| format!("Failed to read email.images_blocked: {error}"))?,
            body_fetch_attempts: row
                .try_get("body_fetch_attempts")
                .map_err(|error| format!("Failed to read email.body_fetch_attempts: {error}"))?,
            last_body_fetch_attempt: parse_opt_dt(
                row.try_get("last_body_fetch_attempt").map_err(|error| {
                    format!("Failed to read email.last_body_fetch_attempt: {error}")
                })?,
                "email.last_body_fetch_attempt",
            )?,
            change_key: row
                .try_get("change_key")
                .map_err(|error| format!("Failed to read email.change_key: {error}"))?,
            last_modified_at: parse_opt_dt(
                row.try_get("last_modified_at")
                    .map_err(|error| format!("Failed to read email.last_modified_at: {error}"))?,
                "email.last_modified_at",
            )?,
            deleted_at: parse_opt_dt(
                row.try_get("deleted_at")
                    .map_err(|error| format!("Failed to read email.deleted_at: {error}"))?,
                "email.deleted_at",
            )?,
            deletion_source: row
                .try_get("deletion_source")
                .map_err(|error| format!("Failed to read email.deletion_source: {error}"))?,
            created_at: parse_dt(
                row.try_get("created_at")
                    .map_err(|error| format!("Failed to read email.created_at: {error}"))?,
                "email.created_at",
            )?,
            updated_at: parse_dt(
                row.try_get("updated_at")
                    .map_err(|error| format!("Failed to read email.updated_at: {error}"))?,
                "email.updated_at",
            )?,
            size: row
                .try_get("size")
                .map_err(|error| format!("Failed to read email.size: {error}"))?,
        })
    }

    async fn has_recent_notification_record(
        pool: &SqlitePool,
        email_id: Uuid,
        remind_at: DateTime<Utc>,
    ) -> Result<bool, String> {
        let remind_at_str = remind_at.to_rfc3339();

        let row = sqlx::query(
            r#"
            SELECT id
            FROM notification_reminder_events
            WHERE email_id = ?
              AND remind_at = ?
            LIMIT 1
            "#,
        )
        .bind(email_id.to_string())
        .bind(remind_at_str)
        .fetch_optional(pool)
        .await
        .map_err(|error| format!("Failed to query reminder notification record: {error}"))?;

        Ok(row.is_some())
    }

    async fn store_notification_record(
        pool: &SqlitePool,
        email_id: Uuid,
        remind_at: DateTime<Utc>,
    ) -> Result<(), String> {
        let remind_at_str = remind_at.to_rfc3339();
        let notified_at_str = Utc::now().to_rfc3339();

        sqlx::query(
            r#"
            INSERT INTO notification_reminder_events (
                id,
                email_id,
                remind_at,
                notified_at
            )
            VALUES (?, ?, ?, ?)
            "#,
        )
        .bind(Uuid::now_v7().to_string())
        .bind(email_id.to_string())
        .bind(remind_at_str)
        .bind(notified_at_str)
        .execute(pool)
        .await
        .map_err(|error| format!("Failed to store reminder notification record: {error}"))?;

        Ok(())
    }

    async fn prune_notification_records(pool: &SqlitePool) -> Result<(), String> {
        let cutoff = (Utc::now() - chrono::Duration::seconds(REMINDER_DEDUP_TTL_SECS)).to_rfc3339();

        sqlx::query(
            r#"
            DELETE FROM notification_reminder_events
            WHERE notified_at < ?
            "#,
        )
        .bind(cutoff)
        .execute(pool)
        .await
        .map_err(|error| format!("Failed to prune reminder notification records: {error}"))?;

        Ok(())
    }
}
