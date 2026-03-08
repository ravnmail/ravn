<script lang="ts" setup>
import FolderSelection from '~/components/Ravn/FolderSelection.vue'
import LabelSelection from '~/components/Ravn/LabelSelection.vue'
import ListFilterBuilder from '~/components/Ravn/ListFilterBuilder.vue'
import { Button } from '~/components/ui/button'
import { Checkbox } from '~/components/ui/checkbox'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '~/components/ui/dialog'
import EmailLabel from '~/components/ui/EmailLabel.vue'
import EmptyState from '~/components/ui/empty/EmptyState.vue'
import IconName from '~/components/ui/IconName.vue'
import IconNameField from '~/components/ui/IconNameField.vue'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '~/components/ui/select'
import type {
  View,
  CreateViewRequest,
  UpdateViewRequest,
  KanbanSwimlane,
  Label,
  CreateLabelRequest,
  CalendarDateField,
  CalendarViewConfig,
  ListViewConfig,
  ListFilterGroup,
  ListViewFilters,
} from '~/types/view'

const props = defineProps<{
  viewId?: string | null
}>()

const emit = defineEmits<{
  close: []
  saved: [view: View]
}>()

const { t } = useI18n()
const { createView, updateView, useGetView } = useViews()
const { data: currentView, refetch } = useGetView(computed(() => props.viewId ?? ''))
const { labels, createLabel, useUpdateLabelMutation, useDeleteLabelMutation } = useLabels()
const { settings } = useSettings()

const { isPending: isDeletingLabel, mutate: deleteLabel } = useDeleteLabelMutation()
const { isPending: isUpdatingLabel, mutate: updateLabel } = useUpdateLabelMutation()

const isDialogOpen = defineModel<boolean>('open', { default: false })
const isLoading = ref(false)
const allFolders = ref<any[]>([])

// Labels section
const showLabelsSection = computed(() => {
  return (
    formData.value.config.type === 'kanban' &&
    settings.value?.views?.kanban?.showLabelsSection !== false
  )
})
const editingLabel = ref<Label | null>(null)
const labelForm = ref<CreateLabelRequest>({ name: '', color: undefined, icon: undefined })
const isLabelFormLoading = computed(() => isDeletingLabel.value || isUpdatingLabel.value)

const formData = ref<CreateViewRequest | UpdateViewRequest>({
  name: '',
  view_type: 'kanban',
  icon: undefined,
  color: undefined,
  folders: [] as string[],
  config: {
    type: 'kanban',
    swimlanes: [] as KanbanSwimlane[],
  },
})

const calendarDateFieldOptions = computed(() => [
  { value: 'remind_at', label: t('components.viewWizard.calendar.dateFields.remindAt') },
  { value: 'received_at', label: t('components.viewWizard.calendar.dateFields.receivedAt') },
  { value: 'sent_at', label: t('components.viewWizard.calendar.dateFields.sentAt') },
])

const calendarDateFieldModel = computed({
  get: () =>
    ((formData.value.config as CalendarViewConfig).date_field || 'remind_at') as CalendarDateField,
  set: (value: string) => {
    if (formData.value.config.type !== 'calendar') return
    formData.value.config = {
      ...(formData.value.config as CalendarViewConfig),
      date_field: value as CalendarDateField,
    }
  },
})

const createEmptyListFilterGroup = (): ListFilterGroup => ({
  id: crypto.randomUUID(),
  operator: 'and',
  rules: [],
})

const normalizeListFilterGroups = (
  groups: ListFilterGroup[],
  { preserveEmpty = false }: { preserveEmpty?: boolean } = {}
): ListFilterGroup[] => {
  const normalizedGroups = (groups || []).map((group, groupIndex) => ({
    id: group.id || `group-${groupIndex}`,
    operator: group.operator || 'and',
    negated: group.negated ?? false,
    rules: (group.rules || [])
      .map((rule, ruleIndex) => ({
        id: rule.id || `${group.id || `group-${groupIndex}`}-rule-${ruleIndex}`,
        source: rule.source,
        values: Array.from(new Set((rule.values || []).filter(Boolean))),
        operator: rule.operator || 'or',
        negated: rule.negated ?? false,
      }))
      .filter((rule) => rule.values.length > 0),
  }))

  if (preserveEmpty) {
    return normalizedGroups.length > 0 ? normalizedGroups : [createEmptyListFilterGroup()]
  }

  return normalizedGroups.filter((group) => group.rules.length > 0)
}

