<script lang="ts" setup>
import type { Label, CreateLabelRequest, UpdateLabelRequest } from '~/types/view'
import IconNameField from '~/components/ui/IconNameField.vue'
import EmailLabel from '~/components/ui/EmailLabel.vue'
import { Button } from '~/components/ui/button'

const { t } = useI18n()
const { alert } = useAlertDialog()
const { labels, createLabel, useUpdateLabelMutation, useDeleteLabelMutation } = useLabels()

const { isPending: isDeleting, mutate: deleteLabel } = useDeleteLabelMutation()
const { isPending: isUpdating, mutate: updateLabel } = useUpdateLabelMutation()

const isDialogOpen = defineModel<boolean>('open', { default: false })
const isLoading = computed(() => {
  return isDeleting.value || isUpdating.value
})

const editingLabel = ref<Label | null>(null)
const formData = ref<CreateLabelRequest>({
  color: undefined,
  icon: undefined,
  name: '',
})

const resetForm = () => {
  formData.value = {
    name: '',
    icon: undefined,
    color: undefined
  }
  editingLabel.value = null
}

const startEdit = (label: Label) => {
  editingLabel.value = label
  formData.value = {
    name: label.name,
    icon: label.icon,
    color: label.color
  }
}

const cancelEdit = () => {
  resetForm()
}

const handleSubmit = async () => {
  if (!formData.value.name.trim()) return

  if (editingLabel.value) {
    await updateLabel({
      id: editingLabel.value.id,
      ...formData.value,
    } as UpdateLabelRequest)
  } else {
    await createLabel(formData.value as CreateLabelRequest)
  }
  resetForm()
}

const handleDelete = async (label: Label) => {
  const confirmed = await alert.confirm(
    t('dialogs.confirmDelete.message', label),
    {
      title: t('dialogs.confirmDelete.title'),
      confirmLabel: t('actions.delete'),
      variant: 'destructive'
    }
  )
  if (!confirmed) return
  deleteLabel(label.id)
}
</script>

<template>
  <UiDialog v-model:open="isDialogOpen">
    <UiDialogContent class="max-w-2xl max-h-[80vh]">
      <UiDialogHeader>
        <UiDialogTitle>{{ t('components.labelManager.title') }}</UiDialogTitle>
        <UiDialogDescription>
          {{ t('components.labelManager.description') }}
        </UiDialogDescription>
      </UiDialogHeader>

      <div class="space-y-6 py-4">
        <div class="space-y-4">
          <IconNameField
            :model-value="formData"
            @update:model-value="formData = $event as CreateLabelRequest"
          />

          <div class="flex gap-2">
            <UiButton
              :disabled="!formData.name.trim() || isLoading"
              class="flex-1"
              @click="handleSubmit"
            >
              <Icon
                v-if="isLoading"
                class="mr-2 h-4 w-4 animate-spin"
                name="lucide:loader-2"
              />
              {{
                editingLabel ? t('components.labelManager.actions.update') : t('components.labelManager.actions.create')
              }} {{ t('common.labels.name') }}
            </UiButton>
            <UiButton
              v-if="editingLabel"
              variant="outline"
              @click="cancelEdit"
            >
              {{ t('common.actions.cancel') }}
            </UiButton>
          </div>
        </div>
        <div class="bg-surface rounded-lg divide-y divide-border max-h-96 overflow-y-auto">
          <div
            v-for="label in labels"
            :key="label.id"
            class="flex items-center justify-between p-2"
          >
            <EmailLabel v-bind="label"/>
            <div class="flex gap-2">
              <Button
                size="sm"
                variant="ghost"
                @click="startEdit(label)"
              >
                <Icon
                  class="h-4 w-4"
                  name="lucide:edit-2"
                />
              </Button>
              <Button
                size="sm"
                variant="ghost"
                @click="handleDelete(label)"
              >
                <Icon
                  class="h-4 w-4 text-red-500"
                  name="lucide:trash-2"
                />
              </Button>
            </div>
          </div>

          <div
            v-if="labels.length === 0"
            class="p-8 text-center text-gray-500"
          >
            {{ t('components.labelManager.emptyState') }}
          </div>
        </div>
      </div>

      <UiDialogFooter>
        <UiButton
          variant="outline"
          @click="isDialogOpen = false"
        >
          {{ t('common.actions.close') }}
        </UiButton>
      </UiDialogFooter>
    </UiDialogContent>
  </UiDialog>
</template>
