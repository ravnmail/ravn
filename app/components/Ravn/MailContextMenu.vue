<script lang="ts" setup>
import FolderMenu from '~/components/Ravn/FolderMenu.vue'
import LabelMenu from '~/components/Ravn/LabelMenu.vue'
import RemindAtMenu from '~/components/Ravn/RemindAtMenu.vue'
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

const handleRemindAtSelect = async (value: string | null) => {
  try {
    await props.onExecuteAction?.('setRemindAt', value)
    closeMenu()
  } catch (error) {
    throw error
  }
}

const isProcessingFolderChange = ref(false)
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
  if (!props.activeEmail?.id) return

  const { labelId, selected } = payload
  const previousSelected = selectedLabelIds.value
  const nextSelected = selected
    ? [...previousSelected, labelId]
    : previousSelected.filter((currentLabelId) => currentLabelId !== labelId)

  optimisticSelectedLabelIds.value = nextSelected

  try {
    await props.onExecuteAction?.(selected ? 'assignLabel' : 'removeLabel', labelId)
  } catch (error) {
    optimisticSelectedLabelIds.value = previousSelected
    throw error
  }
}
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
          <ContextMenuSubContent
            class="p-0"
            @open-auto-focus.prevent
          >
            <RemindAtMenu @select="handleRemindAtSelect" />
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
