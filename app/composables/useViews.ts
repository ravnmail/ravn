import { invoke } from '@tauri-apps/api/core'
import { useQuery, useMutation, useQueryClient } from '@tanstack/vue-query'
import type {
  View,
  CreateViewRequest,
  UpdateViewRequest,
} from '~/types/view'

const QUERY_KEYS = {
  all: ['views'] as const,
  lists: () => [...QUERY_KEYS.all, 'list'] as const,
  details: () => [...QUERY_KEYS.all, 'detail'] as const,
  detail: (id: string) => [...QUERY_KEYS.details(), id] as const,
}

export const useViews = () => {
  const queryClient = useQueryClient()

  // Note: Event listeners for 'view:created', 'view:updated', 'view:deleted'
  // are now registered globally in useGlobalEventListeners

  const {
    data: views,
    isLoading,
    refetch: refetchViews,
  } = useQuery({
    queryKey: QUERY_KEYS.lists(),
    queryFn: async () => {
      const result = await invoke<View[]>('get_views')
      return result.sort((a, b) => a.sort_order - b.sort_order)
    },
  })

  const useGetView = (viewId: string | Ref<string>) => {
    return useQuery({
      queryKey: QUERY_KEYS.detail(computed(() =>
        typeof viewId === 'string' ? viewId : viewId.value
      ).value),
      queryFn: async () => {
        const id = typeof viewId === 'string' ? viewId : viewId.value
        return await invoke<View | null>('get_view', { viewId: id })
      },
      enabled: computed(() => {
        const id = typeof viewId === 'string' ? viewId : viewId.value
        return !!id
      }),
    })
  }

  const createViewMutation = useMutation({
    mutationFn: async (request: CreateViewRequest) => {
      return await invoke<View>('create_view', { request })
    },
    onMutate: async (request) => {
      await queryClient.cancelQueries({ queryKey: QUERY_KEYS.lists() })
      const previousViews = queryClient.getQueryData<View[]>(QUERY_KEYS.lists())

      const optimisticView: View = {
        id: `temp-${Date.now()}`,
        ...request,
        is_default: false,
        sort_order: (previousViews?.length || 0) + 1,
        swimlanes: [],
        created_at: new Date().toISOString(),
        updated_at: new Date().toISOString(),
      } as View

      queryClient.setQueryData(QUERY_KEYS.lists(), (old: View[] | undefined) => {
        const updated = [...(old || []), optimisticView]
        return updated.sort((a, b) => a.sort_order - b.sort_order)
      })

      return { previousViews }
    },
    onSuccess: (newView) => {
      queryClient.setQueryData(QUERY_KEYS.lists(), (old: View[] | undefined) => {
        const updated = (old || []).filter(v => !v.id.startsWith('temp-'))
        updated.push(newView)
        return updated.sort((a, b) => a.sort_order - b.sort_order)
      })
    },
    onError: (_error, _variables, context) => {
      if (context?.previousViews) {
        queryClient.setQueryData(QUERY_KEYS.lists(), context.previousViews)
      }
    },
  })

  const useUpdateViewMutation = () => useMutation({
    mutationFn: async (request: UpdateViewRequest) => {
      return await invoke<View>('update_view', { request })
    },
    onMutate: async (request) => {
      await queryClient.cancelQueries({ queryKey: QUERY_KEYS.lists() })
      const previousViews = queryClient.getQueryData<View[]>(QUERY_KEYS.lists())

      queryClient.setQueryData(QUERY_KEYS.lists(), (old: View[] | undefined) => {
        return (old || []).map(v =>
          v.id === request.id
            ? { ...v, ...request, updated_at: new Date().toISOString() }
            : v
        ).sort((a, b) => a.sort_order - b.sort_order)
      })

      return { previousViews }
    },
    onSuccess: (updatedView, variables) => {
      queryClient.setQueryData(QUERY_KEYS.lists(), (old: View[] | undefined) => {
        return (old || []).map(v =>
          v.id === variables.id ? updatedView : v
        ).sort((a, b) => a.sort_order - b.sort_order)
      })
    },
    onError: (_error, _variables, context) => {
      if (context?.previousViews) {
        queryClient.setQueryData(QUERY_KEYS.lists(), context.previousViews)
      }
    },
  })

  const useDeleteViewMutation = () => useMutation({
    mutationFn: async (viewId: string) => {
      await invoke('delete_view', { viewId })
    },
    onMutate: async (viewId) => {
      await queryClient.cancelQueries({ queryKey: QUERY_KEYS.lists() })
      const previousViews = queryClient.getQueryData<View[]>(QUERY_KEYS.lists())

      queryClient.setQueryData(QUERY_KEYS.lists(), (old: View[] | undefined) => {
        return (old || []).filter(v => v.id !== viewId)
      })

      return { previousViews }
    },
    onError: (_error, _viewId, context) => {
      if (context?.previousViews) {
        queryClient.setQueryData(QUERY_KEYS.lists(), context.previousViews)
      }
    },
  })

  return {
    views: computed(() => views.value || []),
    isLoading: computed(() => isLoading.value),
    refetchViews,
    useGetView,
    createView: createViewMutation.mutateAsync,
    createViewMutation,
    useUpdateViewMutation,
    useDeleteViewMutation,
  }
}
