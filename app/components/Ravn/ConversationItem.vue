<script lang="ts" setup>
import type { ConversationListItem } from '~/types/conversation'
import useFormatting from '~/composables/useFormatting'
import EmailLabel from '~/components/ui/EmailLabel.vue'
import { useDraggable } from '~/composables/useDragAndDrop'
import type { EmailCategory } from '~/types/email'

interface Props {
  conversation: ConversationListItem
  leftActions?: SwipeAction[]
  rightActions?: SwipeAction[]
  excludeLabels?: string[]
  folderId: string
  isSelected?: boolean
  isMultiSelected?: boolean
  selectedIds?: string[]
  selectedMessageIds?: string[]
}

const props = withDefaults(defineProps<Props>(), {
  leftActions: () => [],
  rightActions: () => [],
  excludeLabels: () => [],
  isSelected: false,
  isMultiSelected: false,
  selectedIds: () => [],
  selectedMessageIds: () => [],
})

const emit = defineEmits<{
  (e: 'action', actionId: string, conversationId: string): void
  (e: 'click', event: MouseEvent): void
}>()

const { formatEmailDate } = useFormatting()
const firstMessage = computed(() => props.conversation.messages.filter(m => m.folder_id === props.folderId)[0])

const itemRef = ref<HTMLElement | null>(null)

// Get message IDs from this conversation that are in the current folder
const messageIdsInFolder = computed(() =>
  props.conversation.messages
    .filter(m => m.folder_id === props.folderId)
    .map(m => m.id)
)

// Make conversation draggable with multi-select support
const { isDragging } = useDraggable(itemRef, () => {
  const isMultiDrag = props.selectedIds && props.selectedIds.length > 0 && props.selectedIds.includes(props.conversation.id)

  return {
    type: 'conversation',
    id: props.conversation.id,
    accountId: firstMessage.value?.account_id,
    folderId: props.folderId,
    // For single drag: use this conversation's message IDs
    // For multi-drag: use all selected message IDs from parent
    messageIds: isMultiDrag ? props.selectedMessageIds : messageIdsInFolder.value,
    // Include selected IDs for multi-drag (conversation IDs for tracking)
    selectedIds: isMultiDrag ? props.selectedIds : undefined,
    isMultiDrag,
  }
})

const mappedLeftActions = computed(() =>
  props.leftActions.map(action => ({
    ...action,
    handler: () => emit('action', action.id, props.conversation.id)
  }))
)

const mappedRightActions = computed(() =>
  props.rightActions.map(action => ({
    ...action,
    handler: () => emit('action', action.id, props.conversation.id)
  }))
)

const {
  swipeOffset,
  activeActionSide,
  visibleSide,
  activeActionIndex,
  handleActionClick
} = useSwipeActions(itemRef, {
  leftActions: mappedLeftActions.value,
  rightActions: mappedRightActions.value
})

const contentTransform = computed(() => `translateX(${swipeOffset.value}px)`)

const actionWidth = 5
const leftActionsWidth = computed(() => props.leftActions.length * actionWidth)
const rightActionsWidth = computed(() => props.rightActions.length * actionWidth)

const filteredLabels = computed(() => {
  if (!firstMessage.value) return []
  if (props.excludeLabels && props.excludeLabels.length > 0) {
    return firstMessage.value.labels.filter(l => !props.excludeLabels!.includes(l.id))
  }
  return firstMessage.value.labels
})

// Check if any message in the conversation is unread
const hasUnread = computed(() =>
  props.conversation.messages.some(m => !m.is_read)
)

// Check if any message has attachments
const hasAttachments = computed(() =>
  props.conversation.messages.some(m => m.has_attachments)
)

const categoryIconMap: Record<EmailCategory, { name: string; color: string }> = {
  personal: {
    name: 'lucide:user',
    color: '#3b82f6',
  },
  promotions: {
    name: 'lucide:tag',
    color: '#4caf50',
  },
  updates: {
    name: 'lucide:megaphone',
    color: '#ff9800',
  },
  transactions: {
    name: 'lucide:shopping-cart',
    color: '#f44336',
  },
}

</script>

<template>
  <div
    v-if="firstMessage"
    ref="itemRef"
    :class="[
      'relative overflow-hidden touch-pan-x rounded transition-opacity',
      isDragging ? 'opacity-30 cursor-grabbing' : ''
    ]"
  >
    <div
      v-if="props.leftActions.length > 0"
      :aria-hidden="visibleSide !== 'left'"
      :class="['absolute inset-y-0 left-0 flex h-full', visibleSide !== 'left' && 'hidden']"
      :style="{ width: `${leftActionsWidth}rem` }"
    >
      <RavnSwipeActionItem
        v-for="(action, index) in props.leftActions"
        :key="action.id"
        :action="action"
        :is-active="visibleSide === 'left' && activeActionIndex === index"
        side="left"
        @click="handleActionClick(index, 'left')"
      />
    </div>
    <div
      v-if="rightActions.length > 0"
      :aria-hidden="visibleSide !== 'right'"
      :class="['absolute inset-y-0 right-0 flex h-full', visibleSide !== 'right' && 'hidden']"
      :style="{ width: `${rightActionsWidth}rem` }"
    >
      <RavnSwipeActionItem
        v-for="(action, index) in rightActions"
        :key="action.id"
        :action="action"
        :is-active="visibleSide === 'right' && activeActionIndex === index"
        side="right"
        @click="handleActionClick(index, 'right')"
      />
    </div>

    <div :style="{ transform: contentTransform }">
      <div
        :class="[
          'flex p-2 transition-transform duration-200 ease-out',
          hasUnread ? 'text-primary' : '',
          isSelected ? 'bg-selection text-selection-foreground' : '',
          isMultiSelected ? 'bg-primary/10 ring-1 ring-primary' : ''
        ]"
        @click="(e) => emit('click', e)"
      >
        <RavnAvatar
          v-if="firstMessage.from.address"
          :account-id="firstMessage.account_id"
          :email="firstMessage.from.address"
          :name="firstMessage.from.name"
          class="mr-4 pointer-events-none"
          size="lg"
        />
        <div class="flex-grow">
          <div class="flex items-center gap-2">
            <div
              v-if="hasUnread"
              class="w-2 h-2 bg-accent rounded-full shrink-0"
            />
            <span class="line-clamp-1 text-sm">{{ firstMessage.from.name || firstMessage.from.address }}</span>
            <span class="ml-auto text-sm opacity-60 text-nowrap">{{ formatEmailDate(firstMessage, 2) }}</span>
          </div>
          <div class="flex items-center gap-2">
            <span class="font-bold line-clamp-1">{{ firstMessage.subject }}</span>
            <div class="ml-auto flex items-center gap-2">
              <Icon
                v-if="hasAttachments"
                class="shrink-0"
                name="lucide:paperclip"
              />
              <Icon
                v-if="firstMessage.category"
                :name="categoryIconMap[firstMessage.category].name"
                :style="{ color: categoryIconMap[firstMessage.category].color }"
              />
              <span
                v-if="conversation.message_count > 1"
                class="text-xs font-semibold px-2 py-0.5 rounded bg-background"
              >{{ conversation.message_count }}</span>
            </div>
          </div>
          <div class="text-sm opacity-50 line-clamp-2">
            {{ firstMessage.snippet }}
          </div>
          <div class="flex mt-2 gap-1 flex-wrap">
            <EmailLabel
              v-for="l in filteredLabels"
              :key="l.id"
              v-bind="l"
            />
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
