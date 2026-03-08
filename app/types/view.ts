export type ViewType = 'kanban' | 'calendar' | 'list' | 'smart' | 'unified'

export type CalendarDateField = 'received_at' | 'sent_at' | 'remind_at'

export type CalendarMode = 'month' | 'week'

export type SwimlaneState = 'open' | 'closed'

export type ListFilterOperator = 'and' | 'or'

export type ListFilterSourceType = 'folders' | 'labels'

export interface ListFilterRule {
  id: string
  source: ListFilterSourceType
  values: string[]
  operator?: ListFilterOperator
  negated?: boolean
}

export interface ListFilterGroup {
  id: string
  operator: ListFilterOperator
  negated?: boolean
  rules: ListFilterRule[]
}

export interface ListViewFilters {
  groups: ListFilterGroup[]
}

export interface KanbanSwimlane {
  id: string
  title: string
  icon?: string
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

export type CalendarViewConfig = {
  type: 'calendar'
  date_field: CalendarDateField
  folder_ids: string[]
  mode: CalendarMode
}

export type ListViewConfig = {
  type: 'list'
  filters: ListViewFilters
}

export type ViewConfig =
  | KanbanViewConfig
  | CalendarViewConfig
  | ListViewConfig
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
