import { invoke } from '@tauri-apps/api/core'
import { useQuery, useMutation, useQueryClient } from '@tanstack/vue-query'
import type { Account, Folder, FolderType } from '~/types/sync'
import type { SidebarFolderItem } from '~/composables/useSidebarNavigation'

const QUERY_KEYS = {
  all: ['folders'] as const,
  list: () => [...QUERY_KEYS.all, 'list'] as const,
}

export function useFolders() {
  // const queryClient = useQueryClient()

  const getFolderSortOrder = (folderType: FolderType): number => {
    const orderMap: Record<FolderType, number> = {
      inbox: 1,
      starred: 2,
      draft: 3,
      sent: 4,
      archive: 5,
      spam: 6,
      trash: 7,
      custom: 100,
    }
    return orderMap[folderType] || 999
  }

  const toNavigationFolder = (folder: Folder): SidebarFolderItem => {
    return {
      ...folder,
      href: `/mail/${folder.account_id}/folders/${folder.id}`,
      type: 'folder',
      sort_order: getFolderSortOrder(folder.folder_type),
    }
  }

  const useGetFolders = () => {
    return useQuery({
      queryKey: QUERY_KEYS.list(),
      queryFn: async () => {
        return await invoke<Folder[]>('get_folder_navigation')
      }
    })
  }

  const useInitSyncMutation = () => useMutation({
    mutationFn: async ({ folderId, full = false }: { folderId: string; full: boolean }) => {
      return await invoke('init_folder_sync', { folderId, full })
    },
  })

  const useUpdateExpandedMutation = () => useMutation({
    mutationFn: async ({ folderId, isExpanded }: { folderId: string; isExpanded: boolean }) => {
      return await invoke('update_expanded', { folderId, isExpanded })
    },
  })

  const useUpdateHiddenMutation = () => useMutation({
    mutationFn: async ({ folderId, isHidden }: { folderId: string; isHidden: boolean }) => {
      return await invoke('update_hidden', { folderId, isHidden })
    },
  })

  const useRenameMutation = () => useMutation({
    mutationFn: async ({folderId, request} : {folderId: string, request: {
      name: string
      icon?: string
      color?: string
    }}) => {
      console.log('Renaming folder', folderId, request)
      return await invoke('rename', { folderId, request })
    },
  })

  const useUpdateSettingsMutation = () => useMutation({
    mutationFn: async ({
      folderId,
      settings,
    }: {
      folderId: string
      settings: Record<string, unknown>
    }) => {
      return await invoke('update_settings', { folderId, settings })
    },
  })

  const flatten = (folders: SidebarFolderItem[], level: number = 0) => {
    let result: Array<Folder & { level: number }> = []
    folders.forEach(item => {
      result.push({ ...item, level })
      if (item.children && item.children.length > 0) {
        result = result.concat(flatten(item.children, level + 1))
      }
    })
    return result
  }

  const flattenAccountFolders = (folders: SidebarFolderItem[]) => {
    return folders.map(folder => {
      if (folder.children && folder.children.length > 0) {
        return {
          ...folder,
          children: flatten(folder.children, 0),
        }
      }
      return folder
    })
  }

  const mapFolderTree = (folders: Folder[], accounts: Account[]): SidebarSectionItem[] => {
    const accountMap: Record<string, SidebarSectionItem> = {}
    accounts?.forEach(account => {
      accountMap[account.id] = {
        id: account.id,
        title: account.name,
        type: 'section',
        children: [],
      }
    })

    if (!folders || folders.length === 0) {
      return Object.values(accountMap)
    }

    const folderMap: Record<string, SidebarFolderItem> = {}
    folders?.forEach(folder => {
      if (folder.hidden) {
        return
      }
      const navFolder = toNavigationFolder(folder)
      folderMap[folder.id] = navFolder

      if (folder.parent_id) {
        const parentFolder = folderMap[folder.parent_id]
        if (parentFolder) {
          parentFolder.children = parentFolder.children || []
          parentFolder.children.push(navFolder)
        }
      }
      else {
        const accountSection = accountMap[folder.account_id]
        if (accountSection) {
          accountSection.children = accountSection.children || []
          accountSection.children.push(navFolder)
        }
      }
    })
    const sortChildren = (items: SidebarFolderItem[]) => {
      items.sort((a, b) => {
        const orderA = a.sort_order || 999
        const orderB = b.sort_order || 999
        if (orderA !== orderB) {
          return orderA - orderB
        }
        return a.name.localeCompare(b.name)
      })

      items.forEach(item => {
        if (item.children) {
          sortChildren(item.children)
        }
      })
    }

    Object.values(accountMap).forEach(section => {
      if (section.children) {
        sortChildren(section.children)
      }
    })

    return Object.values(accountMap)
  }

  const {
    data: folders,
  } = useGetFolders()

  return {
    folders,
    useGetFolders,

    mapFolderTree,
    toNavigationFolder,
    useInitSyncMutation,
    initSync: useInitSyncMutation().mutateAsync,
    useUpdateExpandedMutation,
    updateExpanded: useUpdateExpandedMutation().mutateAsync,
    useUpdateHiddenMutation,
    updateHidden: useUpdateHiddenMutation().mutateAsync,
    useRenameMutation,
    updateFolderProperties: useRenameMutation().mutateAsync,
    flatten,
    flattenAccountFolders,
    useUpdateSettingsMutation,
  }
}
