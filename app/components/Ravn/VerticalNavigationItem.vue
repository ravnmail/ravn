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

const props = withDefaults(defineProps<NavigationFolder>(), {})

const { t } = useI18n()
const { updateExpanded, updateHidden, updateFolderProperties, initSync } = useFolders()
const { move } = useEmails()
const localExpanded = ref(props.expanded || false)
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
      accountId: props.accountId,
      folderId: data.id,
      newParentId: props.id || null,
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

// Watch for prop changes (from database updates)
watch(() => props.expanded, (newVal) => {
  localExpanded.value = newVal || false
})

const toggleExpanded = async () => {
  const newExpanded = !localExpanded.value
  localExpanded.value = newExpanded

  try {
    await updateExpanded({ folderId: props.id, isExpanded: newExpanded })
  } catch (err) {
    console.error('Failed to persist expanded state:', err)
    localExpanded.value = !newExpanded
  }
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
    v-if="!hidden"
    class="relative"
  >
    <div
      v-if="isEditing"
      class="px-1 py-2"
    >
      <IconNameField
        v-model="editValue"
        class="mb-2"
      />
      <div class="flex gap-2 justify-end">
        <button
          class="px-3 py-1 text-xs rounded bg-primary text-primary-foreground hover:bg-primary/90"
          @click="saveEdit"
        >
          {{ t('components.verticalNavItem.actions.save') }}
        </button>
        <button
          class="px-3 py-1 text-xs rounded border hover:bg-popover"
          @click="cancelEdit"
        >
          {{ t('components.verticalNavItem.actions.cancel') }}
        </button>
      </div>
    </div>
    <DropdownMenu v-else>
      <div
        ref="folderRef"
        :class="[
          'overflow-hidden transition-all duration-200 rounded group',
          isDragging ? 'opacity-50 cursor-grabbing' : 'cursor-grab'
        ]"
      >
        <NuxtLink
          :class="[
              'flex flex-col px-1 py-1.5 transition-all duration-200',
              'hover:bg-selection',
              isOver && canDrop ? 'ring-2 ring-primary ring-offset-1 bg-accent' : '',
              isOver && !canDrop ? 'ring-2 ring-destructive ring-offset-1' : ''
            ]"
          :to="`/mail/${account_id}/folders/${id}`"
          active-class="bg-selection text-primary"
        >
          <div class="flex items-center gap-1">
            <Icon
              v-if="children?.length"
              :class="['shrink-0 transition-transform opacity-50 cursor-pointer', localExpanded ? 'transform rotate-90' : '']"
              name="lucide:chevron-right"
              @click.stop.prevent="toggleExpanded"
            />
            <span
              v-else
              class="w-4 shrink-0"
            />
            <Icon
              :name="`lucide:${icon}`"
              :style="{ color: color }"
              class="mr-2 shrink-0 text-foreground"
            />
            <span class="grow text-sm font-medium">{{ name }}</span>
            <span
              v-if="unread_count"
              class="ml-auto text-xs text-muted mr-2 group-hover:opacity-0"
            >{{ unread_count }}</span>
            <DropdownMenuTrigger as-child>
              <Icon
                class="absolute right-2 opacity-0 group-hover:opacity-50 hover:opacity-100 transition-opacity"
                name="lucide:ellipsis"
                @click.stop.prevent
              />
            </DropdownMenuTrigger>
          </div>
        </NuxtLink>
        <DropdownMenuContent align="start">
          <DropdownMenuItem @click="startEdit">
            <Icon
              class="w-4 h-4 mr-2"
              name="lucide:pen"
            />
            {{ t('components.verticalNavItem.actions.edit') }}
          </DropdownMenuItem>
          <DropdownMenuItem @click="handleHide">
            <Icon
              class="w-4 h-4 mr-2"
              name="lucide:eye-off"
            />
            {{ t('components.verticalNavItem.actions.hide') }}
          </DropdownMenuItem>
          <DropdownMenuItem @click="handleSync">
            <Icon
              class="w-4 h-4 mr-2"
              name="lucide:refresh-cw"
            />
            {{ t('components.verticalNavItem.actions.sync') }}
          </DropdownMenuItem>
        </DropdownMenuContent>
      </div>
    </DropdownMenu>
    <div
      v-if="children?.length && localExpanded && !isEditing"
      class="pl-3"
    >
      <VerticalNavigationItem
        v-for="child in children"
        :key="child.id"
        v-bind="child"
      />
    </div>
  </div>
</template>