export interface SettingItem {
  id: string
  name: string
  description: string
  is: string
  props?: Record<string, any>
  disabled?: boolean
  validate?: (value: any) => boolean | string
}

export interface SettingSection {
  id: string
  name: string
  items: SettingItem[]
}

export interface SettingGroup {
  id: string
  name: string
  sections: SettingSection[]
}

export interface SettingsTreeNode {
  id: string
  name: string
  groupId: string
  children?: SettingsTreeNode[]
}

export type SettingsManifest = SettingGroup[]