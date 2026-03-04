-- Add remind_at column to emails table for calendar-based workflow
ALTER TABLE emails ADD COLUMN remind_at TIMESTAMP;

-- Index for efficient calendar queries
CREATE INDEX IF NOT EXISTS idx_emails_remind_at ON emails(remind_at) WHERE remind_at IS NOT NULL;
