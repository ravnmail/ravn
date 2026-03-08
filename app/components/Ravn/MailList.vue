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
import IconName from '~/components/ui/IconName.vue'
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
import type { EmailListItem } from '~/types/email'

const { t } = useI18n()
const router = useRouter()
const { addContext, removeContext, register, unregister, executeAction } = useActions()

const mailListRef = useTemplateRef<HTMLElement>('mailListRef')
const scrollerRef = useTemplateRef<HTMLElement>('scrollerRef')
const { focused } = useFocusWithin(mailListRef)

import type { ListViewConfig, View } from '~/types/view'

type AdvancedListFilterRule = {
  id?: string
  source?: 'folders' | 'labels'
  values?: string[]
  operator?: 'and' | 'or'
  negated?: boolean
}

type AdvancedListFilterGroup = {
  id?: string
  operator?: 'and' | 'or'
  negated?: boolean
  rules?: AdvancedListFilterRule[]
}

type MailListScope =
  | {
      type: 'folder'
      folderId: string
    }
  | {
      type: 'label'
      labelId: string
    }
  | {
      type: 'combined'
      folderIds: string[]
      labelIds: string[]
      matchAllLabels?: boolean
      filterGroups?: AdvancedListFilterGroup[]
      groupOperator?: 'and' | 'or'
    }

const props = defineProps<{
  folderId?: string
  accountId?: string
  conversationId?: string
  view?: View
  scope?: MailListScope
}>()

const emit = defineEmits<{
  (e: 'select-conversation', conversationId?: string): void
}>()

const {
  useGetConversationsForFolderInfinite,
  useGetConversationsForLabelInfinite,
  useGetConversationsForCombinedScopeInfinite,
} = useConversation()
const { folders, useUpdateSettingsMutation, useInitSyncMutation } = useFolders()
const { labels, useGetLabel } = useLabels()

const { mutateAsync: updateSettings } = useUpdateSettingsMutation()
const { mutateAsync: initSync } = useInitSyncMutation()

const { archive, trash, updateRead, move, addLabelToEmail, removeLabelFromEmail, setRemindAt } =
  useEmails()

const multiSelect = useMultiSelect<ConversationListItem>()

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

const normalizedScope = computed<MailListScope>(() => {
  if (props.scope) {
    return props.scope
  }

  if (props.view?.view_type === 'list' && props.view.config.type === 'list') {
    const config = props.view.config as ListViewConfig
    const filterGroups = Array.isArray(config.filters?.groups)
      ? (config.filters.groups as AdvancedListFilterGroup[])
      : []

    return {
      type: 'combined',
      folderIds: [],
      labelIds: [],
      matchAllLabels: false,
      filterGroups,
      groupOperator: 'and',
    }
  }

  if (props.folderId) {
    return {
      type: 'folder',
      folderId: props.folderId,
    }
  }

  return {
    type: 'combined',
    folderIds: [],
    labelIds: [],
    matchAllLabels: false,
  }
})

const primaryFolderId = computed(() => {
  if (normalizedScope.value.type === 'folder') return normalizedScope.value.folderId
  if (normalizedScope.value.type === 'combined') return normalizedScope.value.folderIds[0] || ''
  return ''
})

const primaryLabelId = computed(() => {
  if (normalizedScope.value.type === 'label') return normalizedScope.value.labelId
  if (normalizedScope.value.type === 'combined') return normalizedScope.value.labelIds[0] || ''
  return ''
})

const currentFolder = computed(() => {
  if (!primaryFolderId.value) return null
  return (folders.value || []).find((f) => f.id === primaryFolderId.value)
})

const { data: currentLabelData } = useGetLabel(primaryLabelId)
const currentLabel = computed(() => currentLabelData.value ?? null)