const ensureListConfigFromValue = (config: ListViewConfig): ListViewConfig => ({
  ...config,
  filters: {
    groups: normalizeListFilterGroups(config.filters?.groups || [], {
      preserveEmpty: true,
    }),
  },
})

const buildListFoldersFromGroups = (groups: ListFilterGroup[]) => {
  return Array.from(
    new Set(
      groups.flatMap((group) =>
        group.rules.filter((rule) => rule.source === 'folders').flatMap((rule) => rule.values)
      )
    )
  )
}

const editableListFilterGroups = ref<ListFilterGroup[]>([createEmptyListFilterGroup()])

const syncEditableListGroupsFromConfig = (config: ListViewConfig | null | undefined) => {
  editableListFilterGroups.value = normalizeListFilterGroups(config?.filters?.groups || [], {
    preserveEmpty: true,
  })
}

watch(
  editableListFilterGroups,
  (value) => {
    if (formData.value.config.type !== 'list') return

    const normalizedGroups = normalizeListFilterGroups(value, { preserveEmpty: true })
    formData.value.config = {
      type: 'list',
      filters: {
        groups: normalizedGroups,
      },
    }
    formData.value.folders = buildListFoldersFromGroups(normalizedGroups)
  },
  { deep: true }
)

const newSwimlane = ref({
  title: '',
  color: '#3B82F6',
  label_ids: [] as string[],
  folder_ids: [] as string[],
})
const editingSwimlaneIndex = ref<number | null>(null)

watch(
  currentView,
  (view) => {
    if (view) {
      const nextFormData = JSON.parse(JSON.stringify(view)) as CreateViewRequest | UpdateViewRequest

      if (nextFormData.config.type === 'list') {
        nextFormData.config = ensureListConfigFromValue(nextFormData.config as ListViewConfig)
        syncEditableListGroupsFromConfig(nextFormData.config as ListViewConfig)
      }

      formData.value = nextFormData
    } else {
      formData.value = {
        name: '',
        view_type: 'kanban',
        icon: undefined,
        color: undefined,
        folders: [] as string[],
        config: {
          type: 'kanban',
          swimlanes: [] as KanbanSwimlane[],
        },
      }
    }
  },
  { immediate: true }
)

// Watch for dialog open
watch(
  () => isDialogOpen.value,
  async (open) => {
    if (open) {
      if (props.viewId) {
        await refetch()
      }
      if (currentView.value) {
        const nextFormData = JSON.parse(JSON.stringify(currentView.value)) as
          | CreateViewRequest
          | UpdateViewRequest

        if (nextFormData.config.type === 'list') {
          nextFormData.config = ensureListConfigFromValue(nextFormData.config as ListViewConfig)
          syncEditableListGroupsFromConfig(nextFormData.config as ListViewConfig)
        }

        formData.value = nextFormData
      } else if (formData.value.config.type === 'list') {
        const nextListConfig: ListViewConfig = {
          type: 'list',
          filters: {
            groups: [createEmptyListFilterGroup()],
          },
        }
        formData.value.config = nextListConfig
        formData.value.folders = []
        syncEditableListGroupsFromConfig(nextListConfig)
      }
    } else {
      resetForm()
    }
  }
)

const resetForm = () => {
  formData.value = {
    name: '',
    icon: null,
    color: null,
    view_type: 'kanban',
    folders: [],
    config: {
      type: 'kanban',
      swimlanes: [],
    },
  }
  editableListFilterGroups.value = [createEmptyListFilterGroup()]
  resetSwimlaneForm()
}

const resetSwimlaneForm = () => {
  newSwimlane.value = {
    title: '',
    color: '#3B82F6',
    icon: undefined,
    label_ids: [],
    folder_ids: [],
  }
  editingSwimlaneIndex.value = null
}

