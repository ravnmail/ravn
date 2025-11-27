import type { ViewType, CreateViewRequest, KanbanSwimlane } from '~/types/view'
import type { ViewTemplate, ProcessedTemplate } from '~/types/viewTemplate'
import { VIEW_TEMPLATES } from '~/data/viewTemplates'

export type WizardStep = 'type' | 'template' | 'preview' | 'customize'

export const useViewWizard = () => {
  const { createLabel } = useLabels()
  const { createView } = useViews(ref(null))

  const currentStep = useState<WizardStep>('wizardStep', () => 'type')
  const selectedViewType = useState<ViewType | null>('wizardViewType', () => null)
  const selectedTemplate = useState<ViewTemplate | null>('wizardTemplate', () => null)
  const processedTemplate = useState<ProcessedTemplate | null>('wizardProcessedTemplate', () => null)
  const isProcessing = useState('wizardProcessing', () => false)

  const availableTemplates = computed(() => {
    if (!selectedViewType.value) return []
    return VIEW_TEMPLATES.filter(t => t.viewType === selectedViewType.value)
  })

  const reset = () => {
    currentStep.value = 'type'
    selectedViewType.value = null
    selectedTemplate.value = null
    processedTemplate.value = null
    isProcessing.value = false
  }

  const selectViewType = (viewType: ViewType) => {
    selectedViewType.value = viewType
    currentStep.value = 'template'
  }

  const selectTemplate = (template: ViewTemplate | null) => {
    selectedTemplate.value = template
    if (template) {
      currentStep.value = 'preview'
    } else {
      // Skip to customize with blank template
      processedTemplate.value = {
        title: 'New View',
        description: '',
        viewType: selectedViewType.value!,
        labels: [],
        swimlanes: [],
      }
      currentStep.value = 'customize'
    }
  }

  const processTemplate = async (template: ViewTemplate): Promise<ProcessedTemplate> => {
    const labelIdMap = new Map<string, string>()
    const createdLabels = []

    // Create labels and build ID mapping
    for (const labelDef of template.labels) {
      const realId = crypto.randomUUID()
      labelIdMap.set(labelDef.id, realId)

      createdLabels.push({
        tempId: labelDef.id,
        realId,
        name: labelDef.name,
        color: labelDef.color,
        icon: labelDef.icon,
      })
    }

    // Process swimlanes with replaced label IDs
    const processedSwimlanes = template.swimlanes.map(swimlane => ({
      name: swimlane.name,
      description: swimlane.description,
      icon: swimlane.icon,
      color: swimlane.color,
      labelIds: swimlane.labels.map(tempId => labelIdMap.get(tempId) || tempId),
    }))

    return {
      title: template.title,
      description: template.description,
      viewType: template.viewType,
      labels: createdLabels,
      swimlanes: processedSwimlanes,
    }
  }

  const confirmTemplate = async () => {
    if (!selectedTemplate.value) return

    isProcessing.value = true
    try {
      const processed = await processTemplate(selectedTemplate.value)
      processedTemplate.value = processed
      currentStep.value = 'customize'
    } catch (error) {
      console.error('Failed to process template:', error)
      throw error
    } finally {
      isProcessing.value = false
    }
  }

  const createViewFromTemplate = async (
    customizations: {
      name?: string
      icon?: string
      color?: string
      folders?: string[]
    } = {}
  ) => {
    if (!processedTemplate.value) {
      throw new Error('No processed template available')
    }

    isProcessing.value = true
    try {
      const template = processedTemplate.value

      // Create labels first
      const createdLabelIds = new Map<string, string>()
      for (const label of template.labels) {
        const createdLabel = await createLabel({
          name: label.name,
          color: label.color,
          icon: label.icon,
        })
        createdLabelIds.set(label.realId, createdLabel.id)
      }

      // Build swimlanes with actual created label IDs
      const swimlanes: KanbanSwimlane[] = template.swimlanes.map((swimlane, index) => ({
        id: crypto.randomUUID(),
        title: swimlane.name,
        color: swimlane.color,
        label_ids: swimlane.labelIds.map(id => createdLabelIds.get(id) || id),
        state: 'open' as const,
        sort_order: index,
      }))

      // Create view with processed config
      const viewRequest: CreateViewRequest = {
        name: customizations.name || template.title,
        icon: customizations.icon,
        color: customizations.color,
        view_type: template.viewType,
        config: {
          type: 'kanban',
          swimlanes,
        },
        folders: customizations.folders || [],
      }

      const view = await createView(viewRequest)
      reset()
      return view
    } catch (error) {
      console.error('Failed to create view from template:', error)
      throw error
    } finally {
      isProcessing.value = false
    }
  }

  const goBack = () => {
    switch (currentStep.value) {
      case 'template':
        currentStep.value = 'type'
        selectedViewType.value = null
        break
      case 'preview':
        currentStep.value = 'template'
        selectedTemplate.value = null
        break
      case 'customize':
        if (selectedTemplate.value) {
          currentStep.value = 'preview'
          processedTemplate.value = null
        } else {
          currentStep.value = 'template'
        }
        break
    }
  }

  return {
    currentStep: readonly(currentStep),
    selectedViewType: readonly(selectedViewType),
    selectedTemplate: readonly(selectedTemplate),
    processedTemplate: readonly(processedTemplate),
    availableTemplates,
    isProcessing: readonly(isProcessing),
    reset,
    selectViewType,
    selectTemplate,
    confirmTemplate,
    createViewFromTemplate,
    goBack,
  }
}
