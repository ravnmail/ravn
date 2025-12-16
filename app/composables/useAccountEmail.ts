import { invoke } from '@tauri-apps/api/core'
import type { EmailAddress, EmailDetail } from '~/types/email'

export interface AccountForSending {
  id: string
  name: string
  email: string
  account_type: string
  has_smtp_config: boolean
}

export interface SendFromAccountRequest {
  account_id: string
  to: EmailAddress[]
  cc: EmailAddress[]
  bcc: EmailAddress[]
  subject: string
  body: string
  attachments: AttachmentData[]
  draft_id?: string
  conversation_id?: string
}

export interface SaveDraftRequest {
  account_id: string
  draft_id?: string
  to: EmailAddress[]
  cc: EmailAddress[]
  bcc: EmailAddress[]
  subject: string
  body: string
  scheduled_send_at?: string
  conversation_id?: string
}

export interface AttachmentData {
  filename: string
  content: number[]
  content_type?: string
}

export interface SendEmailResponse {
  success: boolean
  message: string
}

export interface SaveDraftResponse {
  success: boolean
  draft_id: string
  message: string
}

/**
 * Composable for account-based email sending and draft management
 */
export function useAccountEmail() {
  const isSending = ref(false)
  const isSavingDraft = ref(false)
  const error = ref<string | null>(null)
  const accounts = ref<AccountForSending[]>([])

  /**
   * Get all accounts that can send emails
   */
  const loadAccounts = async () => {
    try {
      error.value = null
      accounts.value = await invoke<AccountForSending[]>('get_accounts_for_sending')
      return accounts.value
    }
    catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
      console.error('Failed to load accounts:', error.value)
      throw e
    }
  }

  /**
   * Send an email from a specific account
   */
  const sendFromAccount = async (request: SendFromAccountRequest): Promise<SendEmailResponse> => {
    isSending.value = true
    error.value = null

    try {
      const response = await invoke<SendEmailResponse>('send_email_from_account', { request })
      return response
    }
    catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
      console.error('Failed to send email:', error.value)
      throw e
    }
    finally {
      isSending.value = false
    }
  }

  /**
   * Save or update a draft
   */
  const saveDraft = async (request: SaveDraftRequest): Promise<SaveDraftResponse> => {
    isSavingDraft.value = true
    error.value = null

    try {
      const response = await invoke<SaveDraftResponse>('save_draft', { request })
      return response
    }
    catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
      console.error('Failed to save draft:', error.value)
      throw e
    }
    finally {
      isSavingDraft.value = false
    }
  }

  /**
   * Get drafts for a specific account
   */
  const getDrafts = async (accountId: string): Promise<EmailDetail[]> => {
    try {
      error.value = null
      const drafts = await invoke<EmailDetail[]>('get_drafts', { accountId })
      return drafts
    }
    catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
      console.error('Failed to get drafts:', error.value)
      throw e
    }
  }

  /**
   * Delete a draft
   */
  const deleteDraft = async (draftId: string): Promise<SendEmailResponse> => {
    try {
      error.value = null
      const response = await invoke<SendEmailResponse>('delete_draft', { draftId })
      return response
    }
    catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
      console.error('Failed to delete draft:', error.value)
      throw e
    }
  }

  /**
   * Convert File to AttachmentData
   */
  const fileToAttachmentData = async (file: File): Promise<AttachmentData> => {
    return new Promise((resolve, reject) => {
      const reader = new FileReader()
      reader.onload = () => {
        const arrayBuffer = reader.result as ArrayBuffer
        const uint8Array = new Uint8Array(arrayBuffer)
        const content = Array.from(uint8Array)

        resolve({
          filename: file.name,
          content,
          content_type: file.type || undefined,
        })
      }
      reader.onerror = () => reject(reader.error)
      reader.readAsArrayBuffer(file)
    })
  }

  /**
   * Convert multiple files to attachment data
   */
  const filesToAttachmentData = async (files: File[]): Promise<AttachmentData[]> => {
    return Promise.all(files.map(file => fileToAttachmentData(file)))
  }

  return {
    isSending: readonly(isSending),
    isSavingDraft: readonly(isSavingDraft),
    error: readonly(error),
    accounts: readonly(accounts),
    loadAccounts,
    sendFromAccount,
    saveDraft,
    getDrafts,
    deleteDraft,
    fileToAttachmentData,
    filesToAttachmentData,
  }
}
