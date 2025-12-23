import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import type { EmailDetail } from '~/types/email'
import { useQuery } from '@tanstack/vue-query'

interface ChatMessage {
  role: string
  content: string
}

interface AskAiContext {
  history: ChatMessage[]
}

interface AskAiResult {
  completion: string
  error?: string
}

interface EmailMetadata {
  sender: string
  subject: string
  is_reply: boolean
  recipients: string[]
}

interface EmailCompletionContext {
  metadata: EmailMetadata
  prior_email?: string
  current_text: string
  cursor_position: number
}

interface CompletionResult {
  completion: string
  error?: string
}

interface GenerateSubjectContext {
  body_content: string
  sender: string
  recipients: string[]
  is_reply: boolean
  current_subject?: string
}

interface EmailAnalysis {
  gist: string
  responses: Array<{
    title: string
    content: string
  }>
}

interface EmailAnalysisResult {
  analysis: EmailAnalysis | null
  error?: string
}

interface AvailableModel {
  id: string
  name: string
}


const QUERY_KEYS = {
  all: ['corvus'] as const,
  models: () => [...QUERY_KEYS.all, 'models'] as const,
}

interface WritingStyleResult {
  style?: string
  error?: string
}

let listenerRegistered = false
const globalUnlistenFn = ref<(() => void) | null>(null)

