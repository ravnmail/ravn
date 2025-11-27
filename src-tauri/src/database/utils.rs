use chrono::{DateTime, Utc};
use sha2::{Digest, Sha256};
use std::path::PathBuf;
use uuid::Uuid;

pub struct DatabaseUtils;

impl DatabaseUtils {
    /// Generates a cache path for an attachment
    pub fn generate_attachment_path(
        account_id: Uuid,
        received_date: DateTime<Utc>,
        original_filename: &str,
        content_hash: &str,
    ) -> PathBuf {
        let year_month = received_date.format("%Y-%m");
        let extension = std::path::Path::new(original_filename)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("bin");

        PathBuf::from("attachments")
            .join(account_id.to_string())
            .join(year_month.to_string())
            .join(format!("{}.{}", content_hash, extension))
    }

    /// Calculates SHA-256 hash of content for deduplication
    pub fn calculate_hash(content: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(content);
        format!("{:x}", hasher.finalize())
    }

    /// Sanitizes a filename for safe storage
    pub fn sanitize_filename(filename: &str) -> String {
        filename
            .chars()
            .map(|c| match c {
                '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
                _ => c,
            })
            .collect()
    }
}
