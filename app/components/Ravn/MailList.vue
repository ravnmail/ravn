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

const { t } = useI18n()
const router = useRouter()

const props = defineProps<{
  folderId: string
  accountId: string
}>()

const { useGetConversationsForFolder } = useConversation()
const { data: conversations } = useGetConversationsForFolder(
  props.folderId,
  200,
  0
)

const { useNavigationFolders, useUpdateSettingsMutation, useInitSyncMutation } = useFolders()
const accountFolders = useNavigationFolders(props.accountId)

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
  return (accountFolders.value || [])
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
  await initSync({ folderId: props.folderId })
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
      alert(t('components.mailList.errors.credentials'))
    } else if (errorMsg.includes('Archive folder not found')) {
      alert(t('components.mailList.errors.archiveFolder'))
    } else {
      alert(t('components.mailList.errors.generic'))
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
  if (selectedConvIds.length === 0) return []

  const selectedConversations = conversations?.value.filter(c =>
    selectedConvIds.includes(c.id)
  )

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
      <h1 class="text-primary font-semibold">
        {{ currentFolder?.name }}
      </h1>
      <div class="ml-auto flex items-center">
        <Popover>
          <PopoverTrigger as-child>
            <Button
              size="bar"
              variant="ghost"
            >
              <Icon
                name="lucide:filter"
              />
            </Button>
          </PopoverTrigger>
          <PopoverContent class="w-80">
            <select
              v-model="filterRead"
              class="text-xs px-2 py-1 rounded border border-border bg-background"
            >
              <option :value="null">{{ $t('components.mailList.filtering.all') }}</option>
              <option :value="false">{{ $t('components.mailList.filtering.unread') }}</option>
              <option :value="true">{{ $t('components.mailList.filtering.read') }}</option>
            </select>

            <!-- Attachments filter -->
            <Button
              class="text-xs px-2 py-1 rounded border border-border hover:bg-accent"
              @click="filterHasAttachments = filterHasAttachments === true ? null : true"
            >
              <Icon
                class="w-3 h-3"
                name="lucide:paperclip"
              />
            </Button>
          </PopoverContent>
        </Popover>
        <Popover>
          <PopoverTrigger as-child>
            <Button
              size="bar"
              variant="ghost"
            >
              <Icon
                name="lucide:settings-2"
              />
            </Button>
          </PopoverTrigger>
          <PopoverContent class="w-80">
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
    <ScrollArea class="p-1 m-1 flex-1">
      <div
        v-if="groupedConversations.total === 0"
        class="h-full flex items-center justify-center py-20"
      >
        <div class="text-center">
          <div class="text-4xl mb-4">
            {{ currentFolder?.folder_type === 'inbox' ? '🎉' : '📭' }}
          </div>
          <div class="text-lg font-semibold mb-2">
            {{
              currentFolder?.folder_type === 'inbox' ? $t('components.mailList.emptyState.inbox.title') : $t('components.mailList.emptyState.generic.title')
            }}
          </div>
          <div class="text-sm text-muted">
            {{
              currentFolder?.folder_type === 'inbox' ? $t('components.mailList.emptyState.inbox.message') : $t('components.mailList.emptyState.generic.message')
            }}
          </div>
        </div>
      </div>

      <template v-else>
        <template v-if="shouldGroup">
          <div
            v-for="groupKey in groupedConversations.groupOrder"
            :key="groupKey"
          >
            <template v-if="groupedConversations.groups[groupKey].length">
              <!-- Group header with expand/collapse -->
              <div
                class="flex items-center gap-2 text-sm font-bold py-2 text-muted hover:text-primary"
                @click="toggleGroup(groupKey)"
              >
                <Icon
                  :name="isGroupExpanded(groupKey) ? 'lucide:chevron-down' : 'lucide:chevron-right'"
                  class="w-4 h-4"
                />
                <span>{{ $t(getGroupLabel(groupKey)) }}</span>
                <span class="text-xs opacity-50">({{ groupedConversations.groups[groupKey].length }})</span>
              </div>
              <div v-if="isGroupExpanded(groupKey)">
                <transition-group
                  enter-active-class="transition-all duration-200"
                  enter-from-class="opacity-0 -translate-x-full"
                  enter-to-class="opacity-100 translate-x-0"
                  leave-active-class="transition-all duration-200"
                  leave-from-class="opacity-100 translate-x-0"
                  leave-to-class="opacity-0 translate-x-full"
                >
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
                </transition-group>
              </div>
            </template>
          </div>
        </template>

        <transition-group
          v-else
          enter-active-class="transition-all duration-200"
          enter-from-class="opacity-0 -translate-x-full"
          enter-to-class="opacity-100 translate-x-0"
          leave-active-class="transition-all duration-200"
          leave-from-class="opacity-100 translate-x-0"
          leave-to-class="opacity-0 translate-x-full"
        >
          <ConversationItem
            v-for="conversation in sortedConversations"
            :key="conversation.id"
            :conversation="conversation"
            :folder-id="folderId"
            :is-selected="useRoute().params.conversation === conversation.id"
            :left-actions="leftActions"
            :right-actions="rightActions"
            @action="handleAction"
            @click.exact="handleSelect(conversation)"
          />
        </transition-group>
      </template>
    </ScrollArea>
  </div>
</template>