watch(
  currentFolder,
  () => {
    const folder = currentFolder.value
    if (folder?.settings) {
      sortBy.value = folder.settings.sort_by || 'received_at'
      sortOrder.value = folder.settings.sort_order || 'desc'
      groupingEnabled.value = folder.settings.grouping_enabled ?? true
      expandedGroups.value = new Set(
        folder.settings.expanded_groups || [
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
        ]
      )
      filterRead.value = folder.settings.filter_read ?? null
      filterHasAttachments.value = folder.settings.filter_has_attachments ?? null
      return
    }

    sortBy.value = 'received_at'
    sortOrder.value = 'desc'
    groupingEnabled.value = true
    expandedGroups.value = new Set([
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
    filterRead.value = null
    filterHasAttachments.value = null
  },
  { immediate: true }
)

let settingsSaveTimeout: ReturnType<typeof setTimeout> | null = null
const saveFolderSettings = () => {
  if (!primaryFolderId.value || normalizedScope.value.type !== 'folder') return

  if (settingsSaveTimeout) {
    clearTimeout(settingsSaveTimeout)
  }

  settingsSaveTimeout = setTimeout(async () => {
    try {
      await updateSettings({
        folderId: primaryFolderId.value,
        settings: {
          ...currentFolder.value,
          sort_by: sortBy.value,
          sort_order: sortOrder.value,
          grouping_enabled: groupingEnabled.value,
          expanded_groups: Array.from(expandedGroups.value),
          filter_read: filterRead.value,
          filter_has_attachments: filterHasAttachments.value,
        },
      })
    } catch (error) {
      console.error('[MailList] Failed to save folder settings:', error)
    }
  }, 500)
}

watch([sortBy, sortOrder, filterRead, filterHasAttachments], async () => {
  saveFolderSettings()
})

watch(
  [groupingEnabled, expandedGroups],
  () => {
    saveFolderSettings()
  },
  { deep: true }
)

const folderQuery = useGetConversationsForFolderInfinite(primaryFolderId, {
  sortBy,
  sortOrder,
  filterRead,
  filterHasAttachments,
})

const labelQuery = useGetConversationsForLabelInfinite(primaryLabelId, {
  sortBy,
  sortOrder,
  filterRead,
  filterHasAttachments,
})

const mapListRuleToScopedFilter = (rule: AdvancedListFilterRule) => {
  const values = Array.from(new Set((rule.values || []).filter(Boolean)))
  if (values.length === 0 || !rule.source) return null

  if (rule.source === 'folders') {
    return {
      type: 'folder' as const,
      folderIds: values,
      operator: rule.operator || 'or',
      negated: rule.negated ?? false,
    }
  }

  return {
    type: 'label' as const,
    labelIds: values,
    operator: rule.operator || 'or',
    negated: rule.negated ?? false,
    matchAllLabels: (rule.operator || 'or') === 'and',
  }
}

const combinedScope = computed(() =>
  normalizedScope.value.type === 'combined'
    ? {
        folderIds: normalizedScope.value.folderIds,
        labelIds: normalizedScope.value.labelIds,
        matchAllLabels: normalizedScope.value.matchAllLabels,
        filterGroups: (normalizedScope.value.filterGroups || [])
          .map((group) => ({
            operator: group.operator || 'and',
            filters: (group.rules || []).map(mapListRuleToScopedFilter).filter(Boolean),
          }))
          .filter((group) => group.filters.length > 0),
        rootOperator: normalizedScope.value.groupOperator || 'and',
      }
    : {
        folderIds: [],
        labelIds: [],
        matchAllLabels: false,
        filterGroups: [],
        rootOperator: 'and' as const,
      }
)

const combinedQuery = useGetConversationsForCombinedScopeInfinite(combinedScope, {
  sortBy,
  sortOrder,
  filterRead,
  filterHasAttachments,
})

const activeQuery = computed(() => {
  switch (normalizedScope.value.type) {
    case 'folder':
      return folderQuery
    case 'label':
      return labelQuery
    case 'combined':
      return combinedQuery
  }
})

const data = computed(() => activeQuery.value.data.value)

const fetchNextPage = async () => {
  await activeQuery.value.fetchNextPage()
}

const hasNextPage = computed(() => activeQuery.value.hasNextPage.value ?? false)

const isFetchingNextPage = computed(() => activeQuery.value.isFetchingNextPage.value ?? false)

const status = computed(() => activeQuery.value.status.value)

const conversations = computed<ConversationListItem[]>(() => {
  return activeQuery.value.data.value?.pages.flatMap((p) => p.items) ?? []
})

const cloneEmail = (email: EmailListItem): EmailListItem => ({
  ...email,
  labels: [...email.labels],
})

const cloneConversation = (conversation: ConversationListItem): ConversationListItem => ({
  ...conversation,
  messages: conversation.messages.map(cloneEmail),
})

const stableConversationCopies = shallowRef<Record<string, ConversationListItem>>({})

watch(
  conversations,
  (items) => {
    const nextMap: Record<string, ConversationListItem> = {}

    for (const conversation of items) {
      const existing = stableConversationCopies.value[conversation.id]
      if (!existing) {
        nextMap[conversation.id] = cloneConversation(conversation)
        continue
      }

      const sourceById = new Map(conversation.messages.map((message) => [message.id, message]))
      const mergedMessages = existing.messages
        .filter((message) => sourceById.has(message.id))
        .map((message) => {
          const source = sourceById.get(message.id)!
          return {
            ...source,
            labels: [...message.labels],
          }
        })

      for (const source of conversation.messages) {
        if (!mergedMessages.some((message) => message.id === source.id)) {
          mergedMessages.push(cloneEmail(source))
        }
      }

      nextMap[conversation.id] = {
        ...conversation,
        messages: mergedMessages,
      }
    }

    stableConversationCopies.value = nextMap
  },
  { immediate: true }
)

const getStableConversation = (conversationId: string) => {
  return stableConversationCopies.value[conversationId] ?? null
}

const getFolderScopedMessage = (
  conversation: ConversationListItem | null | undefined
): EmailListItem | null => {
  if (!conversation) return null

  if (normalizedScope.value.type === 'folder') {
    return (
      conversation.messages.find(
        (message) => message.folder_id === normalizedScope.value.folderId
      ) ??
      conversation.messages[0] ??
      null
    )
  }

  if (normalizedScope.value.type === 'combined' && normalizedScope.value.folderIds.length > 0) {
    return (
      conversation.messages.find((message) =>
        normalizedScope.value.folderIds.includes(message.folder_id)
      ) ??
      conversation.messages[0] ??
      null
    )
  }

  if (normalizedScope.value.type === 'combined' && normalizedScope.value.labelIds.length > 0) {
    return (
      conversation.messages.find((message) =>
        message.labels.some((label) => normalizedScope.value.labelIds.includes(label.id))
      ) ??
      conversation.messages[0] ??
      null
    )
  }

  return conversation.messages[0] ?? null
}

const contextMenuTarget = shallowRef<{
  conversationId: string
  emailId: string
} | null>(null)

const getContextConversation = () => {
  if (!contextMenuTarget.value) return null
  return getStableConversation(contextMenuTarget.value.conversationId)
}

const getContextMenuActiveEmail = () => {
  const conversation = getContextConversation()
  if (!conversation || !contextMenuTarget.value) return null

  return (
    conversation.messages.find((message) => message.id === contextMenuTarget.value?.emailId) ??
    getFolderScopedMessage(conversation)
  )
}

const getContextMenuFirstMessageId = (): string | null => {
  return getContextMenuActiveEmail()?.id ?? null
}

const setContextMenuTarget = (conversationId: string) => {
  const conversation = getStableConversation(conversationId)
  const email = getFolderScopedMessage(conversation)
  if (!conversation || !email) {
    contextMenuTarget.value = null
    return
  }

  contextMenuTarget.value = {
    conversationId: conversation.id,
    emailId: email.id,
  }
}

const replaceContextConversation = (nextConversation: ConversationListItem) => {
  stableConversationCopies.value = {
    ...stableConversationCopies.value,
    [nextConversation.id]: nextConversation,
  }
}

const updateConversationById = (
  conversationId: string,
  updater: (conversation: ConversationListItem) => ConversationListItem
) => {
  const conversation = getStableConversation(conversationId)
  if (!conversation) return null

  const nextConversation = updater(cloneConversation(conversation))
  replaceContextConversation(nextConversation)
  return nextConversation
}

const updateEmailInConversation = (
  conversationId: string,
  emailId: string,
  updater: (email: EmailListItem) => EmailListItem
) => {
  return updateConversationById(conversationId, (conversation) => ({
    ...conversation,
    messages: conversation.messages.map((message) =>
      message.id === emailId ? updater(message) : message
    ),
  }))
}

const handleConversationContextMenu = (conversationId: string) => {
  setContextMenuTarget(conversationId)
}

const toggleLabelForContextEmail = async (labelId: string) => {
  const target = contextMenuTarget.value
  const activeEmail = getContextMenuActiveEmail()
  if (!target || !activeEmail) return

  const existingLabel = activeEmail.labels.find((label) => label.id === labelId)
  const availableLabel = labels.value.find((label) => label.id === labelId)
  if (!existingLabel && !availableLabel) return

  const previousConversation = cloneConversation(getContextConversation()!)
  const nextLabels = existingLabel
    ? activeEmail.labels.filter((label) => label.id !== labelId)
    : [...activeEmail.labels, availableLabel!]

  updateEmailInConversation(target.conversationId, target.emailId, (email) => ({
    ...email,
    labels: nextLabels,
  }))

  try {
    if (existingLabel) {
      await removeLabelFromEmail(target.emailId, labelId)
    } else {
      await addLabelToEmail({ email_id: target.emailId, label_id: labelId })
    }
  } catch (error) {
    replaceContextConversation(previousConversation)
    throw error
  }
}

const handleConversationSelect = (conversationId?: string) => {
  emit('select-conversation', conversationId)
}

const handleAction = async (actionId: string, conversationId: string) => {
  const conversation = getStableConversation(conversationId) || null
  if (!conversation || !conversation.messages[0]) {
    console.error('[MailList] Conversation not found:', conversationId)
    return
  }

  const firstEmail = getFolderScopedMessage(conversation) ?? conversation.messages[0]

  try {
    switch (actionId) {
      case 'archive':
        await archive(firstEmail.id)
        break
      case 'delete':
        await trash(firstEmail.id)
        break
      case 'reply':
        console.log('[MailList] Reply to conversation:', conversationId)
        break
      case 'more':
        console.log('[MailList] More actions for conversation:', conversationId)
        break
      default:
        console.warn('[MailList] Unknown action:', actionId)
    }
  } catch (error) {
    const errorMsg = error instanceof Error ? error.message : String(error)
    console.error('[MailList] ❌ Action failed:', actionId, error)

    if (errorMsg.includes('IMAP config not set') || errorMsg.includes('credentials')) {
      toast.error(t('components.mailList.errors.credentials') as string)
    } else if (errorMsg.includes('Archive folder not found')) {
      toast.error(t('components.mailList.errors.archiveFolder') as string)
    } else {
      toast.error(t('components.mailList.errors.generic') as string)
    }
  }
}

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

const isGroupExpanded = (groupKey: GroupKey) => {
  return expandedGroups.value.has(groupKey)
}

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
  } else {
    return `common.time.${groupKey}`
  }
}

