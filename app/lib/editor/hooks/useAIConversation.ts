import { ref } from 'vue'
import type { Editor } from '@tiptap/vue-3'

export function useAIConversation(editor: Editor) {
  const result = ref<string>('')
  const status = ref<'init' | 'generating' | 'completed'>('init')
  const conversationHistory = ref<Array<{ role: string; content: string }>>([])
  const toast = function() { }
  const abortController = ref<AbortController | null>(null)

  async function handleCompletion(context: string, prompt: string) {
    status.value = 'generating'
    result.value = ''
    const AIOptions = editor.extensionManager.extensions.find(e => e.name === 'AI')?.options

    try {
      if (conversationHistory.value.length === 0) {
        conversationHistory.value.push({
          role: 'user',
          content: `Question: ${prompt} Context:${context}`,
        })
      } else {
        conversationHistory.value.push({
          role: 'user',
          content: prompt,
        })
      }

      abortController.value = new AbortController()
      const stream = await AIOptions.completions(conversationHistory.value, abortController.value.signal)
      if (!stream) {
        throw new Error('Failed to create stream')
      }

      const assistantResponse = stream.completion
      result.value += stream.completion

      conversationHistory.value.push({
        role: 'assistant',
        content: assistantResponse,
      })

      status.value = 'completed'
      return assistantResponse
    } catch (error: any) {
      if (error.name === 'AbortError') {
        status.value = 'init'
      } else {
        toast({
          title: error?.message || 'Failed to generate AI completion',
          variant: 'destructive',
        })
      }
      throw error
    }
  }

  function resetConversation() {
    result.value = ''
    status.value = 'init'
    conversationHistory.value = []
    abortController.value = null
  }

  const stopGeneration = () => {
    if (abortController.value) {
      abortController.value.abort()
      abortController.value = null
    }
  }

  return {
    result,
    status,
    conversationHistory,
    handleCompletion,
    resetConversation,
    stopGeneration
  }
}
