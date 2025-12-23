import { useFilter } from 'reka-ui'
import { settingsManifest } from '~/config/settings-manifest'
import type {
  SettingsManifest,
  SettingGroup,
  SettingSection,
  SettingItem,
  SettingsTreeNode
} from '~/types/settings-manifest'

const { contains } = useFilter({ sensitivity: 'base' })

const filter = ref<string|null>(null)
export function useSettingsManifest() {
  const { t } = useI18n()
  const manifest = computed<SettingsManifest>(() => settingsManifest.map(group => ({
    ...group,
    name: t(group.name),
    sections: group.sections.map(section => ({
      ...section,
      name: t(section.name),
      items: section.items.map(item => ({
        ...item,
        name: t(item.name),
        description: item.description ? t(item.description) : undefined
      }))
    }))
  })))

  const filteredManifest = computed<SettingsManifest>(() => {
    if (!filter.value) {
      return manifest.value
    }

    const filteredGroups: SettingGroup[] = []

    for (const group of manifest.value) {
      const filteredSections: SettingSection[] = []

      for (const section of group.sections) {
        const filteredItems: SettingItem[] = section.items.filter(item =>
          contains(item.id, filter.value!) ||
          contains(item.name, filter.value!) ||
          (item.description && contains(item.description, filter.value!))
        )

        if (filteredItems.length > 0) {
          filteredSections.push({
            ...section,
            items: filteredItems
          })
        }

      }

      if (filteredSections.length > 0) {
        filteredGroups.push({
          ...group,
          sections: filteredSections
        })
      }
    }

    return filteredGroups
  })

  const filteredNavigation = computed<SettingsTreeNode[]>(() => {
    return filteredManifest.value.map(group => ({
      id: group.id,
      groupId: group.id,
      name: group.name,
      children: group.sections.map(section => ({
        id: section.id,
        groupId: group.id,
        name: section.name
      }))
    }))
  })

  function getGroup(id: string): SettingGroup | undefined {
    return filteredManifest.value.find(group => group.id === id)
  }

  function getSelection(id: string): SettingsTreeNode | undefined {
    return filteredNavigation.value.find(group => group.id === id) ?? filteredNavigation.value[0] ?? undefined
  }

  return {
    filter,
    getGroup,
    getSelection,
    filteredManifest,
    filteredNavigation,
    manifest: readonly(manifest)
  }
}
