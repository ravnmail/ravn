use async_trait::async_trait;
use std::path::{Path, PathBuf};
use tokio::fs;
use tokio::io::AsyncWriteExt;

use super::error::SyncResult;

/// Trait for file storage operations (Open/Closed Principle)
/// Allows different storage backends (local, S3, etc.) without modifying existing code
#[async_trait]
pub trait FileStorage: Send + Sync {
    /// Store file data and return the storage path
    async fn store(&self, path: &Path, data: &[u8]) -> SyncResult<()>;

    /// Retrieve file data from storage
    async fn retrieve(&self, path: &Path) -> SyncResult<Vec<u8>>;

    /// Check if file exists in storage
    async fn exists(&self, path: &Path) -> bool;

    /// Delete file from storage
    async fn delete(&self, path: &Path) -> SyncResult<()>;

    /// Delete entire directory
    async fn delete_directory(&self, path: &Path) -> SyncResult<()>;
}

/// Local filesystem storage implementation
pub struct LocalFileStorage {
    base_dir: PathBuf,
}

impl LocalFileStorage {
    pub fn new(base_dir: PathBuf) -> Self {
        Self { base_dir }
    }

    /// Get full path by joining base_dir with relative path
    fn full_path(&self, path: &Path) -> PathBuf {
        self.base_dir.join(path)
    }

    /// Ensure parent directory exists
    async fn ensure_parent_dir(&self, path: &Path) -> SyncResult<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await?;
        }
        Ok(())
    }
}

#[async_trait]
impl FileStorage for LocalFileStorage {
    async fn store(&self, path: &Path, data: &[u8]) -> SyncResult<()> {
        let full_path = self.full_path(path);
        self.ensure_parent_dir(&full_path).await?;

        let mut file = fs::File::create(&full_path).await?;
        file.write_all(data).await?;
        file.sync_all().await?;

        Ok(())
    }

    async fn retrieve(&self, path: &Path) -> SyncResult<Vec<u8>> {
        let full_path = self.full_path(path);
        let data = fs::read(&full_path).await?;
        Ok(data)
    }

    async fn exists(&self, path: &Path) -> bool {
        let full_path = self.full_path(path);
        full_path.exists()
    }

    async fn delete(&self, path: &Path) -> SyncResult<()> {
        let full_path = self.full_path(path);
        if full_path.exists() {
            fs::remove_file(&full_path).await?;
        }
        Ok(())
    }

    async fn delete_directory(&self, path: &Path) -> SyncResult<()> {
        let full_path = self.full_path(path);
        if full_path.exists() {
            fs::remove_dir_all(&full_path).await?;
        }
        Ok(())
    }
}

/// Utility for generating and sanitizing file paths (Single Responsibility)
pub struct PathGenerator;

impl PathGenerator {
    pub fn sanitize_filename(filename: &str) -> String {
        filename.replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|'], "_")
    }

    pub fn generate_cache_path(account_id: &str, email_id: &str, filename: &str) -> String {
        let safe_filename = Self::sanitize_filename(filename);
        format!("{}/{}/{}", account_id, email_id, safe_filename)
    }

    pub fn cache_path_to_pathbuf(cache_path: &str) -> PathBuf {
        PathBuf::from(cache_path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_sanitize_filename() {
        assert_eq!(PathGenerator::sanitize_filename("test.pdf"), "test.pdf");
        assert_eq!(
            PathGenerator::sanitize_filename("test/file.pdf"),
            "test_file.pdf"
        );
        assert_eq!(
            PathGenerator::sanitize_filename("test:file?.pdf"),
            "test_file_.pdf"
        );
        assert_eq!(
            PathGenerator::sanitize_filename("test<>|.pdf"),
            "test___.pdf"
        );
    }

    #[test]
    fn test_generate_cache_path() {
        let path = PathGenerator::generate_cache_path(
            "7472b127-0955-4a80-9e14-4dc846be1f0f",
            "9216529d-a0c5-4cd3-8844-4ca86bffe3c7",
            "document.pdf",
        );
        assert_eq!(path, "7472b127-0955-4a80-9e14-4dc846be1f0f/9216529d-a0c5-4cd3-8844-4ca86bffe3c7/document.pdf");

        let path = PathGenerator::generate_cache_path(
            "7472b127-0955-4a80-9e14-4dc846be1f0f",
            "9216529d-a0c5-4cd3-8844-4ca86bffe3c7",
            "unsafe/file.pdf",
        );
        assert_eq!(path, "7472b127-0955-4a80-9e14-4dc846be1f0f/9216529d-a0c5-4cd3-8844-4ca86bffe3c7/unsafe_file.pdf");
    }

    #[tokio::test]
    async fn test_local_storage_operations() {
        let temp_dir = TempDir::new().unwrap();
        let storage = LocalFileStorage::new(temp_dir.path().to_path_buf());

        let test_path = Path::new("test/file.txt");
        let test_data = b"Hello, World!";

        storage.store(test_path, test_data).await.unwrap();
        assert!(storage.exists(test_path).await);

        let retrieved = storage.retrieve(test_path).await.unwrap();
        assert_eq!(retrieved, test_data);

        storage.delete(test_path).await.unwrap();
        assert!(!storage.exists(test_path).await);
    }

    #[tokio::test]
    async fn test_storage_directory_operations() {
        let temp_dir = TempDir::new().unwrap();
        let storage = LocalFileStorage::new(temp_dir.path().to_path_buf());

        let dir_path = Path::new("account1");
        storage
            .store(&dir_path.join("file1.txt"), b"data1")
            .await
            .unwrap();
        storage
            .store(&dir_path.join("file2.txt"), b"data2")
            .await
            .unwrap();

        storage.delete_directory(dir_path).await.unwrap();
        assert!(!storage.exists(&dir_path.join("file1.txt")).await);
        assert!(!storage.exists(&dir_path.join("file2.txt")).await);
    }
}
