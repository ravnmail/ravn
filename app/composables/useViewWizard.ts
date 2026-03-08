import { VIEW_TEMPLATES } from '~/data/viewTemplates'
import type {
  CalendarDateField,
  CalendarViewConfig,
  CreateViewRequest,
  KanbanSwimlane,
  ListFilterGroup,
  ListViewConfig,
  ViewType,
} from '~/types/view'
import type { ProcessedTemplate, ViewTemplate } from '~/types/viewTemplate'

export type WizardStep = 'type' | 'template' | 'customize'

export function useViewWizard() {
  const { createLabel } = useLabels()
  const { createView } = useViews()

  const currentStep = useState<WizardStep>('wizardStep', () => 'type')
  const selectedViewType = useState<ViewType | null>('wizardViewType', () => null)
  const selectedTemplate = useState<ViewTemplate | null>('wizardTemplate', () => null)
  const processedTemplate = useState<ProcessedTemplate | null>(
    'wizardProcessedTemplate',
    () => null
  )
  const isProcessing = useState('wizardProcessing', () => false)

  // Calendar-specific configuration state
  const calendarDateField = useState<CalendarDateField>(
    'wizardCalendarDateField',
    () => 'remind_at'
  )
  const calendarShowRemindAtList = useState<boolean>('wizardCalendarShowRemindAtList', () => true)
  const calendarFolderIds = useState<string[]>('wizardCalendarFolderIds', () => [])

  const createEmptyListFilterGroup = (): ListFilterGroup => ({
    id: crypto.randomUUID(),
    operator: 'and',
    rules: [],
  })

  // List-specific configuration state
  const listFilterGroups = useState<ListFilterGroup[]>('wizardListFilterGroups', () => [
    createEmptyListFilterGroup(),
  ])

  const availableTemplates = computed(() => {
    if (!selectedViewType.value) return []
    return VIEW_TEMPLATES.filter((t) => t.viewType === selectedViewType.value)
  })

  const normalizeListFilterGroups = (
    groups: ListFilterGroup[],
    { preserveEmpty = false }: { preserveEmpty?: boolean } = {}
  ): ListFilterGroup[] => {
    const normalizedGroups = (groups || []).map((group, groupIndex) => ({
      id: group.id || `group-${groupIndex}`,
      operator: group.operator || 'and',
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

  const reset = () => {
    currentStep.value = 'type'
    selectedViewType.value = null
    selectedTemplate.value = null
    processedTemplate.value = null
    isProcessing.value = false
    calendarDateField.value = 'remind_at'
    calendarShowRemindAtList.value = false
    calendarFolderIds.value = []
    listFilterGroups.value = [createEmptyListFilterGroup()]
  }

  const selectViewType = (viewType: ViewType) => {
    selectedViewType.value = viewType
    // Calendar and list views skip the template step — go straight to customize
    if (viewType === 'calendar' || viewType === 'list') {
      processedTemplate.value = {
        title: viewType === 'calendar' ? 'New Calendar' : 'New List',
        description: '',
        viewType,
        labels: [],
        swimlanes: [],
      }
      if (viewType === 'calendar') {
        calendarDateField.value = 'remind_at'
        calendarShowRemindAtList.value = true
        calendarFolderIds.value = []
      }
      if (viewType === 'list') {
        listFilterGroups.value = [createEmptyListFilterGroup()]
      }
      currentStep.value = 'customize'
    } else {
      currentStep.value = 'template'
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
    const processedSwimlanes = template.swimlanes.map((swimlane) => ({
      name: swimlane.name,
      description: swimlane.description,
      icon: swimlane.icon,
      color: swimlane.color,
      labelIds: swimlane.labels.map((tempId) => labelIdMap.get(tempId) || tempId),
    }))

    return {
      title: template.title,
      description: template.description,
      viewType: template.viewType,
      labels: createdLabels,
      swimlanes: processedSwimlanes,
    }
  }

  const selectTemplate = async (template: ViewTemplate | null) => {
    selectedTemplate.value = template
    if (template) {
      isProcessing.value = true
      try {
        const processed = await processTemplate(template)
        processedTemplate.value = processed
      } catch (error) {
        console.error('Failed to process template:', error)
        throw error
      } finally {
        isProcessing.value = false
      }
    } else {
      // Start from scratch with blank template
      processedTemplate.value = {
        title: 'New View',
        description: '',
        viewType: selectedViewType.value!,
        labels: [],
        swimlanes: [],
      }
      if (selectedViewType.value === 'list') {
        listFilterGroups.value = [createEmptyListFilterGroup()]
      }
    }
    currentStep.value = 'customize'
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

      // Calendar views have their own config structure
      if (selectedViewType.value === 'calendar') {
        const config: CalendarViewConfig = {
          type: 'calendar',
          date_field: calendarDateField.value,
          folder_ids: calendarFolderIds.value,
          mode: 'month',
          show_secondary_remind_list:
            calendarDateField.value !== 'remind_at' ? calendarShowRemindAtList.value : false,
        }

        const viewRequest: CreateViewRequest = {
          name: customizations.name || template.title,
          icon: customizations.icon,
          color: customizations.color,
          view_type: 'calendar',
          config,
          folders: calendarFolderIds.value,
        }
        const view = await createView(viewRequest)
        reset()
        return view
      }

      if (selectedViewType.value === 'list') {
        const normalizedGroups = normalizeListFilterGroups(listFilterGroups.value, {
          preserveEmpty: false,
        })

        const folderIds = Array.from(
          new Set(
            normalizedGroups.flatMap((group) =>
              group.rules.filter((rule) => rule.source === 'folders').flatMap((rule) => rule.values)
            )
          )
        )

        const config: ListViewConfig = {
          type: 'list',
          filters: {
            groups: normalizedGroups,
          },
        }

        const folders = Array.from(new Set([...folderIds, ...(customizations.folders || [])]))

        const viewRequest: CreateViewRequest = {
          name: customizations.name || template.title,
          icon: customizations.icon,
          color: customizations.color,
          view_type: 'list',
          config,
          folders,
        }
        const view = await createView(viewRequest)
        reset()
        return view
      }

      // Create labels first (kanban flow)
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
        label_ids: swimlane.labelIds.map((id) => createdLabelIds.get(id) || id),
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
      case 'customize':
        // Calendar and list skip template step, go back to type
        if (selectedViewType.value === 'calendar' || selectedViewType.value === 'list') {
          currentStep.value = 'type'
          selectedViewType.value = null
          processedTemplate.value = null
        } else {
          currentStep.value = 'template'
          selectedTemplate.value = null
          processedTemplate.value = null
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
    calendarDateField,
    calendarShowRemindAtList,
    calendarFolderIds,
    listFilterGroups,
    reset,
    selectViewType,
    selectTemplate,
    createViewFromTemplate,
    goBack,
  }
}
