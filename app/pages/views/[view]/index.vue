<script lang="ts" setup>

const { t } = useI18n()
const route = useRoute()
const viewId = computed(() => route.params.view as string)

const { views } = useViews()

const currentView = computed(() => {
  return views.value.find(view => view.id === viewId.value) || null
})

</script>

<template>
  <div class="flex flex-col h-full w-full select-none">
    <div class="p-4">
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

    <RavnKanbanBoard
      v-if="currentView && currentView.view_type === 'kanban'"
      :view="currentView"
    />

    <!-- Empty State -->
    <div
      v-else
      class="flex items-center justify-center h-full"
    >
      <div class="text-center">
        <Icon
          class="h-24 w-24 mx-auto mb-6 text-gray-300 dark:text-gray-700"
          name="lucide:layout-dashboard"
        />
        <h2 class="text-2xl font-bold mb-2">{{ t('pages.view.emptyState.title') }}</h2>
        <p class="text-gray-600 dark:text-gray-400 mb-6 max-w-md">
          {{ t('pages.view.emptyState.description') }}
        </p>
        <div class="space-y-4">
          <p class="text-sm text-gray-500">
            {{ t('pages.view.emptyState.getStarted') }}
          </p>
        </div>
      </div>
    </div>
  </div>
</template>
