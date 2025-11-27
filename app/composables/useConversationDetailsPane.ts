/**
 * Composable for managing conversation details pane visibility
 */
export function useConversationDetailsPane() {
  // Use useState for shared state across components
  const isVisible = useState('conversationDetailsPaneVisible', () => false)

  const toggle = () => {
    isVisible.value = !isVisible.value
  }

  const show = () => {
    isVisible.value = true
  }

  const hide = () => {
    isVisible.value = false
  }

  return {
    isVisible,
    toggle,
    show,
    hide,
  }
}
