<script lang="ts" setup>
const { t } = useI18n()
const { alert } = useAlertDialog()

const { views, useDeleteViewMutation } = useViews()
const { isPending: isDeleting, mutate: deleteView } = useDeleteViewMutation()

const isLabelManagerOpen = ref(false)
const isViewEditorOpen = ref(false)
const editingViewId = ref<string | null>(null)
const currentView = computed(() => useRoute().params.view)

const handleEditView = (viewId: string) => {
  editingViewId.value = viewId
  isViewEditorOpen.value = true
}

const handleDeleteView = async (viewId: string) => {
  const view = views.value.find(v => v.id === viewId)
  if (!view) return
  const confirmed = await alert.confirm(
    t('dialogs.confirmDelete.message', view),
    {
      title: t('dialogs.confirmDelete.title'),
      confirmLabel: t('actions.delete'),
      variant: 'destructive'
    }
  )

  if (!confirmed) return

  try {
    deleteView(viewId)
  } catch (error) {
    console.error('Failed to delete view:', error)
  }
}

const handleViewSaved = () => {
  isViewEditorOpen.value = false
  editingViewId.value = null
}
</script>

<template>
  <div class="flex items-center gap-2">
    <UiDropdownMenu v-if="currentView">
      <UiDropdownMenuTrigger as-child>
        <UiButton
          size="sm"
          variant="ghost"
        >
          <Icon
            class="h-4 w-4"
            name="lucide:more-vertical"
          />
        </UiButton>
      </UiDropdownMenuTrigger>

      <UiDropdownMenuContent>
        <UiDropdownMenuItem @click="handleEditView(currentView)">
          <Icon
            class="mr-2 h-4 w-4"
            name="lucide:edit"
          />
          {{ t('components.viewSwitcher.actions.edit') }}
        </UiDropdownMenuItem>

        <UiDropdownMenuSeparator/>

        <UiDropdownMenuItem
          class="text-red-600"
          @click="handleDeleteView(currentView)"
        >
          <Icon
            class="mr-2 h-4 w-4"
            name="lucide:trash-2"
          />
          {{ t('components.viewSwitcher.actions.delete') }}
        </UiDropdownMenuItem>
      </UiDropdownMenuContent>
    </UiDropdownMenu>

    <!-- Manage Labels Button -->
    <UiButton
      size="sm"
      variant="outline"
      @click="isLabelManagerOpen = true"
    >
      <Icon
        class="mr-2 h-4 w-4"
        name="lucide:tag"
      />
      {{ t('components.viewSwitcher.labels') }}
    </UiButton>

    <!-- Label Manager Dialog -->
    <RavnLabelManager v-model:open="isLabelManagerOpen"/>

    <!-- View Editor Dialog -->
    <RavnViewEditor
      v-model:open="isViewEditorOpen"
      :view-id="editingViewId"
      @saved="handleViewSaved"
    />
  </div>
</template>
