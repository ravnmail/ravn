export interface EmailAddress {
  address: string;
  name?: string;
}

export interface LabelInfo {
  id: string;
  name: string;
  color?: string;
  icon?: string;
}

export interface AttachmentInfo {
  id: string;
  email_id: string;
  filename: string;
  content_type: string;
  size: number;
  is_inline: boolean;
  is_cached: boolean;
  content_id?: string;
  cache_path?: string;
  hash: string;
}

export type EmailCategory = 'personal' | 'transactions' | 'updates' | 'promotions'

/**
 * Minimal email data for list views
 * Optimized for performance with only essential fields
 */
export interface EmailListItem {
  id: string;
  account_id: string;
  folder_id: string;
  message_id: string;
  conversation_id?: string;

  from: EmailAddress;
  to: EmailAddress[];
  cc: EmailAddress[];
  bcc: EmailAddress[];

  subject?: string;
  snippet?: string;
  category?: EmailCategory;

  received_at: string; // ISO date string
  sent_at?: string; // ISO date string

  is_read: boolean;
  is_draft: boolean;
  is_flagged: boolean;
  sync_status: string;
  has_attachments: boolean;
  size: number;

  labels: LabelInfo[];
}

/**
 * Full email data for detail view
 * Includes all fields and related data
 */
export interface EmailDetail extends EmailListItem {
  remote_id?: string;

  reply_to?: EmailAddress;

  body_plain?: string;
  body_html?: string;
  other_mails?: string;
  ai_cache?: string;

  headers?: string;
  size: number;

  scheduled_send_at?: string; // ISO date string

  body_fetch_attempts: number;
  last_body_fetch_attempt?: string; // ISO date string

  tracking_blocked: boolean;
  images_blocked: boolean;

  created_at: string; // ISO date string
  updated_at: string; // ISO date string

  attachments: AttachmentInfo[];
}

/**
 * Legacy Email interface - kept for backward compatibility
 * @deprecated Use EmailListItem or EmailDetail instead
 */
export interface Email {
  id: string;
  account_id: string;
  folder_id: string;
  message_id: string;
  conversation_id?: string;
  remote_id?: string;

  from: EmailAddress;
  to: EmailAddress[];
  cc?: EmailAddress[];
  bcc?: EmailAddress[];
  reply_to?: EmailAddress[];

  subject?: string;
  snippet?: string;
  body_plain?: string;
  body_html?: string;
  ai_cache?: string;
  received_at: string; // ISO date string
  sent_at?: string; // ISO date string
  scheduled_send_at?: string; // ISO date string
  is_read: boolean;
  is_flagged: boolean;
  has_attachments: boolean;
  is_draft: boolean;
  is_deleted: boolean;
  sync_status: string;
  tracking_blocked: boolean;
  images_blocked: boolean;
  message_count?: number;

  created_at?: string; // ISO date string
  updated_at?: string; // ISO date string
}

export interface EmailAnalysisResponse {
  title: string;
  content: string;
}

export interface EmailAnalysis {
  gist: string;
  responses: EmailAnalysisResponse[];
}

export interface Attachment {
  id: string
  email_id: string
  filename: string
  content_type: string
  size: number
  is_inline: boolean
  is_cached: boolean
  full_path?: string
}