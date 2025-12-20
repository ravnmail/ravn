<script lang="ts" setup>
import { Button } from '~/components/ui/button'
import { DropdownMenu, DropdownMenuContent, DropdownMenuTrigger } from '~/components/ui/dropdown-menu'
import DropdownMenuItemRich from '~/components/ui/dropdown-menu/DropdownMenuItemRich.vue'

const { t } = useI18n()
const { alert } = useAlertDialog()

const { views, useDeleteViewMutation } = useViews()
const { isPending: isDeleting, mutate: deleteView } = useDeleteViewMutation()

const isLabelManagerOpen = ref(false)
const isViewEditorOpen = ref(false)
const editingViewId = ref<string | null>(null)
const currentViewId = computed(() => useRoute().params.view)

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
  <div class="flex items-center gap-2 relative z-10">
    <DropdownMenu v-if="currentViewId">
      <DropdownMenuTrigger>
        <Button
          size="sm"
          variant="ghost"
        >
          <Icon name="lucide:more-vertical"/>
        </Button>
      </DropdownMenuTrigger>
      <DropdownMenuContent>
        <DropdownMenuItemRich
          :label="t('components.viewSwitcher.actions.edit')"
          icon="lucide:edit"
          @select="handleEditView(currentViewId)"
        />
        <DropdownMenuItemRich
          :label="t('components.viewSwitcher.actions.delete')"
          class="text-red-600"
          icon="lucide:trash-2"
          @select="handleDeleteView(currentViewId)"
        />
      </DropdownMenuContent>
    </DropdownMenu>
    <Button
      size="sm"
      variant="outline"
      @click="isLabelManagerOpen = true"
    >
      <Icon
        name="lucide:tag"
      />
      {{ t('components.viewSwitcher.labels') }}
    </Button>

    <RavnLabelManager v-model:open="isLabelManagerOpen"/>

    <RavnViewEditor
      v-model:open="isViewEditorOpen"
      :view-id="editingViewId"
      @saved="handleViewSaved"
    />
  </div>
</template>