export function useCorvus() {
  const isAskingAi = ref(false)
  const askAiError = ref<string | null>(null)
  const askAiResponse = ref<string | null>(null)

  const isGeneratingCompletion = ref(false)
  const completionError = ref<string | null>(null)
  const completionSuggestion = ref<string | null>(null)

  const isGeneratingSubject = ref(false)
  const subjectError = ref<string | null>(null)
  const generatedSubject = ref<string | null>(null)

  const isAnalyzing = ref(false)
  const analysisError = ref<string | null>(null)
  const currentAnalysis = ref<EmailAnalysis | null>(null)
  const analyzingEmailId = ref<string | null>(null)

  const isLoadingModels = ref(false)
  const modelsError = ref<string | null>(null)
  const availableModels = ref<AvailableModel[]>([])

  const isLoadingWritingStyle = ref(false)
  const isSavingWritingStyle = ref(false)
  const writingStyleError = ref<string | null>(null)
  const writingStyle = ref<string | null>(null)

  const askAi = async (history: ChatMessage[]): Promise<string | null> => {
    try {
      isAskingAi.value = true
      askAiError.value = null
      askAiResponse.value = null

      const context: AskAiContext = { history }
      const result = await invoke<AskAiResult>('ask_ai', { context })

      if (result.error) {
        askAiError.value = result.error
        return null
      }

      askAiResponse.value = result.completion
      return result.completion
    } catch (error) {
      const message = error instanceof Error ? error.message : 'Failed to get AI response'
      console.error('askAi error:', error)
      askAiError.value = message
      return null
    } finally {
      isAskingAi.value = false
    }
  }

  const generateEmailCompletion = async (
    metadata: EmailMetadata,
    currentText: string,
    cursorPosition: number,
    priorEmail?: string
  ): Promise<string | null> => {
    try {
      isGeneratingCompletion.value = true
      completionError.value = null
      completionSuggestion.value = null

      const context: EmailCompletionContext = {
        metadata,
        current_text: currentText,
        cursor_position: cursorPosition,
        prior_email: priorEmail,
      }

      const result = await invoke<CompletionResult>('generate_email_completion', { context })

      if (result.error) {
        completionError.value = result.error
        return null
      }

      completionSuggestion.value = result.completion
      return result.completion
    } catch (error) {
      const message = error instanceof Error ? error.message : 'Failed to generate completion'
      console.error('generateEmailCompletion error:', error)
      completionError.value = message
      return null
    } finally {
      isGeneratingCompletion.value = false
    }
  }

  const generateSubject = async (
    bodyContent: string,
    sender: string,
    recipients: string[],
    isReply: boolean,
    currentSubject?: string
  ): Promise<string | null> => {
    try {
      isGeneratingSubject.value = true
      subjectError.value = null
      generatedSubject.value = null

      const context: GenerateSubjectContext = {
        body_content: bodyContent,
        sender,
        recipients,
        is_reply: isReply,
        current_subject: currentSubject,
      }

      const result = await invoke<CompletionResult>('generate_subject', { context })

      if (result.error) {
        subjectError.value = result.error
        return null
      }

      generatedSubject.value = result.completion
      return result.completion
    } catch (error) {
      const message = error instanceof Error ? error.message : 'Failed to generate subject'
      console.error('generateSubject error:', error)
      subjectError.value = message
      return null
    } finally {
      isGeneratingSubject.value = false
    }
  }

  const analyzeEmail = async (email: EmailDetail): Promise<EmailAnalysis | null> => {
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
      const message = error instanceof Error ? error.message : 'Failed to analyze email'
      console.error('analyzeEmail error:', error)
      analysisError.value = message
      return null
    } finally {
      isAnalyzing.value = false
      analyzingEmailId.value = null
    }
  }

  const useGetModels = () => useQuery({
    queryKey: QUERY_KEYS.models(),
    queryFn: async() => {
      const { models } = await invoke<{models: AvailableModel[], error: string | null}>('get_available_models')
      return models
    }
  })

  const parseAnalysisFromCache = (email: EmailDetail): EmailAnalysis | null => {
    if (!email.ai_cache) return null

    try {
      return JSON.parse(email.ai_cache) as EmailAnalysis
    } catch (error) {
      console.error('Failed to parse ai_cache:', error)
      return null
    }
  }

  const clearAiState = () => {
    askAiResponse.value = null
    askAiError.value = null
    isAskingAi.value = false
  }

  const clearCompletionState = () => {
    completionSuggestion.value = null
    completionError.value = null
    isGeneratingCompletion.value = false
  }

  const clearSubjectState = () => {
    generatedSubject.value = null
    subjectError.value = null
    isGeneratingSubject.value = false
  }

  const clearAnalysisState = () => {
    currentAnalysis.value = null
    analysisError.value = null
    isAnalyzing.value = false
    analyzingEmailId.value = null
  }

  const getWritingStyle = async (): Promise<string | null> => {
    try {
      isLoadingWritingStyle.value = true
      writingStyleError.value = null

      const result = await invoke<WritingStyleResult>('get_writing_style')

      if (result.error) {
        writingStyleError.value = result.error
        return null
      }

      writingStyle.value = result.style || null

      return writingStyle.value
    } catch (error) {
      const message = error instanceof Error ? error.message : 'Failed to fetch writing style'
      console.error('getWritingStyle error:', error)
      writingStyleError.value = message
      return null
    } finally {
      isLoadingWritingStyle.value = false
    }
  }

  const setWritingStyle = async (styleStr: string): Promise<boolean> => {
    try {
      isSavingWritingStyle.value = true
      writingStyleError.value = null

      const request = { style: styleStr }
      const result = await invoke<WritingStyleResult>('set_writing_style', { request })

      if (result.error) {
        writingStyleError.value = result.error
        return false
      }

      writingStyle.value = result.style || null

      return true
    } catch (error) {
      const message = error instanceof Error ? error.message : 'Failed to save writing style'
      console.error('setWritingStyle error:', error)
      writingStyleError.value = message
      return false
    } finally {
      isSavingWritingStyle.value = false
    }
  }

  const clearWritingStyleState = () => {
    writingStyle.value = null
    writingStyleError.value = null
    isLoadingWritingStyle.value = false
    isSavingWritingStyle.value = false
  }

  const clearAllStates = () => {
    clearAiState()
    clearCompletionState()
    clearSubjectState()
    clearAnalysisState()
    clearWritingStyleState()
  }

  const askAiStreaming = async (history: ChatMessage[]): Promise<string | null> => {
    try {
      isAskingAi.value = true
      askAiError.value = null
      askAiResponse.value = ''

      const context: AskAiContext = { history }
      const requestId = await invoke<string>('ask_ai_streaming', { context })

      let fullResponse = ''

      return new Promise((resolve, reject) => {
        let chunkUnlisten: (() => void) | null = null
        let completeUnlisten: (() => void) | null = null
        let errorUnlisten: (() => void) | null = null

        const setupListeners = async () => {
          chunkUnlisten = await listen<string>(
            `corvus:ask-ai-chunk-${requestId}`,
            (event) => {
              fullResponse += event.payload
              askAiResponse.value = fullResponse
            }
          )

          completeUnlisten = await listen<string>(
            `corvus:ask-ai-complete-${requestId}`,
            () => {
              cleanup()
              isAskingAi.value = false
              resolve(fullResponse)
            }
          )

          errorUnlisten = await listen<string>(
            `corvus:ask-ai-error-${requestId}`,
            (event) => {
              cleanup()
              askAiError.value = event.payload
              isAskingAi.value = false
              reject(new Error(event.payload))
            }
          )
        }

        const cleanup = () => {
          chunkUnlisten?.()
          completeUnlisten?.()
          errorUnlisten?.()
        }

        setupListeners().catch(reject)
      })
    } catch (error) {
      const message = error instanceof Error ? error.message : 'Failed to stream AI response'
      console.error('askAiStreaming error:', error)
      askAiError.value = message
      isAskingAi.value = false
      return null
    }
  }

  const generateEmailCompletionStreaming = async (
    metadata: EmailMetadata,
    currentText: string,
    cursorPosition: number,
    priorEmail?: string
  ): Promise<string | null> => {
    try {
      isGeneratingCompletion.value = true
      completionError.value = null
      completionSuggestion.value = ''

      const context: EmailCompletionContext = {
        metadata,
        current_text: currentText,
        cursor_position: cursorPosition,
        prior_email: priorEmail,
      }

      const requestId = await invoke<string>('generate_email_completion_streaming', { context })

      let fullCompletion = ''

      return new Promise((resolve, reject) => {
        let chunkUnlisten: (() => void) | null = null
        let completeUnlisten: (() => void) | null = null
        let errorUnlisten: (() => void) | null = null

        const setupListeners = async () => {
          chunkUnlisten = await listen<string>(
            `corvus:completion-chunk-${requestId}`,
            (event) => {
              fullCompletion += event.payload
              completionSuggestion.value = fullCompletion
            }
          )

          completeUnlisten = await listen<string>(
            `corvus:completion-complete-${requestId}`,
            () => {
              cleanup()
              isGeneratingCompletion.value = false
              resolve(fullCompletion)
            }
          )

          errorUnlisten = await listen<string>(
            `corvus:completion-error-${requestId}`,
            (event) => {
              cleanup()
              completionError.value = event.payload
              isGeneratingCompletion.value = false
              reject(new Error(event.payload))
            }
          )
        }

        const cleanup = () => {
          chunkUnlisten?.()
          completeUnlisten?.()
          errorUnlisten?.()
        }

        setupListeners().catch(reject)
      })
    } catch (error) {
      const message = error instanceof Error ? error.message : 'Failed to stream completion'
      console.error('generateEmailCompletionStreaming error:', error)
      completionError.value = message
      isGeneratingCompletion.value = false
      return null
    }
  }

  const generateSubjectStreaming = async (
    bodyContent: string,
    sender: string,
    recipients: string[],
    isReply: boolean,
    currentSubject?: string
  ): Promise<string | null> => {
    try {
      isGeneratingSubject.value = true
      subjectError.value = null
      generatedSubject.value = ''

      const context: GenerateSubjectContext = {
        body_content: bodyContent,
        sender,
        recipients,
        is_reply: isReply,
        current_subject: currentSubject,
      }

      const requestId = await invoke<string>('generate_subject_streaming', { context })

      let fullSubject = ''

      return new Promise((resolve, reject) => {
        let chunkUnlisten: (() => void) | null = null
        let completeUnlisten: (() => void) | null = null
        let errorUnlisten: (() => void) | null = null

        const setupListeners = async () => {
          chunkUnlisten = await listen<string>(
            `corvus:subject-chunk-${requestId}`,
            (event) => {
              fullSubject += event.payload
              generatedSubject.value = fullSubject
            }
          )

          completeUnlisten = await listen<string>(
            `corvus:subject-complete-${requestId}`,
            () => {
              cleanup()
              isGeneratingSubject.value = false
              resolve(fullSubject)
            }
          )

          errorUnlisten = await listen<string>(
            `corvus:subject-error-${requestId}`,
            (event) => {
              cleanup()
              subjectError.value = event.payload
              isGeneratingSubject.value = false
              reject(new Error(event.payload))
            }
          )
        }

        const cleanup = () => {
          chunkUnlisten?.()
          completeUnlisten?.()
          errorUnlisten?.()
        }

        setupListeners().catch(reject)
      })
    } catch (error) {
      const message = error instanceof Error ? error.message : 'Failed to stream subject'
      console.error('generateSubjectStreaming error:', error)
      subjectError.value = message
      isGeneratingSubject.value = false
      return null
    }
  }

  onMounted(async () => {
    if (listenerRegistered) return
    listenerRegistered = true

    try {
      const unlisten = await listen<string>('email:ai-analysis-complete', (event) => {
        console.log('AI analysis complete for email:', event.payload)
      })

      globalUnlistenFn.value = unlisten
    } catch (error) {
      console.error('Failed to set up event listener:', error)
    }
  })

  onUnmounted(() => {
    if (globalUnlistenFn.value) {
      globalUnlistenFn.value()
      globalUnlistenFn.value = null
      listenerRegistered = false
    }
  })

  return {
    isAskingAi,
    askAiError,
    askAiResponse,
    askAi,
    askAiStreaming,
    clearAiState,

    isGeneratingCompletion,
    completionError,
    completionSuggestion,
    generateEmailCompletion,
    generateEmailCompletionStreaming,
    clearCompletionState,

    isGeneratingSubject,
    subjectError,
    generatedSubject,
    generateSubject,
    generateSubjectStreaming,
    clearSubjectState,

    isAnalyzing,
    analysisError,
    currentAnalysis,
    analyzingEmailId,
    analyzeEmail,
    clearAnalysisState,

    isLoadingModels,
    modelsError,
    availableModels,
    useGetModels,

    isLoadingWritingStyle,
    isSavingWritingStyle,
    writingStyleError,
    writingStyle,
    getWritingStyle,
    setWritingStyle,
    clearWritingStyleState,

    parseAnalysisFromCache,
    clearAllStates,
  }
}
