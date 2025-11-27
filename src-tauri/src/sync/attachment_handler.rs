use sqlx::SqlitePool;
use std::sync::Arc;
use uuid::Uuid;

use super::error::{SyncError, SyncResult};
use super::storage::{FileStorage, PathGenerator};
use super::types::SyncAttachment;

/// AttachmentHandler coordinates attachment operations between storage and database
/// Follows Single Responsibility and Dependency Inversion principles
pub struct AttachmentHandler<S: FileStorage> {
    pool: SqlitePool,
    storage: Arc<S>,
}

impl<S: FileStorage> AttachmentHandler<S> {
    pub fn new(pool: SqlitePool, storage: Arc<S>) -> Self {
        Self { pool, storage }
    }

    /// Process attachments for an email
    /// - Saves attachment metadata to database
    /// - Caches attachment data if present (IMAP case)
    /// Returns list of (attachment_id, is_inline, has_data) tuples
    pub async fn process_attachments(
        &self,
        email_id: Uuid,
        account_id: Uuid,
        attachments: &[SyncAttachment],
    ) -> SyncResult<Vec<(Uuid, bool)>> {
        let mut result = Vec::new();

        for attachment in attachments {
            let attachment_id = self.upsert_attachment(email_id, attachment).await?;

            if let Some(data) = &attachment.data {
                self.cache_attachment(
                    attachment_id,
                    account_id,
                    email_id,
                    data,
                    &attachment.filename,
                )
                .await?;
            }

            result.push((attachment_id, attachment.is_inline));
        }

        Ok(result)
    }

    /// Upsert an attachment into the database
    /// Only stores metadata - cache_path is set to None initially
    /// Checks for duplicates by content_id first (for inline attachments), then by email_id+hash
    async fn upsert_attachment(
        &self,
        email_id: Uuid,
        attachment: &SyncAttachment,
    ) -> SyncResult<Uuid> {
        let email_id_str = email_id.to_string();

        let existing = if let Some(ref content_id) = attachment.content_id {
            sqlx::query!(
                "SELECT id FROM attachments WHERE email_id = ? AND content_id = ?",
                email_id_str,
                content_id
            )
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| SyncError::DatabaseError(e.to_string()))?
        } else {
            None
        };

