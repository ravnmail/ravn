<script lang="ts" setup>
import type { NavigationFolder } from '~/types/sync'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '~/components/ui/dropdown-menu'
import IconNameField from '~/components/ui/IconNameField.vue'
import type { DragData } from '~/composables/useDragAndDrop'
import { useDraggable, useDropTarget } from '~/composables/useDragAndDrop'
import { invoke } from '@tauri-apps/api/core'
import { Button } from '~/components/ui/button'
import { PopoverContent, Popover, PopoverAnchor } from '~/components/ui/popover'
import { SimpleTooltip } from '~/components/ui/tooltip'
import type { SidebarNavigationItem, SidebarSectionItem } from '~/composables/useSidebarNavigation'
import DropdownMenuItemRich from '~/components/ui/dropdown-menu/DropdownMenuItemRich.vue'

const emits = defineEmits<{
  (e: 'expanded', isExpanded: boolean): void
}>()

const props = withDefaults(defineProps<SidebarNavigationItem | SidebarSectionItem & { isExpanded: boolean }>(), {})

const { t } = useI18n()
const { updateHidden, updateFolderProperties, initSync } = useFolders()
const { move } = useEmails()
const isEditing = ref(false)
const editValue = ref({ icon: props.icon, color: props.color, name: props.name })

const folderRef = ref<HTMLElement | null>(null)

const { isDragging } = useDraggable(folderRef, () => ({
  type: 'folder',
  id: String(props.id),
  accountId: props.account_id,
  parentId: props.parent_id || null,
  name: props.name,
}))

const { isOver, canDrop } = useDropTarget(folderRef, {
  getData: () => ({
    type: 'folder',
    id: String(props.id),
    accepts: ['folder', 'email', 'conversation'],
  }),
  canDrop: (data: DragData) => {
    if (!props.folder_type) {
      return false
    }
    if (data.type === 'folder') {
      return data.id !== String(props.id)
    }
    return true
  },
  onDrop: async (data: DragData) => {
    if (data.type === 'folder') {
      await handleFolderDrop(data)
    } else if (data.type === 'email' || data.type === 'conversation') {
      await handleEmailDrop(data)
    }
  },
})

const handleFolderDrop = async (data: DragData) => {
  try {
    await invoke('move_folder', {
      request: {
        account_id: props.account_id,
        folder_id: data.id,
        new_parent_id: props.id || null,
      }
    })
  } catch (error) {
    console.error('Failed to move folder:', error)
  }
}

const handleEmailDrop = async (data: DragData) => {
  try {
    if (data.type === 'conversation' && data.messageIds && data.messageIds.length > 0) {
      for (const emailId of data.messageIds) {
        try {
          move(emailId, props.id)
        } catch (error) {
          console.error(`Failed to move email ${emailId}:`, error)
        }
      }
      console.log(`Moved ${data.messageIds.length} message(s) to folder ${props.name}`)
    }
    // Handle multi-drag scenario for emails
    else if (data.isMultiDrag && data.selectedIds && data.selectedIds.length > 1) {
      for (const emailId of data.selectedIds) {
        try {
          move(emailId, props.id)
        } catch (error) {
          console.error(`Failed to move email ${emailId}:`, error)
        }
      }
      console.log(`Moved ${data.selectedIds.length} items to folder ${props.name}`)
    }
    // Single email drag
    else {
      move(data.id, props.id)
    }
  } catch (error) {
    console.error('Failed to move email to folder:', error)
  }
}

const toggleExpanded = async () => {
  emits('expanded', !props.isExpanded)
}

const handleHide = async () => {
  try {
    await updateHidden({ folderId: props.id, isHidden: true })
  } catch (err) {
    console.error('Failed to hide folder:', err)
  }
}

const handleSync = async () => {
  try {
    await initSync({ folderId: props.id, full: true })
  } catch (err) {
    console.error('Failed to hide folder:', err)
  }
}

