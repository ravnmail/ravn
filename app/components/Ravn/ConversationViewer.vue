<script lang="ts" setup>
import { Button } from '~/components/ui/button'
import MessageView from '~/components/Ravn/MessageView.vue'
import Composer from '~/components/Composer.vue'
import ConversationDetailsPane from '~/components/Ravn/ConversationDetailsPane.vue'
import type { EmailDetail } from '~/types/email'
import { ScrollArea } from '~/components/ui/scroll-area'
import EmptyState from '~/components/ui/empty/EmptyState.vue'
import { ResizablePanelGroup, ResizablePanel, ResizableHandle } from '~/components/ui/resizable'
import { toast } from 'vue-sonner'
import { cn } from '~/lib/utils'

const props = defineProps<{
  conversationId: string
  titleClass?: string
}>()

const { t } = useI18n()
const { useGetConversation } = useConversation()
const { archive, trash } = useEmails()
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

const activeComposer = ref<{
  type: 'reply' | 'reply-all' | 'forward',
  originalMessage: EmailDetail,
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

const sentfolderIds = computed(() => folders.value?.filter(folder => ['sent', 'draft'].includes(folder.folder_type)).map(f => f.id) || [])

const isSentMessage = (message: EmailDetail) => {
  return sentfolderIds.value.includes(message.folder_id)
}

</script>

<template>
  <ResizablePanelGroup
    auto-save-id="conversation-layout-sidebar"
    class="flex h-screen bg-background w-full"
    direction="horizontal"
  >
    <ResizablePanel class="flex-1 flex flex-col">
      <div
        v-if="conversation"
        class="flex-1 flex flex-col h-full"
      >
        <div class="px-3 pt-2">
          <div class="flex items-start justify-between">
            <h1
              :class="cn('text-xl font-semibold select-auto text-primary relative z-10', titleClass ?? '')"
            >
              {{ subject }}
            </h1>
            <Button
              size="icon"
              variant="ghost"
              @click="togglePanel(panelCollapsed)"
            >
              <Icon name="lucide:info"/>
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
              :forward="activeComposer.type === 'forward' ? activeComposer.originalMessage : undefined"
              :initial-content="activeComposer.initialContent"
              :is-reply-all="activeComposer.type === 'reply-all'"
              :reply-to="activeComposer.type === 'forward' ? undefined : activeComposer.originalMessage"
              @discarded="handleComposerDiscarded"
              @sent="handleComposerSent"
            />
          </div>
          <div class="px-3 pt-3 space-y-6">
            <MessageView
              v-for="(message, index) in conversation?.messages"
              :key="message.id"
              :auto-analyze="true"
              :class="[ settings.email.conversation.insetOutgoing && isSentMessage(message) ? 'ml-12' : '' ]"
              :initial-reduced="settings.email.conversation.collapseMessages && index > 0"
              :is-first="index === 0"
              v-bind="message"
              @forward="handleForward"
              @reply="handleReply"
              @reply-all="handleReplyAll"
              @quick-reply="handleQuickReply"
            />
          </div>
        </ScrollArea>
      </div>
      <EmptyState
        v-else
        :title="t('components.conversationViewer.notFound')"
        icon="📭"
      />
    </ResizablePanel>
    <ResizableHandle @dblclick="togglePanel(panelCollapsed)"/>
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
