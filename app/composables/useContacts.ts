import { invoke } from '@tauri-apps/api/core'
import { useMutation, useQuery, useQueryClient } from '@tanstack/vue-query'
import type {
  Contact,
  ContactSummary,
  GetContactsRequest,
  GetTopContactsRequest,
} from '~/types/contact'

const QUERY_KEYS = {
  all: ['contacts'] as const,
  lists: () => [...QUERY_KEYS.all, 'list'] as const,
  list: (accountId?: string) => [...QUERY_KEYS.lists(), { accountId }] as const,
  search: () => [...QUERY_KEYS.all, 'search'] as const,
  searchResults: (query?: string) => [...QUERY_KEYS.search(), { query }] as const,
  top: () => [...QUERY_KEYS.all, 'top'] as const,
  topList: (accountId?: string) => [...QUERY_KEYS.top(), { accountId }] as const,
  details: () => [...QUERY_KEYS.all, 'detail'] as const,
  detail: (id: string) => [...QUERY_KEYS.details(), id] as const,
  byEmail: (email: string) => [...QUERY_KEYS.all, 'email', email] as const,
}

export function useContacts() {
  const queryClient = useQueryClient()

  const {
    data: contacts,
    isLoading,
    error,
    refetch: refetchContacts,
  } = useQuery({
    queryKey: QUERY_KEYS.lists(),
    queryFn: async () => {
      return await invoke<Contact[]>('get_contacts', {})
    },
    enabled: false, // Manual control
  })

  const useGetContacts = (request: GetContactsRequest) => {
    return useQuery({
      queryKey: QUERY_KEYS.list(request.account_id),
      queryFn: async () => {
        return await invoke<Contact[]>('get_contacts', { request })
      },
    })
  }

  const useGetTopContacts = (request: GetTopContactsRequest) => {
    return useQuery({
      queryKey: QUERY_KEYS.topList(request.account_id),
      queryFn: async () => {
        return await invoke<ContactSummary[]>('get_top_contacts', { request })
      },
    })
  }

  const useSearchContacts = (q: MaybeRef<string>, options: { account_id: string, limit: number }) => {
    const params = computed(() => ({
      ...options,
      query: unref(q),
    }))

    return useQuery({
      queryKey: computed(() => QUERY_KEYS.searchResults(params.value.query)),
      queryFn: async () => {
        console.log('Searching contacts with request:', params.value)
        return await invoke<ContactSummary[]>('search_contacts', {
          request: params.value
        })
      },
    })
  }

  const useGetContactById = (contactId: string | Ref<string>) => {
    return useQuery({
      queryKey: QUERY_KEYS.detail(computed(() =>
        typeof contactId === 'string' ? contactId : contactId.value
      ).value),
      queryFn: async () => {
        const id = typeof contactId === 'string' ? contactId : contactId.value
        return await invoke<Contact | null>('get_contact_by_id', { contactId: id })
      },
      enabled: computed(() => {
        const id = typeof contactId === 'string' ? contactId : contactId.value
        return !!id
      }),
    })
  }

  const useGetContactByEmail = (email: string) => {
    return useQuery({
      queryKey: QUERY_KEYS.byEmail(email),
      queryFn: async () => {
        return await invoke<Contact | null>('get_contact_by_email', {
          email,
        })
      },
      enabled: computed(() => !!email),
    })
  }

  const resetContactCountersMutation = useMutation({
    mutationFn: async (accountId: string) => {
      await invoke('resync_contact_counters', { request: { account_id: accountId } })
    },
    onSuccess: (_data, accountId) => {
      queryClient.invalidateQueries({ queryKey: QUERY_KEYS.list(accountId) })
    },
  })

  const createContactMutation = useMutation({
    mutationFn: async (contact: Contact) => {
      return await invoke<string>('create_contact', { contact })
    },
    onMutate: async (newContact) => {
      await queryClient.cancelQueries({ queryKey: QUERY_KEYS.list(newContact.account_id) })
      const previousContacts = queryClient.getQueryData<Contact[]>(QUERY_KEYS.list(newContact.account_id))

      const optimisticContact: Contact = {
        ...newContact,
        id: `temp-${Date.now()}`,
      }

      queryClient.setQueryData(QUERY_KEYS.list(newContact.account_id), (old: Contact[] | undefined) => {
        return [...(old || []), optimisticContact]
      })

      return { previousContacts }
    },
    onSuccess: (_contactId, newContact) => {
      queryClient.invalidateQueries({ queryKey: QUERY_KEYS.list(newContact.account_id) })
    },
    onError: (_error, newContact, context) => {
      if (context?.previousContacts) {
        queryClient.setQueryData(QUERY_KEYS.list(newContact.account_id), context.previousContacts)
      }
    },
  })

  const useUpdateContactMutation = () => useMutation({
    mutationFn: async (contact: Contact) => {
      await invoke('update_contact', { contact })
      return contact
    },
    onMutate: async (updatedContact) => {
      await queryClient.cancelQueries({ queryKey: QUERY_KEYS.list(updatedContact.account_id) })
      await queryClient.cancelQueries({ queryKey: QUERY_KEYS.detail(updatedContact.id) })

      const previousContacts = queryClient.getQueryData<Contact[]>(QUERY_KEYS.list(updatedContact.account_id))
      const previousDetail = queryClient.getQueryData<Contact>(QUERY_KEYS.detail(updatedContact.id))

      queryClient.setQueryData(QUERY_KEYS.list(updatedContact.account_id), (old: Contact[] | undefined) => {
        return (old || []).map(c => c.id === updatedContact.id ? updatedContact : c)
      })

      queryClient.setQueryData(QUERY_KEYS.detail(updatedContact.id), updatedContact)

      return { previousContacts, previousDetail }
    },
    onSuccess: (updatedContact) => {
      queryClient.setQueryData(QUERY_KEYS.detail(updatedContact.id), updatedContact)
    },
    onError: (_error, updatedContact, context) => {
      if (context?.previousContacts) {
        queryClient.setQueryData(QUERY_KEYS.list(updatedContact.account_id), context.previousContacts)
      }
      if (context?.previousDetail) {
        queryClient.setQueryData(QUERY_KEYS.detail(updatedContact.id), context.previousDetail)
      }
    },
  })

  const useDeleteContactMutation = () => useMutation({
    mutationFn: async (contactId: string) => {
      await invoke('delete_contact', { contactId })
    },
    onMutate: async (contactId) => {
      await queryClient.cancelQueries({ queryKey: QUERY_KEYS.lists() })
      await queryClient.cancelQueries({ queryKey: QUERY_KEYS.detail(contactId) })

      const previousContacts = queryClient.getQueryData<Contact[]>(QUERY_KEYS.lists())
      const previousDetail = queryClient.getQueryData<Contact>(QUERY_KEYS.detail(contactId))

      queryClient.setQueryData(QUERY_KEYS.lists(), (old: Contact[] | undefined) => {
        return (old || []).filter(c => c.id !== contactId)
      })

      return { previousContacts, previousDetail }
    },
    onError: (_error, contactId, context) => {
      if (context?.previousContacts) {
        queryClient.setQueryData(QUERY_KEYS.lists(), context.previousContacts)
      }
      if (context?.previousDetail) {
        queryClient.setQueryData(QUERY_KEYS.detail(contactId), context.previousDetail)
      }
    },
  })

  return {
    // State
    contacts: computed(() => contacts.value || []),
    isLoading: computed(() => isLoading.value),
    error: computed(() => error.value),

    // Query hooks
    useGetContacts,
    useGetTopContacts,
    useSearchContacts,
    useGetContactById,
    useGetContactByEmail,

    // Mutations
    resetContactCounters: resetContactCountersMutation.mutateAsync,
    resetContactCountersMutation,
    createContact: createContactMutation.mutateAsync,
    createContactMutation,
    useUpdateContactMutation,
    useDeleteContactMutation,

    // Utilities
    refetchContacts,
  }
}
