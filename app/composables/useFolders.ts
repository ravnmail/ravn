import { invoke } from '@tauri-apps/api/core'
import { useQuery, useMutation, useQueryClient } from '@tanstack/vue-query'
import type { Folder, FolderType } from '~/types/sync'

const QUERY_KEYS = {
  all: ['folders'] as const,
  lists: () => [...QUERY_KEYS.all, 'list'] as const,
  list: (accountId: string) => [...QUERY_KEYS.lists(), accountId] as const,
}

export function useFolders() {
  const queryClient = useQueryClient()

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

  const toNavigationFolder = (folder: Folder): Folder => {
    return {
      ...folder,
      sort_order: getFolderSortOrder(folder.folder_type),
    }
  }

  const useGetFolders = (accountId: string | Ref<string>) => {
    return useQuery({
      queryKey: QUERY_KEYS.list(computed(() =>
        typeof accountId === 'string' ? accountId : accountId.value
      ).value),
      queryFn: async () => {
        const id = typeof accountId === 'string' ? accountId : accountId.value
        return await invoke<Folder[]>('get_folders', { accountId: id })
      },
      enabled: computed(() => {
        const id = typeof accountId === 'string' ? accountId : accountId.value
        return !!id
      }),
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

  const useNavigationFolders = (accountId: string) => {
    const { data: folders } = useGetFolders(accountId)

    return computed(() => {
      const folderList = folders.value || []
      const folderMap = new Map<string, Folder>()
      const rootFolders: Folder[] = []

      folderList.forEach((folder) => {
        folderMap.set(folder.id, toNavigationFolder(folder))
      })

      folderMap.forEach((folder) => {
        if (folder.parent_id) {
          const parent = folderMap.get(folder.parent_id)
          if (parent) {
            if (!parent.children) {
              parent.children = []
            }
            parent.children.push(folder)
          } else {
            rootFolders.push(folder)
          }
        } else {
          rootFolders.push(folder)
        }
      })

      const sortFolders = (folderList: Folder[]) => {
        folderList.sort((a, b) => {
          const orderA = a.sort_order || 999
          const orderB = b.sort_order || 999
          if (orderA !== orderB) {
            return orderA - orderB
          }
          return a.name.localeCompare(b.name)
        })

        folderList.forEach((folder) => {
          if (folder.children?.length) {
            sortFolders(folder.children)
          }
        })
      }

      sortFolders(rootFolders)
      return rootFolders
    })
  }

  const flatten = (folders: Folder[], level: number = 0) => {
    let result: Array<Folder & { level: number }> = []
    folders.forEach(item => {
      result.push({ ...item, level })
      if (item.children && item.children.length > 0) {
        result = result.concat(flatten(item.children, level + 1))
      }
    })
    return result
  }

  return {
    useGetFolders,
    useNavigationFolders,
    useInitSyncMutation,
    initSync: useInitSyncMutation().mutateAsync,
    useUpdateExpandedMutation,
    updateExpanded: useUpdateExpandedMutation().mutateAsync,
    useUpdateHiddenMutation,
    updateHidden: useUpdateHiddenMutation().mutateAsync,
    useRenameMutation,
    updateFolderProperties: useRenameMutation().mutateAsync,
    flatten,
    useUpdateSettingsMutation,
  }
}
