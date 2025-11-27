mod account_repository;
mod attachment_repository;
mod contact_repository;
mod conversation_repository;
mod email_repository;
mod folder_repository;
mod label_repository;
mod sync_state_repository;
mod view_repository;

pub use account_repository::*;
pub use attachment_repository::*;
pub use contact_repository::*;
pub use conversation_repository::*;
pub use email_repository::*;
pub use folder_repository::*;
pub use label_repository::*;
pub use sync_state_repository::*;
pub use view_repository::*;

use sqlx::SqlitePool;

pub struct RepositoryFactory {
    pool: SqlitePool,
}

impl RepositoryFactory {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub fn account_repository(&self) -> SqliteAccountRepository {
        SqliteAccountRepository::new(self.pool.clone())
    }

    pub fn email_repository(&self) -> SqliteEmailRepository {
        SqliteEmailRepository::new(self.pool.clone())
    }

    pub fn folder_repository(&self) -> SqliteFolderRepository {
        SqliteFolderRepository::new(self.pool.clone())
    }

    pub fn label_repository(&self) -> SqliteLabelRepository {
        SqliteLabelRepository::new(self.pool.clone())
    }

    pub fn attachment_repository(&self) -> SqliteAttachmentRepository {
        SqliteAttachmentRepository::new(self.pool.clone())
    }

    pub fn contact_repository(&self) -> SqliteContactRepository {
        SqliteContactRepository::new(self.pool.clone())
    }

    pub fn view_repository(&self) -> SqliteViewRepository {
        SqliteViewRepository::new(self.pool.clone())
    }

    pub fn conversation_repository(&self) -> SqliteConversationRepository {
        SqliteConversationRepository::new(self.pool.clone())
    }

    pub fn sync_state_repository(&self) -> SqliteSyncStateRepository {
        SqliteSyncStateRepository::new(self.pool.clone())
    }
}
