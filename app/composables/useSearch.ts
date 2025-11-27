import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { EmailListItem } from '~/types/email'

export interface SearchOptions {
  query: string
  accountId?: string
  folderId?: string
  unreadOnly?: boolean
  flaggedOnly?: boolean
  limit?: number
  offset?: number
}

export interface SearchResults {
  emails: EmailListItem[]
  total: number
}

export interface ReindexResult {
  total_indexed: number
  success: boolean
}

export function useSearch() {
  const loading = ref(false)
  const emails = ref<EmailListItem[]>([])
  const total = ref(0)
  const error = ref<string | null>(null)

  /**
   * Perform a full-text search for emails
   */
  const search = async (options: SearchOptions): Promise<SearchResults | null> => {
    if (!options.query.trim()) {
      emails.value = []
      total.value = 0
      return null
    }

    loading.value = true
    error.value = null

    try {
      const result = await invoke<SearchResults>('search_emails', {
        query: options.query,
        accountId: options.accountId,
        folderId: options.folderId,
        limit: options.limit ?? 100,
        offset: options.offset ?? 0,
      })

      emails.value = result.emails
      total.value = result.total

      return result
    } catch (err) {
      console.error('Search failed:', err)
      error.value = err instanceof Error ? err.message : 'Search failed'
      emails.value = []
      total.value = 0
      return null
    } finally {
      loading.value = false
    }
  }

  /**
   * Reindex all emails in the search index
   */
  const reindexAll = async (): Promise<ReindexResult | null> => {
    loading.value = true
    error.value = null

    try {
      const result = await invoke<ReindexResult>('reindex_all_emails')
      return result
    } catch (err) {
      console.error('Reindex failed:', err)
      error.value = err instanceof Error ? err.message : 'Reindex failed'
      return null
    } finally {
      loading.value = false
    }
  }

  /**
   * Reindex emails for a specific account
   */
  const reindexAccount = async (accountId: string): Promise<ReindexResult | null> => {
    loading.value = true
    error.value = null

    try {
      const result = await invoke<ReindexResult>('reindex_account_emails', {
        accountId,
      })
      return result
    } catch (err) {
      console.error('Reindex account failed:', err)
      error.value = err instanceof Error ? err.message : 'Reindex account failed'
      return null
    } finally {
      loading.value = false
    }
  }

  return {
    // State
    loading,
    emails,
    total,
    error,

    // Methods
    search,
    reindexAll,
    reindexAccount,
  }
}