const startEditSwimlane = (index: number) => {
  const s = formData.value.config.swimlanes[index]
  newSwimlane.value = {
    title: s.title,
    icon: s.icon,
    color: s.color,
    label_ids: [...(s.label_ids ?? [])],
    folder_ids: [...(s.folder_ids ?? [])],
  }
  editingSwimlaneIndex.value = index
}

const addSwimlaneToForm = () => {
  if (!newSwimlane.value.title.trim()) return

  if (editingSwimlaneIndex.value !== null) {
    // Update existing swimlane in place
    const existing = formData.value.config.swimlanes[editingSwimlaneIndex.value]
    formData.value.config.swimlanes[editingSwimlaneIndex.value] = {
      ...existing,
      title: newSwimlane.value.title.trim(),
      icon: newSwimlane.value.icon,
      color: newSwimlane.value.color,
      label_ids: newSwimlane.value.label_ids,
      folder_ids:
        newSwimlane.value.folder_ids.length > 0 ? newSwimlane.value.folder_ids : undefined,
    }
  } else {
    const swimlane: KanbanSwimlane = {
      id: crypto.randomUUID(),
      title: newSwimlane.value.title.trim(),
      icon: newSwimlane.value.icon,
      color: newSwimlane.value.color,
      label_ids: newSwimlane.value.label_ids,
      folder_ids:
        newSwimlane.value.folder_ids.length > 0 ? newSwimlane.value.folder_ids : undefined,
      state: 'open',
      sort_order: formData.value.config.swimlanes.length,
    }
    formData.value.config.swimlanes.push(swimlane)
  }

  resetSwimlaneForm()
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

  if (formData.value.config.type === 'kanban' && formData.value.config.swimlanes.length === 0) {
    alert(t('components.viewEditor.swimlanes.emptyState'))
    return
  }

  if (formData.value.config.type === 'list') {
    const normalizedGroups = normalizeListFilterGroups(
      (formData.value.config as ListViewConfig).filters?.groups || [],
      { preserveEmpty: false }
    )
    const folderIds = Array.from(
      new Set(
        normalizedGroups.flatMap((group) =>
          group.rules.filter((rule) => rule.source === 'folders').flatMap((rule) => rule.values)
        )
      )
    )

    formData.value.config = {
      ...(formData.value.config as ListViewConfig),
      filters: {
        groups: normalizedGroups,
      },
    }
    formData.value.folders = folderIds
  }

  if (formData.value.config.type === 'calendar') {
    formData.value.folders = [...(formData.value.config as CalendarViewConfig).folder_ids]
  }

  isLoading.value = true
  try {
    if (currentView.value) {
      const view = await updateView({
        ...formData.value,
        id: currentView.value.id,
      } as UpdateViewRequest)
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
  return labels.value.find((l) => l.id === labelId)?.name || 'Unknown'
}

const getFolderName = (folderId: string) => {
  return allFolders.value.find((f) => String(f.id) === folderId)?.name || 'Unknown'
}

// --- Label management ---
const resetLabelForm = () => {
  labelForm.value = { name: '', color: undefined, icon: undefined }
  editingLabel.value = null
}

const startEditLabel = (label: Label) => {
  editingLabel.value = label
  labelForm.value = { name: label.name, color: label.color, icon: label.icon }
}

const handleLabelSubmit = async () => {
  if (!labelForm.value.name.trim()) return
  if (editingLabel.value) {
    await updateLabel({ id: editingLabel.value.id, ...labelForm.value })
  } else {
    await createLabel(labelForm.value as CreateLabelRequest)
  }
  resetLabelForm()
}

const { alert } = useAlertDialog()

const handleDeleteLabel = async (label: Label) => {
  const confirmed = await alert.confirm(t('dialogs.confirmDelete.message', label), {
    title: t('dialogs.confirmDelete.title'),
    confirmLabel: t('actions.delete'),
    variant: 'destructive',
  })
  if (!confirmed) return
  deleteLabel(label.id)
}
</script>

<template>
  <Dialog v-model:open="isDialogOpen">
    <DialogContent class="max-h-[90vh] max-w-4xl overflow-y-auto">
      <DialogHeader>
        <DialogTitle>
          {{
            currentView
              ? t('components.viewEditor.title.edit')
              : t('components.viewEditor.title.create')
          }}
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
          v-if="formData.config.type === 'kanban'"
          :model-value="formData.folders"
          @update:model-value="(v) => (formData.folders = v)"
        />

        <div
          v-if="formData.config.type === 'calendar'"
          class="space-y-4 rounded-lg border bg-muted/30 p-4"
        >
          <h4 class="text-sm font-medium">
            {{ t('components.viewWizard.viewTypes.calendar.name') }}
          </h4>

          <div class="space-y-2">
            <label class="text-sm font-medium">
              {{ t('components.viewWizard.calendar.dateField') }}
            </label>
            <Select v-model="calendarDateFieldModel">
              <SelectTrigger class="w-full">
                <SelectValue :placeholder="t('components.viewWizard.calendar.dateField')" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem
                  v-for="option in calendarDateFieldOptions"
                  :key="option.value"
                  :value="option.value"
                >
                  {{ option.label }}
                </SelectItem>
              </SelectContent>
            </Select>
          </div>

          <div class="space-y-2">
            <label class="text-sm font-medium">
              {{ t('components.viewWizard.calendar.folders') }}
            </label>
            <FolderSelection
              :model-value="(formData.config as CalendarViewConfig).folder_ids"
              @update:model-value="
                (v) => {
                  formData.config = { ...(formData.config as CalendarViewConfig), folder_ids: v }
                  formData.folders = v
                }
              "
            />
          </div>
        </div>

        <div
          v-else-if="formData.config.type === 'list'"
          class="space-y-4 rounded-lg border bg-muted/30 p-4"
        >
          <div class="space-y-1">
            <h4 class="text-sm font-medium">
              {{ t('components.viewWizard.viewTypes.list.name') }}
            </h4>
            <p class="text-muted-foreground text-sm">
              Build this list by adding one or more groups with folder and label rules.
            </p>
          </div>

          <ListFilterBuilder
            :model-value="editableListFilterGroups"
            :folders="allFolders"
            :labels="labels"
            title="Filter groups"
            description="Add one or more groups and combine folder and label rules inside each group."
            @update:model-value="(value) => (editableListFilterGroups = value)"
          />
        </div>

        <template v-else>
          <div class="rounded-xl bg-surface p-4">
            <label class="text-sm font-medium">
              {{
                editingSwimlaneIndex !== null
                  ? t('components.viewEditor.actions.editSwimlane')
                  : t('components.viewEditor.swimlanes.title')
              }}
            </label>
            <div class="space-y-2">
              <IconNameField
                :model-value="{ ...newSwimlane, name: newSwimlane.title }"
                name="title"
                @update:model-value="(e) => (newSwimlane = { ...e, title: e.name })"
              />
              <LabelSelection
                :model-value="newSwimlane.label_ids"
                @update:model-value="(v) => (newSwimlane.label_ids = v)"
              />
              <FolderSelection
                :model-value="newSwimlane.folder_ids"
                @update:model-value="(v) => (newSwimlane.folder_ids = v)"
              />
              <div class="flex gap-2">
                <Button
                  :disabled="!newSwimlane.title.trim()"
                  size="sm"
                  @click="addSwimlaneToForm"
                >
                  <Icon
                    class="mr-2 h-4 w-4"
                    :name="editingSwimlaneIndex !== null ? 'lucide:check' : 'lucide:plus'"
                  />
                  {{
                    editingSwimlaneIndex !== null
                      ? t('components.viewEditor.actions.updateSwimlane')
                      : t('components.viewEditor.actions.addSwimlane')
                  }}
                </Button>
                <Button
                  v-if="editingSwimlaneIndex !== null"
                  size="sm"
                  variant="outline"
                  @click="resetSwimlaneForm"
                >
                  {{ t('common.actions.cancel') }}
                </Button>
              </div>
            </div>
          </div>
          <div class="space-y-1 rounded-xl bg-surface p-2">
            <div
              v-for="(swimlane, index) in formData.config.swimlanes"
              :key="swimlane.id"
              :class="[
                'flex items-center gap-3 rounded-lg border bg-background p-1',
                editingSwimlaneIndex === index ? 'border-primary/50 bg-primary/5' : 'border-border',
              ]"
            >
              <div class="flex flex-col">
                <Button
                  :disabled="index === 0"
                  size="bar"
                  variant="ghost"
                  @click="moveSwimlane(index, index - 1)"
                >
                  <Icon name="lucide:chevron-up" />
                </Button>
                <Button
                  :disabled="index === formData.config.swimlanes.length - 1"
                  size="bar"
                  variant="ghost"
                  @click="moveSwimlane(index, index + 1)"
                >
                  <Icon name="lucide:chevron-down" />
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
                  {{ t('components.viewEditor.swimlanes.labels') }}:
                  {{ swimlane.label_ids.map(getLabelName).join(', ') }}
                </div>
                <div
                  v-if="swimlane.folder_ids?.length > 0"
                  class="text-xs text-muted"
                >
                  {{ t('components.viewEditor.swimlanes.folders') }}:
                  {{ swimlane.folder_ids.map(getFolderName).join(', ') }}
                </div>
              </div>
              <div class="flex gap-1">
                <Button
                  size="sm"
                  variant="ghost"
                  @click="startEditSwimlane(index)"
                >
                  <Icon name="lucide:edit-2" />
                </Button>
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
            </div>

            <EmptyState
              v-if="formData.config.swimlanes.length === 0"
              :description="t('components.viewEditor.swimlanes.emptyState')"
            />
          </div>
        </template>

        <!-- Labels section -->
        <div
          v-if="showLabelsSection"
          class="space-y-3 rounded-xl bg-surface p-4"
        >
          <label class="text-sm font-medium">{{ t('components.viewEditor.labels.title') }}</label>

          <!-- Add / Edit label form -->
          <div class="space-y-2">
            <IconNameField
              :model-value="{ ...labelForm, name: labelForm.name }"
              name="name"
              @update:model-value="(e) => (labelForm = { ...labelForm, ...e })"
            />
            <div class="flex gap-2">
              <Button
                :disabled="!labelForm.name.trim() || isLabelFormLoading"
                size="sm"
                @click="handleLabelSubmit"
              >
                <Icon
                  v-if="isLabelFormLoading"
                  class="mr-2 h-4 w-4 animate-spin"
                  name="lucide:loader-2"
                />
                {{
                  editingLabel
                    ? t('components.labelManager.actions.update')
                    : t('components.labelManager.actions.create')
                }}
                {{ t('common.labels.name') }}
              </Button>
              <Button
                v-if="editingLabel"
                size="sm"
                variant="outline"
                @click="resetLabelForm"
              >
                {{ t('common.actions.cancel') }}
              </Button>
            </div>
          </div>

          <!-- Labels list -->
          <div class="max-h-64 divide-y divide-border overflow-y-auto rounded-lg bg-background">
            <div
              v-for="label in labels"
              :key="label.id"
              class="flex items-center justify-between px-2 py-1.5"
            >
              <EmailLabel v-bind="label" />
              <div class="flex gap-1">
                <Button
                  size="sm"
                  variant="ghost"
                  @click="startEditLabel(label)"
                >
                  <Icon
                    class="h-4 w-4"
                    name="lucide:edit-2"
                  />
                </Button>
                <Button
                  size="sm"
                  variant="ghost"
                  @click="handleDeleteLabel(label)"
                >
                  <Icon
                    class="h-4 w-4 text-destructive"
                    name="lucide:trash-2"
                  />
                </Button>
              </div>
            </div>
            <div
              v-if="labels.length === 0"
              class="text-muted-foreground p-6 text-center text-sm"
            >
              {{ t('components.labelManager.emptyState') }}
            </div>
          </div>
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
          :disabled="
            !formData.name.trim() ||
            (formData.config.type === 'kanban' && formData.config.swimlanes.length === 0) ||
            isLoading
          "
          size="sm"
          @click="handleSubmit"
        >
          <Icon
            v-if="isLoading"
            class="mr-2 h-4 w-4 animate-spin"
            name="lucide:loader-2"
          />
          {{
            currentView
              ? t('components.viewEditor.actions.updateView')
              : t('components.viewEditor.actions.createView')
          }}
        </Button>
      </DialogFooter>
    </DialogContent>
  </Dialog>
</template>
