<script lang="ts" setup>
import CalendarBoard from '~/components/Ravn/CalendarBoard.vue'
import KanbanBoard from '~/components/Ravn/KanbanBoard.vue'
import ListViewBoard from '~/components/Ravn/ListViewBoard.vue'
import EmptyState from '~/components/ui/empty/EmptyState.vue'
import IconName from '~/components/ui/IconName.vue'

const { t } = useI18n()
const route = useRoute()
const viewId = computed(() => route.params.view as string)

const { views } = useViews()

const currentView = computed(() => {
  return views.value.find((view) => view.id === viewId.value) || null
})

const component = computed(() => {
  if (!currentView.value) return null

  switch (currentView.value.view_type) {
    case 'kanban':
      return KanbanBoard
    case 'calendar':
      return CalendarBoard
    case 'list':
      return ListViewBoard
    default:
      return null
  }
})
</script>

<template>
  <div class="flex h-full w-full flex-col">
    <component
      :is="component"
      v-if="currentView"
      :view="currentView"
    >
      <div class="flex items-center gap-2">
        <IconName
          :color="currentView?.color"
          :icon="currentView?.icon || 'grid'"
          :name="currentView?.name"
          class="text-primary"
        />
      </div>
      <RavnViewSwitcher class="ml-auto" />
    </component>
    <EmptyState
      v-else
      :description="t('pages.view.emptyState.getStarted')"
      :title="t('pages.view.emptyState.description')"
      class="flex h-full items-center justify-center"
      icon-name="lucide:layout-dashboard"
    />
  </div>
</template>
