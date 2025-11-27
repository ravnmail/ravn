<script lang="ts" setup>
import type { View } from '~/types/view'
import type { ViewTemplate } from '~/types/viewTemplate'
import { Button } from '~/components/ui/button'
import IconNameField from '~/components/ui/IconNameField.vue'
import IconName from '~/components/ui/IconName.vue'

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

const { useNavigationFolders, flatten } = useFolders()
const accountFolders = useNavigationFolders('019ab4ec-729d-71b0-a370-c5116ca9358c')

const allFolders = computed(() => flatten(accountFolders?.value) || [])

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
  <UiDialog v-model:open="isDialogOpen">
    <UiDialogContent class="max-w-4xl max-h-[90vh] overflow-y-auto">
      <UiDialogHeader>
        <UiDialogTitle>{{ dialogTitle }}</UiDialogTitle>
        <UiDialogDescription>{{ dialogDescription }}</UiDialogDescription>
      </UiDialogHeader>

      <div class="py-4">
        <!-- Step 1: Select View Type -->
        <div
          v-if="currentStep === 'type'"
          class="space-y-4"
        >
          <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
            <button
              v-for="viewType in viewTypes"
              :key="viewType.type"
              :class="[
                'p-6 border rounded-lg text-left transition-all',
                viewType.enabled
                  ? 'hover:border-primary hover:bg-accent cursor-pointer'
                  : 'opacity-50 cursor-not-allowed',
                selectedViewType === viewType.type
                  ? 'border-primary bg-accent'
                  : 'border-border'
              ]"
              :disabled="!viewType.enabled"
              @click="viewType.enabled && handleViewTypeSelect(viewType.type)"
            >
              <div class="flex flex-col items-start gap-3">
                <Icon
                  :class="viewType.enabled ? 'text-primary' : 'text-muted-foreground'"
                  :name="viewType.icon"
                  class="h-8 w-8"
                />
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
            </button>
          </div>
        </div>

        <!-- Step 2: Select Template -->
        <div
          v-else-if="currentStep === 'template'"
          class="space-y-4"
        >
          <!-- Blank Option -->
          <button
            class="w-full p-6 border rounded-lg text-left hover:border-primary hover:bg-accent transition-all"
            @click="handleTemplateSelect(null)"
          >
            <div class="flex items-start gap-4">
              <div class="h-12 w-12 rounded-lg bg-muted flex items-center justify-center flex-shrink-0">
                <Icon
                  class="h-6 w-6 text-muted-foreground"
                  name="lucide:plus"
                />
              </div>
              <div>
                <h3 class="font-semibold">{{ t('components.viewWizard.startFromScratch.title') }}</h3>
                <p class="text-sm text-muted-foreground mt-1">
                  {{ t('components.viewWizard.startFromScratch.description') }}
                </p>
              </div>
            </div>
          </button>

          <!-- Templates -->
          <div class="grid grid-cols-1 gap-4">
            <button
              v-for="template in availableTemplates"
              :key="template.id"
              class="p-6 border rounded-lg text-left hover:border-primary hover:bg-accent transition-all"
              @click="handleTemplateSelect(template)"
            >
              <div class="flex items-start gap-4">
                <div class="h-12 w-12 rounded-lg bg-primary/10 flex items-center justify-center flex-shrink-0">
                  <Icon
                    class="h-6 w-6 text-primary"
                    name="lucide:layout-template"
                  />
                </div>
                <div class="flex-1">
                  <h3 class="font-semibold">{{ template.title }}</h3>
                  <p class="text-sm text-muted-foreground mt-1">
                    {{ template.description }}
                  </p>
                  <div class="flex gap-2 mt-3 flex-wrap">
                    <span
                      v-for="label in template.labels.slice(0, 5)"
                      :key="label.id"
                      :style="{ backgroundColor: label.color }"
                      class="px-2 py-0.5 rounded text-xs text-white"
                    >
                      {{ label.name }}
                    </span>
                    <span
                      v-if="template.labels.length > 5"
                      class="px-2 py-0.5 rounded text-xs bg-muted text-muted-foreground"
                    >
                      +{{ template.labels.length - 5 }} more
                    </span>
                  </div>
                </div>
              </div>
            </button>
          </div>
        </div>

        <!-- Step 3: Preview Template -->
        <div
          v-else-if="currentStep === 'preview'"
          class="space-y-6"
        >
          <div
            v-if="selectedTemplate"
            class="space-y-6"
          >
            <!-- Template Info -->
            <div class="space-y-2">
              <h3 class="font-semibold text-lg">{{ selectedTemplate.title }}</h3>
              <p class="text-muted-foreground">{{ selectedTemplate.description }}</p>
            </div>

            <!-- Labels -->
            <div class="space-y-2">
              <h4 class="font-medium">{{ t('components.viewWizard.preview.labelsTitle') }}</h4>
              <div class="flex flex-wrap gap-2">
                <div
                  v-for="label in selectedTemplate.labels"
                  :key="label.id"
                  class="flex items-center gap-2 px-3 py-2 rounded-lg border bg-card"
                >
                  <Icon
                    :name="`lucide:${label.icon}`"
                    :style="{ color: label.color }"
                    class="h-4 w-4"
                  />
                  <span class="text-sm font-medium">{{ label.name }}</span>
                  <div
                    :style="{ backgroundColor: label.color }"
                    class="h-3 w-3 rounded-full"
                  />
                </div>
              </div>
            </div>

            <!-- Swimlanes -->
            <div class="space-y-2">
              <h4 class="font-medium">{{ t('components.viewWizard.preview.swimlanesTitle') }}</h4>
              <div class="space-y-2">
                <div
                  v-for="(swimlane, index) in selectedTemplate.swimlanes"
                  :key="index"
                  class="p-4 rounded-lg border bg-card"
                >
                  <div class="flex items-start gap-3">
                    <Icon
                      :name="`lucide:${swimlane.icon}`"
                      :style="{ color: swimlane.color }"
                      class="h-5 w-5 flex-shrink-0 mt-0.5"
                    />
                    <div class="flex-1">
                      <div class="flex items-center gap-2">
                        <h5 class="font-medium">{{ swimlane.name }}</h5>
                        <div
                          :style="{ backgroundColor: swimlane.color }"
                          class="h-3 w-3 rounded-full"
                        />
                      </div>
                      <p class="text-sm text-muted-foreground mt-1">{{ swimlane.description }}</p>
                      <div
                        v-if="swimlane.labels.length > 0"
                        class="flex gap-2 mt-2"
                      >
                        <span
                          v-for="labelId in swimlane.labels"
                          :key="labelId"
                          :style="{
                            backgroundColor: selectedTemplate.labels.find(l => l.id === labelId)?.color
                          }"
                          class="px-2 py-0.5 rounded text-xs text-white"
                        >
                          {{ selectedTemplate.labels.find(l => l.id === labelId)?.name }}
                        </span>
                      </div>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>

        <!-- Step 4: Customize -->
        <div
          v-else-if="currentStep === 'customize'"
          class="space-y-6"
        >
          <!-- View Name & Icon -->
          <IconNameField
            :model-value="customizations"
            @update:model-value="Object.assign(customizations, $event)"
          />

          <!-- Folders Selection -->
          <div class="space-y-2">
            <label class="text-sm font-medium">{{ t('components.viewWizard.customize.includeFolders') }}</label>
            <div class="border rounded-lg p-3 space-y-2 max-h-48 overflow-y-auto">
              <div
                v-for="folder in allFolders"
                :key="folder.id"
                :style="{ paddingLeft: `${folder.level}rem` }"
                class="flex items-center gap-2"
              >
                <UiCheckbox
                  :checked="customizations.folders.includes(String(folder.id))"
                  @update:model-value="toggleFolder(String(folder.id))"
                />
                <IconName
                  :color="folder.color"
                  :icon="folder.icon"
                  :name="folder.name"
                />
              </div>
              <div
                v-if="allFolders.length === 0"
                class="text-center py-4 text-muted-foreground text-sm"
              >
                {{ t('components.viewWizard.customize.noFolders') }}
              </div>
            </div>
          </div>

          <!-- Preview of Configuration -->
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
          <UiButton
            v-if="currentStep !== 'type'"
            variant="outline"
            @click="goBack"
          >
            <Icon
              class="mr-2 h-4 w-4"
              name="lucide:arrow-left"
            />
            {{ t('common.actions.back') }}
          </UiButton>
          <div v-else/>

          <div class="flex gap-2">
            <UiButton
              variant="outline"
              @click="isDialogOpen = false"
            >
              {{ t('common.actions.cancel') }}
            </UiButton>

            <UiButton
              v-if="currentStep === 'preview'"
              :disabled="isProcessing"
              @click="handleConfirmTemplate"
            >
              <Icon
                v-if="isProcessing"
                class="mr-2 h-4 w-4 animate-spin"
                name="lucide:loader-2"
              />
              {{ t('common.actions.continue') }}
            </UiButton>

            <UiButton
              v-else-if="currentStep === 'customize'"
              :disabled="!customizations.name.trim() || isProcessing"
              @click="handleCreateView"
            >
              <Icon
                v-if="isProcessing"
                class="mr-2 h-4 w-4 animate-spin"
                name="lucide:loader-2"
              />
              {{ t('components.viewWizard.actions.createView') }}
            </UiButton>
          </div>
        </div>
      </UiDialogFooter>
    </UiDialogContent>
  </UiDialog>
</template>