const shouldUseSentAt = computed(() => {
  return currentFolder.value?.folder_type === 'sent' || currentFolder.value?.folder_type === 'draft'
})

const shouldGroup = computed(() => {
  return (
    groupingEnabled.value &&
    (sortBy.value === 'received_at' || sortBy.value === 'sent_at' || sortBy.value === 'size')
  )
})

const getGroupOrder = (): GroupKey[] => {
  if (sortBy.value === 'size') {
    const sizeGroups: GroupKey[] = ['enormous', 'huge', 'veryLarge', 'large', 'medium', 'small']
    return sortOrder.value === 'desc' ? sizeGroups : [...sizeGroups].reverse()
  } else {
    const dateGroups: GroupKey[] = ['today', 'yesterday', 'thisWeek', 'thisMonth', 'older']
    return sortOrder.value === 'desc' ? dateGroups : [...dateGroups].reverse()
  }
}

const getFolderMessages = (conversation: ConversationListItem) => {
  const scopedMessages =
    normalizedScope.value.type === 'folder'
      ? conversation.messages.filter((m) => m.folder_id === normalizedScope.value.folderId)
      : normalizedScope.value.type === 'combined' && normalizedScope.value.folderIds.length > 0
        ? conversation.messages.filter((m) => normalizedScope.value.folderIds.includes(m.folder_id))
        : normalizedScope.value.type === 'combined' && normalizedScope.value.labelIds.length > 0
          ? conversation.messages.filter((m) =>
              m.labels.some((label) => normalizedScope.value.labelIds.includes(label.id))
            )
          : conversation.messages

  const messagesToSort = scopedMessages.length > 0 ? scopedMessages : conversation.messages

  return messagesToSort.sort((a, b) => {
    let aValue: number
    let bValue: number
    switch (sortBy.value) {
      case 'sent_at':
        aValue = a.sent_at ? new Date(a.sent_at).getTime() : 0
        bValue = b.sent_at ? new Date(b.sent_at).getTime() : 0
        break
      case 'size':
        aValue = a.size || 0
        bValue = b.size || 0
        break
      default:
        aValue = new Date(a.received_at).getTime()
        bValue = new Date(b.received_at).getTime()
    }
    return sortOrder.value === 'desc' ? bValue - aValue : aValue - bValue
  })
}

