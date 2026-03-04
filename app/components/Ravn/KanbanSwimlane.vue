<script lang="ts" setup>
import type { EmailListItem } from '~/types/email'
import type { DragData } from '~/composables/useDragAndDrop'
import { useDropTarget } from '~/composables/useDragAndDrop'
import KanbanEmailItem from '~/components/Ravn/KanbanEmailItem.vue'
import EmptyState from '~/components/ui/empty/EmptyState.vue'
import MailContextMenu from '~/components/Ravn/MailContextMenu.vue'
import { Badge } from '~/components/ui/badge'
import IconName from '~/components/ui/IconName.vue'
import { Popover, PopoverAnchor, PopoverContent } from '~/components/ui/popover'
import { SimpleTooltip } from '~/components/ui/tooltip'
import IconNameField from '~/components/ui/IconNameField.vue'
import { Button } from '~/components/ui/button'
import { DropdownMenu, DropdownMenuContent, DropdownMenuTrigger } from '~/components/ui/dropdown-menu'
import DropdownMenuItemRich from '~/components/ui/dropdown-menu/DropdownMenuItemRich.vue'

const props = defineProps<{
  swimlane: {
    id: string
    title: string
    icon?: string
    color?: string
    label_ids?: string[]
    folder_ids?: string[]
  }
  emails: EmailListItem[]
  selectedConversationId?: string
}>()

const editValue = ref<{ icon?: string | null, name: string, color?: string | null } | null>(null)

const emit = defineEmits<{
  (e: 'update', value: { icon?: string; color?: string; title: string }): void
  (e: 'drop', dragData: DragData, targetSwimlaneId: string): void
  (e: 'emailClick', email: EmailListItem): void
  (e: 'refresh'): void
}>()

const { t } = useI18n()
const swimlaneRef = ref<HTMLElement | null>(null)

const { isOver, canDrop } = useDropTarget(swimlaneRef, {
  getData: () => ({
    type: 'swimlane',
    id: props.swimlane.id,
  }),
  canDrop: (data: DragData) => {
    return data.type === 'email' && data.fromSwimlaneId !== props.swimlane.id
  },
  onDrop: async (data: DragData) => {
    emit('drop', data, props.swimlane.id)
  },
})

const backgroundColor = computed(() => {
  if (isOver.value) {
    return canDrop.value ? `${props.swimlane.color}40` : '#FF000040'
  }
  return props.swimlane.color ? `${props.swimlane.color}20` : 'transparent'
})

const collapsed = ref(false)

const startEdit = () => {
  editValue.value = { icon: props.swimlane.icon, color: props.swimlane.color, name: props.swimlane.title }
}
const cancelEdit = () => {
  editValue.value = null
}

const isEditing = computed({
  get: () => editValue.value !== null,
  set: (v: boolean) => {
    if (!v) {
      cancelEdit()
    } else {
      startEdit()
    }
  }
})

const handleRename = () => {
  if (editValue.value) {
    emit('update', {
      ...props.swimlane,
      icon: editValue.value.icon,
      color: editValue.value.color,
      title: editValue.value.name,
    })
    cancelEdit()
  }
}

// Context menu — one instance per lane, tracks the right-clicked email
const { archive, trash, move, updateRead, addLabelToEmail } = useEmails()
const contextEmail = ref<EmailListItem | null>(null)

const executeAction = async (actionId: string, arg?: unknown) => {
  const email = contextEmail.value
  if (!email) return
  switch (actionId) {
    case 'archiveEmail':
      await archive(email.id)
      emit('refresh')
      break
    case 'deleteEmail':
      await trash(email.id)
      emit('refresh')
      break
    case 'moveEmail':
      await move(email.id, arg as string)
      emit('refresh')
      break
    case 'markRead':
      await updateRead(email.id, true)
      break
    case 'markUnread':
      await updateRead(email.id, false)
      break
    case 'assignLabel':
      await addLabelToEmail({ email_id: email.id, label_id: arg as string })
      break
  }
}
</script>

<template>
  <div
    ref="swimlaneRef"
    class="flex-shrink-0 w-80 flex flex-col group/swimlane"
  >
    <!-- Header -->
    <div class="flex flex-col gap-2 mb-3">
      <Popover
        :open="isEditing"
        @update:open="(v: boolean) => { isEditing = v }"
      >
        <PopoverAnchor/>
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
            @submit="handleRename"
          />
          <Button
            size="xs"
            variant="ghost"
            @click="handleRename"
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

      <div class="flex items-center gap-2">
        <IconName
          :color="swimlane.color"
          :icon="swimlane.icon || 'folder-open'"
          :name="swimlane.title"
          class="flex-1 min-w-0"
          @dblclick="isEditing = true"
        />
        <Badge
          size="sm"
          variant="background"
        >
          {{ emails.length > 99 ? '+99' : emails.length }}
        </Badge>

        <!-- Swimlane actions dropdown -->
        <DropdownMenu>
          <DropdownMenuTrigger as-child>
            <Button
              class="opacity-0 group-hover/swimlane:opacity-100 transition-opacity"
              size="xs"
              variant="ghost"
            >
              <Icon name="lucide:more-horizontal"/>
            </Button>
          </DropdownMenuTrigger>
          <DropdownMenuContent align="end">
            <DropdownMenuItemRich
              :label="t('components.kanban.swimlane.actions.rename')"
              icon="lucide:edit-2"
              @select="isEditing = true"
            />
            <DropdownMenuItemRich
              :icon="collapsed ? 'lucide:chevron-down' : 'lucide:chevron-up'"
              :label="collapsed ? t('components.kanban.swimlane.actions.expand') : t('components.kanban.swimlane.actions.collapse')"
              @select="collapsed = !collapsed"
            />
          </DropdownMenuContent>
        </DropdownMenu>
      </div>

      <div
        :style="{ backgroundColor: swimlane.color }"
        class="h-1 rounded-full w-full"
      />
    </div>

    <!-- Email list (collapsible) -->
    <div
      v-show="!collapsed"
      :style="{ backgroundColor }"
      class="flex-1 min-h-0 rounded-lg p-2 transition-all duration-200"
    >
      <MailContextMenu
        :selected-email-ids="contextEmail ? [contextEmail.id] : []"
        :on-execute-action="executeAction"
      >
        <div>
          <KanbanEmailItem
            v-for="email in emails"
            :key="email.id"
            :email="email"
            :exclude-labels="swimlane.label_ids"
            :is-selected="email.conversation_id === selectedConversationId"
            :swimlane-id="swimlane.id"
            @contextmenu.capture="contextEmail = email"
            @click="emit('emailClick', email)"
          />
        </div>
      </MailContextMenu>
      <EmptyState
        v-if="emails.length === 0"
        :style="{ color: swimlane.color || undefined }"
        :title="t('components.kanban.emptyState.noEmails')"
        class="opacity-30"
        icon-name="lucide:folder-open"
        variant="coloredSticker"
      />
    </div>
  </div>
</template>
