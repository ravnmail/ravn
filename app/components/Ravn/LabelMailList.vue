<script lang="ts" setup>
import { useVirtualizer } from '@tanstack/vue-virtual'
import { useFocusWithin } from '@vueuse/core'
import dayjs from 'dayjs'
import { toast } from 'vue-sonner'

import ConversationItem from '~/components/Ravn/ConversationItem.vue'
import MailContextMenu from '~/components/Ravn/MailContextMenu.vue'
import { Badge } from '~/components/ui/badge'
import { Button } from '~/components/ui/button'
import EmptyState from '~/components/ui/empty/EmptyState.vue'
import { FormField } from '~/components/ui/form'
import { Popover, PopoverContent, PopoverTrigger } from '~/components/ui/popover'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '~/components/ui/select'
import { Switch } from '~/components/ui/switch'
import { useMultiSelect } from '~/composables/useDragAndDrop'
import type { ConversationListItem } from '~/types/conversation'

const { t } = useI18n()
const router = useRouter()
const { addContext, removeContext, register, unregister, executeAction } = useActions()

const mailListRef = useTemplateRef<HTMLElement>('mailListRef')
const scrollerRef = useTemplateRef<HTMLElement>('scrollerRef')
const { focused } = useFocusWithin(mailListRef)

const props = defineProps<{
  labelId: string
  conversationId?: string
}>()

const { useGetConversationsForLabelInfinite } = useConversation()
const { archive, trash, updateRead, addLabelToEmail, setRemindAt } = useEmails()
const { useGetLabel } = useLabels()

const { data: labelData } = useGetLabel(computed(() => props.labelId))
const label = computed(() => labelData.value ?? null)

const multiSelect = useMultiSelect<ConversationListItem>()

const contextMenuConvId = ref<string | null>(null)

const getContextMenuFirstMessageId = (): string | null => {
  if (!contextMenuConvId.value) return null
  const conv = conversations.value.find((c) => c.id === contextMenuConvId.value)
  if (!conv) return null
  return conv.messages[0]?.id ?? null
}

const sortBy = ref<string>('received_at')
const sortOrder = ref<string>('desc')
const filterRead = ref<boolean | null>(null)
const filterHasAttachments = ref<boolean | null>(null)

const groupingEnabled = ref<boolean>(true)
const expandedGroups = ref<Set<string>>(
  new Set([
    'today',
    'yesterday',
    'thisWeek',
    'thisMonth',
    'older',
    'enormous',
    'huge',
    'veryLarge',
    'large',
    'medium',
    'small',
  ])
)

onMounted(() => {
  addContext('mailList', focused)

  const ns = 'mailList'
  register({
    namespace: ns,
    id: 'archiveEmail',
    icon: 'lucide:archive',
    handler: () => {
      const id = getContextMenuFirstMessageId()
      if (id) archive(id)
    },
  })
  register({
    namespace: ns,
    id: 'deleteEmail',
    icon: 'lucide:trash-2',
    handler: () => {
      const id = getContextMenuFirstMessageId()
      if (id) trash(id)
    },
  })
  register({
    namespace: ns,
    id: 'markRead',
    icon: 'lucide:mail-open',
    handler: () => {
      const id = getContextMenuFirstMessageId()
      if (id) updateRead(id, true)
    },
  })
  register({
    namespace: ns,
    id: 'markUnread',
    icon: 'lucide:mail',
    handler: () => {
      const id = getContextMenuFirstMessageId()
      if (id) updateRead(id, false)
    },
  })
  register({
    namespace: ns,
    id: 'assignLabel',
    icon: 'lucide:tag',
    handler: (arg) => {
      const id = getContextMenuFirstMessageId()
      if (id && arg) addLabelToEmail({ email_id: id, label_id: arg as string })
    },
  })
  register({
    namespace: ns,
    id: 'setRemindAt',
    icon: 'lucide:bell',
    handler: (arg) => {
      const id = getContextMenuFirstMessageId()
      if (id) setRemindAt(id, (arg as string | null) ?? null)
    },
  })
})

