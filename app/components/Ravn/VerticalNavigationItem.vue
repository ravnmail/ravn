<script lang="ts" setup>
import { invoke } from '@tauri-apps/api/core'
import { toast } from 'vue-sonner'

import { Button } from '~/components/ui/button'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '~/components/ui/dropdown-menu'
import DropdownMenuItemRich from '~/components/ui/dropdown-menu/DropdownMenuItemRich.vue'
import IconName from '~/components/ui/IconName.vue'
import IconNameField from '~/components/ui/IconNameField.vue'
import { PopoverContent, Popover, PopoverAnchor } from '~/components/ui/popover'
import { SimpleTooltip } from '~/components/ui/tooltip'
import type { DragData } from '~/composables/useDragAndDrop'
import { useDraggable, useDropTarget } from '~/composables/useDragAndDrop'
import type { SidebarNavigationItem, SidebarSectionItem } from '~/composables/useSidebarNavigation'

const emits = defineEmits<{
  (e: 'expanded', isExpanded: boolean): void
}>()

const props = withDefaults(
  defineProps<(SidebarNavigationItem | SidebarSectionItem) & { isExpanded: boolean }>(),
  {}
)

const { t } = useI18n()
const { updateHidden, updateFolderProperties, initSync } = useFolders()
const { move, emptyFolder } = useEmails()
const { alert } = useAlertDialog()
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
      },
    })
  } catch (error) {
    console.error('Failed to move folder:', error)
  }
}

const handleEmailDrop = async (data: DragData) => {
  try {
    if (data.type === 'conversation' && data.messageIds && data.messageIds.length > 0) {
      await Promise.all(
        data.messageIds.map(async (emailId) => {
          try {
            await move(emailId, props.id)
          } catch (error) {
            console.error(`Failed to move email ${emailId}:`, error)
          }
        })
      )
      console.log(`Moved ${data.messageIds.length} message(s) to folder ${props.name}`)
    }
    // Handle multi-drag scenario for emails
    else if (data.isMultiDrag && data.selectedIds && data.selectedIds.length > 1) {
      const emailIds =
        data.messageIds && data.messageIds.length > 0 ? data.messageIds : data.selectedIds
      await Promise.all(
        emailIds.map(async (emailId) => {
          try {
            await move(emailId, props.id)
          } catch (error) {
            console.error(`Failed to move email ${emailId}:`, error)
          }
        })
      )
      console.log(`Moved ${emailIds.length} items to folder ${props.name}`)
    }
    // Single email drag
    else {
      await move(data.id, props.id)
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
      },
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

const canEmpty = computed(() => props.folder_type === 'trash' || props.folder_type === 'spam')

const handleEmpty = async () => {
  const confirmed = await alert.confirm(
    t('components.verticalNavItem.actions.emptyFolderConfirm'),
    {
      title: t('components.verticalNavItem.actions.emptyFolder'),
      variant: 'destructive',
    }
  )
  if (!confirmed) return

  try {
    await emptyFolder(props.id)
    toast(t('components.verticalNavItem.actions.emptyFolderSuccess'))
  } catch (err) {
    console.error('Failed to empty folder:', err)
  }
}
</script>

<template>
  <div
    :class="[
      isOver && canDrop ? 'bg-selection ring-1 ring-primary ring-offset-1' : '',
      isOver && !canDrop ? 'ring-1 ring-destructive ring-offset-1' : '',
    ]"
    class="relative"
  >
    <Popover
      :open="isEditing"
      @update:open="
        (v: boolean) => {
          isEditing = v
        }
      "
    >
      <PopoverAnchor />
      <PopoverContent
        :align-offset="8"
        :side-offset="28"
        align="start"
        class="flex items-center gap-1"
        side="bottom"
      >
        <IconNameField
          v-model="editValue"
          name="icon"
          @cancel="cancelEdit"
          @submit="saveEdit"
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
        :class="['flex items-center py-0.5', isDragging ? 'cursor-grabbing opacity-50' : '']"
      >
        <div
          v-if="children?.length"
          class="h-full w-5 pl-1 text-primary opacity-50 transition-opacity hover:opacity-100"
          @click.stop.prevent="toggleExpanded"
        >
          <Icon
            :class="['opacity-50 transition-transform', isExpanded ? 'rotate-90 transform' : '']"
            :size="14"
            name="lucide:chevron-right"
          />
        </div>
        <div
          v-else
          :class="[folder_type ? 'w-5' : 'w-2']"
          class="h-6 shrink-0"
        />
        <IconName
          :color="color"
          :icon="icon"
          :name="name"
          class="text-sm"
        />
        <span
          v-if="unread_count"
          :class="[
            'mr-2 ml-auto text-xs font-semibold text-muted',
            open ? 'opacity-0' : 'group-hover:opacity-0',
          ]"
          >{{ unread_count }}</span
        >
        <DropdownMenuTrigger
          v-if="folder_type"
          as-child
        >
          <Icon
            :class="[
              'absolute right-2 opacity-0 transition-opacity',
              open ? 'opacity-100' : 'group-hover:opacity-50 hover:opacity-100',
            ]"
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
        <DropdownMenuSeparator v-if="canEmpty" />
        <DropdownMenuItemRich
          v-if="canEmpty"
          :label="t('components.verticalNavItem.actions.emptyFolder')"
          class="text-destructive"
          icon="lucide:trash-2"
          @click="handleEmpty"
        />
      </DropdownMenuContent>
    </DropdownMenu>
  </div>
</template>
