import { invoke } from '@tauri-apps/api/core'
import { open, save } from '@tauri-apps/plugin-dialog'
import { toast } from 'vue-sonner'

import type { AttachmentData } from '~/composables/useAccountEmail'
import { getFileIconForMimeType } from '~/lib/utils/fileIcons'
import type { Attachment } from '~/types/email'

interface AttachmentInfo {
  id: string
  email_id: string
  filename: string
  content_type: string
  size: number
  is_inline: boolean
  is_cached: boolean
  full_path?: string
}

type DialogResult = string | string[] | null

export function useAttachments() {
  const attachments = ref<Attachment[]>([])
  const isLoading = ref(false)
  const error = ref<string | null>(null)

  const getAttachmentPath = (attachment: Attachment) => {
    if (!attachment.full_path) {
      const message = `Attachment "${attachment.filename}" is not available locally yet`
      console.error(message)
      toast.error(message)
      return null
    }

    return attachment.full_path
  }

  const normalizeDialogPath = (result: DialogResult) => {
    if (!result) {
      return null
    }

    if (Array.isArray(result)) {
      return result[0] ?? null
    }

    return result
  }

  const joinPath = (basePath: string, fileName: string) => {
    const normalizedBase = basePath.replace(/[\\/]$/, '')
    return `${normalizedBase}/${fileName}`
  }

  const sanitizeFilename = (filename: string | null | undefined) => {
    const trimmed = (filename || 'attachment').trim()
    return trimmed.length > 0 ? trimmed : 'attachment'
  }

  const notifySaveSuccess = (destinationPath: string) => {
    toast.success('Attachment saved', {
      description: destinationPath,
    })
  }

  const notifyMultiSaveSuccess = (count: number, destinationPath: string) => {
    toast.success(`Saved ${count} attachment${count === 1 ? '' : 's'}`, {
      description: destinationPath,
    })
  }

  const notifySaveError = (err: unknown, filename: string) => {
    const message = err instanceof Error ? err.message : String(err)
    console.error(`Failed to save attachment "${filename}":`, err)
    toast.error(`Failed to save ${filename}`, {
      description: message,
    })
  }

  const loadAttachments = async (emailId: string) => {
    isLoading.value = true
    error.value = null

    try {
      const result = await invoke<Attachment[]>('get_email_attachments', {
        emailId: emailId.toString(),
      })
      attachments.value = result
    } catch (err: any) {
      error.value = err?.message || 'Failed to load attachments'
      console.error('Failed to load attachments:', err)
    } finally {
      isLoading.value = false
    }
  }

  const openAttachment = async (attachment: Attachment) => {
    const filePath = getAttachmentPath(attachment)
    if (!filePath) {
      return
    }

    try {
      await invoke('open_attachment', {
        filePath,
      })
    } catch (err: any) {
      console.error('Failed to open attachment:', err)
      toast.error(`Failed to open ${attachment.filename}`, {
        description: err?.message || String(err),
      })
    }
  }

  const quicklookAttachments = async (attachmentList: Attachment[]) => {
    const filePaths = attachmentList
      .map(getAttachmentPath)
      .filter((path): path is string => Boolean(path))

    if (filePaths.length === 0) {
      console.error('No cached attachments to preview')
      toast.error('No attachments available to preview')
      return
    }

    try {
      await invoke('quicklook_attachment', {
        filePaths,
      })
    } catch (err: any) {
      console.error('Failed to Quick Look attachments:', err)
      toast.error('Failed to preview attachments', {
        description: err?.message || String(err),
      })
    }
  }

  const saveAttachmentToPath = async (attachment: Attachment, destinationPath: string) => {
    const sourcePath = getAttachmentPath(attachment)
    if (!sourcePath) {
      return false
    }

    try {
      await invoke('save_attachment', {
        sourcePath,
        destinationPath,
      })

      notifySaveSuccess(destinationPath)
      return true
    } catch (err) {
      notifySaveError(err, attachment.filename)
      return false
    }
  }

  const saveToDownloads = async (attachment: Attachment) => {
    const sourcePath = getAttachmentPath(attachment)
    if (!sourcePath) {
      return false
    }

    try {
      const downloadsPath = await invoke<string>('get_downloads_path')
      const filename = sanitizeFilename(attachment.filename)
      const destinationPath = joinPath(downloadsPath, filename)

      await invoke('save_attachment', {
        sourcePath,
        destinationPath,
      })

      notifySaveSuccess(destinationPath)
      return true
    } catch (err) {
      notifySaveError(err, attachment.filename)
      return false
    }
  }

  const pickSaveDirectory = async () => {
    const result = await open({
      title: 'Choose Folder',
      directory: true,
      multiple: false,
    })

    return normalizeDialogPath(result)
  }

  const saveMultipleToDirectory = async (attachmentList: Attachment[], directoryPath?: string) => {
    const destinationDirectory = directoryPath ?? (await pickSaveDirectory())
    if (!destinationDirectory) {
      return false
    }

    let savedCount = 0

    for (const attachment of attachmentList) {
      const filename = sanitizeFilename(attachment.filename)
      const destinationPath = joinPath(destinationDirectory, filename)
      const saved = await saveAttachmentToPath(attachment, destinationPath)

      if (saved) {
        savedCount += 1
      }
    }

    if (savedCount > 1) {
      notifyMultiSaveSuccess(savedCount, destinationDirectory)
    }

    return savedCount > 0
  }

  const saveToCustomLocation = async (attachment: Attachment) => {
    const sourcePath = getAttachmentPath(attachment)
    if (!sourcePath) {
      return false
    }

    try {
      const result = await save({
        defaultPath: sanitizeFilename(attachment.filename),
        title: 'Save Attachment',
      })

      const destinationPath = normalizeDialogPath(result)
      if (!destinationPath) {
        return false
      }

      await invoke('save_attachment', {
        sourcePath,
        destinationPath,
      })

      notifySaveSuccess(destinationPath)
      return true
    } catch (err) {
      notifySaveError(err, attachment.filename)
      return false
    }
  }

  const formatFileSize = (bytes: number): string => {
    if (bytes === 0) return '0 B'
    const k = 1024
    const sizes = ['B', 'KB', 'MB', 'GB']
    const i = Math.floor(Math.log(bytes) / Math.log(k))
    return `${Number.parseFloat((bytes / k ** i).toFixed(1))} ${sizes[i]}`
  }

  const getFileIcon = (contentType: string, filename: string): string => {
    return getFileIconForMimeType(contentType, filename)
  }

  const loadAttachmentsForForward = async (emailId: string): Promise<AttachmentData[]> => {
    try {
      const attachmentList = await invoke<AttachmentInfo[]>('get_email_attachments', { emailId })

      const attachmentDataPromises = attachmentList.map(async (att) => {
        try {
          const data = await invoke<AttachmentData>('read_attachment_for_forward', {
            attachmentId: att.id,
          })
          return data
        } catch (err) {
          console.warn(`Failed to load attachment ${att.filename}:`, err)
          return null
        }
      })

      const loadedAttachments = (await Promise.all(attachmentDataPromises)).filter(
        (att): att is AttachmentData => att !== null
      )

      return loadedAttachments
    } catch (err) {
      console.error('Failed to load forwarded attachments:', err)
      return []
    }
  }

  return {
    attachments,
    isLoading,
    error,
    loadAttachments,
    openAttachment,
    quicklookAttachments,
    saveAttachmentToPath,
    saveToDownloads,
    saveToCustomLocation,
    saveMultipleToDirectory,
    pickSaveDirectory,
    formatFileSize,
    getFileIcon,
    loadAttachmentsForForward,
  }
}
