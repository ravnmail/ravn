import type { FolderType } from '~/types/sync'

export type SidebarFolderItem = {
  id: string

  name: string
  icon?: string
  color?: string
  href: string
  type: 'folder'

  folder_type?: FolderType
  hidden?: boolean
  unread_count?: number
  total_count?: number
  synced_at?: string
  remote_id?: string
  account_id?: string
  parent_id?: string
  shortcut?: string

  sort_order?: number
  children?: SidebarFolderItem[]
}

export type SidebarViewItem = {
  id: string

  name: string
  icon?: string
  color?: string
  href?: string
  type: 'view'
  tooltip?: string
  shortcut?: string

  click?: () => void
}

export type SidebarLabelItem = {
  id: string

  name: string
  icon?: string
  color?: string
  href: string
  type: 'label'
}

export type SidebarSectionItem = {
  id: string
  title?: string
  type: 'section'
  children?: SidebarNavigationItem[]
}

export type SidebarNavigationItem = SidebarViewItem | SidebarFolderItem | SidebarLabelItem
export type SideBarNavigation = Array<SidebarSectionItem>

export const useSidebarNavigation = () => {
  const { accounts } = useAccounts()
  const { executeAction } = useActions()
  const { folders, mapFolderTree } = useFolders()
  const { views } = useViews()
  const { labels } = useLabels()
  const { settings } = useSettings()
  const { t } = useI18n()

  const showLabelsSection = computed(() => {
    return settings.value?.views?.sidebar?.showLabelsSection !== false
  })

  const sections = computed(() => {
    const viewItems = views.value.map((view) => ({
      id: view.id,
      name: view.name,
      icon: view.icon || 'grid-3x3',
      color: view.color,
      type: 'view',
      href: `/views/${view.id}`,
    })) as SidebarViewItem[]

    viewItems.push({
      id: 'new-view',
      name: t('components.viewNav.newView') as string,
      icon: 'plus',
      type: 'view',
      click: () => {
        executeAction('global:openCreateViewWizard')
      },
    })

    const result: SideBarNavigation = [
      {
        id: 'views',
        title: 'Views',
        children: viewItems,
        type: 'section',
      },
    ]

    const folderSections = mapFolderTree(folders.value, accounts.value)

    if (showLabelsSection.value) {
      const labelItems = labels.value.map((label) => ({
        id: label.id,
        name: label.name,
        icon: label.icon || 'tag',
        color: label.color,
        href: `/labels/${label.id}`,
        type: 'label' as const,
      })) as SidebarLabelItem[]

      // Sentinel item — rendered as an "Add Label" button at the bottom of the list.
      // SidebarSection handles the click directly by detecting id === 'new-label'.
      const newLabelItem: SidebarViewItem = {
        id: 'new-label',
        name: t('components.sidebarNav.newLabel') as string,
        icon: 'plus',
        type: 'view',
      }

      const labelsSection: SidebarSectionItem = {
        id: 'labels',
        title: t('components.sidebarNav.labelsSection') as string,
        type: 'section',
        children: [...labelItems, newLabelItem],
      }

      return [...result, ...folderSections, labelsSection]
    }

    return [...result, ...folderSections]
  })

  return { sections }
}
