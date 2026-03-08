<script lang="ts" setup>
import { RadioGroupItem } from 'reka-ui'

import ListFilterBuilder from '~/components/Ravn/ListFilterBuilder.vue'
import { Button } from '~/components/ui/button'
import { Dialog, DialogContent, DialogHeaderCombined } from '~/components/ui/dialog'
import EmailLabel from '~/components/ui/EmailLabel.vue'
import IconName from '~/components/ui/IconName.vue'
import IconNameField from '~/components/ui/IconNameField.vue'
import { RadioGroup } from '~/components/ui/radio-group'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '~/components/ui/select'
import type { ListFilterGroup, View } from '~/types/view'
import type { ViewTemplate } from '~/types/viewTemplate'

const props = defineProps<{
  initialViewType?: 'kanban' | 'calendar' | 'list'
}>()

const emit = defineEmits<{
  close: []
  created: [view: View]
}>()

const isDialogOpen = defineModel<boolean>('open', { default: false })

const { t } = useI18n()

const {
  currentStep,
  selectedViewType,
  selectedTemplate,
  processedTemplate,
  availableTemplates,
  isProcessing,
  calendarDateField,
  calendarFolderIds,
  listFilterGroups,
  reset,
  selectViewType,
  selectTemplate,
  createViewFromTemplate,
  goBack,
} = useViewWizard()

const { labels } = useLabels()
const { accounts } = useAccounts()
const { useGetFolders, mapFolderTree, flatten } = useFolders()
const { data: accountFolders } = useGetFolders()
const folders = computed(() => flatten(mapFolderTree(accountFolders.value, accounts.value)) || [])

const customizations = ref({
  name: '',
  icon: undefined as string | undefined,
  color: undefined as string | undefined,
  folders: [] as string[],
})

const availableLabels = computed(() => labels.value || [])

const handleListFilterGroupsUpdate = (groups: ListFilterGroup[]) => {
  listFilterGroups.value = groups
}

// View types with enabled status
const viewTypes = computed(() => [
  {
    type: 'kanban' as const,
    name: t('components.viewWizard.viewTypes.kanban.name'),
    description: t('components.viewWizard.viewTypes.kanban.description'),
    icon: 'lucide:columns',
    enabled: true,
  },
  {
    type: 'calendar' as const,
    name: t('components.viewWizard.viewTypes.calendar.name'),
    description: t('components.viewWizard.viewTypes.calendar.description'),
    icon: 'lucide:calendar',
    enabled: true,
  },
  {
    type: 'list' as const,
    name: t('components.viewWizard.viewTypes.list.name'),
    description: t('components.viewWizard.viewTypes.list.description'),
    icon: 'lucide:list',
    enabled: true,
  },
])

const calendarDateFieldOptions = computed(() => [
  { value: 'remind_at', label: t('components.viewWizard.calendar.dateFields.remindAt') },
  { value: 'received_at', label: t('components.viewWizard.calendar.dateFields.receivedAt') },
  { value: 'sent_at', label: t('components.viewWizard.calendar.dateFields.sentAt') },
])

const calendarDateFieldModel = computed({
  get: () => calendarDateField.value,
  set: (v: string) => {
    calendarDateField.value = v as 'received_at' | 'sent_at' | 'remind_at'
  },
})

const toggleCalendarFolder = (folderId: string) => {
  const index = calendarFolderIds.value.indexOf(folderId)
  if (index > -1) {
    calendarFolderIds.value.splice(index, 1)
  } else {
    calendarFolderIds.value.push(folderId)
  }
}

watch(
  () => isDialogOpen.value,
  async (open, wasOpen) => {
    if (open && !wasOpen) {
      if (props.initialViewType && currentStep.value === 'type') {
        selectViewType(props.initialViewType)
      }
    } else if (!open && wasOpen) {
      reset()
      customizations.value = {
        name: '',
        icon: undefined,
        color: undefined,
        folders: [],
      }
    }
  }
)

const handleViewTypeSelect = (type: 'kanban' | 'calendar' | 'list') => {
  selectViewType(type)
}

