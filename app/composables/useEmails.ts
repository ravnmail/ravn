import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import type { EmailDetail, EmailListItem } from '~/types/email'

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

let emailListenersRegistered = false
const emailUnlistenFns: Array<() => void> = []

export function useEmails() {
  const isLoading = useState('emailsLoading', () => false)
  const error = useState<string | null>('emailsError', () => null)

  onMounted(async () => {
    if (emailListenersRegistered) return
    emailListenersRegistered = true

    const unlistenEmailUpdated = await listen<EmailUpdatedEvent>('email:updated', () => {
      // Event handled by components/composables that track email state
    })

    const unlistenEmailDeleted = await listen<EmailDeletedEvent>('email:deleted', () => {
      // Event handled by components/composables that track email state
    })

    emailUnlistenFns.push(unlistenEmailUpdated, unlistenEmailDeleted)
  })

  onUnmounted(() => {
    emailUnlistenFns.forEach(unlisten => unlisten())
  })

  const fetch = async (id: string): Promise<EmailDetail | null> => {
    isLoading.value = true
    error.value = null

    try {
      return await invoke<EmailDetail>('get_emails', { id })
    }
    catch (err) {
      const errorMessage = err instanceof Error ? err.message : String(err)
      error.value = errorMessage
      console.error('Failed to fetch email:', errorMessage)
      return null
    }
    finally {
      isLoading.value = false
    }
  }

  const fetchForFolder = async (
    folderId: string,
    limit?: number,
    offset?: number,
  ): Promise<EmailListItem[]> => {
    isLoading.value = true
    error.value = null

    try {
      return await invoke<EmailListItem[]>('get_emails_for_folders', {
        folderId,
        limit: limit || 50,
        offset: offset || 0,
      })
    }
    catch (err) {
      const errorMessage = err instanceof Error ? err.message : String(err)
      error.value = errorMessage
      console.error('Failed to fetch emails for folder:', errorMessage)
      return []
    }
    finally {
      isLoading.value = false
    }
  }

  const fetchForLabels = async (
    labelIds: string[],
    matchAll: boolean = false,
    limit?: number,
    offset?: number,
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
    }
    catch (err) {
      const errorMessage = err instanceof Error ? err.message : String(err)
      error.value = errorMessage
      console.error('Failed to fetch emails by labels:', errorMessage)
      return []
    }
    finally {
      isLoading.value = false
    }
  }

  const updateRead = async (emailId: string, isRead: boolean): Promise<void> => {
    error.value = null

    try {
      await invoke('update_read', { emailId, isRead })
    }
    catch (err) {
      const errorMessage = err instanceof Error ? err.message : String(err)
      error.value = errorMessage
      console.error('Failed to update read status:', errorMessage)
      throw new Error(errorMessage)
    }
  }

  const move = async (emailId: string, folderId: string): Promise<void> => {
    error.value = null

    try {
      await invoke('move_email', { emailId, folderId })
    }
    catch (err) {
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
    }
    catch (err) {
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
    }
    catch (err) {
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
    }
    catch (err) {
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
    }
    catch (err) {
      const errorMessage = err instanceof Error ? err.message : String(err)
      error.value = errorMessage
      console.error('Failed to delete email:', errorMessage)
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
    try {
      await invoke('add_label_to_email', { request })
    } catch (error) {
      console.error('Failed to add label to email:', error)
      throw error
    }
  }

  const removeLabelFromEmail = async (emailId: string, labelId: string): Promise<void> => {
    try {
      await invoke('remove_label_from_email', { emailId, labelId })
    } catch (error) {
      console.error('Failed to remove label from email:', error)
      throw error
    }
  }

  return {
    isLoading: readonly(isLoading),
    error: readonly(error),
    fetch,
    fetchForFolder,
    fetchForLabels,
    updateRead,
    move,
    archive,
    junk,
    trash,
    deleteEmail,
    updateImageBlocking,
    addLabelToEmail,
    removeLabelFromEmail,
    allowImages,
    allowAll,
  }
}
