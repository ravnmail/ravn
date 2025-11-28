import { invoke } from '@tauri-apps/api/core'
import { useQuery, useMutation, useQueryClient } from '@tanstack/vue-query'
import type { Label, CreateLabelRequest, UpdateLabelRequest } from '~/types/view'

const QUERY_KEYS = {
  all: ['labels'] as const,
  lists: () => [...QUERY_KEYS.all, 'list'] as const,
  list: (filters?: string) => [...QUERY_KEYS.lists(), { filters }] as const,
  details: () => [...QUERY_KEYS.all, 'detail'] as const,
  detail: (id: string) => [...QUERY_KEYS.details(), id] as const,
}

export const useLabels = () => {
  const queryClient = useQueryClient()

  const {
    data: labels,
    isLoading,
    error,
    refetch: refetchLabels,
  } = useQuery({
    queryKey: QUERY_KEYS.lists(),
    queryFn: async () => {
      const result = await invoke<Label[]>('get_labels')
      return result.sort((a, b) => a.name.localeCompare(b.name))
    },
  })

  const useGetLabel = (labelId: string | Ref<string>) => {
    return useQuery({
      queryKey: QUERY_KEYS.detail(computed(() =>
        typeof labelId === 'string' ? labelId : labelId.value
      ).value),
      queryFn: async () => {
        const id = typeof labelId === 'string' ? labelId : labelId.value
        return await invoke<Label | null>('get_label', { labelId: id })
      },
      enabled: computed(() => {
        const id = typeof labelId === 'string' ? labelId : labelId.value
        return !!id
      }),
    })
  }

  const createLabelMutation = useMutation({
    mutationFn: async (request: CreateLabelRequest) => {
      return await invoke<Label>('create_label', { request })
    },
    onMutate: async (newLabel) => {
      await queryClient.cancelQueries({ queryKey: QUERY_KEYS.lists() })
      const previousLabels = queryClient.getQueryData<Label[]>(QUERY_KEYS.lists())
      const optimisticLabel: Label = {
        id: `temp-${Date.now()}`,
        ...newLabel,
      } as Label

      queryClient.setQueryData(QUERY_KEYS.lists(), (old: Label[] | undefined) => {
        const updated = [...(old || []), optimisticLabel]
        return updated.sort((a, b) => a.name.localeCompare(b.name))
      })

      return { previousLabels }
    },
    onSuccess: (newLabel) => {
      queryClient.setQueryData(QUERY_KEYS.lists(), (old: Label[] | undefined) => {
        const updated = (old || []).filter(l => !l.id.startsWith('temp-'))
        updated.push(newLabel)
        return updated.sort((a, b) => a.name.localeCompare(b.name))
      })
    },
    onError: (_error, _variables, context) => {
      if (context?.previousLabels) {
        queryClient.setQueryData(QUERY_KEYS.lists(), context.previousLabels)
      }
    },
  })

  const useUpdateLabelMutation = () => useMutation({
    mutationFn: async (request: UpdateLabelRequest) => {
      return await invoke<Label>('update_label', { request })
    },
    onMutate: async (updates) => {
      await queryClient.cancelQueries({ queryKey: QUERY_KEYS.lists() })
      await queryClient.cancelQueries({ queryKey: QUERY_KEYS.detail(updates.id) })

      const previousLabels = queryClient.getQueryData<Label[]>(QUERY_KEYS.lists())
      const previousLabel = queryClient.getQueryData<Label>(QUERY_KEYS.detail(updates.id))

      queryClient.setQueryData(QUERY_KEYS.lists(), (old: Label[] | undefined) => {
        return (old || []).map(l =>
          l.id === updates.id ? { ...l, ...updates } : l
        ).sort((a, b) => a.name.localeCompare(b.name))
      })

      queryClient.setQueryData(QUERY_KEYS.detail(updates.id), (old: Label | undefined) => {
        return old ? { ...old, ...updates } : undefined
      })

      return { previousLabels, previousLabel }
    },
    onSuccess: (updatedLabel, variables) => {
      queryClient.setQueryData(QUERY_KEYS.lists(), (old: Label[] | undefined) => {
        return (old || []).map(l =>
          l.id === variables.id ? updatedLabel : l
        ).sort((a, b) => a.name.localeCompare(b.name))
      })

      queryClient.setQueryData(QUERY_KEYS.detail(variables.id), updatedLabel)
    },
    onError: (_error, variables, context) => {
      if (context?.previousLabels) {
        queryClient.setQueryData(QUERY_KEYS.lists(), context.previousLabels)
      }
      if (context?.previousLabel) {
        queryClient.setQueryData(QUERY_KEYS.detail(variables.id), context.previousLabel)
      }
    },
  })

  const useDeleteLabelMutation = () => useMutation({
    mutationFn: async (labelId: string) => {
      await invoke('delete_label', { labelId })
    },
    onMutate: async (labelId) => {
      await queryClient.cancelQueries({ queryKey: QUERY_KEYS.lists() })
      await queryClient.cancelQueries({ queryKey: QUERY_KEYS.detail(labelId) })

      const previousLabels = queryClient.getQueryData<Label[]>(QUERY_KEYS.lists())
      const previousLabel = queryClient.getQueryData<Label>(QUERY_KEYS.detail(labelId))

      queryClient.setQueryData(QUERY_KEYS.lists(), (old: Label[] | undefined) => {
        return (old || []).filter(l => l.id !== labelId)
      })

      return { previousLabels, previousLabel }
    },
    onError: (_error, labelId, context) => {
      if (context?.previousLabels) {
        queryClient.setQueryData(QUERY_KEYS.lists(), context.previousLabels)
      }
      if (context?.previousLabel) {
        queryClient.setQueryData(QUERY_KEYS.detail(labelId), context.previousLabel)
      }
    },
  })

  return {
    labels: computed(() => labels.value || []),
    isLoading: computed(() => isLoading.value),
    error: computed(() => error.value),
    refetchLabels,
    useGetLabel,

    createLabel: createLabelMutation.mutateAsync,
    createLabelMutation,
    useUpdateLabelMutation,
    useDeleteLabelMutation,
  }
}