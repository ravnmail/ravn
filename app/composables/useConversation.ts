import { useInfiniteQuery, useQuery } from '@tanstack/vue-query'
import { invoke } from '@tauri-apps/api/core'

import type { ConversationDetail, ConversationListItem } from '~/types/conversation'

const PAGE_SIZE = 50

const QUERY_KEYS = {
  all: ['conversations'] as const,
  lists: () => [...QUERY_KEYS.all, 'list'] as const,
  listByFolder: (
    folderId: string,
    sortBy?: string,
    sortOrder?: string,
    filterRead?: boolean | null,
    filterHasAttachments?: boolean | null
  ) =>
    [
      ...QUERY_KEYS.lists(),
      { folderId, sortBy, sortOrder, filterRead, filterHasAttachments },
    ] as const,
  listByLabel: (
    labelId: string,
    sortBy?: string,
    sortOrder?: string,
    filterRead?: boolean | null,
    filterHasAttachments?: boolean | null
  ) =>
    [
      ...QUERY_KEYS.lists(),
      { labelId, sortBy, sortOrder, filterRead, filterHasAttachments },
    ] as const,
  details: () => [...QUERY_KEYS.all, 'detail'] as const,
  detail: (id: string) => [...QUERY_KEYS.details(), id] as const,
  detailByMessage: (messageId: string) => [...QUERY_KEYS.details(), 'message', messageId] as const,
}

export type ConversationFolderFilters = {
  sortBy?: MaybeRef<string>
  sortOrder?: MaybeRef<string>
  filterRead?: MaybeRef<boolean | null>
  filterHasAttachments?: MaybeRef<boolean | null>
}

export type ConversationLabelFilters = ConversationFolderFilters

export function useConversation() {
  const useGetConversation = (conversationId: MaybeRef<string>) => {
    const resolvedConversationId = computed(() => unref(conversationId))

    return useQuery({
      queryKey: computed(() => QUERY_KEYS.detail(resolvedConversationId.value)),
      queryFn: async () => {
        return await invoke<ConversationDetail>('get_conversation_by_id', {
          conversationId: resolvedConversationId.value,
        })
      },
      enabled: computed(() => {
        return !!resolvedConversationId.value
      }),
    })
  }

  /**
   * Infinite query for folder conversations.
   * Each page fetches PAGE_SIZE conversations using backend sort/filter.
   * The `pageParam` is the offset (number of conversations already loaded).
   */
  const useGetConversationsForFolderInfinite = (
    folderId: MaybeRef<string>,
    filters: ConversationFolderFilters = {}
  ) => {
    const resolvedFolderId = computed(() => unref(folderId))
    const resolvedSortBy = computed(() => unref(filters.sortBy) ?? 'received_at')
    const resolvedSortOrder = computed(() => unref(filters.sortOrder) ?? 'desc')
    const resolvedFilterRead = computed(() => unref(filters.filterRead) ?? null)
    const resolvedFilterHasAttachments = computed(() => unref(filters.filterHasAttachments) ?? null)

    return useInfiniteQuery({
      queryKey: computed(() =>
        QUERY_KEYS.listByFolder(
          resolvedFolderId.value,
          resolvedSortBy.value,
          resolvedSortOrder.value,
          resolvedFilterRead.value,
          resolvedFilterHasAttachments.value
        )
      ),
      queryFn: async ({ pageParam = 0 }) => {
        const items = await invoke<ConversationListItem[]>('get_conversations_for_folder', {
          folderId: resolvedFolderId.value,
          limit: PAGE_SIZE,
          offset: pageParam as number,
          sortBy: resolvedSortBy.value,
          sortOrder: resolvedSortOrder.value,
          filterRead: resolvedFilterRead.value,
          filterHasAttachments: resolvedFilterHasAttachments.value,
        })
        return { items, nextOffset: (pageParam as number) + items.length }
      },
      initialPageParam: 0,
      getNextPageParam: (lastPage) => {
        // Stop fetching when a page returns fewer items than PAGE_SIZE
        if (lastPage.items.length < PAGE_SIZE) return undefined
        return lastPage.nextOffset
      },
      enabled: computed(() => !!resolvedFolderId.value),
    })
  }

  const useGetConversationForMessage = (messageId: MaybeRef<string>) => {
    const resolvedMessageId = computed(() => unref(messageId))

    return useQuery({
      queryKey: computed(() => QUERY_KEYS.detailByMessage(resolvedMessageId.value)),
      queryFn: async () => {
        return await invoke<ConversationDetail>('get_conversation_for_message_id', {
          messageId: resolvedMessageId.value,
        })
      },
      enabled: computed(() => {
        return !!resolvedMessageId.value
      }),
    })
  }

  /**
   * Infinite query for label conversations.
   * Each page fetches PAGE_SIZE conversations using backend sort/filter.
   * The `pageParam` is the offset (number of conversations already loaded).
   */
  const useGetConversationsForLabelInfinite = (
    labelId: MaybeRef<string>,
    filters: ConversationLabelFilters = {}
  ) => {
    const resolvedLabelId = computed(() => unref(labelId))
    const resolvedSortBy = computed(() => unref(filters.sortBy) ?? 'received_at')
    const resolvedSortOrder = computed(() => unref(filters.sortOrder) ?? 'desc')
    const resolvedFilterRead = computed(() => unref(filters.filterRead) ?? null)
    const resolvedFilterHasAttachments = computed(() => unref(filters.filterHasAttachments) ?? null)

    return useInfiniteQuery({
      queryKey: computed(() =>
        QUERY_KEYS.listByLabel(
          resolvedLabelId.value,
          resolvedSortBy.value,
          resolvedSortOrder.value,
          resolvedFilterRead.value,
          resolvedFilterHasAttachments.value
        )
      ),
      queryFn: async ({ pageParam = 0 }) => {
        const items = await invoke<ConversationListItem[]>('get_conversations_for_label', {
          labelId: resolvedLabelId.value,
          limit: PAGE_SIZE,
          offset: pageParam as number,
          sortBy: resolvedSortBy.value,
          sortOrder: resolvedSortOrder.value,
          filterRead: resolvedFilterRead.value,
          filterHasAttachments: resolvedFilterHasAttachments.value,
        })
        return { items, nextOffset: (pageParam as number) + items.length }
      },
      initialPageParam: 0,
      getNextPageParam: (lastPage) => {
        if (lastPage.items.length < PAGE_SIZE) return undefined
        return lastPage.nextOffset
      },
      enabled: computed(() => !!resolvedLabelId.value),
    })
  }

  return {
    useGetConversation,
    useGetConversationsForFolderInfinite,
    useGetConversationsForLabelInfinite,
    useGetConversationForMessage,
  }
}
