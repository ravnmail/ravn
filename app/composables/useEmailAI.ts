import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { ref, onMounted, onUnmounted } from 'vue'
import type { Email, EmailAnalysis } from '~/types/email'

interface EmailAnalysisResult {
  analysis: EmailAnalysis | null
  error: string | null
}

let listenerRegistered = false
const globalUnlistenFn = ref<(() => void) | null>(null)

export function useEmailAI() {
  const isAnalyzing = ref(false)
  const analysisError = ref<string | null>(null)
  const currentAnalysis = ref<EmailAnalysis | null>(null)
  const analyzingEmailId = ref<string | null>(null)

  const parseAnalysisFromCache = (email: Email): EmailAnalysis | null => {
    if (!email.ai_cache) return null

    try {
      return JSON.parse(email.ai_cache) as EmailAnalysis
    } catch (error) {
      console.error('Failed to parse ai_cache:', error)
      return null
    }
  }

  const analyzeEmail = async (email: Email): Promise<EmailAnalysis | null> => {
    if (!email || !email.id) {
      analysisError.value = 'Invalid email'
      return null
    }

    const cached = parseAnalysisFromCache(email)
    if (cached) {
      currentAnalysis.value = cached
      return cached
    }

    try {
      isAnalyzing.value = true
      analyzingEmailId.value = email.id
      analysisError.value = null

      const result = await invoke<EmailAnalysisResult>('analyze_email_with_ai', {
        emailId: email.id,
      })

      if (result.error) {
        analysisError.value = result.error
        return null
      }

      currentAnalysis.value = result.analysis
      return result.analysis
    } catch (error) {
      console.error('Failed to analyze email:', error)
      analysisError.value = error instanceof Error ? error.message : 'Failed to analyze email'
      return null
    } finally {
      isAnalyzing.value = false
      analyzingEmailId.value = null
    }
  }

  const clearAnalysis = () => {
    currentAnalysis.value = null
    analysisError.value = null
    isAnalyzing.value = false
    analyzingEmailId.value = null
  }

  onMounted(async () => {
    if (listenerRegistered) return
    listenerRegistered = true

    const unlisten = await listen<number>('email:ai-analysis-complete', (event) => {
      console.log('AI analysis complete for email:', event.payload)
    })

    globalUnlistenFn.value = unlisten
  })

  onUnmounted(() => {
    if (globalUnlistenFn.value) {
      globalUnlistenFn.value()
      globalUnlistenFn.value = null
      listenerRegistered = false
    }
  })

  return {
    isAnalyzing,
    analysisError,
    currentAnalysis,
    analyzingEmailId,
    analyzeEmail,
    clearAnalysis,
    parseAnalysisFromCache,
  }
}
