<script lang="ts" setup>
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
import FolderMenu from '~/components/Ravn/FolderMenu.vue'
import LabelMenu from '~/components/Ravn/LabelMenu.vue'

const props = defineProps<{
  selectedEmailIds?: string[]
  onExecuteAction?: (id: string, arg?: unknown) => void
}>()

const open = ref(false)

const handleFolderSelect = (v: string | string[]) => {
  const id = Array.isArray(v) ? v[0] : v
  if (!id) return
  props.onExecuteAction?.('moveEmail', id)
  nextTick(() => { open.value = false })
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
          <ContextMenuSubContent class="p-0" @open-auto-focus.prevent>
            <FolderMenu
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
          <ContextMenuSubContent class="p-0" @open-auto-focus.prevent>
            <LabelMenu
              @update:selected-labels="(v) => { const id = Array.isArray(v) ? v[0] : v; if (id) onExecuteAction?.('assignLabel', id) }"
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
