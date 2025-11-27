import type { ConversationListItem } from '~/types/conversation'

/**
 * Composable for navigating through conversation lists with keyboard
 */
export function useConversationNavigation(
  conversations: Ref<ConversationListItem[]>,
  currentConversationId: Ref<string | undefined>,
  onSelect: (conversation: ConversationListItem) => void
) {
  const selectNext = () => {
    if (conversations.value.length === 0) return

    // If no conversation is selected, select the first one
    if (!currentConversationId.value) {
      onSelect(conversations.value[0])
      return
    }

    // Find current index
    const currentIndex = conversations.value.findIndex(
      c => c.id === currentConversationId.value
    )

    // If current conversation not found, select first
    if (currentIndex === -1) {
      onSelect(conversations.value[0])
      return
    }

    // Select next conversation if available
    if (currentIndex < conversations.value.length - 1) {
      onSelect(conversations.value[currentIndex + 1])
    }
  }

  const selectPrevious = () => {
    if (conversations.value.length === 0) return

    // If no conversation is selected, select the last one
    if (!currentConversationId.value) {
      onSelect(conversations.value[conversations.value.length - 1])
      return
    }

    // Find current index
    const currentIndex = conversations.value.findIndex(
      c => c.id === currentConversationId.value
    )

    // If current conversation not found, select last
    if (currentIndex === -1) {
      onSelect(conversations.value[conversations.value.length - 1])
      return
    }

    // Select previous conversation if available
    if (currentIndex > 0) {
      onSelect(conversations.value[currentIndex - 1])
    }
  }

  const selectCurrent = () => {
    if (!currentConversationId.value || conversations.value.length === 0) return

    const current = conversations.value.find(
      c => c.id === currentConversationId.value
    )

    if (current) {
      onSelect(current)
    }
  }

  return {
    selectNext,
    selectPrevious,
    selectCurrent,
  }
}