onBeforeUnmount(() => {
  removeContext('mailList')
  const ns = 'mailList'
  for (const id of [
    'archiveEmail',
    'deleteEmail',
    'markRead',
    'markUnread',
    'assignLabel',
    'setRemindAt',
  ]) {
    unregister(ns, id)
  }
})

// ─── Infinite query ───────────────────────────────────────────────────────────

const { data, fetchNextPage, hasNextPage, isFetchingNextPage, status } =
  useGetConversationsForLabelInfinite(
    computed(() => props.labelId),
    {
      sortBy,
      sortOrder,
      filterRead,
      filterHasAttachments,
    }
  )

const conversations = computed<ConversationListItem[]>(
  () => data.value?.pages.flatMap((p) => p.items) ?? []
)

// ─── Actions ─────────────────────────────────────────────────────────────────

const handleAction = async (actionId: string, conversationId: string) => {
  const conversation = conversations.value.find((c) => c.id === conversationId) || null
  if (!conversation || !conversation.messages[0]) {
    console.error('[LabelMailList] Conversation not found:', conversationId)
    return
  }

  const firstEmail = conversation.messages[0]
  try {
    switch (actionId) {
      case 'archive':
        await archive(firstEmail.id)
        break
      case 'delete':
        await trash(firstEmail.id)
        break
      default:
        console.warn('[LabelMailList] Unknown action:', actionId)
    }
  } catch (error) {
    const errorMsg = error instanceof Error ? error.message : String(error)
    console.error('[LabelMailList] ❌ Action failed:', actionId, error)
    if (errorMsg.includes('IMAP config not set') || errorMsg.includes('credentials')) {
      toast.error(t('components.mailList.errors.credentials') as string)
    } else if (errorMsg.includes('Archive folder not found')) {
      toast.error(t('components.mailList.errors.archiveFolder') as string)
    } else {
      toast.error(t('components.mailList.errors.generic') as string)
    }
  }
}

// ─── Grouping ─────────────────────────────────────────────────────────────────

type GroupKey =
  | 'today'
  | 'yesterday'
  | 'thisWeek'
  | 'thisMonth'
  | 'older'
  | 'enormous'
  | 'huge'
  | 'veryLarge'
  | 'large'
  | 'medium'
  | 'small'

const toggleGroup = (groupKey: GroupKey) => {
  if (expandedGroups.value.has(groupKey)) {
    expandedGroups.value.delete(groupKey)
  } else {
    expandedGroups.value.add(groupKey)
  }
}

const isGroupExpanded = (groupKey: GroupKey) => expandedGroups.value.has(groupKey)

const getSizeGroup = (size: number): GroupKey => {
  const sizeInMB = size / (1024 * 1024)
  const sizeInKB = size / 1024
  if (sizeInMB > 25) return 'enormous'
  if (sizeInMB > 10) return 'huge'
  if (sizeInMB > 5) return 'veryLarge'
  if (sizeInMB > 1) return 'large'
  if (sizeInKB > 25) return 'medium'
  return 'small'
}

const getGroupLabel = (groupKey: GroupKey): string => {
  if (sortBy.value === 'size') {
    return `components.mailList.grouping.size.${groupKey}`
  }
  return `common.time.${groupKey}`
}

const shouldGroup = computed(
  () =>
    groupingEnabled.value &&
    (sortBy.value === 'received_at' || sortBy.value === 'sent_at' || sortBy.value === 'size')
)

const getGroupOrder = (): GroupKey[] => {
  if (sortBy.value === 'size') {
    const sizeGroups: GroupKey[] = ['enormous', 'huge', 'veryLarge', 'large', 'medium', 'small']
    return sortOrder.value === 'desc' ? sizeGroups : [...sizeGroups].reverse()
  }
  const dateGroups: GroupKey[] = ['today', 'yesterday', 'thisWeek', 'thisMonth', 'older']
  return sortOrder.value === 'desc' ? dateGroups : [...dateGroups].reverse()
}