const getPrimaryMessage = (conversation: ConversationListItem) => {
  return getFolderMessages(conversation)[0]
}

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
        const dateToUse =
          shouldUseSentAt.value && msg.sent_at ? dayjs(msg.sent_at) : dayjs(msg.received_at)

        if (dateToUse.isSame(today, 'day')) groups.today.push(conv)
        else if (dateToUse.isSame(yesterday, 'day')) groups.yesterday.push(conv)
        else if (dateToUse.isAfter(thisWeek)) groups.thisWeek.push(conv)
        else if (dateToUse.isAfter(thisMonth)) groups.thisMonth.push(conv)
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

const handleSelect = (conversation: ConversationListItem, event?: MouseEvent) => {
  if (event?.metaKey || event?.ctrlKey || event?.shiftKey) {
    if (props.conversationId && !multiSelect.selectedIds.value.includes(props.conversationId)) {
      const primaryConversation = conversations.value.find((c) => c.id === props.conversationId)
      if (primaryConversation) {
        multiSelect.toggleSelect(primaryConversation)
      }
    }

    multiSelect.toggleSelect(conversation, event)
    handleConversationSelect(conversation.id)
    return
  }

  multiSelect.clearSelection()

  if (normalizedScope.value.type === 'folder' && props.accountId && primaryFolderId.value) {
    router.push(
      `/mail/${props.accountId}/folders/${primaryFolderId.value}/conversations/${conversation.id}`
    )
    return
  }

  if (normalizedScope.value.type === 'label' && primaryLabelId.value) {
    router.push(`/labels/${primaryLabelId.value}/conversations/${conversation.id}`)
    return
  }

  handleConversationSelect(conversation.id)
}

