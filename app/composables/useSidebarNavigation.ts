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

export type SidebarSectionItem = {
  id: string
  title?: string
  type: 'section'
  children?: SidebarNavigationItem[]
}

export type SidebarNavigationItem = SidebarViewItem | SidebarFolderItem
export type SideBarNavigation = Array<SidebarSectionItem>

export const useSidebarNavigation = () => {
  const isCreateViewWizardOpen = inject('isCreateViewWizardOpen')
  const { accounts } = useAccounts()
  const { folders, mapFolderTree } = useFolders()
  const { views } = useViews()
  const { t } = useI18n()

  const sections = computed(() => {
    const viewItems = views.value.map(view => ({
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
      click: () => { isCreateViewWizardOpen.value = true },
    })

    const result: SideBarNavigation = [
      {
        id: 'views',
        title: 'Views',
        children: viewItems,
        type: 'section'
      }
    ]

    return [...result, ...mapFolderTree(folders.value, accounts.value)]
  })

  return { sections }
}