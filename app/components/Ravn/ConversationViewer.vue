<script lang="ts" setup>
import { Button } from '~/components/ui/button'
import MessageView from '~/components/Ravn/MessageView.vue'
import Composer from '~/components/Composer.vue'
import ConversationDetailsPane from '~/components/Ravn/ConversationDetailsPane.vue'
import type { EmailDetail } from '~/types/email'
import { useIntersectionObserver } from '@vueuse/core'
import { ScrollArea } from '~/components/ui/scroll-area'
import EmptyState from '~/components/ui/empty/EmptyState.vue'
import { ResizablePanelGroup, ResizablePanel, ResizableHandle } from '~/components/ui/resizable'

const props = defineProps<{
  conversationId: string
}>()

const { t } = useI18n()
const { useGetConversation } = useConversation()
const { archive, trash } = useEmails()
const { updateRead } = useEmails()
const { data: conversation } = useGetConversation(props.conversationId)
const { useGetFolders } = useFolders()
const { data: folders } = useGetFolders()

const panelRef = useTemplateRef<HTMLElement | null>('panelRef')
const conversationContainer = useTemplateRef<HTMLElement | null>('conversationContainer')
const panelCollapsed = ref(false)

const togglePanel = (collapse: boolean) => {
  if (collapse) {
    panelRef.value?.collapse()
  } else {
    panelRef.value?.expand()
  }
}

const onTogglePanel = (collapsed: boolean) => {
  panelCollapsed.value = !collapsed
}

const markedAsRead = ref<Set<string>>(new Set())
const visibilityTimers = ref<Map<string, NodeJS.Timeout>>(new Map())
const activeComposer = ref<{
  type: 'reply' | 'reply-all' | 'forward',
  originalMessage: EmailDetail,
  initialContent?: string
} | null>(null)

const isArchiving = ref(false)
const isDeleting = ref(false)

const latestMessage = computed(() => {
  return conversation.value?.messages.slice().reverse()[0] || null
})

const subject = computed(() => {
  return latestMessage.value?.subject || t('components.emailViewer.noSubject')
})

const scrollToComposer = () => {
  nextTick(() => {
    if (conversationContainer.value) {
      conversationContainer.value.scrollTo({
        top: 0,
        behavior: 'smooth'
      })
    }
  })
}

const handleReply = (message: EmailDetail) => {
  activeComposer.value = {
    type: 'reply',
    originalMessage: message
  }
  scrollToComposer()
}

const handleReplyAll = (message: EmailDetail) => {
  activeComposer.value = {
    type: 'reply-all',
    originalMessage: message
  }
  scrollToComposer()
}

const handleForward = (message: EmailDetail) => {
  activeComposer.value = {
    type: 'forward',
    originalMessage: message
  }
  scrollToComposer()
}

const handleQuickReply = (message: EmailDetail, content: string) => {
  activeComposer.value = {
    type: 'reply',
    originalMessage: message,
    initialContent: content
  }
  scrollToComposer()
}

const handleComposerSent = () => {
  activeComposer.value = null
}

const handleComposerDiscarded = () => {
  activeComposer.value = null
}

const handleError = (action: string, error: unknown) => {
  const errorMsg = error instanceof Error ? error.message : String(error)

  if (errorMsg.includes('IMAP config not set') || errorMsg.includes('credentials')) {
    alert(t('components.conversationViewer.errors.credentials'))
  } else if (errorMsg.includes('Archive folder not found')) {
    alert(t('components.conversationViewer.errors.archiveFolder'))
  } else {
    alert(`Failed to ${action.toLowerCase()}: ${errorMsg}`)
  }
}

const handleArchive = async () => {
  if (!latestMessage.value || isArchiving.value) return

  isArchiving.value = true
  try {
    await archive(latestMessage.value.id)
  } catch (error) {
    handleError('Archive', error)
  } finally {
    isArchiving.value = false
  }
}

const handleDelete = async () => {
  if (!latestMessage.value || isDeleting.value) return

  isDeleting.value = true
  try {
    await trash(latestMessage.value.id)
  } catch (error) {
    handleError('Delete', error)
  } finally {
    isDeleting.value = false
  }
}

// Setup keyboard bindings at top level (not inside onMounted)
useKeyboardBindings({
  archive: handleArchive,
  delete: handleDelete,
  reply: () => {
    if (latestMessage.value) {
      handleReply(latestMessage.value)
    }
  },
  replyAll: () => {
    if (latestMessage.value) {
      handleReplyAll(latestMessage.value)
    }
  },
  forward: () => {
    if (latestMessage.value) {
      handleForward(latestMessage.value)
    }
  },
})

