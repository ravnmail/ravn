<script lang="ts" setup>
import type { EmailListItem } from '~/types/email'
import type { DragData } from '~/composables/useDragAndDrop'
import { useDropTarget } from '~/composables/useDragAndDrop'
import { ScrollArea } from '~/components/ui/scroll-area'
import KanbanEmailItem from '~/components/Ravn/KanbanEmailItem.vue'

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
      <ScrollArea class="h-full space-y-2">
        <KanbanEmailItem
          v-for="email in emails"
          :key="email.id"
          :email="email"
          :exclude-labels="swimlane.label_ids"
          :swimlane-id="swimlane.id"
          @click.exact="emit('emailClick', email)"
        />

        <div
          v-if="emails.length === 0"
          class="flex items-center justify-center h-32 text-gray-400 text-sm"
        >
          <div class="text-center">
            <Icon
              class="h-8 w-8 mx-auto mb-2 opacity-30"
              name="lucide:inbox"
            />
            <p>{{ t('components.kanban.emptyState.noEmails') }}</p>
          </div>
        </div>
      </ScrollArea>
    </div>
  </div>
</template>
