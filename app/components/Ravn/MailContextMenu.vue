<script lang="ts" setup>
import FolderMenu from '~/components/Ravn/FolderMenu.vue'
import LabelMenu from '~/components/Ravn/LabelMenu.vue'
import {
  ContextMenu,
  ContextMenuContent,
  ContextMenuGroup,
  ContextMenuSeparator,
  ContextMenuSub,
  ContextMenuSubContent,
  ContextMenuSubTrigger,
  ContextMenuTrigger,
} from '~/components/ui/context-menu'
import DropdownMenuItemRich from '~/components/ui/dropdown-menu/DropdownMenuItemRich.vue'
import type { EmailListItem } from '~/types/email'

const props = defineProps<{
  selectedEmailIds?: string[]
  activeEmail?: EmailListItem | null
  onExecuteAction?: (id: string, arg?: unknown) => void | Promise<void>
}>()

const { t } = useI18n()
const open = ref(false)

const closeMenu = () => {
  nextTick(() => {
    open.value = false
  })
}

const isProcessingFolderChange = ref(false)
const isProcessingLabelChange = ref(false)
const optimisticSelectedFolderIds = ref<string[] | null>(null)
const optimisticSelectedLabelIds = ref<string[] | null>(null)

const selectedLabelIds = computed(() => {
  return (
    optimisticSelectedLabelIds.value ?? props.activeEmail?.labels?.map((label) => label.id) ?? []
  )
})

const selectedFolderIds = computed(() => {
  return (
    optimisticSelectedFolderIds.value ??
    (props.activeEmail?.folder_id ? [props.activeEmail.folder_id] : [])
  )
})

watch(
  () => props.activeEmail?.id,
  () => {
    optimisticSelectedFolderIds.value = null
    optimisticSelectedLabelIds.value = null
    isProcessingFolderChange.value = false
    isProcessingLabelChange.value = false
  }
)

watch(
  () => props.activeEmail?.folder_id,
  (folderId) => {
    if (optimisticSelectedFolderIds.value && optimisticSelectedFolderIds.value[0] === folderId) {
      optimisticSelectedFolderIds.value = null
      isProcessingFolderChange.value = false
    }
  }
)

watch(
  () =>
    props.activeEmail?.labels
      ?.map((label) => label.id)
      .sort()
      .join('|') ?? '',
  (labelKey) => {
    const optimisticKey = optimisticSelectedLabelIds.value?.slice().sort().join('|') ?? ''
    if (optimisticSelectedLabelIds.value && optimisticKey === labelKey) {
      optimisticSelectedLabelIds.value = null
      isProcessingLabelChange.value = false
    }
  }
)

const handleFolderSelect = async (v: string | string[]) => {
  const id = Array.isArray(v) ? v[0] : v
  const currentFolderId = props.activeEmail?.folder_id

  if (!id || !props.activeEmail?.id || id === currentFolderId || isProcessingFolderChange.value)
    return

  optimisticSelectedFolderIds.value = [id]
  isProcessingFolderChange.value = true

  try {
    await props.onExecuteAction?.('moveEmail', id)
    closeMenu()
  } catch (error) {
    optimisticSelectedFolderIds.value = null
    isProcessingFolderChange.value = false
    throw error
  }
}

const handleLabelToggle = async (payload: { labelId: string; selected: boolean }) => {
  if (!props.activeEmail?.id || isProcessingLabelChange.value) return

  const { labelId, selected } = payload
  const previousSelected = selectedLabelIds.value
  const nextSelected = selected
    ? [...previousSelected, labelId]
    : previousSelected.filter((currentLabelId) => currentLabelId !== labelId)

  optimisticSelectedLabelIds.value = nextSelected
  isProcessingLabelChange.value = true

  try {
    await props.onExecuteAction?.(selected ? 'assignLabel' : 'removeLabel', labelId)
  } catch (error) {
    optimisticSelectedLabelIds.value = null
    isProcessingLabelChange.value = false
    throw error
  }
}

interface ReminderPreset {
  labelKey: string
  icon: string
  getValue: () => string | null
}

