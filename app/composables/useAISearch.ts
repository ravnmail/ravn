import { invoke } from '@tauri-apps/api/core'

export interface GenerateSearchQueryRequest {
  naturalLanguageQuery: string
}

export interface GenerateSearchQueryResponse {
  query: string
  explanation?: string
}

export const useAISearch = () => {
  const generating = ref<boolean>(false)
  const error = ref<string | null>(null)

  const generateSearchQuery = async (
    naturalLanguageQuery: string
  ): Promise<string | null> => {
    if (!naturalLanguageQuery.trim()) {
      error.value = 'Please enter a search query'
      return null
    }

    generating.value = true
    error.value = null

    try {
      const response = await invoke<GenerateSearchQueryResponse>(
        'generate_search_query',
        {
          naturalLanguageQuery,
        }
      )

      return response.query
    } catch (err: any) {
      console.error('[useAISearch] Error generating search query:', err)
      error.value = err.message || 'Failed to generate search query'
      return null
    } finally {
      generating.value = false
    }
  }

  const resetError = () => {
    error.value = null
  }

  return {
    generating,
    error,
    generateSearchQuery,
    resetError,
  }
}
