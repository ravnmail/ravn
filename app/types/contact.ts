export interface Contact {
  id: string
  account_id: string
  display_name: string | null
  first_name: string | null
  last_name: string | null
  company: string | null
  email: string
  source: 'observed' | 'imported' | 'manual'
  avatar_type: 'gravatar' | 'unavatar' | 'favicon' | 'none' | 'unprocessed'
  avatar_path: string
  send_count: number
  receive_count: number
  last_used_at: string | null
  first_seen_at: string
  created_at: string
  updated_at: string
}

export interface ContactSummary {
  id: string
  email: string
  display_name: string | null
  avatar_path: string
  send_count: number
  receive_count: number
  last_used_at: string | null
  usage_score: number
}

export interface SearchContactsRequest {
  account_id: string
  query: string
  limit?: number
}

export interface GetTopContactsRequest {
  account_id: string
  limit?: number
}

export interface GetContactsRequest {
  account_id: string
  limit?: number
  offset?: number
}
