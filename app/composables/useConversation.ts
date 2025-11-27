import { invoke } from '@tauri-apps/api/core'
import { useQuery, useQueryClient } from '@tanstack/vue-query'
import type { ConversationDetail, ConversationListItem } from '~/types/conversation'
import { useSetupQueryListeners } from './useQueryListeners'

const QUERY_KEYS = {
  all: ['conversations'] as const,
  lists: () => [...QUERY_KEYS.all, 'list'] as const,
  listByFolder: (folderId: string, limit?: number, offset?: number) =>
    [...QUERY_KEYS.lists(), { folderId, limit, offset }] as const,
  details: () => [...QUERY_KEYS.all, 'detail'] as const,
  detail: (id: string) => [...QUERY_KEYS.details(), id] as const,
  detailByMessage: (messageId: string) => [...QUERY_KEYS.details(), 'message', messageId] as const,
}

export function useConversation() {
  const queryClient = useQueryClient()

  useSetupQueryListeners(queryClient, [
    { name: 'conversation:created', invalidateKey: QUERY_KEYS.lists() },
    { name: 'conversation:updated', invalidateKey: QUERY_KEYS.lists() },
    { name: 'conversation:deleted', invalidateKey: QUERY_KEYS.lists() },
    { name: 'folder:updated', invalidateKey: QUERY_KEYS.lists() },
  ])

  const useGetConversation = (conversationId: MaybeRef<string>) => {
    const resolvedConversationId = computed(() => unref(conversationId))

    return useQuery({
      queryKey: computed(() => QUERY_KEYS.detail(resolvedConversationId.value)),
      queryFn: async () => {
        return await invoke<ConversationDetail>('get_conversation_by_id', { conversationId: resolvedConversationId.value })
      },
      enabled: computed(() => {
        return !!resolvedConversationId.value
      }),
    })
  }

  const useGetConversationsForFolder = (
    folderId: MaybeRef<string>,
    limit: MaybeRef<number> = 50,
    offset: MaybeRef<number> = 0,
  ) => {
    const resolvedFolderId = computed(() => unref(folderId))
    const resolvedLimit = computed(() => unref(limit))
    const resolvedOffset = computed(() => unref(offset))

    return useQuery({
      queryKey: computed(() =>
        QUERY_KEYS.listByFolder(resolvedFolderId.value, resolvedLimit.value, resolvedOffset.value)
      ),
      queryFn: async () => {
        return await invoke<ConversationListItem[]>('get_conversations_for_folder', {
          folderId: resolvedFolderId.value,
          limit: resolvedLimit.value,
          offset: resolvedOffset.value,
        })
      },
      enabled: computed(() => !!resolvedFolderId.value),
    })
  }

  const useGetConversationForMessage = (messageId: MaybeRef<string>) => {
    const resolvedMessageId = computed(() => unref(messageId))

    return useQuery({
      queryKey: computed(() => QUERY_KEYS.detailByMessage(resolvedMessageId.value)),
      queryFn: async () => {
        return await invoke<ConversationDetail>('get_conversation_for_message_id', { messageId: resolvedMessageId.value })
      },
      enabled: computed(() => {
        return !!resolvedMessageId.value
      }),
    })
  }

  return {
    useGetConversation,
    useGetConversationsForFolder,
    useGetConversationForMessage,
  }
}
