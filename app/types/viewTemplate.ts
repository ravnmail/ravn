import type { ViewType } from './view'

export interface TemplateLabelDefinition {
  id: string // temporary ID to be replaced with UUID
  name: string
  color: string
  icon: string
}

export interface TemplateSwimlaneDefinition {
  name: string
  description: string
  icon: string
  color: string
  labels: string[] // references to template label IDs
}

export interface ViewTemplate {
  id: string
  title: string
  description: string
  viewType: ViewType
  labels: TemplateLabelDefinition[]
  swimlanes: TemplateSwimlaneDefinition[]
}

export interface ProcessedTemplate {
  title: string
  description: string
  viewType: ViewType
  labels: Array<{ tempId: string; realId: string; name: string; color: string; icon: string }>
  swimlanes: Array<{
    name: string
    description: string
    icon: string
    color: string
    labelIds: string[] // real UUIDs
  }>
}
