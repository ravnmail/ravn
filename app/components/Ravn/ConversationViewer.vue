<script lang="ts" setup>
import { Button } from '~/components/ui/button'
import MessageView from '~/components/Ravn/MessageView.vue'
import Composer from '~/components/Composer.vue'
import ConversationDetailsPane from '~/components/Ravn/ConversationDetailsPane.vue'
import type { Email, EmailDetail } from '~/types/email'
import { useIntersectionObserver } from '@vueuse/core'
import { ScrollArea } from '~/components/ui/scroll-area'

const props = defineProps<{
  conversationId: string
}>()

const { t } = useI18n()
const { useGetConversation } = useConversation()
const { archive, trash } = useEmails()
const { isVisible: isDetailsPaneVisible, toggle: toggleDetailsPane } = useConversationDetailsPane()
const { updateRead } = useEmails()

const { data: conversation } = useGetConversation(props.conversationId)
const conversationContainer = useTemplateRef<HTMLElement | null>('conversationContainer')

const markedAsRead = ref<Set<string>>(new Set())
const visibilityTimers = ref<Map<string, NodeJS.Timeout>>(new Map())
const activeComposer = ref<{ type: 'reply' | 'reply-all' | 'forward', originalMessage: Email, initialContent?: string } | null>(null)

const isArchiving = ref(false)
const isDeleting = ref(false)

const latestMessage = computed(() => {
  return conversation.value?.messages.slice().reverse()[0] || null
})

const subject = computed(() => {
  return latestMessage.value?.subject || t('components.emailViewer.noSubject')
})

const emailDetailToEmail = (detail: EmailDetail): Email => {
  return {
    ...detail,
    cc: detail.cc || [],
    bcc: detail.bcc || [],
    reply_to: detail.reply_to ? [detail.reply_to] : undefined,
    is_deleted: false,
  }
}

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
    originalMessage: emailDetailToEmail(message)
  }
  scrollToComposer()
}

const handleReplyAll = (message: EmailDetail) => {
  activeComposer.value = {
    type: 'reply-all',
    originalMessage: emailDetailToEmail(message)
  }
  scrollToComposer()
}

const handleForward = (message: EmailDetail) => {
  activeComposer.value = {
    type: 'forward',
    originalMessage: emailDetailToEmail(message)
  }
  scrollToComposer()
}

const handleQuickReply = (message: EmailDetail, content: string) => {
  activeComposer.value = {
    type: 'reply',
    originalMessage: emailDetailToEmail(message),
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
  }
  else if (errorMsg.includes('Archive folder not found')) {
    alert(t('components.conversationViewer.errors.archiveFolder'))
  }
  else {
    alert(`Failed to ${action.toLowerCase()}: ${errorMsg}`)
  }
}

const handleArchive = async () => {
  if (!latestMessage.value || isArchiving.value) return

  isArchiving.value = true
  try {
    await archive(latestMessage.value.id)
  }
  catch (error) {
    handleError('Archive', error)
  }
  finally {
    isArchiving.value = false
  }
}

const handleDelete = async () => {
  if (!latestMessage.value || isDeleting.value) return

  isDeleting.value = true
  try {
    await trash(latestMessage.value.id)
  }
  catch (error) {
    handleError('Delete', error)
  }
  finally {
    isDeleting.value = false
  }
}

// Setup keyboard bindings at top level (not inside onMounted)
useKeyboardBindings({
  toggleDetailsPane,
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

// Cleanup timers on unmount
onUnmounted(() => {
  visibilityTimers.value.forEach(timer => clearTimeout(timer))
  visibilityTimers.value.clear()
})

// Create refs for message elements to track visibility
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
  <div class="flex h-screen bg-background w-full">
    <div
      v-if="conversation"
      class="flex-1 flex flex-col"
    >
      <div class="px-3 pt-2">
        <div class="flex items-center justify-between">
          <div>
            <h1 class="text-2xl font-semibold select-auto text-primary">
              {{ subject }}
            </h1>
          </div>
          <Button
            :title="`${isDetailsPaneVisible ? 'Hide' : 'Show'} details`"
            size="icon"
            variant="ghost"
            @click="toggleDetailsPane"
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
          class="border-b border-border"
        >
          <Composer
            v-if="activeComposer.type === 'reply'"
            :initial-account-id="activeComposer.originalMessage.account_id"
            :initial-content="activeComposer.initialContent"
            :reply-to="activeComposer.originalMessage"
            @discarded="handleComposerDiscarded"
            @sent="handleComposerSent"
          />
          <Composer
            v-else-if="activeComposer.type === 'forward'"
            :forward="activeComposer.originalMessage"
            :initial-account-id="activeComposer.originalMessage.account_id"
            :initial-content="activeComposer.initialContent"
            @discarded="handleComposerDiscarded"
            @sent="handleComposerSent"
          />
          <Composer
            v-else-if="activeComposer.type === 'reply-all'"
            :initial-account-id="activeComposer.originalMessage.account_id"
            :initial-content="activeComposer.initialContent"
            :is-reply-all="true"
            :reply-to="activeComposer.originalMessage"
            @discarded="handleComposerDiscarded"
            @sent="handleComposerSent"
          />
        </div>
        <div class="px-3 py-6 space-y-6">
          <div
            v-for="(message, index) in conversation?.messages"
            :key="message.id"
            :ref="(el) => { if (el) messageRefs[message.id.toString()] = el as HTMLElement }"
            class="space-y-3"
          >
            <div
              v-if="index > 0"
              class="border-t border-border"
            />
            <MessageView
              :auto-analyze="index === 0"
              :reduced="true"
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
    <div
      v-else
      class="flex-1 flex items-center justify-center"
    >
      <div class="text-center text-muted-foreground">
        <Icon
          class="mx-auto mb-4 h-12 w-12 opacity-50"
          name="lucide:inbox"
        />
        <p>{{ t('components.conversationViewer.notFound') }}</p>
      </div>
    </div>

    <transition
      enter-active-class="transition-all duration-200 ease-out"
      enter-from-class="opacity-0 translate-x-4"
      enter-to-class="opacity-100 translate-x-0"
      leave-active-class="transition-all duration-200 ease-in"
      leave-from-class="opacity-100 translate-x-0"
      leave-to-class="opacity-0 translate-x-4"
    >
      <div
        v-if="conversation && isDetailsPaneVisible"
        class="min-w-32 flex-shrink-0 max-w-128"
      >
        <ConversationDetailsPane :conversation="conversation" />
      </div>
    </transition>
  </div>
</template>
