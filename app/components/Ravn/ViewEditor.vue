<script lang="ts" setup>
import type {
  View,
  CreateViewRequest,
  UpdateViewRequest,
  KanbanSwimlane
} from '~/types/view'
import { Button } from '~/components/ui/button'
import IconNameField from '~/components/ui/IconNameField.vue'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle
} from '~/components/ui/dialog'
import FolderSelection from '~/components/Ravn/FolderSelection.vue'
import IconName from '~/components/ui/IconName.vue'
import EmptyState from '~/components/ui/empty/EmptyState.vue'
import LabelSelection from '~/components/Ravn/LabelSelection.vue'

const props = defineProps<{
  viewId?: string | null
}>()

const emit = defineEmits<{
  close: []
  saved: [view: View]
}>()

const { t } = useI18n()
const { createView, updateView, useGetView } = useViews()
const { data: currentView, refetch } = useGetView(props.viewId)
const { labels } = useLabels()

const isDialogOpen = defineModel<boolean>('open', { default: false })
const isLoading = ref(false)
const allFolders = ref<any[]>([])

const formData = ref<CreateViewRequest | UpdateViewRequest>({
  name: '',
  view_type: 'kanban',
  icon: undefined,
  color: undefined,
  folders: [] as string[],
  config: {
    type: 'kanban',
    swimlanes: [] as KanbanSwimlane[]
  }
})

const newSwimlane = ref({
  title: '',
  color: '#3B82F6',
  label_ids: [] as string[],
  folder_ids: [] as string[]
})

watch(currentView, (view) => {
  if (view) {
    formData.value = JSON.parse(JSON.stringify(view)) as CreateViewRequest | UpdateViewRequest
  }
}, { immediate: true })

// Watch for dialog open
watch(() => isDialogOpen.value, async (open) => {
    await refetch()
  if (open) {
    if (currentView.value) {
      formData.value = JSON.parse(JSON.stringify(currentView.value)) as CreateViewRequest | UpdateViewRequest
    }
  } else {
    resetForm()
  }
})

const resetForm = () => {
  formData.value = {
    name: '',
    icon: null,
    color: null,
    view_type: 'kanban',
    folders: [],
    config: {
      type: 'kanban',
      swimlanes: []
    }
  }
  currentView.value = null
}

const addSwimlaneToForm = () => {
  if (!newSwimlane.value.title.trim()) return

  const swimlane: KanbanSwimlane = {
    id: crypto.randomUUID(),
    title: newSwimlane.value.title.trim(),
    icon: newSwimlane.value.icon,
    color: newSwimlane.value.color,
    label_ids: newSwimlane.value.label_ids,
    folder_ids: newSwimlane.value.folder_ids.length > 0 ? newSwimlane.value.folder_ids : undefined,
    state: 'open',
    sort_order: formData.value.config.swimlanes.length
  }

  formData.value.config.swimlanes.push(swimlane)

  newSwimlane.value = {
    title: '',
    color: '#3B82F6',
    icon: undefined,
    label_ids: [],
    folder_ids: []
  }
}

const removeSwimlane = (index: number) => {
  formData.value.config.swimlanes.splice(index, 1)
  formData.value.config.swimlanes.forEach((s, i) => {
    s.sort_order = i
  })
}

const moveSwimlane = (from: number, to: number) => {
  const swimlanes = [...formData.value.config.swimlanes]
  const [moved] = swimlanes.splice(from, 1)
  swimlanes.splice(to, 0, moved)
  swimlanes.forEach((s, i) => {
    s.sort_order = i
  })
  formData.value.config.swimlanes = swimlanes
}

const handleSubmit = async () => {
  if (!formData.value.name.trim()) return
  if (formData.value.config.swimlanes.length === 0) {
    alert(t('components.viewEditor.swimlanes.emptyState'))
    return
  }

  isLoading.value = true
  try {
    if (currentView.value) {
      const view = await updateView({ ...formData.value, id: currentView.value.id } as UpdateViewRequest)
      emit('saved', view)
    } else {
      const view = await createView(formData.value as CreateViewRequest)
      emit('saved', view)
    }

    isDialogOpen.value = false
    resetForm()
  } catch (error) {
    console.error('Failed to save view:', error)
  } finally {
    isLoading.value = false
  }
}

const getLabelName = (labelId: string) => {
  return labels.value.find(l => l.id === labelId)?.name || 'Unknown'
}