const handleMessageVisibility = (message: EmailDetail, isVisible: boolean) => {
  const messageId = message.id.toString()

  if (message.is_read || markedAsRead.value.has(messageId)) {
    return
  }

  if (isVisible) {
    const timer = setTimeout(async () => {
      if (!markedAsRead.value.has(messageId)) {
        try {
          await updateRead(message.id, true)
          markedAsRead.value.add(messageId)
          console.log('[ConversationViewer] Marked message as read:', messageId)
        } catch (error) {
          console.error('[ConversationViewer] Failed to mark as read:', error)
        }
      }
    }, 2000) // 2 second threshold

    visibilityTimers.value.set(messageId, timer)
  } else {
    const timer = visibilityTimers.value.get(messageId)
    if (timer) {
      clearTimeout(timer)
      visibilityTimers.value.delete(messageId)
    }
  }
}

const sentfolderIds = computed(() => folders.value?.filter(folder => ['sent', 'draft'].includes(folder.folder_type)).map(f => f.id) || [])

const isSentMessage = (message: EmailDetail) => {
  return sentfolderIds.value.includes(message.folder_id)
}

onUnmounted(() => {
  visibilityTimers.value.forEach(timer => clearTimeout(timer))
  visibilityTimers.value.clear()
})

const messageRefs = ref<Record<string, HTMLElement | null>>({})

const setupMessageObservers = () => {
  if (!conversation || !conversation.messages) return

  conversation.messages.forEach((message) => {
    const messageId = message.id.toString()
    const element = messageRefs.value[messageId]

    if (element && !message.is_read && !markedAsRead.value.has(messageId)) {
      const { stop } = useIntersectionObserver(
        element,
        ([{ isIntersecting }]) => {
          handleMessageVisibility(message, isIntersecting)
        },
        { threshold: 0.5 } // Message needs to be 50% visible
      )

      onUnmounted(stop)
    }
  })
}

onMounted(() => {
  nextTick(() => {
    setupMessageObservers()
  })
})
</script>

<template>
  <ResizablePanelGroup
    auto-save-id="conversation-layout-sidebar"
    class="flex h-screen bg-background w-full"
    direction="horizontal"
  >
    <ResizablePanel
      class="flex-1 flex flex-col"
    >
      <div
        v-if="conversation"
        class="flex-1 flex flex-col h-full"
      >
        <div class="px-3 pt-2">
          <div class="flex items-center justify-between">
            <div>
              <h1 class="text-2xl font-semibold select-auto text-primary">
                {{ subject }}
              </h1>
            </div>
            <Button
              size="icon"
              variant="ghost"
              @click="togglePanel(panelCollapsed)"
            >
              <Icon
                class="h-5 w-5"
                name="lucide:info"
              />
            </Button>
          </div>
        </div>
        <ScrollArea
          ref="conversationContainer"
        >
          <div
            v-if="activeComposer"
            class="border-b border-border px-3"
          >
            <Composer
              v-if="activeComposer.type"
              :forward="activeComposer.type === 'forward' ? activeComposer.originalMessage : undefined"
              :initial-content="activeComposer.initialContent"
              :is-reply-all="activeComposer.type === 'reply-all'"
              :reply-to="activeComposer.type === 'forward' ? undefined : activeComposer.originalMessage"
              @discarded="handleComposerDiscarded"
              @sent="handleComposerSent"
            />
          </div>
          <div class="px-3 py-6 space-y-6">
            <div
              v-for="(message, index) in conversation?.messages"
              :key="message.id"
              :ref="(el) => { if (el) messageRefs[message.id.toString()] = el as HTMLElement }"
              :class="[ isSentMessage(message) ? 'ml-12' : '' ]"
              class="space-y-3"
            >
              <MessageView
                :auto-analyze="true"
                :initial-reduced="index > 0"
                v-bind="message"
                @forward="handleForward"
                @reply="handleReply"
                @reply-all="handleReplyAll"
                @quick-reply="handleQuickReply"
              />
            </div>
          </div>
        </ScrollArea>
      </div>
      <EmptyState
        v-else
        :title="t('components.conversationViewer.notFound')"
        icon="📭"
      />
    </ResizablePanel>

    <ResizableHandle
      @dblclick="togglePanel(panelCollapsed)"
    />
    <ResizablePanel
      v-if="conversation"
      id="conversation-panel"
      ref="panelRef"
      :default-size="240"
      :max-size="480"
      :min-size="240"
      collapsible
      size-unit="px"
      @collapse="() => onTogglePanel(true)"
      @expand="() => onTogglePanel(false)"
    >
      <ConversationDetailsPane :conversation="conversation"/>
    </ResizablePanel>
  </ResizablePanelGroup>
</template>
