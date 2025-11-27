-- Accounts: Root entity for email accounts
CREATE TABLE IF NOT EXISTS accounts (
    id TEXT NOT NULL PRIMARY KEY,
    name TEXT NOT NULL,
    email TEXT NOT NULL,
    account_type TEXT NOT NULL CHECK (account_type IN ('gmail', 'office365', 'apple', 'imap')),
    settings TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Contacts: Email contacts with interaction tracking
CREATE TABLE IF NOT EXISTS contacts (
    id TEXT NOT NULL PRIMARY KEY,
    account_id TEXT,
    display_name TEXT,
    first_name TEXT,
    last_name TEXT,
    company TEXT,
    email TEXT NOT NULL,
    notes TEXT,
    source TEXT NOT NULL DEFAULT 'observed' CHECK (source IN ('observed', 'imported', 'manual')),
    avatar_type TEXT NOT NULL CHECK (avatar_type IN ('gravatar', 'unavatar', 'favicon', 'none', 'unprocessed')),
    avatar_path TEXT,
    send_count INTEGER NOT NULL DEFAULT 0,
    receive_count INTEGER NOT NULL DEFAULT 0,
    last_used_at TIMESTAMP,
    first_seen_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (account_id) REFERENCES accounts(id) ON DELETE CASCADE
);
-- Conversations: Email conversation threads
CREATE TABLE IF NOT EXISTS conversations (
    id TEXT NOT NULL PRIMARY KEY,
    remote_id TEXT NOT NULL,
    message_count INTEGER NOT NULL DEFAULT 0,
    ai_cache TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Signatures: Email signatures per account
CREATE TABLE IF NOT EXISTS signatures (
    id TEXT NOT NULL PRIMARY KEY,
    account_id TEXT NOT NULL,
    name TEXT NOT NULL,
    signature TEXT NOT NULL,
    is_default BOOLEAN NOT NULL DEFAULT 0,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (account_id) REFERENCES accounts(id) ON DELETE CASCADE
);

-- Folders: Email folders/mailboxes with hierarchy support
CREATE TABLE IF NOT EXISTS folders (
    id TEXT NOT NULL PRIMARY KEY,
    account_id TEXT NOT NULL,
    name TEXT NOT NULL,
    folder_type TEXT NOT NULL DEFAULT 'custom'
        CHECK (folder_type IN ('inbox', 'sent', 'draft', 'trash', 'spam', 'archive', 'custom', 'starred')),
    remote_id TEXT,
    color TEXT,
    icon TEXT,
    settings TEXT NOT NULL DEFAULT '{"cache_attachments": false}',
    expanded BOOLEAN NOT NULL DEFAULT 0,
    hidden BOOLEAN NOT NULL DEFAULT 0,
    sort_order INTEGER NOT NULL DEFAULT 0,
    parent_id TEXT REFERENCES folders(id),
    sync_interval INTEGER NOT NULL DEFAULT 300,
    unread_count INTEGER NOT NULL DEFAULT 0,
    total_count INTEGER NOT NULL DEFAULT 0,
    synced_at TIMESTAMP NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (account_id) REFERENCES accounts(id) ON DELETE CASCADE
);

-- Labels: User-defined labels for email organization
CREATE TABLE IF NOT EXISTS labels (
    id TEXT NOT NULL PRIMARY KEY,
    name TEXT NOT NULL,
    color TEXT,
    icon TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Emails: Email messages with comprehensive metadata
CREATE TABLE IF NOT EXISTS emails (
    id TEXT NOT NULL PRIMARY KEY,
    account_id TEXT NOT NULL,
    folder_id TEXT NOT NULL,
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
    other_mails TEXT,
    ai_cache TEXT,
    flags TEXT NOT NULL DEFAULT '[]',
    category TEXT CHECK (category IN ('personal', 'transactions', 'updates', 'promotions')),
    headers TEXT,
    size INTEGER NOT NULL DEFAULT 0,
    is_read BOOLEAN NOT NULL DEFAULT 0,
    is_flagged BOOLEAN NOT NULL DEFAULT 0,
    is_draft BOOLEAN NOT NULL DEFAULT 0,
    is_deleted BOOLEAN NOT NULL DEFAULT 0,
    has_attachments BOOLEAN NOT NULL DEFAULT 0,
    tracking_blocked BOOLEAN NOT NULL DEFAULT 1,
    images_blocked BOOLEAN NOT NULL DEFAULT 1,
    change_key TEXT,
    sync_status TEXT NOT NULL DEFAULT 'synced',
    body_fetch_attempts INTEGER NOT NULL DEFAULT 0,
    last_modified_at TIMESTAMP,
    last_body_fetch_attempt TIMESTAMP,
    scheduled_send_at TIMESTAMP,
    received_at TIMESTAMP NOT NULL,
    sent_at TIMESTAMP,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (account_id) REFERENCES accounts(id) ON DELETE CASCADE,
    FOREIGN KEY (folder_id) REFERENCES folders(id) ON DELETE CASCADE,
    FOREIGN KEY (conversation_id) REFERENCES conversations(id) ON DELETE SET NULL
);

-- Email-Label junction table (many-to-many)
CREATE TABLE IF NOT EXISTS email_labels (
    email_id TEXT NOT NULL,
    label_id TEXT NOT NULL,
    value TEXT,
    type TEXT NOT NULL DEFAULT 'user'
        CHECK (type IN ('user', 'assistant', 'system')),
    PRIMARY KEY (email_id, label_id),
    FOREIGN KEY (email_id) REFERENCES emails(id) ON DELETE CASCADE,
    FOREIGN KEY (label_id) REFERENCES labels(id) ON DELETE CASCADE
);

-- Attachments: Email attachments with caching support
CREATE TABLE IF NOT EXISTS attachments (
    id TEXT NOT NULL PRIMARY KEY,
    email_id TEXT NOT NULL,
    filename TEXT NOT NULL,
    content_type TEXT NOT NULL,
    size INTEGER NOT NULL,
    hash TEXT NOT NULL,
    cache_path TEXT,
    is_inline BOOLEAN NOT NULL DEFAULT 0,
    is_cached BOOLEAN NOT NULL DEFAULT 0,
    content_id TEXT,
    remote_url TEXT,
    remote_path TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (email_id) REFERENCES emails(id) ON DELETE CASCADE
);

-- ============================================================================
-- Sync Tables
-- ============================================================================

-- Sync State: Track synchronization progress per folder
CREATE TABLE IF NOT EXISTS sync_state (
    id TEXT NOT NULL PRIMARY KEY,
    account_id TEXT NOT NULL,
    folder_id TEXT,
    last_sync_at TIMESTAMP,
    next_sync_at TIMESTAMP,
    last_uid INTEGER,
    sync_token TEXT,
    sync_status TEXT NOT NULL DEFAULT 'idle'
        CHECK (sync_status IN ('idle', 'syncing', 'error', 'paused')),
    error_message TEXT,
    error_count INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (account_id) REFERENCES accounts(id) ON DELETE CASCADE,
    FOREIGN KEY (folder_id) REFERENCES folders(id) ON DELETE CASCADE,
    UNIQUE(account_id, folder_id)
);

-- OAuth Credentials: Metadata for OAuth tokens (actual tokens in OS keyring)
CREATE TABLE IF NOT EXISTS oauth_credentials (
    id TEXT NOT NULL PRIMARY KEY,
    account_id TEXT NOT NULL,
    provider TEXT NOT NULL CHECK (provider IN ('gmail', 'office365')),
    token_type TEXT NOT NULL DEFAULT 'Bearer',
    expires_at TIMESTAMP,
    scopes TEXT NOT NULL,
    keyring_service TEXT NOT NULL,
    keyring_account TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (account_id) REFERENCES accounts(id) ON DELETE CASCADE,
    UNIQUE(account_id)
);

-- Encrypted Credentials: Fallback encrypted storage when keyring unavailable
CREATE TABLE IF NOT EXISTS encrypted_credentials (
    id TEXT NOT NULL PRIMARY KEY,
    account_id TEXT NOT NULL,
    credential_type TEXT NOT NULL CHECK(credential_type IN ('oauth2', 'imap')),
    encrypted_data BLOB NOT NULL,
    nonce BLOB NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (account_id) REFERENCES accounts(id) ON DELETE CASCADE,
    UNIQUE(account_id, credential_type)
);

-- Views: for Kanban and future view types
CREATE TABLE IF NOT EXISTS views (
    id TEXT NOT NULL PRIMARY KEY,
    icon TEXT,
    color TEXT,
    name TEXT NOT NULL,
    view_type TEXT NOT NULL DEFAULT 'kanban'
        CHECK (view_type IN ('list', 'kanban', 'calendar', 'smart', 'unified')),
    config TEXT NOT NULL DEFAULT '{}',
    folders TEXT NOT NULL DEFAULT '[]',
    sort_order INTEGER NOT NULL DEFAULT 0,
    is_default BOOLEAN NOT NULL DEFAULT 0,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- ============================================================================
-- Indexes
-- ============================================================================

-- Contact indexes
CREATE INDEX IF NOT EXISTS idx_contacts_email ON contacts(email);
CREATE INDEX IF NOT EXISTS idx_contacts_account ON contacts(account_id);

-- Email indexes
CREATE INDEX IF NOT EXISTS idx_emails_account_folder ON emails(account_id, folder_id);
CREATE INDEX IF NOT EXISTS idx_emails_conversation ON emails(conversation_id);
CREATE INDEX IF NOT EXISTS idx_emails_received_at ON emails(received_at);
CREATE INDEX IF NOT EXISTS idx_emails_message_id ON emails(message_id);
CREATE INDEX IF NOT EXISTS idx_emails_remote_id ON emails(remote_id);
CREATE INDEX IF NOT EXISTS idx_emails_account_id ON emails(account_id);
CREATE INDEX IF NOT EXISTS idx_emails_folder_id ON emails(folder_id);
CREATE INDEX IF NOT EXISTS idx_emails_category ON emails(category);
CREATE INDEX IF NOT EXISTS idx_emails_sync_status ON emails(sync_status);
CREATE INDEX IF NOT EXISTS idx_emails_change_key ON emails(change_key) WHERE change_key IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_emails_last_modified_at ON emails(last_modified_at) WHERE last_modified_at IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_emails_needs_body_fetch
    ON emails(sync_status, last_body_fetch_attempt)
    WHERE sync_status IN ('headers_only', 'error');

-- Attachment indexes
CREATE INDEX IF NOT EXISTS idx_attachments_hash ON attachments(hash);
CREATE INDEX IF NOT EXISTS idx_attachments_email_id ON attachments(email_id);
CREATE INDEX IF NOT EXISTS idx_attachments_cached ON attachments(is_cached, email_id);

-- Folder indexes
CREATE INDEX IF NOT EXISTS idx_folders_account ON folders(account_id);

-- Sync indexes
CREATE INDEX IF NOT EXISTS idx_sync_state_account ON sync_state(account_id);
CREATE INDEX IF NOT EXISTS idx_conversations_remote_id ON conversations(remote_id);
CREATE INDEX IF NOT EXISTS idx_sync_state_folder ON sync_state(folder_id);
CREATE INDEX IF NOT EXISTS idx_sync_state_next_sync ON sync_state(next_sync_at);
CREATE INDEX IF NOT EXISTS idx_sync_state_last_uid ON sync_state(last_uid);
CREATE INDEX IF NOT EXISTS idx_oauth_credentials_account ON oauth_credentials(account_id);
CREATE INDEX IF NOT EXISTS idx_encrypted_credentials_account ON encrypted_credentials(account_id);
CREATE INDEX IF NOT EXISTS idx_encrypted_credentials_type ON encrypted_credentials(credential_type);

-- Views indexes
CREATE INDEX IF NOT EXISTS idx_views_view_type ON views(view_type);
CREATE INDEX IF NOT EXISTS idx_views_sort_order ON views(sort_order);

-- ============================================================================
-- Triggers
-- ============================================================================

-- Updated_at triggers for timestamp maintenance
CREATE TRIGGER IF NOT EXISTS accounts_updated_at
   AFTER UPDATE ON accounts
BEGIN
    UPDATE accounts SET updated_at = CURRENT_TIMESTAMP
    WHERE id = NEW.id;
END;

CREATE TRIGGER IF NOT EXISTS emails_updated_at
   AFTER UPDATE ON emails
BEGIN
    UPDATE emails SET updated_at = CURRENT_TIMESTAMP
    WHERE id = NEW.id;
END;

CREATE TRIGGER IF NOT EXISTS labels_updated_at
   AFTER UPDATE ON labels
BEGIN
    UPDATE labels SET updated_at = CURRENT_TIMESTAMP
    WHERE id = NEW.id;
END;

CREATE TRIGGER IF NOT EXISTS folders_updated_at
   AFTER UPDATE ON folders
BEGIN
    UPDATE folders SET updated_at = CURRENT_TIMESTAMP
    WHERE id = NEW.id;
END;

CREATE TRIGGER IF NOT EXISTS sync_state_updated_at
   AFTER UPDATE ON sync_state
BEGIN
    UPDATE sync_state SET updated_at = CURRENT_TIMESTAMP
    WHERE id = NEW.id;
END;

CREATE TRIGGER IF NOT EXISTS oauth_credentials_updated_at
   AFTER UPDATE ON oauth_credentials
BEGIN
    UPDATE oauth_credentials SET updated_at = CURRENT_TIMESTAMP
    WHERE id = NEW.id;
END;

-- Trigger for conversations updated_at
CREATE TRIGGER IF NOT EXISTS conversations_updated_at
   AFTER UPDATE ON conversations
BEGIN
    UPDATE conversations SET updated_at = CURRENT_TIMESTAMP
    WHERE id = NEW.id;
END;

-- Trigger to update conversation message_count when emails are added/removed
CREATE TRIGGER IF NOT EXISTS update_conversation_count_insert
   AFTER INSERT ON emails
   WHEN NEW.conversation_id IS NOT NULL
BEGIN
    UPDATE conversations
    SET message_count = message_count + 1
    WHERE id = NEW.conversation_id;
END;

CREATE TRIGGER IF NOT EXISTS update_conversation_count_delete
   AFTER DELETE ON emails
   WHEN OLD.conversation_id IS NOT NULL
BEGIN
    UPDATE conversations
    SET message_count = message_count - 1
    WHERE id = OLD.conversation_id;
END;

CREATE TRIGGER IF NOT EXISTS update_conversation_count_update
   AFTER UPDATE ON emails
   WHEN OLD.conversation_id != NEW.conversation_id
BEGIN
    -- Decrement old conversation
    UPDATE conversations
    SET message_count = message_count - 1
    WHERE id = OLD.conversation_id AND OLD.conversation_id IS NOT NULL;

    -- Increment new conversation
    UPDATE conversations
    SET message_count = message_count + 1
    WHERE id = NEW.conversation_id AND NEW.conversation_id IS NOT NULL;
END;


-- Folder count maintenance triggers
CREATE TRIGGER update_folder_counts_insert
   AFTER INSERT ON emails
   WHEN NEW.is_deleted = 0
BEGIN
    UPDATE folders
    SET total_count = total_count + 1,
        unread_count = unread_count + (CASE WHEN NEW.is_read = 0 THEN 1 ELSE 0 END)
    WHERE id = NEW.folder_id;
END;

CREATE TRIGGER update_folder_counts_update
   AFTER UPDATE ON emails
   WHEN OLD.is_read != NEW.is_read OR OLD.is_deleted != NEW.is_deleted OR OLD.folder_id != NEW.folder_id
BEGIN
    -- Decrement old folder counts (only if old email wasn't deleted)
    UPDATE folders
    SET total_count = total_count - (CASE WHEN OLD.is_deleted = 0 THEN 1 ELSE 0 END),
        unread_count = unread_count - (CASE WHEN OLD.is_read = 0 AND OLD.is_deleted = 0 THEN 1 ELSE 0 END)
    WHERE id = OLD.folder_id AND OLD.folder_id != NEW.folder_id;

    -- Increment new folder counts (only if new email isn't deleted)
    UPDATE folders
    SET total_count = total_count + (CASE WHEN NEW.is_deleted = 0 THEN 1 ELSE 0 END),
        unread_count = unread_count + (CASE WHEN NEW.is_read = 0 AND NEW.is_deleted = 0 THEN 1 ELSE 0 END)
    WHERE id = NEW.folder_id AND OLD.folder_id != NEW.folder_id;

    -- Update counts for same folder (status changes only)
    UPDATE folders
    SET total_count = total_count +
        (CASE WHEN NEW.is_deleted = 0 THEN 0 ELSE 0 END) -
        (CASE WHEN OLD.is_deleted = 0 THEN 0 ELSE 0 END) +
        (CASE WHEN NEW.is_deleted != OLD.is_deleted THEN (CASE WHEN NEW.is_deleted = 1 THEN -1 ELSE 1 END) ELSE 0 END),
        unread_count = unread_count +
        (CASE WHEN NEW.is_read = 0 AND NEW.is_deleted = 0 THEN 1 ELSE 0 END) -
        (CASE WHEN OLD.is_read = 0 AND OLD.is_deleted = 0 THEN 1 ELSE 0 END)
    WHERE id = NEW.folder_id AND OLD.folder_id = NEW.folder_id;
END;

CREATE TRIGGER update_folder_counts_delete
   AFTER DELETE ON emails
   WHEN OLD.is_deleted = 0
BEGIN
    UPDATE folders
    SET total_count = total_count - 1,
        unread_count = unread_count - (CASE WHEN OLD.is_read = 0 THEN 1 ELSE 0 END)
    WHERE id = OLD.folder_id;
END;

CREATE TRIGGER IF NOT EXISTS update_views_updated_at
AFTER UPDATE ON views
FOR EACH ROW
BEGIN
    UPDATE views SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id;
END;