const getFolderName = (folderId: string) => {
  return allFolders.value.find(f => String(f.id) === folderId)?.name || 'Unknown'
}
</script>

<template>
  <Dialog v-model:open="isDialogOpen">
    <DialogContent class="max-w-4xl max-h-[90vh] overflow-y-auto">
      <DialogHeader>
        <DialogTitle>
          {{ currentView ? t('components.viewEditor.title.edit') : t('components.viewEditor.title.create') }}
        </DialogTitle>
        <DialogDescription>
          {{ t('components.viewEditor.description') }}
        </DialogDescription>
      </DialogHeader>

      <div class="space-y-3">
        <IconNameField
          :model-value="formData"
          name="name"
          @update:model-value="Object.assign(formData, $event)"
        />

        <FolderSelection
          :model-value="formData.folders"
          @update:model-value="(v) => formData.folders = v"
        />

        <div class="bg-surface rounded-xl p-4">
          <label class="text-sm font-medium">{{ t('components.viewEditor.swimlanes.title') }}</label>
          <div class="space-y-2">
            <IconNameField
              :model-value="{ ...newSwimlane, name: newSwimlane.title }"
              name="title"
              @update:model-value="(e) => newSwimlane = { ...e, title: e.name }"
            />
            <LabelSelection
              :model-value="newSwimlane.label_ids"
              @update:model-value="(v) => newSwimlane.label_ids = v"
            />
            <FolderSelection
              :model-value="newSwimlane.folder_ids"
              @update:model-value="(v) => newSwimlane.folder_ids = v"
            />
            <Button
              :disabled="!newSwimlane.title.trim()"
              size="sm"
              @click="addSwimlaneToForm"
            >
              <Icon
                class="mr-2 h-4 w-4"
                name="lucide:plus"
              />
              {{ t('components.viewEditor.actions.addSwimlane') }}
            </Button>
          </div>
        </div>
        <div class="bg-surface rounded-xl space-y-1 p-2">
          <div
            v-for="(swimlane, index) in formData.config.swimlanes"
            :key="swimlane.id"
            class="flex items-center gap-3 p-1 border border-border rounded-lg bg-background"
          >
            <div class="flex flex-col">
              <Button
                :disabled="index === 0"
                size="bar"
                variant="ghost"
                @click="moveSwimlane(index, index - 1)"
              >
                <Icon name="lucide:chevron-up"/>
              </Button>
              <Button
                :disabled="index === formData.config.swimlanes.length - 1"
                size="bar"
                variant="ghost"
                @click="moveSwimlane(index, index + 1)"
              >
                <Icon name="lucide:chevron-down"/>
              </Button>
            </div>

            <div class="flex-1">
              <IconName
                :color="swimlane.color"
                :icon="swimlane.icon || 'folder-open'"
                :name="swimlane.title"
              />
              <div
                v-if="swimlane.label_ids?.length > 0"
                class="text-xs text-muted"
              >
                {{ t('components.viewEditor.swimlanes.labels') }}: {{ swimlane.label_ids.map(getLabelName).join(', ') }}
              </div>
              <div
                v-if="swimlane.folder_ids?.length > 0"
                class="text-xs text-muted"
              >
                {{ t('components.viewEditor.swimlanes.folders') }}: {{
                  swimlane.folder_ids.map(getFolderName).join(', ')
                }}
              </div>
            </div>
            <Button
              size="sm"
              variant="ghost"
              @click="removeSwimlane(index)"
            >
              <Icon
                class="text-destructive"
                name="lucide:trash-2"
              />
            </Button>
          </div>

          <EmptyState
            v-if="formData.config.swimlanes.length === 0"
            :description="t('components.viewEditor.swimlanes.emptyState')"
          />
        </div>
      </div>

      <DialogFooter>
        <Button
          size="sm"
          variant="outline"
          @click="isDialogOpen = false"
        >
          {{ t('common.actions.cancel') }}
        </Button>
        <Button
          :disabled="!formData.name.trim() || formData.config.swimlanes.length === 0 || isLoading"
          size="sm"
          @click="handleSubmit"
        >
          <Icon
            v-if="isLoading"
            class="mr-2 h-4 w-4 animate-spin"
            name="lucide:loader-2"
          />
          {{
            currentView ? t('components.viewEditor.actions.updateView') : t('components.viewEditor.actions.createView')
          }}
        </Button>
      </DialogFooter>
    </DialogContent>
  </Dialog>
</template>
