import type { AttachmentInfo, EmailDetail, EmailListItem } from './email'

/**
 * Conversation list item for displaying in list views
 * Contains minimal data with list of messages
 */
export interface ConversationListItem {
  id: string
  message_count: number
  ai_cache?: string
  messages: EmailListItem[]
}

/**
 * Full conversation details for detail view
 * Includes all messages and attachments
 */
export interface ConversationDetail {
  id: string
  message_count: number
  ai_cache?: string
  attachments: AttachmentInfo[]
  messages: EmailDetail[]
}
