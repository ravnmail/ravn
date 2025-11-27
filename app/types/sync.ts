import type { EmailAddress } from './email'

// Sync types
export interface FolderSettings {
  cache_attachments: boolean
  sort_by: string
  sort_order: string
  grouping_enabled: boolean
  expanded_groups: string[]
  filter_read?: boolean | null
  filter_has_attachments?: boolean | null
}

export interface Folder {
  id: string
  account_id: string
  parent_id?: string
  remote_id: string
  name: string
  folder_type: FolderType
  icon?: string
  color?: string
  hidden?: boolean
  sort_order?: number
  unread_count: number
  total_count: number
  synced_at?: string
  expanded?: boolean
  settings?: FolderSettings
}

export interface NavigationFolder extends Folder {
  count?: number
  children?: Folder[]
}


export type FolderType =
  | 'inbox'
  | 'sent'
  | 'draft'
  | 'trash'
  | 'spam'
  | 'archive'
  | 'custom'
  | 'starred'

export interface SyncEmail {
  id?: string
  account_id: string
  folder_id: string
  message_id: string
  conversation_id?: string
  remote_id: string
  from: EmailAddress
  to: EmailAddress[]
  cc: EmailAddress[]
  bcc: EmailAddress[]
  reply_to?: EmailAddress
  subject?: string
  snippet?: string
  body_plain?: string
  body_html?: string
  ai_cache?: string
  received_at: string
  sent_at?: string
  flags: string[]
  headers?: Record<string, string>
  size: number
  has_attachments: boolean
  attachments: SyncAttachment[]
}

export interface SyncAttachment {
  id?: string
  email_id?: string
  filename: string
  content_type: string
  size: number
  hash: string
  cache_path?: string
  remote_url?: string
  remote_path?: string
  is_inline: boolean
  is_cached: boolean
  content_id?: string
}

export interface SyncReport {
  folders_synced: number
  emails_synced: number
  errors: string[]
}

// Account types
export interface Account {
  id: string
  name: string
  email: string
  account_type: AccountType
  settings: AccountSettings
  created_at: string
  updated_at: string
}

export type AccountType = 'gmail' | 'office365' | 'apple' | 'imap'

export interface AccountSettings {
  imap_host?: string
  imap_port?: number
  imap_use_tls?: boolean
  imap_username?: string
  smtp_host?: string
  smtp_port?: number
  smtp_use_tls?: boolean
  smtp_username?: string
  sync_enabled: boolean
  sync_interval?: number
  sync_on_startup: boolean
  cache_attachments: boolean
  max_attachment_cache_size?: number
  auto_download_inline: boolean
  provider_settings?: Record<string, unkown>
}

// Auth types
export interface StartOAuth2Request {
  provider: string
  redirect_uri: string
}

export interface StartOAuth2Response {
  auth_url: string
  csrf_token: string
}

export interface ExchangeOAuth2CodeRequest {
  provider: string
  code: string
  redirect_uri: string
  csrf_token: string
  account_id: string
}

export interface StoreImapCredentialsRequest {
  account_id: string
  username: string
  password: string
}

export interface ImapConnectionConfig {
  host: string
  port: number
  username: string
  password: string
  use_tls: boolean
}

export interface ProviderConfig {
  id: AccountType
  name: string
  icon: string
  description: string
  auth_type: 'oauth' | 'basic'
  color: string
}

// Auth flow state
export interface AuthFlowState {
  step: 'select' | 'configure' | 'connecting' | 'success' | 'error'
  provider?: AccountType
  error?: string
  account_id?: string
}

export interface CreateAccountRequest {
  name: string
  email: string
  account_type: AccountType
  settings?: AccountSettings
}

export interface CredentialsRequiredEvent {
  account_id: string
  provider: string
  reason: string
}