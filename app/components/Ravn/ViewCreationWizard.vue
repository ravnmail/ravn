<script lang="ts" setup>
import type { View } from '~/types/view'
import type { ViewTemplate } from '~/types/viewTemplate'
import { Button } from '~/components/ui/button'
import IconNameField from '~/components/ui/IconNameField.vue'
import IconName from '~/components/ui/IconName.vue'
import EmailLabel from '~/components/ui/EmailLabel.vue'
import { RadioGroupItem } from 'reka-ui'
import { RadioGroup } from '~/components/ui/radio-group'
import { Dialog, DialogContent, DialogHeader } from '~/components/ui/dialog'
import { Checkbox } from '~/components/ui/checkbox'

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
  reset,
  selectViewType,
  selectTemplate,
  confirmTemplate,
  createViewFromTemplate,
  goBack,
} = useViewWizard()

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
    enabled: false,
  },
  {
    type: 'list' as const,
    name: t('components.viewWizard.viewTypes.list.name'),
    description: t('components.viewWizard.viewTypes.list.description'),
    icon: 'lucide:list',
    enabled: false,
  },
])

watch(() => isDialogOpen.value, async (open, wasOpen) => {
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
})

const handleViewTypeSelect = (type: 'kanban' | 'calendar' | 'list') => {
  selectViewType(type)
}

const handleTemplateSelect = async (template: ViewTemplate | null) => {
  selectTemplate(template)

  if (template) {
    customizations.value.name = template.title
    customizations.value.icon = undefined
    customizations.value.color = undefined
  } else {
    customizations.value.name = t('components.viewNav.newView')
  }
}

