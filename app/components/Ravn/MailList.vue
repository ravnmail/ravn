<script lang="ts" setup>

import dayjs from 'dayjs'
import type { ConversationListItem } from '~/types/conversation'
import ConversationItem from '~/components/Ravn/ConversationItem.vue'
import { useMultiSelect } from '~/composables/useDragAndDrop'
import { ScrollArea } from '~/components/ui/scroll-area'
import { Button } from '~/components/ui/button'
import { Popover, PopoverContent, PopoverTrigger } from '~/components/ui/popover'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '~/components/ui/select'
import { FormField } from '~/components/ui/form'
import { Switch } from '~/components/ui/switch'
import EmptyState from '~/components/ui/empty/EmptyState.vue'
import MailContextMenu from '~/components/Ravn/MailContextMenu.vue'
import { toast } from 'vue-sonner'
import IconName from '~/components/ui/IconName.vue'
import { Badge } from '~/components/ui/badge'

const { t } = useI18n()
const router = useRouter()

const props = defineProps<{
  folderId: string
  accountId: string
  conversationId?: string
}>()

const { useGetConversationsForFolder } = useConversation()
const { data: conversations } = useGetConversationsForFolder(
  props.folderId,
  200,
  0
)

const { folders, getBreadcrumbs, useUpdateSettingsMutation, useInitSyncMutation } = useFolders()

const { mutateAsync: updateSettings } = useUpdateSettingsMutation()
const { mutateAsync: initSync } = useInitSyncMutation()

const { archive, trash } = useEmails()

const multiSelect = useMultiSelect<ConversationListItem>()

const sortBy = ref<string>('received_at')
const sortOrder = ref<string>('desc')
const filterRead = ref<boolean | null>(null)
const filterHasAttachments = ref<boolean | null>(null)

const groupingEnabled = ref<boolean>(true)
const expandedGroups = ref<Set<string>>(new Set(['today', 'yesterday', 'thisWeek', 'thisMonth', 'older', 'enormous', 'huge', 'veryLarge', 'large', 'medium', 'small']))

const currentFolder = computed(() => {
  return (folders.value || [])
    .find(f => f.id === props.folderId)
})

watch(currentFolder, () => {
  const folder = currentFolder.value
  if (folder?.settings) {
    sortBy.value = folder.settings.sort_by || 'received_at'
    sortOrder.value = folder.settings.sort_order || 'desc'
    groupingEnabled.value = folder.settings.grouping_enabled ?? true
    expandedGroups.value = new Set(folder.settings.expanded_groups || ['today', 'yesterday', 'thisWeek', 'thisMonth', 'older', 'enormous', 'huge', 'veryLarge', 'large', 'medium', 'small'])
    filterRead.value = folder.settings.filter_read ?? null
    filterHasAttachments.value = folder.settings.filter_has_attachments ?? null
  }
}, { immediate: true })

let settingsSaveTimeout: ReturnType<typeof setTimeout> | null = null
const saveFolderSettings = () => {
  if (settingsSaveTimeout) {
    clearTimeout(settingsSaveTimeout)
  }

  settingsSaveTimeout = setTimeout(async () => {
    try {
      await updateSettings({
        folderId: props.folderId, settings: {
          ...currentFolder.value,
          sort_by: sortBy.value,
          sort_order: sortOrder.value,
          grouping_enabled: groupingEnabled.value,
          expanded_groups: Array.from(expandedGroups.value),
          filter_read: filterRead.value,
          filter_has_attachments: filterHasAttachments.value,
        }
      })
    } catch (error) {
      console.error('[MailList] Failed to save folder settings:', error)
    }
  }, 500)
}


onMounted(async () => {
  await initSync({ folderId: props.folderId, full: false })
})

watch([sortBy, sortOrder, filterRead, filterHasAttachments], async () => {
  saveFolderSettings()
})

watch([groupingEnabled, expandedGroups], () => {
  saveFolderSettings()
}, { deep: true })