const handleTemplateSelect = async (template: ViewTemplate | null) => {
  if (template) {
    customizations.value.name = template.title
    customizations.value.icon = undefined
    customizations.value.color = undefined
  } else {
    customizations.value.name = t('components.viewNav.newView')
  }
  await selectTemplate(template)
}

const handleCreateView = async () => {
  try {
    const view = await createViewFromTemplate(customizations.value)
    emit('created', view)
    isDialogOpen.value = false
  } catch (error) {
    console.error('Failed to create view:', error)
  }
}

const toggleFolder = (folderId: string) => {
  const index = customizations.value.folders.indexOf(folderId)
  if (index > -1) {
    customizations.value.folders.splice(index, 1)
  } else {
    customizations.value.folders.push(folderId)
  }
}

const dialogTitle = computed(() => {
  switch (currentStep.value) {
    case 'type':
      return t('components.viewWizard.steps.type.title')
    case 'template':
      return t('components.viewWizard.steps.template.title')
    case 'customize':
      return t('components.viewWizard.steps.customize.title')
    default:
      return t('components.viewEditor.title.create')
  }
})

const dialogDescription = computed(() => {
  switch (currentStep.value) {
    case 'type':
      return t('components.viewWizard.steps.type.description')
    case 'template':
      return t('components.viewWizard.steps.template.description')
    case 'customize':
      return t('components.viewWizard.steps.customize.description')
    default:
      return ''
  }
})
</script>