const handleConfirmTemplate = async () => {
  await confirmTemplate()
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

const getLabelById = (labelId: string) => {
  if (!processedTemplate.value) return null
  return processedTemplate.value.labels.find(l => l.realId === labelId)
}

const dialogTitle = computed(() => {
  switch (currentStep.value) {
    case 'type':
      return t('components.viewWizard.steps.type.title')
    case 'template':
      return t('components.viewWizard.steps.template.title')
    case 'preview':
      return t('components.viewWizard.steps.preview.title')
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
    case 'preview':
      return t('components.viewWizard.steps.preview.description')
    case 'customize':
      return t('components.viewWizard.steps.customize.description')
    default:
      return ''
  }
})
</script>

<template>
  <Dialog v-model:open="isDialogOpen">
    <DialogContent class="max-w-4xl max-h-[90vh] overflow-y-auto">
      <DialogHeader>
        <UiDialogTitle>{{ dialogTitle }}</UiDialogTitle>
        <UiDialogDescription>{{ dialogDescription }}</UiDialogDescription>
      </DialogHeader>

      <div>
        <RadioGroup
          v-if="currentStep === 'type'"
          class="grid grid-cols-1 gap-1"
        >
          <RadioGroupItem
            v-for="viewType in viewTypes"
            :key="viewType.type"
            :class="[
                'p-3 border rounded-lg text-left transition-all',
                viewType.enabled
                  ? 'hover:bg-selection-background'
                  : 'opacity-50 cursor-not-allowed',
                selectedViewType === viewType.type
                  ? 'border-selection-border bg-selection-background text-selection-foreground'
                  : 'border-border'
              ]"
            :disabled="!viewType.enabled"
            @click="viewType.enabled && handleViewTypeSelect(viewType.type)"
          >
            <div class="flex items-center gap-3">
              <div class="h-12 w-12 rounded-lg bg-muted/20 flex items-center justify-center flex-shrink-0">
                <Icon
                  :name="viewType.icon"
                  class="h-8 w-8"
                />
              </div>
              <div>
                <h3 class="font-semibold flex items-center gap-2">
                  {{ viewType.name }}
                  <span
                    v-if="!viewType.enabled"
                    class="text-xs bg-muted text-muted-foreground px-2 py-0.5 rounded"
                  >
                      {{ t('components.viewWizard.comingSoon') }}
                    </span>
                </h3>
                <p class="text-sm text-muted-foreground mt-1">
                  {{ viewType.description }}
                </p>
              </div>
            </div>
          </RadioGroupItem>
        </RadioGroup>
        <RadioGroup
          v-else-if="currentStep === 'template'"
          class="grid grid-cols-1 gap-1"
        >
          <RadioGroupItem
            class="w-full p-3 border border-border rounded-lg text-left hover:border-b-selection-border hover:bg-selection-background transition-all"
            @select="handleTemplateSelect(null)"
          >
            <div class="flex items-start gap-4">
              <div class="h-12 w-12 rounded-lg bg-muted/20 flex items-center justify-center flex-shrink-0">
                <Icon
                  class="h-6 w-6 text-muted-foreground"
                  name="lucide:plus"
                />
              </div>
              <div class="flex-1 flex flex-col gap-1">
                <h3 class="font-semibold">{{ t('components.viewWizard.startFromScratch.title') }}</h3>
                <p class="text-sm text-muted">
                  {{ t('components.viewWizard.startFromScratch.description') }}
                </p>
              </div>
            </div>
          </RadioGroupItem>
          <RadioGroupItem
            v-for="template in availableTemplates"
            :key="template.id"
            class="p-3 border border-border rounded-lg text-left hover:border-b-selection-border hover:bg-selection-background transition-all"
            @click="handleTemplateSelect(template)"
          >
            <div class="flex items-start gap-4">
              <div class="h-12 w-12 rounded-lg bg-muted/20 flex items-center justify-center flex-shrink-0">
                <Icon
                  class="h-6 w-6 text-primary"
                  name="lucide:columns-3-cog"
                />
              </div>
              <div class="flex-1 flex flex-col gap-1">
                <h3 class="font-semibold">{{ template.title }}</h3>
                <p class="text-sm text-muted">
                  {{ template.description }}
                </p>
                <div class="flex gap-1 flex-wrap">
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
        <div
          v-else-if="currentStep === 'preview'"
          class="space-y-4"
        >
          <div
            v-if="selectedTemplate"
            class="space-y-4"
          >
            <div class="space-y-2">
              <h3 class="font-semibold text-primary">{{ selectedTemplate.title }}</h3>
              <p class="text-muted">{{ selectedTemplate.description }}</p>
            </div>
            <div class="space-y-2">
              <h4 class="font-medium">{{ t('components.viewWizard.preview.labelsTitle') }}</h4>
              <div class="flex flex-wrap gap-1">
                <EmailLabel
                  v-for="label in selectedTemplate.labels"
                  :key="label.id"
                  v-bind="label"
                />
              </div>
            </div>
            <div class="space-y-2">
              <h4 class="font-medium">{{ t('components.viewWizard.preview.swimlanesTitle') }}</h4>
              <div class="space-y-1">
                <div
                  v-for="(swimlane, index) in selectedTemplate.swimlanes"
                  :key="index"
                  class="p-3 rounded-lg border border-border"
                >
                  <div class="flex flex-col gap-1">
                    <IconName
                      :color="swimlane.color"
                      :icon="swimlane.icon"
                      :name="swimlane.name"
                      class="text-primary"
                    />
                    <p class="text-sm text-muted-foreground">{{ swimlane.description }}</p>
                    <div
                      v-if="swimlane.labels.length > 0"
                      class="flex flex-wrap gap-1"
                    >
                      <EmailLabel
                        v-for="labelId in swimlane.labels"
                        :key="labelId"
                        v-bind=" selectedTemplate.labels.find(l => l.id === labelId)"
                      />
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
        <div
          v-else-if="currentStep === 'customize'"
          class="space-y-3"
        >
          <IconNameField
            :model-value="customizations"
            @update:model-value="Object.assign(customizations, $event)"
          />
          <div class="space-y-2"/>
          <div
            v-if="processedTemplate"
            class="space-y-2"
          >
            <label class="text-sm font-medium">{{ t('components.viewWizard.customize.viewConfig') }}</label>
            <div class="border rounded-lg p-4 bg-muted/50 space-y-3">
              <div class="text-sm">
                <span class="font-medium">{{ processedTemplate.swimlanes.length }}</span> swimlanes
                <span class="mx-2">•</span>
                <span class="font-medium">{{ processedTemplate.labels.length }}</span> labels
              </div>

              <div class="flex flex-wrap gap-2">
                <div
                  v-for="swimlane in processedTemplate.swimlanes"
                  :key="swimlane.name"
                  class="flex items-center gap-2 px-3 py-1.5 rounded-md border bg-card text-sm"
                >
                  <Icon
                    :name="`lucide:${swimlane.icon}`"
                    :style="{ color: swimlane.color }"
                    class="h-4 w-4"
                  />
                  <span>{{ swimlane.name }}</span>
                  <div
                    :style="{ backgroundColor: swimlane.color }"
                    class="h-2 w-2 rounded-full"
                  />
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>

      <UiDialogFooter>
        <div class="flex justify-between w-full">
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
          <div v-else/>

          <div class="flex gap-2">
            <Button
              size="sm"
              variant="outline"
              @click="isDialogOpen = false"
            >
              {{ t('common.actions.cancel') }}
            </Button>

            <Button
              v-if="currentStep === 'preview'"
              :disabled="isProcessing"
              size="sm"
              @click="handleConfirmTemplate"
            >
              <Icon
                v-if="isProcessing"
                class="mr-2 h-4 w-4 animate-spin"
                name="lucide:loader-2"
              />
              {{ t('common.actions.continue') }}
            </Button>

            <Button
              v-else-if="currentStep === 'customize'"
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
