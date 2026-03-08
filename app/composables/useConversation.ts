import { useInfiniteQuery, useQuery } from '@tanstack/vue-query'
import { invoke } from '@tauri-apps/api/core'

import type { ConversationDetail, ConversationListItem } from '~/types/conversation'

const PAGE_SIZE = 50

const QUERY_KEYS = {
  all: ['conversations'] as const,
  lists: () => [...QUERY_KEYS.all, 'list'] as const,
  listByScope: (
    scope: ConversationScopeQuery,
    sortBy?: string,
    sortOrder?: string,
    filterRead?: boolean | null,
    filterHasAttachments?: boolean | null
  ) =>
    [
      ...QUERY_KEYS.lists(),
      {
        scope,
        sortBy,
        sortOrder,
        filterRead,
        filterHasAttachments,
      },
    ] as const,
  details: () => [...QUERY_KEYS.all, 'detail'] as const,
  detail: (id: string) => [...QUERY_KEYS.details(), id] as const,
  detailByMessage: (messageId: string) => [...QUERY_KEYS.details(), 'message', messageId] as const,
}

export type ConversationFilters = {
  sortBy?: MaybeRef<string>
  sortOrder?: MaybeRef<string>
  filterRead?: MaybeRef<boolean | null>
  filterHasAttachments?: MaybeRef<boolean | null>
}

export type ConversationFolderFilters = ConversationFilters
export type ConversationLabelFilters = ConversationFilters

export type ScopedFilterOperator = 'and' | 'or'

export type ScopedFilterCondition =
  | {
      type: 'folder'
      folderIds?: string[]
      operator?: ScopedFilterOperator
      negated?: boolean
    }
  | {
      type: 'label'
      labelIds?: string[]
      operator?: ScopedFilterOperator
      negated?: boolean
      matchAllLabels?: boolean
    }

export interface ScopedFilterGroup {
  id?: string
  operator?: ScopedFilterOperator
  negated?: boolean
  filters?: ScopedFilterCondition[]
}

export type ConversationScopeQuery =
  | { type: 'folder'; folderId: string }
  | { type: 'label'; labelId: string }
  | {
      type: 'combined'
      folderIds?: string[]
      labelIds?: string[]
      matchAllLabels?: boolean
      filters?: ScopedFilterCondition[]
      filterGroups?: ScopedFilterGroup[]
      rootOperator?: ScopedFilterOperator
    }

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

  const useGetConversationsInfinite = (
    scope: MaybeRef<ConversationScopeQuery | null | undefined>,
    filters: ConversationFilters = {}
  ) => {
    const resolvedScope = computed(() => unref(scope) ?? null)
    const resolvedSortBy = computed(() => unref(filters.sortBy) ?? 'received_at')
    const resolvedSortOrder = computed(() => unref(filters.sortOrder) ?? 'desc')
    const resolvedFilterRead = computed(() => unref(filters.filterRead) ?? null)
    const resolvedFilterHasAttachments = computed(() => unref(filters.filterHasAttachments) ?? null)

    return useInfiniteQuery({
      queryKey: computed(() =>
        QUERY_KEYS.listByScope(
          resolvedScope.value ?? { type: 'combined', folderIds: [], labelIds: [] },
          resolvedSortBy.value,
          resolvedSortOrder.value,
          resolvedFilterRead.value,
          resolvedFilterHasAttachments.value
        )
      ),
      queryFn: async ({ pageParam = 0 }) => {
        const currentScope = resolvedScope.value

        if (!currentScope) {
          return { items: [], nextOffset: pageParam as number }
        }

        let items: ConversationListItem[]

        switch (currentScope.type) {
          case 'folder':
            items = await invoke<ConversationListItem[]>('get_conversations_for_folder', {
              folderId: currentScope.folderId,
              limit: PAGE_SIZE,
              offset: pageParam as number,
              sortBy: resolvedSortBy.value,
              sortOrder: resolvedSortOrder.value,
              filterRead: resolvedFilterRead.value,
              filterHasAttachments: resolvedFilterHasAttachments.value,
            })
            break
          case 'label':
            items = await invoke<ConversationListItem[]>('get_conversations_for_label', {
              labelId: currentScope.labelId,
              limit: PAGE_SIZE,
              offset: pageParam as number,
              sortBy: resolvedSortBy.value,
              sortOrder: resolvedSortOrder.value,
              filterRead: resolvedFilterRead.value,
              filterHasAttachments: resolvedFilterHasAttachments.value,
            })
            break
          case 'combined':
            items = await invoke<ConversationListItem[]>('get_conversations_for_scope', {
              folderIds: currentScope.folderIds ?? [],
              labelIds: currentScope.labelIds ?? [],
              matchAllLabels: currentScope.matchAllLabels ?? false,
              filters: currentScope.filters ?? [],
              filterGroups: currentScope.filterGroups ?? [],
              rootOperator: currentScope.rootOperator ?? 'and',
              limit: PAGE_SIZE,
              offset: pageParam as number,
              sortBy: resolvedSortBy.value,
              sortOrder: resolvedSortOrder.value,
              filterRead: resolvedFilterRead.value,
              filterHasAttachments: resolvedFilterHasAttachments.value,
            })
            break
        }

        return { items, nextOffset: (pageParam as number) + items.length }
      },
      initialPageParam: 0,
      getNextPageParam: (lastPage) => {
        if (lastPage.items.length < PAGE_SIZE) return undefined
        return lastPage.nextOffset
      },
      enabled: computed(() => {
        const currentScope = resolvedScope.value
        if (!currentScope) return false

        switch (currentScope.type) {
          case 'folder':
            return !!currentScope.folderId
          case 'label':
            return !!currentScope.labelId
          case 'combined':
            return (
              (currentScope.folderIds?.length ?? 0) > 0 ||
              (currentScope.labelIds?.length ?? 0) > 0 ||
              (currentScope.filters?.length ?? 0) > 0 ||
              (currentScope.filterGroups?.length ?? 0) > 0
            )
        }
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

    return useGetConversationsInfinite(
      computed(() =>
        resolvedFolderId.value ? { type: 'folder', folderId: resolvedFolderId.value } : null
      ),
      filters
    )
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

    return useGetConversationsInfinite(
      computed(() =>
        resolvedLabelId.value ? { type: 'label', labelId: resolvedLabelId.value } : null
      ),
      filters
    )
  }

  const useGetConversationsForCombinedScopeInfinite = (
    scope: MaybeRef<{
      folderIds?: string[]
      labelIds?: string[]
      matchAllLabels?: boolean
      filters?: ScopedFilterCondition[]
      filterGroups?: ScopedFilterGroup[]
      rootOperator?: ScopedFilterOperator
    }>,
    filters: ConversationFilters = {}
  ) => {
    const resolvedScope = computed(() => unref(scope))

    return useGetConversationsInfinite(
      computed(() => ({
        type: 'combined' as const,
        folderIds: resolvedScope.value?.folderIds ?? [],
        labelIds: resolvedScope.value?.labelIds ?? [],
        matchAllLabels: resolvedScope.value?.matchAllLabels ?? false,
        filters: resolvedScope.value?.filters ?? [],
        filterGroups: resolvedScope.value?.filterGroups ?? [],
        rootOperator: resolvedScope.value?.rootOperator ?? 'and',
      })),
      filters
    )
  }

  return {
    useGetConversation,
    useGetConversationsInfinite,
    useGetConversationsForFolderInfinite,
    useGetConversationsForLabelInfinite,
    useGetConversationsForCombinedScopeInfinite,
    useGetConversationForMessage,
  }
}