        if let Some(record) = existing {
            sqlx::query!(
                r#"
                UPDATE attachments
                SET filename = ?, content_type = ?, size = ?,
                    remote_url = ?, remote_path = ?, is_inline = ?,
                    content_id = ?
                WHERE id = ?
                "#,
                attachment.filename,
                attachment.content_type,
                attachment.size,
                attachment.remote_url,
                attachment.remote_path,
                attachment.is_inline,
                attachment.content_id,
                record.id
            )
            .execute(&self.pool)
            .await
            .map_err(|e| SyncError::DatabaseError(e.to_string()))?;

            let attachment_id_str = record.id.as_str();
            let attachment_id = Uuid::parse_str(&attachment_id_str)
                .map_err(|e| SyncError::DatabaseError(format!("Invalid ULID: {}", e)))?;
            Ok(attachment_id)
        } else {
            let attachment_id = Uuid::now_v7();
            let attachment_id_str = attachment_id.to_string();
            sqlx::query!(
                r#"
                INSERT INTO attachments (
                    id, email_id, filename, content_type, size, hash,
                    cache_path, remote_url, remote_path, is_inline, is_cached, content_id
                )
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                "#,
                attachment_id_str,
                email_id_str,
                attachment.filename,
                attachment.content_type,
                attachment.size,
                attachment.hash,
                None::<String>,
                attachment.remote_url,
                attachment.remote_path,
                attachment.is_inline,
                false,
                attachment.content_id
            )
            .execute(&self.pool)
            .await
            .map_err(|e| SyncError::DatabaseError(e.to_string()))?;

            Ok(attachment_id)
        }
    }

    /// Cache an attachment to storage and update database
    /// This is the only place where cache_path is set
    /// Also computes and updates the hash based on content
    pub async fn cache_attachment(
        &self,
        attachment_id: Uuid,
        account_id: Uuid,
        email_id: Uuid,
        data: &[u8],
        filename: &str,
    ) -> SyncResult<String> {
        let content_hash = format!("{:x}", md5::compute(data));

        let cache_path = PathGenerator::generate_cache_path(
            &account_id.to_string(),
            &email_id.to_string(),
            filename,
        );
        let path_buf = PathGenerator::cache_path_to_pathbuf(&cache_path);

        self.storage.store(&path_buf, data).await?;

        let attachment_id_str = attachment_id.to_string();
        sqlx::query!(
            "UPDATE attachments SET cache_path = ?, is_cached = 1, hash = ? WHERE id = ?",
            cache_path,
            content_hash,
            attachment_id_str
        )
        .execute(&self.pool)
        .await
        .map_err(|e| SyncError::DatabaseError(e.to_string()))?;

        log::debug!(
            "Cached attachment {} (ID: {}) to {} with hash {}",
            filename,
            attachment_id,
            cache_path,
            content_hash
        );

        Ok(cache_path)
    }

    /// Get attachment data from cache
    pub async fn get_attachment_data(&self, attachment_id: Uuid) -> SyncResult<Vec<u8>> {
        let attachment_id_str = attachment_id.to_string();
        let record = sqlx::query!(
            "SELECT cache_path, is_cached FROM attachments WHERE id = ?",
            attachment_id_str
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| SyncError::DatabaseError(e.to_string()))?;

        if !record.is_cached {
            return Err(SyncError::AttachmentError(
                "Attachment not cached".to_string(),
            ));
        }

        let cache_path = record.cache_path.ok_or_else(|| {
            SyncError::AttachmentError("Attachment cached but path is missing".to_string())
        })?;

        let path_buf = PathGenerator::cache_path_to_pathbuf(&cache_path);
        self.storage.retrieve(&path_buf).await
    }

    /// Get attachments metadata for an email
    pub async fn get_email_attachments(&self, email_id: Uuid) -> SyncResult<Vec<SyncAttachment>> {
        let email_id_str = email_id.to_string();
        let records = sqlx::query!(
            r#"
            SELECT id, email_id, filename, content_type, size, hash,
                   cache_path, remote_url, remote_path, is_inline, is_cached, content_id
            FROM attachments
            WHERE email_id = ?
            "#,
            email_id_str
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| SyncError::DatabaseError(e.to_string()))?;

        let attachments = records
            .into_iter()
            .filter_map(|record| {
                let id = Some(Uuid::parse_str(record.id.as_str()).unwrap());
                let email_id = Uuid::parse_str(&record.email_id).ok();
                Some(SyncAttachment {
                    id,
                    email_id,
                    filename: record.filename,
                    content_type: record.content_type,
                    size: record.size,
                    hash: record.hash,
                    cache_path: record.cache_path,
                    remote_url: record.remote_url,
                    remote_path: record.remote_path,
                    is_inline: record.is_inline,
                    is_cached: record.is_cached,
                    content_id: record.content_id,
                    data: None,
                })
            })
            .collect();

        Ok(attachments)
    }

    /// Clear attachment cache for an account
    pub async fn clear_cache(&self, account_id: Uuid) -> SyncResult<()> {
        let account_id_str = account_id.to_string();
        let account_path = PathGenerator::cache_path_to_pathbuf(&account_id_str);
        self.storage.delete_directory(&account_path).await?;

        sqlx::query!(
            r#"
            UPDATE attachments
            SET cache_path = NULL, is_cached = 0
            WHERE email_id IN (
                SELECT id FROM emails WHERE account_id = ?
            )
            "#,
            account_id_str
        )
        .execute(&self.pool)
        .await
        .map_err(|e| SyncError::DatabaseError(e.to_string()))?;

        log::info!("Cleared attachment cache for account {}", account_id);
        Ok(())
    }

    /// Check if attachment is cached
    pub async fn is_cached(&self, attachment_id: Uuid) -> SyncResult<bool> {
        let attachment_id_str = attachment_id.to_string();
        let record = sqlx::query!(
            "SELECT is_cached FROM attachments WHERE id = ?",
            attachment_id_str
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| SyncError::DatabaseError(e.to_string()))?;

        Ok(record.is_cached)
    }

    /// Get attachment metadata by ID
    pub async fn get_attachment_metadata(&self, attachment_id: Uuid) -> SyncResult<SyncAttachment> {
        let attachment_id_str = attachment_id.to_string();
        let record = sqlx::query!(
            r#"
            SELECT id, email_id, filename, content_type, size, hash,
                   cache_path, remote_url, remote_path, is_inline, is_cached, content_id
            FROM attachments
            WHERE id = ?
            "#,
            attachment_id_str
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| SyncError::DatabaseError(e.to_string()))?;

        let id = Some(Uuid::parse_str(record.id.as_str()).unwrap());
        let email_id = Uuid::parse_str(&record.email_id).ok();

        Ok(SyncAttachment {
            id,
            email_id,
            filename: record.filename,
            content_type: record.content_type,
            size: record.size,
            hash: record.hash,
            cache_path: record.cache_path,
            remote_url: record.remote_url,
            remote_path: record.remote_path,
            is_inline: record.is_inline,
            is_cached: record.is_cached,
            content_id: record.content_id,
            data: None,
        })
    }
}