const getPrimaryMessage = (conversation: ConversationListItem) =>
  conversation.messages.toSorted(
    (a, b) => new Date(b.received_at).getTime() - new Date(a.received_at).getTime()
  )[0]

// ─── Virtual rows ─────────────────────────────────────────────────────────────

type VirtualRow =
  | { type: 'group-header'; key: GroupKey; count: number }
  | { type: 'conversation'; conversation: ConversationListItem; groupKey: GroupKey }
  | { type: 'load-more' }

const virtualRows = computed<VirtualRow[]>(() => {
  const items = conversations.value
  if (items.length === 0) return []

  const rows: VirtualRow[] = []

  if (shouldGroup.value) {
    const groupOrder = getGroupOrder()
    const groups: Record<GroupKey, ConversationListItem[]> = {
      today: [],
      yesterday: [],
      thisWeek: [],
      thisMonth: [],
      older: [],
      enormous: [],
      huge: [],
      veryLarge: [],
      large: [],
      medium: [],
      small: [],
    }

    if (sortBy.value === 'size') {
      for (const conv of items) {
        const msg = getPrimaryMessage(conv)
        if (!msg) continue
        groups[getSizeGroup(msg.size || 0)].push(conv)
      }
    } else {
      const today = dayjs().startOf('day')
      const yesterday = dayjs().subtract(1, 'day').startOf('day')
      const thisWeek = dayjs().startOf('week')
      const thisMonth = dayjs().startOf('month')

      for (const conv of items) {
        const msg = getPrimaryMessage(conv)
        if (!msg) continue
        const date = dayjs(msg.received_at)
        if (date.isSame(today, 'day')) groups.today.push(conv)
        else if (date.isSame(yesterday, 'day')) groups.yesterday.push(conv)
        else if (date.isAfter(thisWeek)) groups.thisWeek.push(conv)
        else if (date.isAfter(thisMonth)) groups.thisMonth.push(conv)
        else groups.older.push(conv)
      }
    }

    for (const key of groupOrder) {
      const group = groups[key]
      if (group.length === 0) continue
      rows.push({ type: 'group-header', key, count: group.length })
      if (isGroupExpanded(key)) {
        for (const conv of group) {
          rows.push({ type: 'conversation', conversation: conv, groupKey: key })
        }
      }
    }
  } else {
    for (const conv of items) {
      rows.push({ type: 'conversation', conversation: conv, groupKey: 'today' })
    }
  }

  if (hasNextPage.value) {
    rows.push({ type: 'load-more' })
  }

  return rows
})

// ─── Virtualizer ──────────────────────────────────────────────────────────────

const virtualizer = useVirtualizer(
  computed(() => ({
    count: virtualRows.value.length,
    getScrollElement: () => scrollerRef.value ?? null,
    estimateSize: (index) => {
      const row = virtualRows.value[index]
      if (!row) return 110
      if (row.type === 'group-header') return 36
      if (row.type === 'load-more') return 48
      return 110
    },
    measureElement: (el) => el.getBoundingClientRect().height,
    overscan: 10,
  }))
)

const virtualItems = computed(() => virtualizer.value.getVirtualItems())
const totalSize = computed(() => virtualizer.value.getTotalSize())

watch(virtualItems, (items) => {
  if (!hasNextPage.value || isFetchingNextPage.value) return
  const last = items[items.length - 1]
  if (!last) return
  const row = virtualRows.value[last.index]
  if (row?.type === 'load-more') {
    fetchNextPage()
  }
})

// ─── Selection ────────────────────────────────────────────────────────────────

