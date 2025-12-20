export type ViewType = 'kanban' | 'calendar' | 'list' | 'smart' | 'unified'

export type SwimlaneState = 'open' | 'closed'

export interface KanbanSwimlane {
  id: string
  title: string
  color?: string
  label_ids: string[]
  folder_ids?: string[]
  state: SwimlaneState
  sort_order: number
}

export type KanbanViewConfig = {
  type: 'kanban'
  swimlanes: KanbanSwimlane[]
}

export type ViewConfig =
  | KanbanViewConfig
  | { type: 'calendar' }
  | { type: 'list' }
  | { type: 'smart' }
  | { type: 'unified' }

export interface View {
  id: string
  name: string
  icon?: string
  color?: string
  view_type: ViewType
  config: ViewConfig
  folders: string[]
  sort_order: number
  is_default: boolean
  created_at: string
  updated_at: string
}

export interface Label {
  id: string
  name: string
  color?: string
  icon?: string
  created_at: string
  updated_at: string
}

export interface CreateLabelRequest {
  name: string
  color?: string
  icon?: string
}

export interface UpdateLabelRequest extends CreateLabelRequest {
  id: string
}

export interface CreateViewRequest {
  name: string
  icon?: string | null
  color?: string | null
  view_type: ViewType
  config?: ViewConfig
  folders?: string[]
  sort_order?: number
}

export interface UpdateViewRequest extends CreateViewRequest {
  id: string
}