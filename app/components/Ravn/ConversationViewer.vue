<script lang="ts" setup>
import { useFocusWithin } from '@vueuse/core'
import { toast } from 'vue-sonner'

import Composer from '~/components/Composer.vue'
import ConversationDetailsPane from '~/components/Ravn/ConversationDetailsPane.vue'
import MessageView from '~/components/Ravn/MessageView.vue'
import { Button } from '~/components/ui/button'
import EmptyState from '~/components/ui/empty/EmptyState.vue'
import { ResizablePanelGroup, ResizablePanel, ResizableHandle } from '~/components/ui/resizable'
import { ScrollArea } from '~/components/ui/scroll-area'
import { cn } from '~/lib/utils'
import type { EmailDetail } from '~/types/email'

const props = defineProps<{
  conversationId: string
  selectedEmailId?: string
  titleClass?: string
}>()

const { t } = useI18n()
const { useGetConversation } = useConversation()
const { archive, trash } = useEmails()
const conversationId = toRef(props, 'conversationId')
const { data: conversation } = useGetConversation(conversationId)
const { useGetFolders } = useFolders()
const { data: folders } = useGetFolders()
const { register, unregister, addContext, removeContext } = useActions()

const conversationViewerRef = useTemplateRef<HTMLElement | null>('conversationViewerRef')
const panelRef = ref<any>(null)
const conversationContainer = useTemplateRef<HTMLElement | null>('conversationContainer')
const panelCollapsed = ref(false)
const messageElements = new Map<string, HTMLElement>()
const highlightedEmailId = ref<string | null>(null)

const { focused } = useFocusWithin(conversationViewerRef)

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

const activeComposer = ref<{
  type: 'reply' | 'reply-all' | 'forward'
  originalMessage: EmailDetail
  initialContent?: string
} | null>(null)

const isArchiving = ref(false)
const isDeleting = ref(false)
const { settings } = useSettings()

const latestMessage = computed(() => {
  return conversation.value?.messages.slice().reverse()[0] || null
})

const subject = computed(() => {
  return latestMessage.value?.subject || t('components.emailViewer.noSubject')
})

const registerMessageElement = (messageId: string, element: unknown) => {
  if (element instanceof HTMLElement) {
    messageElements.set(messageId, element)
    return
  }

  messageElements.delete(messageId)
}

const focusSelectedEmail = async () => {
  if (!props.selectedEmailId) {
    highlightedEmailId.value = null
    return
  }

  await nextTick()

  const messageElement = messageElements.get(props.selectedEmailId)
  if (!messageElement) return

  highlightedEmailId.value = props.selectedEmailId
  messageElement.scrollIntoView({
    behavior: 'smooth',
    block: 'center',
  })
}

const scrollToComposer = () => {
  nextTick(() => {
    if (conversationContainer.value) {
      conversationContainer.value.scrollTo({
        top: 0,
        behavior: 'smooth',
      })
    }
  })
}

const handleReply = (message: EmailDetail) => {
  activeComposer.value = {
    type: 'reply',
    originalMessage: message,
  }
  scrollToComposer()
}

const handleReplyAll = (message: EmailDetail) => {
  activeComposer.value = {
    type: 'reply-all',
    originalMessage: message,
  }
  scrollToComposer()
}

const handleForward = (message: EmailDetail) => {
  activeComposer.value = {
    type: 'forward',
    originalMessage: message,
  }
  scrollToComposer()
}

