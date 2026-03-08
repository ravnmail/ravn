CREATE TABLE IF NOT EXISTS notification_reminder_events (
    id TEXT PRIMARY KEY NOT NULL,
    email_id TEXT NOT NULL,
    remind_at TIMESTAMP NOT NULL,
    notified_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_notification_reminder_events_email_remind_at
    ON notification_reminder_events(email_id, remind_at);

CREATE INDEX IF NOT EXISTS idx_notification_reminder_events_notified_at
    ON notification_reminder_events(notified_at);
