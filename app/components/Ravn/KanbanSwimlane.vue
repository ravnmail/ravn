<script lang="ts" setup>
import type { EmailListItem } from '~/types/email'
import type { DragData } from '~/composables/useDragAndDrop'
import { useDropTarget } from '~/composables/useDragAndDrop'
import { ScrollArea } from '~/components/ui/scroll-area'
import KanbanEmailItem from '~/components/Ravn/KanbanEmailItem.vue'
import EmptyState from '~/components/ui/empty/EmptyState.vue'
import MailContextMenu from '~/components/Ravn/MailContextMenu.vue'

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

const emit = defineEmits<{
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
      <div class="flex items-center gap-2">
        <Icon
          :name="`lucide:${swimlane.icon || 'folder'}`"
          :style="{ color: swimlane.color }"
        />
        <h3 class="font-medium flex-1">{{ swimlane.title }}</h3>
        <span class="text-xs bg-secondary text-secondary-foreground font-medium px-2 py-1 rounded-full">
          {{ emails.length }}
        </span>
      </div>
      <div
        :style="{ backgroundColor: swimlane.color }"
        class="h-1 rounded-full w-full"
      />
    </div>
    <div
      :style="{
        backgroundColor
      }"
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
        :title="t('components.kanban.emptyState.noEmails')"
        class="opacity-50"
        icon-name="lucide:inbox"
      />
    </div>
  </div>
</template>
