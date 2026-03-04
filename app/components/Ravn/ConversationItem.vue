<script lang="ts" setup>
import { pointerOutsideOfPreview } from '@atlaskit/pragmatic-drag-and-drop/element/pointer-outside-of-preview'
import { setCustomNativeDragPreview } from '@atlaskit/pragmatic-drag-and-drop/element/set-custom-native-drag-preview'
import { Badge } from '~/components/ui/badge'
import EmailLabel from '~/components/ui/EmailLabel.vue'
import { useDraggable } from '~/composables/useDragAndDrop'
import useFormatting from '~/composables/useFormatting'
import type { ConversationListItem } from '~/types/conversation'
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
const firstMessage = computed(
  () => props.conversation.messages.filter((m) => m.folder_id === props.folderId)[0]
)

const itemRef = ref<HTMLElement | null>(null)

// Get message IDs from this conversation that are in the current folder
const messageIdsInFolder = computed(() =>
  props.conversation.messages.filter((m) => m.folder_id === props.folderId).map((m) => m.id)
)

const getDragData = () => {
  const isMultiDrag =
    props.selectedIds &&
    props.selectedIds.length > 0 &&
    props.selectedIds.includes(props.conversation.id)

  return {
    type: 'conversation' as const,
    id: props.conversation.id,
    accountId: firstMessage.value?.account_id,
    folderId: props.folderId,
    messageIds: isMultiDrag ? props.selectedMessageIds : messageIdsInFolder.value,
    selectedIds: isMultiDrag ? props.selectedIds : undefined,
    isMultiDrag,
  }
}

// Make conversation draggable with multi-select support
const { isDragging } = useDraggable(itemRef, getDragData, {
  onGenerateDragPreview: ({ nativeSetDragImage }) => {
    setCustomNativeDragPreview({
      nativeSetDragImage,
      // Anchor preview to the right of the cursor so the drop target stays visible
      getOffset: pointerOutsideOfPreview({ x: '16px', y: '8px' }),
      render({ container }) {
        const data = getDragData()
        const isMulti = data.isMultiDrag && props.selectedIds && props.selectedIds.length > 1
        const label = isMulti
          ? `${props.selectedIds!.length} conversations`
          : (firstMessage.value?.subject?.trim() || firstMessage.value?.from?.name || 'Email')

        const el = document.createElement('div')
        el.style.cssText = [
          'display:flex',
          'align-items:center',
          'gap:6px',
          'padding:6px 10px',
          'border-radius:8px',
          'font-size:13px',
          'font-weight:500',
          'max-width:220px',
          'white-space:nowrap',
          'overflow:hidden',
          'text-overflow:ellipsis',
          'box-shadow:0 4px 16px rgba(0,0,0,0.18)',
          'background:var(--color-background,#fff)',
          'color:var(--color-foreground,#111)',
          'border:1px solid var(--color-border,#e5e7eb)',
          'pointer-events:none',
        ].join(';')

        if (isMulti) {
          const badge = document.createElement('span')
          badge.style.cssText = 'background:var(--color-accent,#6366f1);color:#fff;border-radius:999px;padding:1px 7px;font-size:11px;font-weight:700;flex-shrink:0'
          badge.textContent = String(props.selectedIds!.length)
          el.appendChild(badge)
        }

        const text = document.createElement('span')
        text.style.cssText = 'overflow:hidden;text-overflow:ellipsis'
        text.textContent = label
        el.appendChild(text)

        container.appendChild(el)
      },
    })
  },
})

const mappedLeftActions = computed(() =>
  props.leftActions.map((action) => ({
    ...action,
    handler: () => emit('action', action.id, props.conversation.id),
  }))
)

const mappedRightActions = computed(() =>
  props.rightActions.map((action) => ({
    ...action,
    handler: () => emit('action', action.id, props.conversation.id),
  }))
)

const { swipeOffset, activeActionSide, visibleSide, activeActionIndex, handleActionClick } =
  useSwipeActions(itemRef, {
    leftActions: mappedLeftActions.value,
    rightActions: mappedRightActions.value,
  })

const contentTransform = computed(() => `translateX(${swipeOffset.value}px)`)

const actionWidth = 5
const leftActionsWidth = computed(() => props.leftActions.length * actionWidth)
const rightActionsWidth = computed(() => props.rightActions.length * actionWidth)

const filteredLabels = computed(() => {
  if (!firstMessage.value) return []
  if (props.excludeLabels && props.excludeLabels.length > 0) {
    return firstMessage.value.labels.filter((l) => !props.excludeLabels!.includes(l.id))
  }
  return firstMessage.value.labels
})

// Check if any message in the conversation is unread
const hasUnread = computed(() => props.conversation.messages.some((m) => !m.is_read))

// Check if any message has attachments
const hasAttachments = computed(() => props.conversation.messages.some((m) => m.has_attachments))

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
      'relative touch-pan-x overflow-hidden rounded transition-opacity',
      isDragging ? 'cursor-grabbing opacity-30' : '',
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
          isMultiSelected ? 'bg-primary/10 ring-1 ring-primary' : '',
        ]"
        @click="(e) => emit('click', e)"
      >
        <RavnAvatar
          v-if="firstMessage.from.address"
          :email="firstMessage.from.address"
          :name="firstMessage.from.name"
          :key="firstMessage.from.address"
          class="pointer-events-none mr-4"
          size="lg"
        />
        <div class="grow">
          <div class="flex items-center gap-1">
            <div
              v-if="hasUnread"
              class="size-2 shrink-0 rounded-full bg-accent"
            />
            <span class="line-clamp-1 text-sm">{{
              firstMessage.from.name || firstMessage.from.address
            }}</span>
            <span class="ml-auto text-sm text-nowrap opacity-60">{{
              formatEmailDate(firstMessage, 2)
            }}</span>
          </div>
          <div class="flex items-center gap-2">
            <span class="line-clamp-1 font-bold">{{ firstMessage.subject }}</span>
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
                class="shrink-0"
              />
              <Badge
                v-if="conversation.message_count > 1"
                size="sm"
                variant="background"
                >{{ conversation.message_count }}</Badge
              >
            </div>
          </div>
          <div class="line-clamp-2 text-sm opacity-50">
            {{ firstMessage.snippet?.replace(/\s\s+/, ' ') }}
          </div>
          <div
            v-if="filteredLabels?.length > 0"
            class="mt-2 flex flex-wrap gap-1"
          >
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
