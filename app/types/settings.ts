// Settings Types
// These correspond to the settings in src-tauri/resources/settings.json5

import type { CleanTranslation } from 'nuxt-i18n-micro-types'

export interface AIModelSettings {
  fast: string
  normal: string
}

export interface AISettings {
  models: AIModelSettings
}

// Signature Interface
export interface Signature {
  id: string
  title: string
  content: string
  defaultForAccounts: string[]
}

export interface SignaturesSettings {
  items: Signature[]
  globalDefault: string | null
}

// Keyboard Bindings
export interface KeyboardBindings {
  [action: string]: string[]
}

export interface KeyboardSettings {
  enabled: boolean
  bindings: KeyboardBindings
}

export type ReminderPresetType = 'laterToday' | 'tomorrow' | 'nextWeek' | 'nextMonth' | 'custom'

export type ReminderPresetOffsetUnit = 'minute' | 'hour' | 'day' | 'week' | 'month'

export interface ReminderPresetOffset {
  value: number
  unit: ReminderPresetOffsetUnit
}

export interface ReminderPresetSetting {
  id: string
  label: string
  type: ReminderPresetType
  icon?: string | null
  time?: string | null
  offset?: ReminderPresetOffset | null
}

export interface EmailReminderSettings {
  presets: ReminderPresetSetting[]
}

export interface EmailSettings {
  renderMode: 'simple' | 'normal'
  reminders: EmailReminderSettings
}

export interface NotificationSettings {
  enabled: boolean
  incomingSound: string | null
  outgoingSound: string | null
  reminderSound: string | null
  notificationFolders: string[]
  badgeType: 'count' | 'dot' | null
  badgeFolders: string[]
}

export interface KanbanViewSettings {
  showLabelsSection: boolean
}

export interface SidebarSettings {
  showLabelsSection: boolean
}

export interface ViewsSettings {
  kanban: KanbanViewSettings
  sidebar: SidebarSettings
}

export interface RegionalSettings {
  /** dayjs format token for dates, e.g. "MMM D, YYYY" | "DD.MM.YYYY" | "MM/DD/YYYY" | "YYYY-MM-DD" */
  dateFormat: string
  /** dayjs format token for times, e.g. "HH:mm" (24-hour) | "h:mm A" (12-hour) */
  timeFormat: string
  /** dayjs format token for weekday column headers, e.g. "ddd" (Mon) | "dddd" (Monday) | "dd" (Mo) */
  weekdayFormat: string
  /** First day of week: 0 = Sunday, 1 = Monday */
  startOfWeek: 0 | 1
}

// Root settings interface
export interface Settings {
  ai: AISettings
  signatures: SignaturesSettings
  keyboard: KeyboardSettings
  email: EmailSettings
  notifications: NotificationSettings
  views: ViewsSettings
  regional: RegionalSettings
}

// Navigation item for settings sidebar
export interface SettingsNavItem {
  title: CleanTranslation | string
  name: string
  icon: string
  badge?: string | number
  disabled?: boolean
}

// For partial updates
export type PartialSettings = Partial<Settings>
export type PartialDeep<T> = {
  [P in keyof T]?: T[P] extends object ? PartialDeep<T[P]> : T[P]
}