const handleSelect = (conversation: ConversationListItem, event?: MouseEvent) => {
  if (event?.metaKey || event?.ctrlKey || event?.shiftKey) {
    multiSelect.toggleSelect(conversation, event)
  } else {
    multiSelect.clearSelection()
    router.push(`/labels/${props.labelId}/conversations/${conversation.id}`)
  }
}

const selectedMessageIds = computed(() => {
  const selectedConvIds = multiSelect.selectedIds.value
  const selectedConversations = conversations.value.filter(
    (c) => selectedConvIds.includes(c.id) || props.conversationId === c.id
  )
  // For label view we include all messages of the conversation
  return selectedConversations.flatMap((conv) => conv.messages.map((m) => m.id))
})

watch(
  () => props.labelId,
  () => {
    multiSelect.clearSelection()
  }
)

const leftActions = ref<SwipeAction[]>([
  { id: 'archive', icon: 'lucide:archive', label: 'actions.archive', color: 'bg-warning' },
])

const rightActions = ref<SwipeAction[]>([
  { id: 'more', icon: 'lucide:ellipsis', label: 'actions.more', color: 'bg-gray-500' },
  { id: 'reply', icon: 'lucide:reply', label: 'actions.reply', color: 'bg-accent' },
  { id: 'delete', icon: 'lucide:trash-2', label: 'actions.delete', color: 'bg-destructive' },
])

const route = useRoute()
</script>

