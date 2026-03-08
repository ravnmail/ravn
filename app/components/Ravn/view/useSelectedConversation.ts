export const useSelectedConversation = () => {
  const router = useRouter()
  const route = useRoute()

  const selectedConversationId = computed<string | undefined>({
    get() {
      return route.query.conversation as string | undefined
    },
    set(value) {
      const query = { ...route.query }

      if (value) {
        query.conversation = value
      } else {
        delete query.conversation
      }

      router.replace({ query })
    },
  })

  const selectConversation = (conversationId: string) => {
    selectedConversationId.value = conversationId
  }

  const clearSelectedConversation = () => {
    selectedConversationId.value = undefined
  }

  return {
    selectedConversationId,
    selectConversation,
    clearSelectedConversation,
  }
}