const selectedConversationIds = computed(() => {
  const ids = new Set(multiSelect.selectedIds.value)

  if (props.conversationId) {
    ids.add(props.conversationId)
  }

  return Array.from(ids)
})

const selectedMessageIds = computed(() => {
  const selectedConvIds = selectedConversationIds.value
  const selectedConversations = conversations.value.filter((c) => selectedConvIds.includes(c.id))

  if (normalizedScope.value.type === 'folder') {
    return selectedConversations.flatMap((conv) =>
      conv.messages.filter((m) => m.folder_id === normalizedScope.value.folderId).map((m) => m.id)
    )
  }

  if (normalizedScope.value.type === 'combined' && normalizedScope.value.folderIds.length > 0) {
    return selectedConversations.flatMap((conv) =>
      conv.messages
        .filter((m) => normalizedScope.value.folderIds.includes(m.folder_id))
        .map((m) => m.id)
    )
  }

  if (normalizedScope.value.type === 'combined' && normalizedScope.value.labelIds.length > 0) {
    return selectedConversations.flatMap((conv) =>
      conv.messages
        .filter((m) => m.labels.some((label) => normalizedScope.value.labelIds.includes(label.id)))
        .map((m) => m.id)
    )
  }

  return selectedConversations.flatMap((conv) => conv.messages.map((m) => m.id))
})