<template>
  <div
    ref="mailListRef"
    class="flex h-full flex-col"
  >
    <!-- Header -->
    <div class="flex shrink-0 items-center border-b border-b-border p-3">
      <div class="flex items-center gap-2 font-semibold text-primary">
        <Icon
          :name="`lucide:${label?.icon || 'tag'}`"
          :style="{ color: label?.color }"
          :size="18"
        />
        <span>{{ label?.name }}</span>
      </div>
      <div class="ml-auto flex items-center">
        <Popover>
          <PopoverTrigger as-child>
            <Button
              size="bar"
              variant="ghost"
            >
              <Icon name="lucide:filter" />
            </Button>
          </PopoverTrigger>
          <PopoverContent class="w-40 p-3">
            <div class="flex flex-col gap-4">
              <Select
                v-model="filterRead"
                class="rounded border border-border bg-background px-2 py-1 text-xs"
              >
                <SelectTrigger>
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem :value="null">{{
                    $t('components.mailList.filtering.all')
                  }}</SelectItem>
                  <SelectItem :value="false">{{
                    $t('components.mailList.filtering.unread')
                  }}</SelectItem>
                  <SelectItem :value="true">{{
                    $t('components.mailList.filtering.read')
                  }}</SelectItem>
                </SelectContent>
              </Select>
              <div class="flex">
                <Button @click="filterHasAttachments = filterHasAttachments === true ? null : true">
                  <Icon name="lucide:paperclip" />
                </Button>
              </div>
            </div>
          </PopoverContent>
        </Popover>
        <Popover>
          <PopoverTrigger as-child>
            <Button
              size="bar"
              variant="ghost"
            >
              <Icon name="lucide:settings-2" />
            </Button>
          </PopoverTrigger>
          <PopoverContent class="w-80 p-3">
            <div class="flex flex-col gap-4">
              <FormField
                :label="$t('components.mailList.sorting.label')"
                name="sorting"
              >
                <div class="flex items-stretch gap-2">
                  <Select v-model="sortBy">
                    <SelectTrigger>
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="received_at">
                        {{ $t('components.mailList.sorting.receivedAt') }}
                      </SelectItem>
                      <SelectItem value="sent_at">
                        {{ $t('components.mailList.sorting.sentAt') }}
                      </SelectItem>
                      <SelectItem value="size">
                        {{ $t('components.mailList.sorting.size') }}
                      </SelectItem>
                    </SelectContent>
                  </Select>
                  <Button @click="sortOrder = sortOrder === 'desc' ? 'asc' : 'desc'">
                    <Icon
                      :name="
                        sortOrder === 'desc'
                          ? 'lucide:arrow-down-wide-narrow'
                          : 'lucide:arrow-up-narrow-wide'
                      "
                    />
                  </Button>
                </div>
              </FormField>
              <FormField name="grouping">
                <label class="flex items-center justify-between space-x-2">
                  <span>{{ $t('components.mailList.grouping.toggle') }}</span>
                  <Switch v-model="groupingEnabled" />
                </label>
              </FormField>
            </div>
          </PopoverContent>
        </Popover>
      </div>
    </div>

    <!-- Empty state -->
    <EmptyState
      v-if="status === 'success' && conversations.length === 0"
      :description="$t('components.labelMailList.emptyState.message')"
      :title="$t('components.labelMailList.emptyState.title')"
      icon="🏷️"
      class="flex-1"
    />

    <!-- Virtual scroll container -->
    <div
      v-else
      ref="scrollerRef"
      class="flex-1 overflow-y-auto p-2"
    >
      <MailContextMenu
        :selected-email-ids="selectedMessageIds"
        :on-execute-action="(id, arg) => executeAction('mailList', id, arg)"
      >
        <div
          class="relative w-full"
          :style="{ height: `${totalSize}px` }"
        >
          <div
            v-for="virtualItem in virtualItems"
            :key="virtualItem.index"
            :ref="(el) => el && virtualizer.measureElement(el as Element)"
            class="absolute top-0 left-0 w-full"
            :style="{ transform: `translateY(${virtualItem.start}px)` }"
            :data-index="virtualItem.index"
          >
            <!-- Group header -->
            <template v-if="virtualRows[virtualItem.index]?.type === 'group-header'">
              <div
                class="flex cursor-pointer items-center gap-1 px-2 py-2 text-sm font-bold text-muted hover:text-primary"
                @click="toggleGroup((virtualRows[virtualItem.index] as any).key)"
              >
                <Icon
                  :class="[
                    'opacity-50 transition-transform',
                    isGroupExpanded((virtualRows[virtualItem.index] as any).key)
                      ? 'rotate-90 transform'
                      : '',
                  ]"
                  name="lucide:chevron-right"
                />
                <span>{{ $t(getGroupLabel((virtualRows[virtualItem.index] as any).key)) }}</span>
                <Badge
                  size="sm"
                  variant="background"
                  >{{ (virtualRows[virtualItem.index] as any).count }}</Badge
                >
              </div>
            </template>

            <!-- Conversation item -->
            <template v-else-if="virtualRows[virtualItem.index]?.type === 'conversation'">
              <ConversationItem
                @contextmenu.capture="
                  contextMenuConvId = (virtualRows[virtualItem.index] as any).conversation.id
                "
                :conversation="(virtualRows[virtualItem.index] as any).conversation"
                :folder-id="
                  (virtualRows[virtualItem.index] as any).conversation.messages[0]?.folder_id ?? ''
                "
                :is-multi-selected="
                  multiSelect.isSelected((virtualRows[virtualItem.index] as any).conversation.id)
                    .value
                "
                :is-selected="
                  route.params.conversation ===
                  (virtualRows[virtualItem.index] as any).conversation.id
                "
                :selected-ids="multiSelect.selectedIds.value"
                :selected-message-ids="selectedMessageIds"
                @action="handleAction"
                @click="handleSelect((virtualRows[virtualItem.index] as any).conversation, $event)"
              />
            </template>

            <!-- Load-more sentinel -->
            <template v-else-if="virtualRows[virtualItem.index]?.type === 'load-more'">
              <div class="flex items-center justify-center py-3">
                <Icon
                  v-if="isFetchingNextPage"
                  class="animate-spin text-muted"
                  name="lucide:loader-circle"
                />
              </div>
            </template>
          </div>
        </div>
      </MailContextMenu>
    </div>
  </div>
</template>
