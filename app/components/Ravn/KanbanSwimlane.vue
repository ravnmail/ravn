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
}>()

const editValue = ref<{ icon?: string | null, name: string, color?: string | null } | null>(null)

const emit = defineEmits<{
  (e: 'update', value: { icon?: string; color?: string; title: string }): void
  (e: 'drop', dragData: DragData, targetSwimlaneId: string): void
  (e: 'emailClick', email: EmailListItem): void
}>()

const { t } = useI18n()
const swimlaneRef = ref<HTMLElement | null>(null)

const { isOver, canDrop } = useDropTarget(swimlaneRef, {
  getData: () => ({
    type: 'swimlane',
    id: props.swimlane.id,
  }),
  canDrop: (data: DragData) => {
    // Don't allow dropping on the same swimlane
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



</script>

<template>
  <div
    ref="swimlaneRef"
    class="flex-shrink-0 w-80 flex flex-col"
  >
    <div
      class="flex flex-col gap-2 mb-3"
      @click="collapsed = !collapsed"
    >
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
          class="flex-1"
          @dblclick="isEditing = true"
        />
        <Badge
          size="sm"
          variant="background"
        >
          {{ emails.length > 99 ? '+99' : emails.length }}
        </Badge>
      </div>
      <div
        :style="{ backgroundColor: swimlane.color }"
        class="h-1 rounded-full w-full"
      />
    </div>
    <div
      :style="{ backgroundColor }"
      class="flex-1 min-h-0 rounded-lg p-2 transition-all duration-200"
    >
      <MailContextMenu>
        <KanbanEmailItem
          v-for="email in emails"
          :key="email.id"
          :email="email"
          :exclude-labels="swimlane.label_ids"
          :swimlane-id="swimlane.id"
          @click="emit('emailClick', email)"
        />
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
