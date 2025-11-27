import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'

export interface GenerateSubjectContext {
  body_content: string
  sender: string
  recipients: string[]
  is_reply: boolean
  currentSubject?: string
}

export function useSubjectGeneration() {
  const isGenerating = ref(false)
  const error = ref<string | null>(null)

  const generateSubject = async (context: GenerateSubjectContext) => {
    isGenerating.value = true
    error.value = null

    try {
      console.log('Generating subject with context:', context)
      const result = await invoke('generate_subject', { context })
      return (result as { completion: string; error?: string }).completion
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Failed to generate subject'
      console.log('Error generating subject:', err)
      throw err
    } finally {
      isGenerating.value = false
    }
  }

  return {
    isGenerating,
    error,
    generateSubject
  }
}