<script lang="ts" setup>

import EmptyState from '~/components/ui/empty/EmptyState.vue'
import KanbanBoard from '~/components/Ravn/KanbanBoard.vue'

const { t } = useI18n()
const route = useRoute()
const viewId = computed(() => route.params.view as string)

const { views } = useViews()

const currentView = computed(() => {
  return views.value.find(view => view.id === viewId.value) || null
})

const component = computed(() => {
  if (!currentView.value) return null

  switch (currentView.value.view_type) {
    case 'kanban':
      return KanbanBoard
    default:
      return null
  }
})

</script>

<template>
  <div class="flex flex-col h-full w-full">
    <div class="p-3">
      <div class="flex items-center justify-between">
        <div class="flex items-center gap-4">
          <span class="text-muted">/</span>
          <span class="text-primary flex items-center gap-2">
            <Icon
              :name="`lucide:${currentView?.icon || 'grid'}`"
              :style="{ color: currentView?.color }"
              class="shrink-0 "
            />
            {{ currentView?.name }}
          </span>
        </div>
        <RavnViewSwitcher/>
      </div>
    </div>
    <component
      :is="component"
      v-if="currentView"
      :view="currentView"
    />
    <EmptyState
      v-else
      :description="t('pages.view.emptyState.getStarted')"
      :title="t('pages.view.emptyState.description')"
      class="flex items-center justify-center h-full"
      icon-name="lucide:layout-dashboard"
    />
  </div>
</template>