const reminderPresets = computed<ReminderPreset[]>(() => {
  const now = new Date()

  const laterToday = new Date(now)
  laterToday.setHours(now.getHours() + 3, 0, 0, 0)

  const tomorrow = new Date(now)
  tomorrow.setDate(now.getDate() + 1)
  tomorrow.setHours(9, 0, 0, 0)

  const nextWeek = new Date(now)
  nextWeek.setDate(now.getDate() + 7)
  nextWeek.setHours(9, 0, 0, 0)

  const nextMonth = new Date(now)
  nextMonth.setMonth(now.getMonth() + 1)
  nextMonth.setDate(1)
  nextMonth.setHours(9, 0, 0, 0)

  return [
    {
      labelKey: 'components.remindAt.laterToday',
      icon: 'lucide:clock-3',
      getValue: () => laterToday.toISOString(),
    },
    {
      labelKey: 'components.remindAt.tomorrow',
      icon: 'lucide:sunrise',
      getValue: () => tomorrow.toISOString(),
    },
    {
      labelKey: 'components.remindAt.nextWeek',
      icon: 'lucide:calendar-days',
      getValue: () => nextWeek.toISOString(),
    },
    {
      labelKey: 'components.remindAt.nextMonth',
      icon: 'lucide:calendar-range',
      getValue: () => nextMonth.toISOString(),
    },
    {
      labelKey: 'components.remindAt.clear',
      icon: 'lucide:x-circle',
      getValue: () => null,
    },
  ]
})
</script>

<template>
  <ContextMenu v-model:open="open">
    <ContextMenuTrigger as-child>
      <slot />
    </ContextMenuTrigger>
    <ContextMenuContent>
      <ContextMenuGroup>
        <DropdownMenuItemRich
          icon="lucide:reply"
          label="Reply"
          @select="onExecuteAction?.('replyEmail')"
        />
        <DropdownMenuItemRich
          icon="lucide:reply-all"
          label="Reply All"
          @select="onExecuteAction?.('replyAllEmail')"
        />
        <DropdownMenuItemRich
          icon="lucide:forward"
          label="Forward"
          @select="onExecuteAction?.('forwardEmail')"
        />
      </ContextMenuGroup>
      <ContextMenuSeparator />
      <ContextMenuGroup>
        <DropdownMenuItemRich
          icon="lucide:archive"
          label="Archive"
          @select="onExecuteAction?.('archiveEmail')"
        />
        <DropdownMenuItemRich
          icon="lucide:trash-2"
          label="Delete"
          @select="onExecuteAction?.('deleteEmail')"
        />
        <ContextMenuSub>
          <ContextMenuSubTrigger>
            <Icon
              :size="16"
              name="lucide:folder-input"
            />
            <span>Move to...</span>
          </ContextMenuSubTrigger>
          <ContextMenuSubContent
            class="p-0"
            @open-auto-focus.prevent
          >
            <FolderMenu
              :selected-folders="selectedFolderIds"
              @update:selected-folders="handleFolderSelect"
            />
          </ContextMenuSubContent>
        </ContextMenuSub>
        <ContextMenuSub>
          <ContextMenuSubTrigger>
            <Icon
              :size="16"
              name="lucide:tag"
            />
            <span>Labels</span>
          </ContextMenuSubTrigger>
          <ContextMenuSubContent
            class="p-0"
            @open-auto-focus.prevent
          >
            <LabelMenu
              :email="activeEmail"
              :selected-labels="selectedLabelIds"
              @toggle="handleLabelToggle"
            />
          </ContextMenuSubContent>
        </ContextMenuSub>
        <ContextMenuSub>
          <ContextMenuSubTrigger>
            <Icon
              :size="16"
              name="lucide:bell"
            />
            <span>{{ t('components.remindAt.menuLabel') }}</span>
          </ContextMenuSubTrigger>
          <ContextMenuSubContent @open-auto-focus.prevent>
            <DropdownMenuItemRich
              v-for="preset in reminderPresets"
              :key="preset.labelKey"
              :icon="preset.icon"
              :label="t(preset.labelKey)"
              @select="onExecuteAction?.('setRemindAt', preset.getValue())"
            />
          </ContextMenuSubContent>
        </ContextMenuSub>
      </ContextMenuGroup>
      <ContextMenuSeparator />
      <ContextMenuGroup>
        <DropdownMenuItemRich
          icon="lucide:mail-open"
          label="Mark as Read"
          @select="onExecuteAction?.('markRead')"
        />
        <DropdownMenuItemRich
          icon="lucide:mail"
          label="Mark as Unread"
          @select="onExecuteAction?.('markUnread')"
        />
      </ContextMenuGroup>
    </ContextMenuContent>
  </ContextMenu>
</template>
