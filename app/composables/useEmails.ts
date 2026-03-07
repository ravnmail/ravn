import { useQueryClient } from '@tanstack/vue-query'
import type { InfiniteData } from '@tanstack/vue-query'
import { invoke } from '@tauri-apps/api/core'

import type { EmailDetail, EmailListItem } from '~/types/email'
import type { CalendarDateField } from '~/types/view'

interface EmailUpdatedEvent {
  id: string
  is_read?: boolean
  folder_id?: string
  from_folder_id?: string
  images_blocked?: boolean
  tracking_blocked?: boolean
}

interface EmailDeletedEvent {
  id: string
}

export interface AddLabelToEmailRequest {
  email_id: string
  label_id: string
}

export function useEmails() {
  const isLoading = useState('emailsLoading', () => false)
  const error = useState<string | null>('emailsError', () => null)
  const { updateBadgeCount } = useNotifications()
  const queryClient = useQueryClient()

  type ConversationListPage = {
    items: Array<{
      id: string
      message_count: number
      ai_cache?: string
      messages: EmailListItem[]
    }>
    nextOffset: number
  }

  const updateEmailInConversationCaches = (
    emailId: string,
    updater: (email: EmailListItem) => EmailListItem
  ) => {
    queryClient.setQueriesData<InfiniteData<ConversationListPage>>(
      {
        queryKey: ['conversations', 'list'],
      },
      (oldData) => {
        if (!oldData) return oldData

        return {
          ...oldData,
          pages: oldData.pages.map((page) => ({
            ...page,
            items: page.items.map((conversation) => {
              let hasChanges = false

              const messages = conversation.messages.map((message) => {
                if (message.id !== emailId) return message
                hasChanges = true
                return updater(message)
              })

              return hasChanges
                ? {
                    ...conversation,
                    messages,
                  }
                : conversation
            }),
          })),
        }
      }
    )
  }

  const updateEmailDetailCache = (
    emailId: string,
    updater: (email: EmailDetail) => EmailDetail
  ) => {
    queryClient.setQueryData<EmailDetail>(['emails', 'detail', emailId], (oldData) => {
      if (!oldData) return oldData
      return updater(oldData)
    })
  }

  const invalidateEmailRelatedCaches = async () => {
    await Promise.all([
      queryClient.invalidateQueries({ queryKey: ['conversations', 'list'] }),
      queryClient.invalidateQueries({ queryKey: ['conversations', 'detail'] }),
      queryClient.invalidateQueries({ queryKey: ['emails'] }),
    ])
  }

  const fetch = async (id: string): Promise<EmailDetail | null> => {
    isLoading.value = true
    error.value = null

    try {
      return await invoke<EmailDetail>('get_emails', { id })
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : String(err)
      error.value = errorMessage
      console.error('Failed to fetch email:', errorMessage)
      return null
    } finally {
      isLoading.value = false
    }
  }

  const fetchForFolder = async (
    folderId: string,
    limit?: number,
    offset?: number
  ): Promise<EmailListItem[]> => {
    isLoading.value = true
    error.value = null

    try {
      return await invoke<EmailListItem[]>('get_emails_for_folders', {
        folderId,
        limit: limit || 50,
        offset: offset || 0,
      })
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : String(err)
      error.value = errorMessage
      console.error('Failed to fetch emails for folder:', errorMessage)
      return []
    } finally {
      isLoading.value = false
    }
  }

  const fetchForLabels = async (
    labelIds: string[],
    matchAll: boolean = false,
    limit?: number,
    offset?: number
  ): Promise<EmailListItem[]> => {
    isLoading.value = true
    error.value = null

    try {
      return await invoke<EmailListItem[]>('get_emails_for_labels', {
        labelIds,
        matchAll,
        limit: limit || 50,
        offset: offset || 0,
      })
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : String(err)
      error.value = errorMessage
      console.error('Failed to fetch emails by labels:', errorMessage)
      return []
    } finally {
      isLoading.value = false
    }
  }

  const updateRead = async (emailId: string, isRead: boolean): Promise<void> => {
    error.value = null

    try {
      await invoke('update_read', { emailId, isRead })
      await updateBadgeCount()
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : String(err)
      error.value = errorMessage
      console.error('Failed to update read status:', errorMessage)
      throw new Error(errorMessage)
    }
  }

  const parseBody = async (emailId: string): Promise<void> => {
    error.value = null

    try {
      await invoke('email_parse_body_plain', { emailId })
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : String(err)
      error.value = errorMessage
      console.error('Failed to parse email body:', errorMessage)
      throw new Error(errorMessage)
    }
  }

  const move = async (emailId: string, folderId: string): Promise<void> => {
    error.value = null

    try {
      await invoke('move_email', { emailId, folderId })
      await updateBadgeCount()
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : String(err)
      error.value = errorMessage
      console.error('Failed to move email:', errorMessage)
      throw new Error(errorMessage)
    }
  }

  const archive = async (emailId: string): Promise<void> => {
    error.value = null

    try {
      await invoke('archive', { emailId })
      await updateBadgeCount()
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : String(err)
      error.value = errorMessage
      console.error('Failed to archive email:', errorMessage)
      throw new Error(errorMessage)
    }
  }

  const junk = async (emailId: string): Promise<void> => {
    error.value = null

    try {
      await invoke('junk', { emailId })
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : String(err)
      error.value = errorMessage
      console.error('Failed to move email to junk:', errorMessage)
      throw new Error(errorMessage)
    }
  }

  const trash = async (emailId: string): Promise<void> => {
    error.value = null

    try {
      await invoke('trash', { emailId })
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : String(err)
      error.value = errorMessage
      console.error('Failed to move email to trash:', errorMessage)
      throw new Error(errorMessage)
    }
  }

  const deleteEmail = async (emailId: string): Promise<void> => {
    error.value = null

    try {
      await invoke('delete', { emailId })
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : String(err)
      error.value = errorMessage
      console.error('Failed to delete email:', errorMessage)
      throw new Error(errorMessage)
    }
  }

  const emptyFolder = async (folderId: string): Promise<number> => {
    error.value = null

    try {
      return await invoke<number>('empty_folder', { folderId })
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : String(err)
      error.value = errorMessage
      console.error('Failed to empty folder:', errorMessage)
      throw new Error(errorMessage)
    }
  }

  const updateImageBlocking = async (
    emailId: string,
    imagesBlocked: boolean,
    trackingBlocked: boolean
  ): Promise<boolean> => {
    try {
      await invoke('update_blocking', {
        emailId,
        imagesBlocked,
        trackingBlocked,
      })
      return true
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : String(err)
      console.error('Failed to update image blocking:', errorMessage)
      return false
    }
  }

  const allowImages = async (emailId: string): Promise<boolean> => {
    return updateImageBlocking(emailId, false, true)
  }

  const allowAll = async (emailId: string): Promise<boolean> => {
    return updateImageBlocking(emailId, false, false)
  }

  const addLabelToEmail = async (request: AddLabelToEmailRequest): Promise<void> => {
    error.value = null

    const existingLabel = queryClient
      .getQueriesData<InfiniteData<ConversationListPage>>({ queryKey: ['conversations', 'list'] })
      .flatMap(([, data]) => data?.pages ?? [])
      .flatMap((page) => page.items)
      .flatMap((conversation) => conversation.messages)
      .find((message) => message.id === request.email_id)
      ?.labels.find((label) => label.id === request.label_id)

    if (existingLabel) {
      updateEmailInConversationCaches(request.email_id, (email) => {
        if (email.labels.some((label) => label.id === request.label_id)) {
          return email
        }

        return {
          ...email,
          labels: [...email.labels, existingLabel],
        }
      })

      updateEmailDetailCache(request.email_id, (email) => {
        if (email.labels.some((label) => label.id === request.label_id)) {
          return email
        }

        return {
          ...email,
          labels: [...email.labels, existingLabel],
        }
      })
    }

    try {
      await invoke('add_label_to_email', { request })
      await invalidateEmailRelatedCaches()
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : String(err)
      error.value = errorMessage
      await invalidateEmailRelatedCaches()
      console.error('Failed to add label to email:', errorMessage)
      throw new Error(errorMessage)
    }
  }

  const removeLabelFromEmail = async (emailId: string, labelId: string): Promise<void> => {
    error.value = null

    updateEmailInConversationCaches(emailId, (email) => ({
      ...email,
      labels: email.labels.filter((label) => label.id !== labelId),
    }))

    updateEmailDetailCache(emailId, (email) => ({
      ...email,
      labels: email.labels.filter((label) => label.id !== labelId),
    }))

    try {
      await invoke('remove_label_from_email', { emailId, labelId })
      await invalidateEmailRelatedCaches()
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : String(err)
      error.value = errorMessage
      await invalidateEmailRelatedCaches()
      console.error('Failed to remove label from email:', errorMessage)
      throw new Error(errorMessage)
    }
  }

  const setRemindAt = async (emailId: string, remindAt: string | null): Promise<void> => {
    error.value = null
    try {
      await invoke('set_remind_at', { emailId, remindAt })
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : String(err)
      error.value = errorMessage
      console.error('Failed to set remind_at:', errorMessage)
      throw new Error(errorMessage)
    }
  }

  const fetchForCalendar = async (
    folderIds: string[],
    dateField: CalendarDateField,
    start: string,
    end: string
  ): Promise<EmailListItem[]> => {
    isLoading.value = true
    error.value = null
    try {
      return await invoke<EmailListItem[]>('get_emails_for_calendar', {
        folderIds,
        dateField,
        start,
        end,
      })
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : String(err)
      error.value = errorMessage
      console.error('Failed to fetch emails for calendar:', errorMessage)
      return []
    } finally {
      isLoading.value = false
    }
  }

  return {
    isLoading: readonly(isLoading),
    error: readonly(error),
    fetch,
    fetchForFolder,
    fetchForLabels,
    updateRead,
    parseBody,
    move,
    archive,
    junk,
    trash,
    deleteEmail,
    emptyFolder,
    updateImageBlocking,
    addLabelToEmail,
    removeLabelFromEmail,
    allowImages,
    allowAll,
    setRemindAt,
    fetchForCalendar,
  }
}
