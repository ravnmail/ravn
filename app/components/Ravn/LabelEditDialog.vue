<script lang="ts" setup>
import { Button } from '~/components/ui/button'
import IconNameField from '~/components/ui/IconNameField.vue'
import type { Label, UpdateLabelRequest } from '~/types/view'
// Note: delete is intentionally not handled here — it is triggered directly
// from the sidebar dropdown (SidebarLabelItem) to avoid the two-click pattern.

const props = defineProps<{
  label: Label
}>()

const emit = defineEmits<{
  (e: 'saved', label: Label): void
}>()

const { t } = useI18n()
const { useUpdateLabelMutation } = useLabels()

const isDialogOpen = defineModel<boolean>('open', { default: false })

const { isPending: isUpdating, mutateAsync: updateLabel } = useUpdateLabelMutation()

const isLoading = computed(() => isUpdating.value)

const formData = ref({
  name: props.label.name,
  icon: props.label.icon,
  color: props.label.color,
})

// Sync form data whenever the dialog opens or the label prop changes
watch(
  [() => props.label, isDialogOpen],
  ([label, open]) => {
    if (open) {
      formData.value = {
        name: label.name,
        icon: label.icon,
        color: label.color,
      }
    }
  },
  { immediate: true }
)

const canSubmit = computed(() => formData.value.name.trim().length > 0 && !isLoading.value)

const handleSubmit = async () => {
  if (!canSubmit.value) return

  try {
    const updated = await updateLabel({
      id: props.label.id,
      name: formData.value.name.trim(),
      icon: formData.value.icon,
      color: formData.value.color,
    } as UpdateLabelRequest)
    emit('saved', updated)
    isDialogOpen.value = false
  } catch (err) {
    console.error('[LabelEditDialog] Failed to update label:', err)
  }
}
</script>

<template>
  <UiDialog v-model:open="isDialogOpen">
    <UiDialogContent class="max-w-sm">
      <UiDialogHeader>
        <UiDialogTitle>{{ t('components.labelEditDialog.title') }}</UiDialogTitle>
        <UiDialogDescription>
          {{ t('components.labelEditDialog.description') }}
        </UiDialogDescription>
      </UiDialogHeader>

      <div class="space-y-4 py-2">
        <IconNameField
          :model-value="formData"
          @update:model-value="formData = $event as typeof formData"
        />
      </div>

      <UiDialogFooter class="flex justify-end gap-2">
        <Button
          variant="outline"
          @click="isDialogOpen = false"
        >
          {{ t('common.actions.cancel') }}
        </Button>
        <Button
          :disabled="!canSubmit"
          @click="handleSubmit"
        >
          <Icon
            v-if="isUpdating"
            class="mr-1 h-4 w-4 animate-spin"
            name="lucide:loader-2"
          />
          {{ t('common.actions.save') }}
        </Button>
      </UiDialogFooter>
    </UiDialogContent>
  </UiDialog>
</template>
