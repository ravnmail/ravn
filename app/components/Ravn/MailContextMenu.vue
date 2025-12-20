<script lang="ts" setup>

import {
  ContextMenu,
  ContextMenuContent,
  ContextMenuGroup,
  ContextMenuSeparator, ContextMenuSub, ContextMenuSubContent, ContextMenuSubTrigger,
  ContextMenuTrigger
} from '~/components/ui/context-menu'
import {
 DropdownMenuPortal
} from '~/components/ui/dropdown-menu'
import DropdownMenuItemRich from '~/components/ui/dropdown-menu/DropdownMenuItemRich.vue'
import LabelMenu from '~/components/Ravn/LabelMenu.vue'
import FolderMenu from '~/components/Ravn/FolderMenu.vue'

defineProps<{
  selectedEmailIds?: string[]
}>()


</script>

<template>
  <ContextMenu>
    <ContextMenuTrigger>
      <slot />
    </ContextMenuTrigger>
    <ContextMenuContent>
      <ContextMenuGroup>
        <DropdownMenuItemRich
          icon="lucide:reply"
          label="Reply"
          shortcut="R"
        />
        <DropdownMenuItemRich
          icon="lucide:reply-all"
          label="Reply All"
          shortcut="R"
        />
        <DropdownMenuItemRich
          icon="lucide:forward"
          label="Forward"
          shortcut="F"
        />
      </ContextMenuGroup>
      <ContextMenuSeparator/>
      <ContextMenuGroup>
        <DropdownMenuItemRich
          :shortcut="['Del']"
          icon="lucide:archive"
          label="Archive"
        />
        <DropdownMenuItemRich
          icon="lucide:trash-2"
          label="Delete"
        />
        <ContextMenuSub>
          <ContextMenuSubTrigger>
            <Icon
              :size="16"
              name="lucide:folder-input"
            />
            <span>Move to...</span>
            <DropdownMenuPortal>
              <ContextMenuSubContent>
                <FolderMenu/>
              </ContextMenuSubContent>
            </DropdownMenuPortal>
          </ContextMenuSubTrigger>
        </ContextMenuSub>
        <ContextMenuSub>
          <ContextMenuSubTrigger>
            <Icon
              :size="16"
              name="lucide:tag"
            />
            <span>Labels</span>
            <DropdownMenuPortal>
              <ContextMenuSubContent>
                <LabelMenu/>
              </ContextMenuSubContent>
            </DropdownMenuPortal>
          </ContextMenuSubTrigger>
        </ContextMenuSub>
      </ContextMenuGroup>
      <ContextMenuSeparator/>
      <ContextMenuGroup>
        <DropdownMenuItemRich
          icon="lucide:mail-open"
          label="Mark as read"
        />
        <DropdownMenuItemRich
          icon="lucide:mail"
          label="Mark as unread"
        />
      </ContextMenuGroup>
    </ContextMenuContent>
  </ContextMenu>
</template>