use crate::database::{error::DatabaseError, models::email::Email, models::folder::FolderType};
use async_trait::async_trait;
use sqlx::SqlitePool;
use uuid::Uuid;

#[async_trait]
pub trait EmailRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Email>, DatabaseError>;
    async fn find_by_message_id(&self, message_id: &str) -> Result<Option<Email>, DatabaseError>;
    async fn find_by_remote_id_or_message_id(
        &self,
        account_id: Uuid,
        remote_id: &str,
        message_id: &str,
    ) -> Result<Option<Email>, DatabaseError>;
    async fn find_by_folder(
        &self,
        folder_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Email>, DatabaseError>;
    async fn find_by_folder_with_filters(
        &self,
        folder_id: Uuid,
        limit: i64,
        offset: i64,
        sort_by: &str,
        sort_order: &str,
        filter_read: Option<bool>,
        filter_has_attachments: Option<bool>,
    ) -> Result<Vec<Email>, DatabaseError>;
    async fn find_by_conversation_id(
        &self,
        conversation_id: Uuid,
    ) -> Result<Vec<Email>, DatabaseError>;
    async fn find_by_labels(
        &self,
        label_ids: &[Uuid],
        match_all: bool,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Email>, DatabaseError>;
    async fn create(&self, email: &Email) -> Result<Uuid, DatabaseError>;
    async fn update(&self, email: &Email) -> Result<(), DatabaseError>;
    async fn update_metadata_only(&self, email: &Email) -> Result<(), DatabaseError>;
    async fn soft_delete(&self, id: Uuid) -> Result<(), DatabaseError>;
    async fn undelete(&self, id: Uuid) -> Result<(), DatabaseError>;
    async fn delete(&self, id: Uuid) -> Result<(), DatabaseError>;
    async fn count_unread_all(&self) -> Result<i64, DatabaseError>;
    async fn count_unread_by_folders(&self, folder_ids: &[Uuid]) -> Result<i64, DatabaseError>;
    async fn find_synced_batch(&self, limit: i64, offset: i64)
        -> Result<Vec<Email>, DatabaseError>;
    async fn find_synced_by_account(&self, account_id: Uuid) -> Result<Vec<Email>, DatabaseError>;
    async fn find_with_folder_type(&self) -> Result<Vec<(Email, FolderType)>, DatabaseError>;
    async fn undelete_by_account(&self, account_id: Uuid) -> Result<u64, DatabaseError>;
    // Sync operation methods
    async fn find_for_remote_operation(
        &self,
        id: Uuid,
    ) -> Result<Option<(Uuid, String)>, DatabaseError>;
    async fn update_folder(&self, id: Uuid, folder_id: Uuid) -> Result<(), DatabaseError>;
    async fn update_read_status(&self, id: Uuid, is_read: bool) -> Result<(), DatabaseError>;
    async fn update_flagged_status(&self, id: Uuid, is_flagged: bool) -> Result<(), DatabaseError>;
    async fn update_ai_cache(&self, id: Uuid, ai_cache_json: &str) -> Result<(), DatabaseError>;
    async fn find_pending_ai_analysis(
        &self,
        limit: i64,
    ) -> Result<Vec<(Uuid, Option<String>, Option<String>, Option<String>)>, DatabaseError>;
}

pub struct SqliteEmailRepository {
    pool: SqlitePool,
}

impl SqliteEmailRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl EmailRepository for SqliteEmailRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Email>, DatabaseError> {
        sqlx::query_as::<_, Email>("SELECT * FROM emails WHERE id = ?")
            .bind(id.to_string())
            .fetch_optional(&self.pool)
            .await
            .map_err(DatabaseError::ConnectionError)
    }

    async fn find_by_message_id(&self, message_id: &str) -> Result<Option<Email>, DatabaseError> {
        sqlx::query_as::<_, Email>("SELECT * FROM emails WHERE message_id = ?")
            .bind(message_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(DatabaseError::ConnectionError)
    }

    async fn find_by_remote_id_or_message_id(
        &self,
        account_id: Uuid,
        remote_id: &str,
        message_id: &str,
    ) -> Result<Option<Email>, DatabaseError> {
        let account_id_str = account_id.to_string();
        // Use a placeholder for empty remote_id to avoid matching
        let remote_id_query = if remote_id.is_empty() {
            "__EMPTY__"
        } else {
            remote_id
        };

        sqlx::query_as::<_, Email>(
            r#"
            SELECT * FROM emails
            WHERE account_id = ? AND (remote_id = ? OR message_id = ?)
            ORDER BY CASE WHEN remote_id = ? THEN 0 ELSE 1 END
            LIMIT 1
            "#,
        )
        .bind(account_id_str)
        .bind(remote_id_query)
        .bind(message_id)
        .bind(remote_id_query)
        .fetch_optional(&self.pool)
        .await
        .map_err(DatabaseError::ConnectionError)
    }

    async fn find_by_folder(
        &self,
        folder_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Email>, DatabaseError> {
        sqlx::query_as::<_, Email>(
            "SELECT * FROM emails WHERE folder_id = ? ORDER BY received_at DESC LIMIT ? OFFSET ?",
        )
        .bind(folder_id.to_string())
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(DatabaseError::ConnectionError)
    }

    async fn find_by_folder_with_filters(
        &self,
        folder_id: Uuid,
        limit: i64,
        offset: i64,
        sort_by: &str,
        sort_order: &str,
        filter_read: Option<bool>,
        filter_has_attachments: Option<bool>,
    ) -> Result<Vec<Email>, DatabaseError> {
        let mut query = String::from("SELECT * FROM emails WHERE folder_id = ?");

        // Add filters
        if let Some(is_read) = filter_read {
            query.push_str(&format!(" AND is_read = {}", if is_read { 1 } else { 0 }));
        }

        if let Some(has_attachments) = filter_has_attachments {
            query.push_str(&format!(
                " AND has_attachments = {}",
                if has_attachments { 1 } else { 0 }
            ));
        }

        // Add sorting
        let order_column = match sort_by {
            "sent_at" => "sent_at",
            "size" => "size",
            _ => "received_at",
        };

        let order_direction = if sort_order.to_lowercase() == "asc" {
            "ASC"
        } else {
            "DESC"
        };

        query.push_str(&format!(
            " ORDER BY {} {} LIMIT ? OFFSET ?",
            order_column, order_direction
        ));

        sqlx::query_as::<_, Email>(&query)
            .bind(folder_id.to_string())
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await
            .map_err(DatabaseError::ConnectionError)
    }

    async fn find_by_conversation_id(
        &self,
        conversation_id: Uuid,
    ) -> Result<Vec<Email>, DatabaseError> {
        sqlx::query_as::<_, Email>(
            "SELECT * FROM emails WHERE conversation_id = ? ORDER BY received_at DESC",
        )
        .bind(conversation_id.to_string())
        .fetch_all(&self.pool)
        .await
        .map_err(DatabaseError::ConnectionError)
    }

    async fn find_by_labels(
        &self,
        label_ids: &[Uuid],
        match_all: bool,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Email>, DatabaseError> {
        if label_ids.is_empty() {
            return Ok(Vec::new());
        }

        let label_id_strings: Vec<String> = label_ids.iter().map(|id| id.to_string()).collect();
        let placeholders = label_id_strings
            .iter()
            .map(|_| "?")
            .collect::<Vec<_>>()
            .join(", ");

        let query = if match_all {
            // AND logic: email must have ALL specified labels
            format!(
                r#"
                SELECT DISTINCT e.*
                FROM emails e
                WHERE e.id IN (
                    SELECT email_id
                    FROM email_labels
                    WHERE label_id IN ({})
                    GROUP BY email_id
                    HAVING COUNT(DISTINCT label_id) = ?
                )
                ORDER BY e.received_at DESC
                LIMIT ? OFFSET ?
                "#,
                placeholders
            )
        } else {
            // OR logic: email must have AT LEAST ONE of the specified labels
            format!(
                r#"
                SELECT DISTINCT e.*
                FROM emails e
                JOIN email_labels el ON el.email_id = e.id
                WHERE el.label_id IN ({})
                ORDER BY e.received_at DESC
                LIMIT ? OFFSET ?
                "#,
                placeholders
            )
        };

        let mut sqlx_query = sqlx::query_as::<_, Email>(&query);

        // Bind all label IDs
        for label_id_str in &label_id_strings {
            sqlx_query = sqlx_query.bind(label_id_str);
        }

        // Bind additional parameters
        if match_all {
            sqlx_query = sqlx_query.bind(label_ids.len() as i64);
        }
        sqlx_query = sqlx_query.bind(limit).bind(offset);

        sqlx_query
            .fetch_all(&self.pool)
            .await
            .map_err(DatabaseError::ConnectionError)
    }

    async fn create(&self, email: &Email) -> Result<Uuid, DatabaseError> {
        let id = email.id.to_string();
        let account_id = email.account_id.to_string();
        let folder_id = email.folder_id.to_string();
        let from_json = serde_json::to_string(&email.from.0)?;
        let to_json = serde_json::to_string(&email.to.0)?;
        let cc_json = serde_json::to_string(&email.cc.0)?;
        let bcc_json = serde_json::to_string(&email.bcc.0)?;
        let reply_to_json = email
            .reply_to
            .as_ref()
            .map(|r| serde_json::to_string(&r.0))
            .transpose()?;
        let headers_json = email.headers.as_deref();
        let flags_json = serde_json::to_string(
            &vec![
                if email.is_read { Some("\\Seen") } else { None },
                if email.is_flagged {
                    Some("\\Flagged")
                } else {
                    None
                },
                if email.is_draft {
                    Some("\\Draft")
                } else {
                    None
                },
            ]
            .into_iter()
            .flatten()
            .collect::<Vec<_>>(),
        )?;

        sqlx::query!(
            r#"
            INSERT INTO emails (
                id, account_id, folder_id, message_id, conversation_id, remote_id,
                `from`, `to`, cc, bcc, reply_to, subject, snippet,
                body_plain, body_html, other_mails, category, received_at, sent_at, flags, headers, size,
                is_read, is_flagged, is_draft, has_attachments, sync_status, change_key, last_modified_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            id,
            account_id,
            folder_id,
            email.message_id,
            email.conversation_id,
            email.remote_id,
            from_json,
            to_json,
            cc_json,
            bcc_json,
            reply_to_json,
            email.subject,
            email.snippet,
            email.body_plain,
            email.body_html,
            email.other_mails,
            email.category,
            email.received_at,
            email.sent_at,
            flags_json,
            headers_json,
            email.size,
            email.is_read,
            email.is_flagged,
            email.is_draft,
            email.has_attachments,
            email.sync_status,
            email.change_key,
            email.last_modified_at,
        )
        .execute(&self.pool)
        .await
        .map_err(DatabaseError::ConnectionError)?;

        Ok(email.id)
    }

    async fn update(&self, email: &Email) -> Result<(), DatabaseError> {
        let id = email.id.to_string();
        let folder_id = email.folder_id.to_string();
        let from_json = serde_json::to_string(&email.from.0)?;
        let to_json = serde_json::to_string(&email.to.0)?;
        let cc_json = serde_json::to_string(&email.cc.0)?;
        let bcc_json = serde_json::to_string(&email.bcc.0)?;
        let reply_to_json = email
            .reply_to
            .as_ref()
            .map(|r| serde_json::to_string(&r.0))
            .transpose()?;
        let headers_json = email.headers.as_deref();
        let flags_json = serde_json::to_string(
            &vec![
                if email.is_read { Some("\\Seen") } else { None },
                if email.is_flagged {
                    Some("\\Flagged")
                } else {
                    None
                },
                if email.is_draft {
                    Some("\\Draft")
                } else {
                    None
                },
            ]
            .into_iter()
            .flatten()
            .collect::<Vec<_>>(),
        )?;

        sqlx::query!(
            r#"
            UPDATE emails SET
                folder_id = ?, conversation_id = ?, remote_id = ?,
                `from` = ?, `to` = ?, cc = ?, bcc = ?, reply_to = ?,
                subject = ?, snippet = ?, body_plain = ?, body_html = ?, other_mails = ?, category = ?,
                received_at = ?, sent_at = ?, flags = ?, headers = ?, size = ?,
                is_read = ?, is_flagged = ?, is_draft = ?, is_deleted = ?, ai_cache = ?,
                has_attachments = ?, sync_status = ?, change_key = ?, last_modified_at = ?, updated_at = CURRENT_TIMESTAMP
            WHERE id = ?
            "#,
            folder_id,
            email.conversation_id,
            email.remote_id,
            from_json,
            to_json,
            cc_json,
            bcc_json,
            reply_to_json,
            email.subject,
            email.snippet,
            email.body_plain,
            email.body_html,
            email.other_mails,
            email.category,
            email.received_at,
            email.sent_at,
            flags_json,
            headers_json,
            email.size,
            email.is_read,
            email.is_flagged,
            email.is_draft,
            email.is_deleted,
            email.ai_cache,
            email.has_attachments,
            email.sync_status,
            email.change_key,
            email.last_modified_at,
            id
        )
        .execute(&self.pool)
        .await
        .map_err(DatabaseError::ConnectionError)?;

        Ok(())
    }

    async fn update_metadata_only(&self, email: &Email) -> Result<(), DatabaseError> {
        let id = email.id.to_string();
        let folder_id = email.folder_id.to_string();
        let from_json = serde_json::to_string(&email.from.0)?;
        let to_json = serde_json::to_string(&email.to.0)?;
        let cc_json = serde_json::to_string(&email.cc.0)?;
        let bcc_json = serde_json::to_string(&email.bcc.0)?;
        let reply_to_json = email
            .reply_to
            .as_ref()
            .map(|r| serde_json::to_string(&r.0))
            .transpose()?;
        let headers_json = email.headers.as_deref();
        let flags_json = serde_json::to_string(
            &vec![
                if email.is_read { Some("\\Seen") } else { None },
                if email.is_flagged {
                    Some("\\Flagged")
                } else {
                    None
                },
                if email.is_draft {
                    Some("\\Draft")
                } else {
                    None
                },
            ]
            .into_iter()
            .flatten()
            .collect::<Vec<_>>(),
        )?;

        sqlx::query!(
            r#"
            UPDATE emails SET
                folder_id = ?, remote_id = ?, `from` = ?, `to` = ?, cc = ?,
                bcc = ?, reply_to = ?, subject = ?,
                received_at = ?, sent_at = ?, flags = ?, headers = ?, size = ?,
                is_read = ?, is_flagged = ?, is_draft = ?, has_attachments = ?,
                conversation_id = ?, change_key = ?, last_modified_at = ?, is_deleted = 0, updated_at = CURRENT_TIMESTAMP
            WHERE id = ?
            "#,
            folder_id,
            email.remote_id,
            from_json,
            to_json,
            cc_json,
            bcc_json,
            reply_to_json,
            email.subject,
            email.received_at,
            email.sent_at,
            flags_json,
            headers_json,
            email.size,
            email.is_read,
            email.is_flagged,
            email.is_draft,
            email.has_attachments,
            email.conversation_id,
            email.change_key,
            email.last_modified_at,
            id
        )
        .execute(&self.pool)
        .await
        .map_err(DatabaseError::ConnectionError)?;

        Ok(())
    }

    async fn soft_delete(&self, id: Uuid) -> Result<(), DatabaseError> {
        let id_str = id.to_string();
        sqlx::query!(
            "UPDATE emails SET is_deleted = 1, updated_at = CURRENT_TIMESTAMP WHERE id = ?",
            id_str
        )
        .execute(&self.pool)
        .await
        .map_err(DatabaseError::ConnectionError)?;

        Ok(())
    }

    async fn undelete(&self, id: Uuid) -> Result<(), DatabaseError> {
        let id_str = id.to_string();
        sqlx::query!(
            "UPDATE emails SET is_deleted = 0, updated_at = CURRENT_TIMESTAMP WHERE id = ?",
            id_str
        )
        .execute(&self.pool)
        .await
        .map_err(DatabaseError::ConnectionError)?;

        Ok(())
    }

    async fn delete(&self, id: Uuid) -> Result<(), DatabaseError> {
        let id = id.to_string();
        sqlx::query!("DELETE FROM emails WHERE id = ?", id)
            .execute(&self.pool)
            .await
            .map_err(DatabaseError::ConnectionError)?;

        Ok(())
    }

    async fn count_unread_all(&self) -> Result<i64, DatabaseError> {
        let count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM emails WHERE is_read = 0")
            .fetch_one(&self.pool)
            .await
            .map_err(DatabaseError::ConnectionError)?;

        Ok(count)
    }

    async fn count_unread_by_folders(&self, folder_ids: &[Uuid]) -> Result<i64, DatabaseError> {
        if folder_ids.is_empty() {
            return Ok(0);
        }

        let folder_id_strings: Vec<String> = folder_ids.iter().map(|id| id.to_string()).collect();
        let placeholders = folder_id_strings
            .iter()
            .map(|_| "?")
            .collect::<Vec<_>>()
            .join(", ");

        let query = format!(
            "SELECT COUNT(*) FROM emails WHERE is_read = 0 AND folder_id IN ({})",
            placeholders
        );

        let mut sqlx_query = sqlx::query_scalar::<_, i64>(&query);
        for folder_id_str in &folder_id_strings {
            sqlx_query = sqlx_query.bind(folder_id_str);
        }

        let count = sqlx_query
            .fetch_one(&self.pool)
            .await
            .map_err(DatabaseError::ConnectionError)?;

        Ok(count)
    }

    async fn find_synced_batch(
        &self,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Email>, DatabaseError> {
        sqlx::query_as::<_, Email>(
            "SELECT * FROM emails WHERE is_deleted = 0 AND sync_status = 'synced' ORDER BY received_at DESC LIMIT ? OFFSET ?",
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(DatabaseError::ConnectionError)
    }

    async fn find_synced_by_account(&self, account_id: Uuid) -> Result<Vec<Email>, DatabaseError> {
        let account_id_str = account_id.to_string();
        sqlx::query_as::<_, Email>(
            "SELECT * FROM emails WHERE account_id = ? AND is_deleted = 0 AND sync_status = 'synced' ORDER BY received_at DESC",
        )
        .bind(account_id_str)
        .fetch_all(&self.pool)
        .await
        .map_err(DatabaseError::ConnectionError)
    }

    async fn find_with_folder_type(&self) -> Result<Vec<(Email, FolderType)>, DatabaseError> {
        #[derive(sqlx::FromRow)]
        struct EmailWithFolderType {
            #[sqlx(flatten)]
            email: Email,
            folder_type: FolderType,
        }

        let results = sqlx::query_as::<_, EmailWithFolderType>(
            r#"
            SELECT
                e.id, e.account_id, e.folder_id, e.message_id, e.conversation_id, e.remote_id,
                e.`from`, e.`to`, e.cc, e.bcc, e.reply_to, e.subject, e.snippet,
                e.body_plain, e.body_html, e.other_mails, e.category, e.received_at, e.sent_at,
                e.scheduled_send_at, e.flags, e.headers, e.size, e.is_read, e.is_flagged,
                e.is_draft, e.has_attachments, e.is_deleted, e.sync_status, e.tracking_blocked,
                e.images_blocked, e.ai_cache, e.created_at, e.updated_at,
                f.folder_type
            FROM emails e
            JOIN folders f ON e.folder_id = f.id
            ORDER BY e.sent_at ASC
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(DatabaseError::ConnectionError)?;

        Ok(results
            .into_iter()
            .map(|r| (r.email, r.folder_type))
            .collect())
    }

    async fn undelete_by_account(&self, account_id: Uuid) -> Result<u64, DatabaseError> {
        let account_id_str = account_id.to_string();
        let result = sqlx::query!(
            "UPDATE emails SET is_deleted = 0, updated_at = CURRENT_TIMESTAMP WHERE account_id = ? AND is_deleted = 1",
            account_id_str
        )
        .execute(&self.pool)
        .await
        .map_err(DatabaseError::ConnectionError)?;

        Ok(result.rows_affected())
    }

    async fn find_for_remote_operation(
        &self,
        id: Uuid,
    ) -> Result<Option<(Uuid, String)>, DatabaseError> {
        let id_str = id.to_string();
        let result = sqlx::query!(
            "SELECT folder_id, remote_id FROM emails WHERE id = ?",
            id_str
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(DatabaseError::ConnectionError)?;

        match result {
            Some(record) => {
                let folder_id = Uuid::parse_str(&record.folder_id)
                    .map_err(|e| DatabaseError::InvalidData(format!("Invalid folder_id: {}", e)))?;
                let remote_id = record
                    .remote_id
                    .ok_or_else(|| DatabaseError::InvalidData("No remote_id found".to_string()))?;
                Ok(Some((folder_id, remote_id)))
            }
            None => Ok(None),
        }
    }

    async fn update_folder(&self, id: Uuid, folder_id: Uuid) -> Result<(), DatabaseError> {
        let id_str = id.to_string();
        let folder_id_str = folder_id.to_string();
        sqlx::query!(
            "UPDATE emails SET folder_id = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?",
            folder_id_str,
            id_str
        )
        .execute(&self.pool)
        .await
        .map_err(DatabaseError::ConnectionError)?;

        Ok(())
    }

    async fn update_read_status(&self, id: Uuid, is_read: bool) -> Result<(), DatabaseError> {
        let id_str = id.to_string();
        sqlx::query!(
            "UPDATE emails SET is_read = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?",
            is_read,
            id_str
        )
        .execute(&self.pool)
        .await
        .map_err(DatabaseError::ConnectionError)?;

        Ok(())
    }

    async fn update_flagged_status(&self, id: Uuid, is_flagged: bool) -> Result<(), DatabaseError> {
        let id_str = id.to_string();
        sqlx::query!(
            "UPDATE emails SET is_flagged = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?",
            is_flagged,
            id_str
        )
        .execute(&self.pool)
        .await
        .map_err(DatabaseError::ConnectionError)?;

        Ok(())
    }

    async fn update_ai_cache(&self, id: Uuid, ai_cache_json: &str) -> Result<(), DatabaseError> {
        let id_str = id.to_string();
        sqlx::query!(
            "UPDATE emails SET ai_cache = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?",
            ai_cache_json,
            id_str
        )
        .execute(&self.pool)
        .await
        .map_err(DatabaseError::ConnectionError)?;

        Ok(())
    }

    async fn find_pending_ai_analysis(
        &self,
        limit: i64,
    ) -> Result<Vec<(Uuid, Option<String>, Option<String>, Option<String>)>, DatabaseError> {
        let results = sqlx::query!(
            r#"
            SELECT e.id, e.subject, e.body_plain, e.body_html
            FROM emails e
            INNER JOIN folders f ON e.folder_id = f.id
            WHERE e.ai_cache IS NULL
              AND e.is_deleted = 0
              AND e.category = 'personal'
              AND f.folder_type = 'inbox'
              AND (e.body_plain IS NOT NULL OR e.body_html IS NOT NULL)
              AND e.sync_status = 'synced'
            ORDER BY e.received_at DESC
            LIMIT ?
            "#,
            limit
        )
        .fetch_all(&self.pool)
        .await
        .map_err(DatabaseError::ConnectionError)?;

        results
            .into_iter()
            .map(|record| {
                let id = Uuid::parse_str(&record.id)
                    .map_err(|e| DatabaseError::InvalidData(format!("Invalid email ID: {}", e)))?;
                Ok((id, record.subject, record.body_plain, record.body_html))
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::models::email::{Email, EmailAddress};
    use chrono::{TimeZone, Utc};
    use sqlx::{sqlite::SqlitePoolOptions, types::Json};
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
            CREATE TABLE IF NOT EXISTS emails (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                account_id INTEGER NOT NULL,
                folder_id INTEGER NOT NULL,
                message_id TEXT NOT NULL,
                conversation_id TEXT,
                remote_id TEXT,
                `from` TEXT NOT NULL,
                `to` TEXT NOT NULL DEFAULT '[]',
                cc TEXT NOT NULL DEFAULT '[]',
                bcc TEXT NOT NULL DEFAULT '[]',
                reply_to TEXT,
                subject TEXT,
                snippet TEXT,
                body_plain TEXT,
                body_html TEXT,
                received_at TIMESTAMP NOT NULL,
                sent_at TIMESTAMP,
                scheduled_send_at TIMESTAMP,
                is_read BOOLEAN NOT NULL DEFAULT 0,
                is_flagged BOOLEAN NOT NULL DEFAULT 0,
                is_draft BOOLEAN NOT NULL DEFAULT 0,
                has_attachments BOOLEAN NOT NULL DEFAULT 0,
                is_deleted BOOLEAN NOT NULL DEFAULT 0,
                sync_status TEXT NOT NULL DEFAULT 'synced',
                tracking_blocked BOOLEAN NOT NULL DEFAULT 1,
                images_blocked BOOLEAN NOT NULL DEFAULT 1,
                created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
            );

            CREATE INDEX IF NOT EXISTS idx_emails_message_id ON emails(message_id);
            CREATE INDEX IF NOT EXISTS idx_emails_folder_id ON emails(folder_id);
            CREATE INDEX IF NOT EXISTS idx_emails_conversation_id ON emails(conversation_id);
            "#,
        )
        .execute(pool)
        .await
        .expect("Failed to create test schema");
    }

    /// Helper function to create an email address
    fn create_email_address(address: &str, name: Option<&str>) -> EmailAddress {
        EmailAddress {
            address: address.to_string(),
            name: name.map(String::from),
        }
    }

    /// Helper function to create a test email
    fn create_test_email(account_id: Uuid, folder_id: Uuid) -> Email {
        Email {
            id: Uuid::now_v7(),
            account_id,
            folder_id,
            message_id: format!("<test{}@example.com>", Utc::now().timestamp()),
            conversation_id: Some("conv123".to_string()),
            remote_id: Some("remote123".to_string()),
            from: Json(create_email_address("sender@example.com", Some("Sender"))),
            to: Json(vec![create_email_address(
                "recipient@example.com",
                Some("Recipient"),
            )]),
            cc: Json(vec![]),
            bcc: Json(vec![]),
            category: Some("personal".to_string()),
            other_mails: None,
            size: 512,
            ai_cache: None,
            headers: None,
            reply_to: None,
            subject: Some("Test Subject".to_string()),
            snippet: Some("Test snippet...".to_string()),
            body_plain: Some("Test email body".to_string()),
            body_html: Some("<p>Test email body</p>".to_string()),
            received_at: Utc::now(),
            sent_at: Some(Utc::now()),
            scheduled_send_at: None,
            is_read: false,
            is_flagged: false,
            is_draft: false,
            has_attachments: false,
            is_deleted: false,
            sync_status: "synced".to_string(),
            tracking_blocked: true,
            images_blocked: true,
            body_fetch_attempts: 0,
            last_body_fetch_attempt: None,
            change_key: None,
            last_modified_at: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[tokio::test]
    async fn test_create_email() {
        let pool = create_test_pool().await;
        setup_test_schema(&pool).await;

        let repository = SqliteEmailRepository::new(pool);
        let test_email = create_test_email(Uuid::now_v7(), Uuid::now_v7());

        let result = repository.create(&test_email).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_find_by_id() {
        let pool = create_test_pool().await;
        setup_test_schema(&pool).await;

        let repository = SqliteEmailRepository::new(pool);
        let test_email = create_test_email(Uuid::now_v7(), Uuid::now_v7());
        let id = test_email.id;

        repository.create(&test_email).await.unwrap();

        let result = repository.find_by_id(id).await.unwrap();
        assert!(result.is_some());

        let found_email = result.unwrap();
        assert_eq!(found_email.subject, test_email.subject);
        assert_eq!(found_email.from.0.address, test_email.from.0.address);
        assert_eq!(found_email.body_plain, test_email.body_plain);
    }

    #[tokio::test]
    async fn test_find_by_message_id() {
        let pool = create_test_pool().await;
        setup_test_schema(&pool).await;

        let repository = SqliteEmailRepository::new(pool);
        let test_email = create_test_email(Uuid::now_v7(), Uuid::now_v7());
        let message_id = test_email.message_id.clone();

        repository.create(&test_email).await.unwrap();

        let result = repository.find_by_message_id(&message_id).await.unwrap();
        assert!(result.is_some());

        let found_email = result.unwrap();
        assert_eq!(found_email.message_id, message_id);
    }

    #[tokio::test]
    async fn test_find_by_folder() {
        let pool = create_test_pool().await;
        setup_test_schema(&pool).await;

        let repository = SqliteEmailRepository::new(pool);
        let account_id = Uuid::now_v7();
        let folder_id = Uuid::now_v7();

        // Create multiple test emails
        for _ in 0..5 {
            let test_email = create_test_email(account_id, folder_id);
            repository.create(&test_email).await.unwrap();
        }

        // Test pagination
        let result = repository.find_by_folder(folder_id, 3, 0).await.unwrap();
        assert_eq!(result.len(), 3);

        let result = repository.find_by_folder(folder_id, 3, 3).await.unwrap();
        assert_eq!(result.len(), 2);
    }

    #[tokio::test]
    async fn test_update_email() {
        let pool = create_test_pool().await;
        setup_test_schema(&pool).await;

        let repository = SqliteEmailRepository::new(pool);
        let mut test_email = create_test_email(Uuid::now_v7(), Uuid::now_v7());
        let id = test_email.id;

        repository.create(&test_email).await.unwrap();

        // Update email
        test_email.subject = Some("Updated Subject".to_string());
        test_email.is_read = true;
        test_email.is_flagged = true;

        let update_result = repository.update(&test_email).await;
        assert!(update_result.is_ok());

        let updated = repository.find_by_id(id).await.unwrap().unwrap();
        assert_eq!(updated.subject, Some("Updated Subject".to_string()));
        assert!(updated.is_read);
        assert!(updated.is_flagged);
    }

    #[tokio::test]
    async fn test_delete_email() {
        let pool = create_test_pool().await;
        setup_test_schema(&pool).await;

        let repository = SqliteEmailRepository::new(pool);
        let test_email = create_test_email(Uuid::now_v7(), Uuid::now_v7());
        let id = test_email.id;

        repository.create(&test_email).await.unwrap();

        let delete_result = repository.delete(id).await;
        assert!(delete_result.is_ok());

        let find_result = repository.find_by_id(id).await.unwrap();
        assert!(find_result.is_none());
    }

    #[tokio::test]
    async fn test_email_with_multiple_recipients() {
        let pool = create_test_pool().await;
        setup_test_schema(&pool).await;

        let repository = SqliteEmailRepository::new(pool);
        let mut test_email = create_test_email(Uuid::now_v7(), Uuid::now_v7());
        let id = test_email.id;

        // Add multiple recipients
        test_email.to = Json(vec![
            create_email_address("recipient1@example.com", Some("Recipient 1")),
            create_email_address("recipient2@example.com", Some("Recipient 2")),
        ]);

        test_email.cc = Json(vec![
            create_email_address("cc1@example.com", Some("CC 1")),
            create_email_address("cc2@example.com", Some("CC 2")),
        ]);

        repository.create(&test_email).await.unwrap();

        let found_email = repository.find_by_id(id).await.unwrap().unwrap();
        assert_eq!(found_email.to.0.len(), 2);
        assert_eq!(found_email.cc.0.len(), 2);
    }

    #[tokio::test]
    async fn test_email_conversation_threading() {
        let pool = create_test_pool().await;
        setup_test_schema(&pool).await;

        let repository = SqliteEmailRepository::new(pool);
        let conversation_id = "conv123";
        let account_id = Uuid::now_v7();
        let folder_id = Uuid::now_v7();

        // Create multiple emails in the same conversation
        for i in 1..=3 {
            let mut test_email = create_test_email(account_id, folder_id);
            test_email.conversation_id = Some(conversation_id.to_string());
            test_email.subject = Some(format!("Re: Test Subject ({})", i));
            repository.create(&test_email).await.unwrap();
        }

        // Query all emails with the conversation ID
        let emails = sqlx::query_as::<_, Email>(
            "SELECT * FROM emails WHERE conversation_id = ? ORDER BY received_at",
        )
        .bind(conversation_id)
        .fetch_all(&repository.pool)
        .await
        .unwrap();

        assert_eq!(emails.len(), 3);
        assert!(emails[0].subject.as_ref().unwrap().contains("(1)"));
        assert!(emails[2].subject.as_ref().unwrap().contains("(3)"));
    }

    #[tokio::test]
    async fn test_scheduled_email() {
        let pool = create_test_pool().await;
        setup_test_schema(&pool).await;

        let repository = SqliteEmailRepository::new(pool);
        let mut test_email = create_test_email(Uuid::now_v7(), Uuid::now_v7());
        let id = test_email.id;

        // Schedule email for future delivery
        let scheduled_time = Utc.with_ymd_and_hms(2025, 1, 1, 12, 0, 0).unwrap();
        test_email.scheduled_send_at = Some(scheduled_time);
        test_email.is_draft = true;

        repository.create(&test_email).await.unwrap();

        let found_email = repository.find_by_id(id).await.unwrap().unwrap();
        assert_eq!(found_email.scheduled_send_at, Some(scheduled_time));
        assert!(found_email.is_draft);
    }

    #[tokio::test]
    async fn test_error_handling() {
        let pool = create_test_pool().await;
        setup_test_schema(&pool).await;

        let repository = SqliteEmailRepository::new(pool);

        // Test finding non-existent email
        let result = repository.find_by_id(Uuid::now_v7()).await.unwrap();
        assert!(result.is_none());

        // Test finding by non-existent message ID
        let result = repository.find_by_message_id("non_existent").await.unwrap();
        assert!(result.is_none());

        // Test finding emails in non-existent folder
        let result = repository
            .find_by_folder(Uuid::now_v7(), 10, 0)
            .await
            .unwrap();
        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn test_email_flags() {
        let pool = create_test_pool().await;
        setup_test_schema(&pool).await;

        let repository = SqliteEmailRepository::new(pool);
        let mut test_email = create_test_email(Uuid::now_v7(), Uuid::now_v7());
        let id = test_email.id;

        repository.create(&test_email).await.unwrap();

        // Test various flag combinations
        let flag_tests = vec![
            (true, false, false, false, false), // Read only
            (true, true, false, false, false),  // Read and flagged
            (false, false, true, false, false), // Draft only
            (false, false, false, true, false), // Deleted only
            (false, false, false, false, true), // has attachments only
        ];

        for (is_read, is_flagged, is_draft, is_deleted, has_attachments) in flag_tests {
            test_email.is_read = is_read;
            test_email.is_flagged = is_flagged;
            test_email.is_draft = is_draft;
            test_email.is_deleted = is_deleted;
            test_email.has_attachments = has_attachments;

            repository.update(&test_email).await.unwrap();

            let updated = repository.find_by_id(id).await.unwrap().unwrap();
            assert_eq!(updated.is_read, is_read);
            assert_eq!(updated.is_flagged, is_flagged);
            assert_eq!(updated.is_draft, is_draft);
            assert_eq!(updated.is_deleted, is_deleted);
            assert_eq!(updated.has_attachments, has_attachments);
        }
    }
}