<template>
  <Dialog v-model:open="isDialogOpen">
    <DialogContent class="max-h-[90vh] max-w-4xl overflow-y-auto">
      <DialogHeaderCombined
        :description="dialogDescription"
        :title="dialogTitle"
      />

      <div>
        <!-- Step: Type selection -->
        <RadioGroup
          v-if="currentStep === 'type'"
          class="grid grid-cols-1 gap-1"
        >
          <RadioGroupItem
            v-for="viewType in viewTypes"
            :key="viewType.type"
            :class="[
              'rounded-lg border p-3 text-left transition-all',
              viewType.enabled ? 'hover:bg-selection-background' : 'cursor-not-allowed opacity-50',
              selectedViewType === viewType.type
                ? 'border-selection-border bg-selection-background text-selection-foreground'
                : 'border-border',
            ]"
            :disabled="!viewType.enabled"
            @click="viewType.enabled && handleViewTypeSelect(viewType.type)"
          >
            <div class="flex items-center gap-3">
              <div
                class="flex h-12 w-12 flex-shrink-0 items-center justify-center rounded-lg bg-muted/20"
              >
                <Icon
                  :name="viewType.icon"
                  class="h-8 w-8"
                />
              </div>
              <div>
                <h3 class="flex items-center gap-2 font-semibold">
                  {{ viewType.name }}
                  <span
                    v-if="!viewType.enabled"
                    class="text-muted-foreground rounded bg-muted px-2 py-0.5 text-xs"
                  >
                    {{ t('components.viewWizard.comingSoon') }}
                  </span>
                </h3>
                <p class="text-muted-foreground mt-1 text-sm">
                  {{ viewType.description }}
                </p>
              </div>
            </div>
          </RadioGroupItem>
        </RadioGroup>

        <!-- Step: Template selection -->
        <RadioGroup
          v-else-if="currentStep === 'template'"
          class="grid grid-cols-1 gap-1"
        >
          <RadioGroupItem
            class="w-full rounded-lg border border-border p-3 text-left transition-all hover:border-b-selection-border hover:bg-selection-background"
            :disabled="isProcessing"
            @select="handleTemplateSelect(null)"
          >
            <div class="flex items-start gap-4">
              <div
                class="flex h-12 w-12 flex-shrink-0 items-center justify-center rounded-lg bg-muted/20"
              >
                <Icon
                  class="text-muted-foreground h-6 w-6"
                  name="lucide:plus"
                />
              </div>
              <div class="flex flex-1 flex-col gap-1">
                <h3 class="font-semibold">
                  {{ t('components.viewWizard.startFromScratch.title') }}
                </h3>
                <p class="text-sm text-muted">
                  {{ t('components.viewWizard.startFromScratch.description') }}
                </p>
              </div>
            </div>
          </RadioGroupItem>
          <RadioGroupItem
            v-for="template in availableTemplates"
            :key="template.id"
            class="rounded-lg border border-border p-3 text-left transition-all hover:border-b-selection-border hover:bg-selection-background"
            :disabled="isProcessing"
            @click="handleTemplateSelect(template)"
          >
            <div class="flex items-start gap-4">
              <div
                class="flex h-12 w-12 flex-shrink-0 items-center justify-center rounded-lg bg-muted/20"
              >
                <Icon
                  class="h-6 w-6 text-primary"
                  name="lucide:columns-3-cog"
                />
              </div>
              <div class="flex flex-1 flex-col gap-1">
                <h3 class="font-semibold">{{ template.title }}</h3>
                <p class="text-sm text-muted">
                  {{ template.description }}
                </p>
                <div class="flex flex-wrap gap-1">
                  <EmailLabel
                    v-for="label in template.labels"
                    :key="label.id"
                    v-bind="label"
                  />
                </div>
              </div>
            </div>
          </RadioGroupItem>
        </RadioGroup>

        <!-- Step: Customize (merged preview + customize) -->
        <div
          v-else-if="currentStep === 'customize'"
          class="space-y-4"
        >
          <!-- Name / Icon / Color -->
          <IconNameField
            :model-value="customizations"
            @update:model-value="Object.assign(customizations, $event)"
          />

          <!-- Calendar-specific configuration -->
          <template v-if="selectedViewType === 'calendar'">
            <div class="space-y-4 rounded-lg border bg-muted/30 p-4">
              <h4 class="flex items-center gap-2 text-sm font-medium">
                <Icon
                  class="text-muted-foreground h-4 w-4"
                  name="lucide:calendar-clock"
                />
                {{ t('components.viewWizard.calendar.configTitle') }}
              </h4>

              <!-- Date field selector -->
              <div class="space-y-1.5">
                <label class="text-sm font-medium">
                  {{ t('components.viewWizard.calendar.dateFieldLabel') }}
                </label>
                <Select v-model="calendarDateFieldModel">
                  <SelectTrigger class="w-full">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem
                      v-for="opt in calendarDateFieldOptions"
                      :key="opt.value"
                      :value="opt.value"
                    >
                      {{ opt.label }}
                    </SelectItem>
                  </SelectContent>
                </Select>
                <p class="text-muted-foreground text-xs">
                  {{ t('components.viewWizard.calendar.dateFieldHint') }}
                </p>
              </div>

              <!-- Mailbox / folder selector -->
              <div class="space-y-1.5">
                <label class="text-sm font-medium">
                  {{ t('components.viewWizard.calendar.foldersLabel') }}
                </label>
                <p class="text-muted-foreground text-xs">
                  {{ t('components.viewWizard.calendar.foldersHint') }}
                </p>
                <div
                  v-if="folders.length > 0"
                  class="mt-2 max-h-48 space-y-1 overflow-y-auto"
                >
                  <div
                    v-for="folder in folders"
                    :key="folder.id"
                    class="flex cursor-pointer items-center gap-2 rounded-md px-2 py-1.5 hover:bg-muted/50"
                    @click="toggleCalendarFolder(folder.id)"
                  >
                    <Checkbox
                      :checked="calendarFolderIds.includes(folder.id)"
                      :id="`cal-folder-${folder.id}`"
                    />
                    <label
                      :for="`cal-folder-${folder.id}`"
                      class="flex flex-1 cursor-pointer items-center gap-1.5 text-sm"
                    >
                      <Icon
                        :name="folder.icon || 'lucide:folder'"
                        class="text-muted-foreground h-3.5 w-3.5 flex-shrink-0"
                      />
                      {{ folder.name }}
                    </label>
                  </div>
                </div>
                <p
                  v-else
                  class="text-muted-foreground text-sm italic"
                >
                  {{ t('components.viewWizard.customize.noFolders') }}
                </p>
              </div>
            </div>
          </template>

          <!-- List-specific configuration -->
          <template v-else-if="selectedViewType === 'list'">
            <div class="space-y-4 rounded-lg border bg-muted/30 p-4">
              <div>
                <h4 class="flex items-center gap-2 text-sm font-medium">
                  <Icon
                    class="text-muted-foreground h-4 w-4"
                    name="lucide:list-filter"
                  />
                  {{ t('components.viewWizard.customize.list.title') }}
                </h4>
                <p class="text-muted-foreground mt-1 text-xs">
                  Build a tailored view by adding one or more groups with folder and label rules.
                </p>
              </div>

              <ListFilterBuilder
                :folders="folders"
                :labels="availableLabels"
                :model-value="listFilterGroups"
                description="Add one or more groups and combine folder and label rules inside each group."
                title="Filter groups"
                @update:model-value="handleListFilterGroupsUpdate"
              />
            </div>
          </template>

          <!-- Template overview (shown when a template was selected, for non-calendar views) -->
          <div
            v-else-if="selectedTemplate && processedTemplate"
            class="space-y-3 rounded-lg border bg-muted/30 p-4"
          >
            <div>
              <h4 class="text-sm font-medium">
                {{ t('components.viewWizard.preview.swimlanesTitle') }}
              </h4>
              <div class="mt-2 space-y-1">
                <div
                  v-for="(swimlane, index) in processedTemplate.swimlanes"
                  :key="index"
                  class="flex items-start gap-2 rounded-md border bg-card px-3 py-1.5 text-sm"
                >
                  <IconName
                    :color="swimlane.color"
                    :icon="swimlane.icon"
                    :name="swimlane.name"
                    class="flex-shrink-0"
                  />
                  <div
                    v-if="swimlane.labelIds?.length > 0"
                    class="ml-auto flex flex-wrap gap-1"
                  >
                    <EmailLabel
                      v-for="labelId in swimlane.labelIds"
                      :key="labelId"
                      v-bind="processedTemplate.labels.find((l) => l.realId === labelId)"
                    />
                  </div>
                </div>
              </div>
            </div>
            <div v-if="processedTemplate.labels.length > 0">
              <h4 class="text-sm font-medium">
                {{ t('components.viewWizard.preview.labelsTitle') }}
              </h4>
              <div class="mt-2 flex flex-wrap gap-1">
                <EmailLabel
                  v-for="label in processedTemplate.labels"
                  :key="label.realId"
                  v-bind="label"
                />
              </div>
            </div>
          </div>

          <!-- Loading indicator while processing template -->
          <div
            v-else-if="isProcessing"
            class="text-muted-foreground flex items-center gap-2 py-2 text-sm"
          >
            <Icon
              class="h-4 w-4 animate-spin"
              name="lucide:loader-2"
            />
          </div>
        </div>
      </div>

      <UiDialogFooter>
        <div class="flex w-full justify-between">
          <Button
            v-if="currentStep !== 'type'"
            size="sm"
            variant="outline"
            @click="goBack"
          >
            <Icon
              class="mr-2 h-4 w-4"
              name="lucide:arrow-left"
            />
            {{ t('common.actions.back') }}
          </Button>
          <div v-else />

          <div class="flex gap-2">
            <Button
              size="sm"
              variant="outline"
              @click="isDialogOpen = false"
            >
              {{ t('common.actions.cancel') }}
            </Button>

            <Button
              v-if="currentStep === 'customize'"
              :disabled="!customizations.name.trim() || isProcessing"
              size="sm"
              @click="handleCreateView"
            >
              <Icon
                v-if="isProcessing"
                class="mr-2 h-4 w-4 animate-spin"
                name="lucide:loader-2"
              />
              {{ t('components.viewWizard.actions.createView') }}
            </Button>
          </div>
        </div>
      </UiDialogFooter>
    </DialogContent>
  </Dialog>
</template>