const startEdit = () => {
  editValue.value = { icon: props.icon, color: props.color, name: props.name }
  isEditing.value = true
}

const saveEdit = async () => {
  try {
    await updateFolderProperties({
      folderId: props.id,
      request: {
        name: editValue.value.name,
        icon: editValue.value.icon,
        color: editValue.value.color,
      }
    })
    isEditing.value = false
  } catch (err) {
    console.error('Failed to update folder properties:', err)
  }
}

const cancelEdit = () => {
  isEditing.value = false
  editValue.value = { icon: props.icon, color: props.color, name: props.name }
}
</script>

<template>
  <div
    :class="[
      isOver && canDrop ? 'ring-1 ring-primary ring-offset-1 bg-selection' : '',
      isOver && !canDrop ? 'ring-1 ring-destructive ring-offset-1' : '']"
    class="relative"
  >
    <Popover
      :open="isEditing"
      @update:open="(isOpen) => { isEditing = isOpen }"
    >
      <PopoverAnchor/>
      <PopoverContent
        :align-offset="8"
        :side-offset="28"
        align="start"
        class="flex items-center"
        side="bottom"
      >
        <IconNameField
          v-model="editValue"
          class="mr-1"
        />
        <Button
          size="xs"
          variant="ghost"
          @click="saveEdit"
        >
          <Icon
            class="text-success"
            name="lucide:check"
          />
        </Button>
        <SimpleTooltip tooltip-markdown="press `ESC` to cancel">
          <Button
            size="xs"
            variant="ghost"
            @click="cancelEdit"
          >
            <Icon
              class="text-destructive"
              name="lucide:x"
            />
          </Button>
        </SimpleTooltip>
      </PopoverContent>
    </Popover>
    <DropdownMenu v-slot="{ open }">
      <div
        ref="folderRef"
        :class="[
          'flex items-center py-0.5',
          isDragging ? 'opacity-50 cursor-grabbing' : '',
        ]"
      >
        <div
          v-if="children?.length"
          class="w-5 pl-1 h-full opacity-50 hover:opacity-100 text-primary transition-opacity"
          @click.stop.prevent="toggleExpanded"
        >
          <Icon
            :class="['transition-transform opacity-50', isExpanded ? 'transform rotate-90' : '']"
            :size="14"
            name="lucide:chevron-right"
          />
        </div>
        <div
          v-else
          class="w-5 h-6 shrink-0"
        />
        <Icon
          :name="`lucide:${icon}`"
          :size="18"
          :style="{ color: color }"
          class="mr-1.5 shrink-0 text-foreground"
        />
        <span class="grow text-sm font-medium">{{ name }}</span>
        <span
          v-if="unread_count"
          :class="['ml-auto font-semibold text-xs text-muted mr-2', open ? 'opacity-0' : 'group-hover:opacity-0']"
        >{{ unread_count }}</span>
        <DropdownMenuTrigger
          v-if="folder_type"
          as-child
        >
          <Icon
            :class="['absolute right-2 opacity-0 transition-opacity', open ? 'opacity-100' : ' group-hover:opacity-50 hover:opacity-100']"
            name="lucide:ellipsis"
            @click.stop.prevent
          />
        </DropdownMenuTrigger>
      </div>
      <DropdownMenuContent align="start">
        <DropdownMenuItemRich
          :label="t('components.verticalNavItem.actions.edit')"
          icon="lucide:pen"
          @click="startEdit"
        />
        <DropdownMenuItemRich
          :label="t('components.verticalNavItem.actions.hide')"
          icon="lucide:eye-off"
          @click="handleHide"
        />
        <DropdownMenuItemRich
          :label="t('components.verticalNavItem.actions.sync')"
          icon="lucide:refresh-cw"
          @click="handleSync"
        />
      </DropdownMenuContent>
    </DropdownMenu>
  </div>
</template>