const handleQuickReply = (message: EmailDetail, content: string) => {
  activeComposer.value = {
    type: 'reply-all',
    originalMessage: message,
    initialContent: content,
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
    toast.error(t('components.conversationViewer.errors.credentials') as string)
  } else if (errorMsg.includes('Archive folder not found')) {
    toast.error(t('components.conversationViewer.errors.archiveFolder') as string)
  } else {
    toast.error(`Failed to ${action.toLowerCase()}: ${errorMsg}`)
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

const sentfolderIds = computed(
  () =>
    folders.value
      ?.filter((folder) => ['sent', 'draft'].includes(folder.folder_type))
      .map((f) => f.id) || []
)

const isSentMessage = (message: EmailDetail) => {
  return sentfolderIds.value.includes(message.folder_id)
}

watch(
  () => [conversation.value?.id, props.selectedEmailId],
  async () => {
    await focusSelectedEmail()
  },
  { immediate: true }
)

onMounted(() => {
  addContext('ConversationView', focused)
  register({
    id: 'archiveConversation',
    namespace: 'ConversationView',
    handler: handleArchive,
  })
  register({
    id: 'showConversationDetails',
    namespace: 'ConversationView',
    handler: () => togglePanel(panelCollapsed.value),
  })
})

onBeforeUnmount(() => {
  removeContext('ConversationView')
  unregister('ConversationView', 'archiveConversation')
  unregister('ConversationView', 'showConversationDetails')
})
</script>

<template>
  <ResizablePanelGroup
    ref="conversationViewerRef"
    auto-save-id="conversation-layout-sidebar"
    class="flex h-screen w-full bg-background"
    direction="horizontal"
  >
    <ResizablePanel class="flex flex-1 flex-col">
      <div
        v-if="conversation"
        class="flex h-full flex-1 flex-col"
      >
        <div class="px-3 pt-2">
          <div class="flex items-start justify-between">
            <h1
              :class="
                cn('relative z-10 text-xl font-semibold text-primary select-auto', titleClass ?? '')
              "
            >
              {{ subject }}
            </h1>
            <Button
              size="icon"
              variant="ghost"
              @click="togglePanel(panelCollapsed)"
            >
              <Icon name="lucide:info" />
            </Button>
          </div>
        </div>
        <ScrollArea ref="conversationContainer">
          <div
            v-if="activeComposer"
            class="border-b border-border px-3"
          >
            <Composer
              v-if="activeComposer.type"
              :forward="
                activeComposer.type === 'forward' ? activeComposer.originalMessage : undefined
              "
              :initial-content="activeComposer.initialContent"
              :is-reply-all="activeComposer.type === 'reply-all'"
              :reply-to="
                activeComposer.type === 'forward' ? undefined : activeComposer.originalMessage
              "
              @discarded="handleComposerDiscarded"
              @sent="handleComposerSent"
            />
          </div>
          <div class="space-y-6 px-3 pt-3">
            <template
              v-for="(message, index) in conversation?.messages"
              :key="message.id"
            >
              <div
                :ref="(element) => registerMessageElement(message.id, element)"
                v-if="message.is_draft"
                class="rounded-lg border border-border p-3"
              >
                <Composer
                  :draft="message"
                  @discarded="handleComposerDiscarded"
                  @sent="handleComposerSent"
                />
              </div>
              <div
                v-else
                :ref="(element) => registerMessageElement(message.id, element)"
              >
                <MessageView
                  :auto-analyze="true"
                  :class="[
                    settings?.email?.conversation?.insetOutgoing && isSentMessage(message)
                      ? 'ml-12'
                      : '',
                  ]"
                  :initial-reduced="settings?.email?.conversation?.collapseMessages && index > 0"
                  :is-first="index === 0"
                  :is-highlighted="highlightedEmailId === message.id"
                  v-bind="message"
                  @forward="handleForward"
                  @reply="handleReply"
                  @reply-all="handleReplyAll"
                  @quick-reply="handleQuickReply"
                />
              </div>
            </template>
          </div>
        </ScrollArea>
      </div>
      <EmptyState
        v-else
        :title="t('components.conversationViewer.notFound')"
        icon="📭"
      />
    </ResizablePanel>
    <ResizableHandle @dblclick="togglePanel(panelCollapsed)" />
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
      <ConversationDetailsPane :conversation="conversation" />
    </ResizablePanel>
  </ResizablePanelGroup>
</template>