watch(
  normalizedScope,
  () => {
    multiSelect.clearSelection()
    contextMenuTarget.value = null
  },
  { deep: true }
)

onMounted(async () => {
  if (normalizedScope.value.type === 'folder' && normalizedScope.value.folderId) {
    await initSync({ folderId: normalizedScope.value.folderId, full: false })
  }

  addContext('mailList', focused)

  const ns = 'mailList'
  register({
    namespace: ns,
    id: 'archiveEmail',
    icon: 'lucide:archive',
    handler: async () => {
      const id = getContextMenuFirstMessageId()
      if (id) await archive(id)
    },
  })
  register({
    namespace: ns,
    id: 'deleteEmail',
    icon: 'lucide:trash-2',
    handler: async () => {
      const id = getContextMenuFirstMessageId()
      if (id) await trash(id)
    },
  })
  register({
    namespace: ns,
    id: 'markRead',
    icon: 'lucide:mail-open',
    handler: async () => {
      const target = contextMenuTarget.value
      const id = getContextMenuFirstMessageId()
      if (!id || !target) return

      const previousConversation = cloneConversation(getContextConversation()!)
      updateEmailInConversation(target.conversationId, target.emailId, (email) => ({
        ...email,
        is_read: true,
      }))

      try {
        await updateRead(id, true)
      } catch (error) {
        replaceContextConversation(previousConversation)
        throw error
      }
    },
  })
  register({
    namespace: ns,
    id: 'markUnread',
    icon: 'lucide:mail',
    handler: async () => {
      const target = contextMenuTarget.value
      const id = getContextMenuFirstMessageId()
      if (!id || !target) return

      const previousConversation = cloneConversation(getContextConversation()!)
      updateEmailInConversation(target.conversationId, target.emailId, (email) => ({
        ...email,
        is_read: false,
      }))

      try {
        await updateRead(id, false)
      } catch (error) {
        replaceContextConversation(previousConversation)
        throw error
      }
    },
  })
  register({
    namespace: ns,
    id: 'moveEmail',
    icon: 'lucide:folder-input',
    handler: async (arg) => {
      const target = contextMenuTarget.value
      const id = getContextMenuFirstMessageId()
      const nextFolderId = arg as string | undefined
      if (!id || !target || !nextFolderId) return

      const previousConversation = cloneConversation(getContextConversation()!)
      updateEmailInConversation(target.conversationId, target.emailId, (email) => ({
        ...email,
        folder_id: nextFolderId,
      }))

      try {
        await move(id, nextFolderId)
      } catch (error) {
        replaceContextConversation(previousConversation)
        throw error
      }
    },
  })
  register({
    namespace: ns,
    id: 'assignLabel',
    icon: 'lucide:tag',
    handler: async (arg) => {
      const labelId = typeof arg === 'string' ? arg : undefined
      if (!labelId) return
      await toggleLabelForContextEmail(labelId)
    },
  })
  register({
    namespace: ns,
    id: 'removeLabel',
    icon: 'lucide:tag',
    handler: async (arg) => {
      const labelId = typeof arg === 'string' ? arg : undefined
      if (!labelId) return
      await toggleLabelForContextEmail(labelId)
    },
  })
  register({
    namespace: ns,
    id: 'setRemindAt',
    icon: 'lucide:bell',
    handler: async (arg) => {
      const target = contextMenuTarget.value
      const id = getContextMenuFirstMessageId()
      if (!id || !target) return

      const remindAt = (arg as string | null) ?? null
      const previousConversation = cloneConversation(getContextConversation()!)
      updateEmailInConversation(target.conversationId, target.emailId, (email) => ({
        ...email,
        remind_at: remindAt ?? undefined,
      }))

      try {
        await setRemindAt(id, remindAt)
      } catch (error) {
        replaceContextConversation(previousConversation)
        throw error
      }
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
    'moveEmail',
    'assignLabel',
    'removeLabel',
    'setRemindAt',
  ]) {
    unregister(ns, id)
  }
})

const route = useRoute()
</script>

<template>
  <div
    ref="mailListRef"
    class="flex h-full flex-col"
  >
    <div class="flex shrink-0 items-center border-b border-b-border p-3">
      <IconName
        v-if="currentFolder"
        :color="currentFolder.color || 'inherit'"
        :icon="currentFolder.icon || 'folder-opened'"
        :name="currentFolder.name || ''"
        class="font-semibold text-primary"
      />
      <div
        v-else-if="currentLabel"
        class="flex items-center gap-2 font-semibold text-primary"
      >
        <Icon
          :name="`lucide:${currentLabel.icon || 'tag'}`"
          :style="{ color: currentLabel.color }"
          :size="18"
        />
        <span>{{ currentLabel.name }}</span>
      </div>
      <div
        v-else-if="normalizedScope.type === 'combined'"
        class="font-semibold text-primary"
      >
        {{ view?.name || t('components.viewWizard.viewTypes.list.name') }}
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

    <EmptyState
      v-if="status === 'success' && conversations.length === 0"
      :description="
        currentFolder?.folder_type === 'inbox'
          ? $t('components.mailList.emptyState.inbox.message')
          : normalizedScope.type === 'label'
            ? $t('components.labelMailList.emptyState.message')
            : $t('components.mailList.emptyState.generic.message')
      "
      :icon="
        currentFolder?.folder_type === 'inbox'
          ? '🎉'
          : normalizedScope.type === 'label'
            ? '🏷️'
            : '📭'
      "
      :title="
        currentFolder?.folder_type === 'inbox'
          ? $t('components.mailList.emptyState.inbox.title')
          : normalizedScope.type === 'label'
            ? $t('components.labelMailList.emptyState.title')
            : $t('components.mailList.emptyState.generic.title')
      "
      class="flex-1"
    />

    <div
      v-else
      ref="scrollerRef"
      class="flex-1 overflow-y-auto p-2"
    >
      <MailContextMenu
        :active-email="getContextMenuActiveEmail()"
        :selected-email-ids="contextMenuTarget ? [contextMenuTarget.emailId] : selectedMessageIds"
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
                  >{{ (virtualRows[virtualItem.index] as any).count }}
                </Badge>
              </div>
            </template>

            <template v-else-if="virtualRows[virtualItem.index]?.type === 'conversation'">
              <ConversationItem
                :conversation="
                  getStableConversation((virtualRows[virtualItem.index] as any).conversation.id) ??
                  (virtualRows[virtualItem.index] as any).conversation
                "
                :folder-id="
                  primaryFolderId ||
                  getPrimaryMessage((virtualRows[virtualItem.index] as any).conversation)
                    ?.folder_id ||
                  ''
                "
                :is-multi-selected="
                  selectedConversationIds.includes(
                    (virtualRows[virtualItem.index] as any).conversation.id
                  ) && selectedConversationIds.length > 1
                "
                :is-selected="
                  selectedConversationIds.includes(
                    (virtualRows[virtualItem.index] as any).conversation.id
                  )
                "
                :selected-ids="selectedConversationIds"
                :selected-message-ids="selectedMessageIds"
                @action="handleAction"
                @click="handleSelect((virtualRows[virtualItem.index] as any).conversation, $event)"
                @contextmenu.capture="
                  handleConversationContextMenu(
                    (virtualRows[virtualItem.index] as any).conversation.id
                  )
                "
              />
            </template>

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