const handleAction = async (actionId: string, conversationId: string) => {
  console.log('[MailList] Action triggered:', { actionId, conversationId })
  const conversation = conversations.value?.find(c => c.id === conversationId) || null
  if (!conversation || !conversation.messages[0]) {
    console.error('[MailList] Conversation not found:', conversationId)
    return
  }

  const firstEmail = conversation.messages[0]
  console.log('[MailList] Processing action:', actionId, 'for conversation:', firstEmail.subject, 'accountId:', firstEmail.account_id)
  try {
    switch (actionId) {
      case 'archive':
        await archive(firstEmail.id)
        break
      case 'delete':
        await trash(firstEmail.id)
        break
      case 'reply':
        // TODO: Implement reply action
        console.log('[MailList] Reply to conversation:', conversationId)
        break
      case 'more':
        // TODO: Implement more actions menu
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
  'today'
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

type GroupedConversationList = {
  groups: Record<GroupKey, ConversationListItem[]>
  total: number
  groupOrder: GroupKey[]
}

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

const getFolderMessages = (conversation: ConversationListItem) => {
  const folderMessages = conversation.messages.filter(m => m.folder_id === props.folderId)

  return folderMessages.sort((a, b) => {
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
      default: // received_at
        aValue = new Date(a.received_at).getTime()
        bValue = new Date(b.received_at).getTime()
    }

    return sortOrder.value === 'desc' ? bValue - aValue : aValue - bValue
  })
}

const getPrimaryMessage = (conversation: ConversationListItem) => {
  const folderMessages = getFolderMessages(conversation)
  return folderMessages[0]
}

const sortedConversations = computed(() => {
  return [...(conversations.value ?? [])].sort((a, b) => {
    const aMsg = getPrimaryMessage(a)
    const bMsg = getPrimaryMessage(b)

    if (!aMsg) return 1
    if (!bMsg) return -1

    let aValue: number
    let bValue: number

    switch (sortBy.value) {
      case 'sent_at':
        aValue = aMsg.sent_at ? new Date(aMsg.sent_at).getTime() : 0
        bValue = bMsg.sent_at ? new Date(bMsg.sent_at).getTime() : 0
        break
      case 'size':
        aValue = aMsg.size || 0
        bValue = bMsg.size || 0
        break
      default: // received_at
        aValue = new Date(aMsg.received_at).getTime()
        bValue = new Date(bMsg.received_at).getTime()
    }

    return sortOrder.value === 'desc' ? bValue - aValue : aValue - bValue
  })
})

const shouldUseSentAt = computed(() => {
  return currentFolder.value?.folder_type === 'sent' || currentFolder.value?.folder_type === 'draft'
})

const shouldGroup = computed(() => {
  return groupingEnabled.value && (sortBy.value === 'received_at' || sortBy.value === 'sent_at' || sortBy.value === 'size')
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

const groupedConversations = computed((): GroupedConversationList => {
  const initialGroups: Record<GroupKey, ConversationListItem[]> = {
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
    small: []
  }

  if (sortBy.value === 'size') {
    return sortedConversations.value.reduce((acc: GroupedConversationList, conversation): GroupedConversationList => {
      const primaryMessage = getPrimaryMessage(conversation)
      if (!primaryMessage) return acc

      const sizeGroup = getSizeGroup(primaryMessage.size || 0)
      acc.groups[sizeGroup].push(conversation)
      acc.total += 1
      return acc
    }, {
      groups: { ...initialGroups },
      total: 0,
      groupOrder: getGroupOrder()
    })
  } else {
    const today = dayjs().startOf('day')
    const yesterday = dayjs().subtract(1, 'day').startOf('day')
    const thisWeek = dayjs().startOf('week')
    const thisMonth = dayjs().startOf('month')

    return sortedConversations.value.reduce((acc: GroupedConversationList, conversation): GroupedConversationList => {
      const primaryMessage = getPrimaryMessage(conversation)
      if (!primaryMessage) return acc

      const dateToUse = shouldUseSentAt.value && primaryMessage.sent_at
        ? dayjs(primaryMessage.sent_at)
        : dayjs(primaryMessage.received_at)

      if (dateToUse.isSame(today, 'day')) {
        acc.groups.today.push(conversation)
      } else if (dateToUse.isSame(yesterday, 'day')) {
        acc.groups.yesterday.push(conversation)
      } else if (dateToUse.isAfter(thisWeek)) {
        acc.groups.thisWeek.push(conversation)
      } else if (dateToUse.isAfter(thisMonth)) {
        acc.groups.thisMonth.push(conversation)
      } else {
        acc.groups.older.push(conversation)
      }
      acc.total += 1
      return acc
    }, {
      groups: { ...initialGroups },
      total: 0,
      groupOrder: getGroupOrder()
    })
  }
})

const leftActions = ref<SwipeAction[]>([
  {
    id: 'archive',
    icon: 'lucide:archive',
    label: 'actions.archive',
    color: 'bg-warning'
  }
])

const rightActions = ref<SwipeAction[]>([
  {
    id: 'more',
    icon: 'lucide:ellipsis',
    label: 'actions.more',
    color: 'bg-gray-500'
  },
  {
    id: 'reply',
    icon: 'lucide:reply',
    label: 'actions.reply',
    color: 'bg-accent'
  },
  {
    id: 'delete',
    icon: 'lucide:trash-2',
    label: 'actions.delete',
    color: 'bg-destructive'
  }
])

const handleSelect = (conversation: ConversationListItem, event?: MouseEvent) => {
  if (event?.metaKey || event?.ctrlKey || event?.shiftKey) {
    multiSelect.toggleSelect(conversation, event)
  } else {
    multiSelect.clearSelection()

    router.push(`/mail/${props.accountId}/folders/${props.folderId}/conversations/${conversation.id}`)
  }
}

const selectedMessageIds = computed(() => {
  const selectedConvIds = multiSelect.selectedIds.value
  const selectedConversations = conversations.value?.filter(c =>
    selectedConvIds.includes(c.id) || props.conversationId === c.id
  ) || []

  return selectedConversations.flatMap(conv =>
    conv.messages
      .filter(m => m.folder_id === props.folderId)
      .map(m => m.id)
  )
})

watch(() => props.folderId, () => {
  multiSelect.clearSelection()
})
</script>

<template>
  <div class="flex flex-col">
    <div class="flex p-3 items-center border-b border-b-border">
      <IconName
        v-if="currentFolder"
        :color="currentFolder.color || 'inherit'"
        :icon="currentFolder.icon || 'folder-opened'"
        :name="currentFolder.name || ''"
        class="text-primary font-semibold"
      />
      <div class="ml-auto flex items-center">
        <Popover>
          <PopoverTrigger as-child>
            <Button
              size="bar"
              variant="ghost"
            >
              <Icon name="lucide:filter"/>
            </Button>
          </PopoverTrigger>
          <PopoverContent class="w-40 p-3">
            <div class="flex flex-col gap-4">
              <Select
                v-model="filterRead"
                class="text-xs px-2 py-1 rounded border border-border bg-background"
              >
                <SelectTrigger>
                  <SelectValue/>
                </SelectTrigger>
                <SelectContent>
                  <SelectItem :value="null">{{ $t('components.mailList.filtering.all') }}</SelectItem>
                  <SelectItem :value="false">{{ $t('components.mailList.filtering.unread') }}</SelectItem>
                  <SelectItem :value="true">{{ $t('components.mailList.filtering.read') }}</SelectItem>
                </SelectContent>
              </Select>
              <div class="flex">
                <Button @click="filterHasAttachments = filterHasAttachments === true ? null : true">
                  <Icon name="lucide:paperclip"/>
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
              <Icon name="lucide:settings-2"/>
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
                      <SelectValue/>
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
                  <Button
                    @click="sortOrder = sortOrder === 'desc' ? 'asc' : 'desc'"
                  >
                    <Icon
                      :name="sortOrder === 'desc' ? 'lucide:arrow-down-wide-narrow' : 'lucide:arrow-up-narrow-wide'"
                    />
                  </Button>
                </div>
              </FormField>
              <FormField name="grouping">
                <label class="flex justify-between items-center space-x-2">
                  <span>{{ $t('components.mailList.grouping.toggle') }}</span>
                  <Switch v-model="groupingEnabled"/>
                </label>
              </FormField>
            </div>
          </PopoverContent>
        </Popover>
      </div>
    </div>
    <ScrollArea class="p-2">
      <EmptyState
        v-if="groupedConversations.total === 0"
        :description="currentFolder?.folder_type === 'inbox' ? $t('components.mailList.emptyState.inbox.message') : $t('components.mailList.emptyState.generic.message')"
        :icon="currentFolder?.folder_type === 'inbox' ? '🎉' : '📭'"
        :title="currentFolder?.folder_type === 'inbox' ? $t('components.mailList.emptyState.inbox.title') : $t('components.mailList.emptyState.generic.title')"
        class="h-full"
      />
      <MailContextMenu
        v-else
        :selected-email-ids="selectedMessageIds"
      >
        <template v-if="shouldGroup">
          <div
            v-for="groupKey in groupedConversations.groupOrder"
            :key="groupKey"
          >
            <template v-if="groupedConversations.groups[groupKey].length">
              <div
                class="flex items-center gap-1 text-sm font-bold py-2 text-muted hover:text-primary"
                @click="toggleGroup(groupKey)"
              >
                <Icon
                  :class="['transition-transform opacity-50', isGroupExpanded(groupKey) ? 'transform rotate-90' : '']"
                  name="lucide:chevron-right"
                />
                <span>{{ $t(getGroupLabel(groupKey)) }}</span>
                <Badge
                  size="sm"
                  variant="background"
                >{{ groupedConversations.groups[groupKey].length }}
                </Badge>
              </div>
              <div v-if="isGroupExpanded(groupKey)">
                <ConversationItem
                  v-for="conversation in groupedConversations.groups[groupKey]"
                  :key="conversation.id"
                  :conversation="conversation"
                  :folder-id="folderId"
                  :is-multi-selected="multiSelect.isSelected(conversation.id).value"
                  :is-selected="useRoute().params.conversation === conversation.id"
                  :left-actions="leftActions"
                  :right-actions="rightActions"
                  :selected-ids="multiSelect.selectedIds.value"
                  :selected-message-ids="selectedMessageIds"
                  @action="handleAction"
                  @click="handleSelect(conversation, $event)"
                />
              </div>
            </template>
          </div>
        </template>
        <template v-else>
          <ConversationItem
            v-for="conversation in sortedConversations"
            :key="conversation.id"
            :conversation="conversation"
            :folder-id="folderId"
            :is-selected="useRoute().params.conversation === conversation.id"
            :left-actions="leftActions"
            :right-actions="rightActions"
            @action="handleAction"
            @contextmenu="handleSelect(conversation, $event)"
            @click.exact="handleSelect(conversation)"
          />
        </template>
      </MailContextMenu>
    </ScrollArea>
  </div>
</template>

