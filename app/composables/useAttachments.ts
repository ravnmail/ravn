import { invoke } from '@tauri-apps/api/core'
import { save } from '@tauri-apps/plugin-dialog'

import type { Attachment } from '~/types/email'
import type { AttachmentData } from '~/composables/useAccountEmail'

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

export function useAttachments() {
  const attachments = ref<Attachment[]>([])
  const isLoading = ref(false)
  const error = ref<string | null>(null)

  const loadAttachments = async (emailId: string) => {
    isLoading.value = true
    error.value = null

    try {
      const result = await invoke<Attachment[]>('get_email_attachments', {
        emailId: emailId.toString(),
      })
      console.log('Loaded attachments:', result)
      attachments.value = result
    }
    catch (err: any) {
      error.value = err?.message || 'Failed to load attachments'
      console.error('Failed to load attachments:', err)
    }
    finally {
      isLoading.value = false
    }
  }

  const openAttachment = async (attachment: Attachment) => {
    if (!attachment.full_path) {
      console.error('Attachment not cached')
      return
    }

    try {
      await invoke('open_attachment', {
        filePath: attachment.full_path,
      })
    }
    catch (err: any) {
      console.error('Failed to open attachment:', err)
      alert(`Failed to open attachment: ${err?.message || err}`)
    }
  }

  const quicklookAttachments = async (attachmentList: Attachment[]) => {
    const filePaths = attachmentList
      .filter(a => a.full_path)
      .map(a => a.full_path!)

    if (filePaths.length === 0) {
      console.error('No cached attachments to preview')
      return
    }

    try {
      await invoke('quicklook_attachment', {
        filePaths,
      })
    }
    catch (err: any) {
      console.error('Failed to QuickLook attachments:', err)
      alert(`Failed to preview attachments: ${err?.message || err}`)
    }
  }

  const saveToDownloads = async (attachment: Attachment) => {
    if (!attachment.full_path) {
      console.error('Attachment not cached')
      return
    }

    try {
      const downloadsPath = await invoke<string>('get_downloads_path')
      const destinationPath = `${downloadsPath}/${attachment.filename}`

      await invoke('save_attachment', {
        sourcePath: attachment.full_path,
        destinationPath,
      })

      alert(`Saved to ${destinationPath}`)
    }
    catch (err: any) {
      console.error('Failed to save attachment:', err)
      alert(`Failed to save attachment: ${err?.message || err}`)
    }
  }

  const saveToCustomLocation = async (attachment: Attachment) => {
    if (!attachment.full_path) {
      console.error('Attachment not cached')
      return
    }

    try {
      const destinationPath = await save({
        defaultPath: attachment.filename,
        filters: [{
          name: 'All Files',
          extensions: ['*'],
        }],
      })

      if (!destinationPath) {
        return
      }

      await invoke('save_attachment', {
        sourcePath: attachment.full_path,
        destinationPath,
      })

      alert(`Saved to ${destinationPath}`)
    }
    catch (err: any) {
      console.error('Failed to save attachment:', err)
      alert(`Failed to save attachment: ${err?.message || err}`)
    }
  }

  const formatFileSize = (bytes: number): string => {
    if (bytes === 0)
      return '0 B'
    const k = 1024
    const sizes = ['B', 'KB', 'MB', 'GB']
    const i = Math.floor(Math.log(bytes) / Math.log(k))
    return `${Number.parseFloat((bytes / k ** i).toFixed(1))} ${sizes[i]}`
  }

  const getFileIcon = (contentType: string, filename: string): string => {
    if (contentType.startsWith('image/'))
      return 'lucide:image'
    if (contentType.startsWith('video/'))
      return 'lucide:video'
    if (contentType.startsWith('audio/'))
      return 'lucide:music'
    if (contentType === 'application/pdf')
      return 'lucide:file-text'
    if (contentType.includes('word') || filename.endsWith('.doc') || filename.endsWith('.docx'))
      return 'lucide:file-text'
    if (contentType.includes('excel') || contentType.includes('spreadsheet') || filename.endsWith('.xls') || filename.endsWith('.xlsx'))
      return 'lucide:file-spreadsheet'
    if (contentType.includes('powerpoint') || contentType.includes('presentation') || filename.endsWith('.ppt') || filename.endsWith('.pptx'))
      return 'lucide:presentation'
    if (contentType.includes('zip') || contentType.includes('compressed') || filename.endsWith('.zip') || filename.endsWith('.rar'))
      return 'lucide:file-archive'
    if (contentType.includes('text/') || filename.endsWith('.txt'))
      return 'lucide:file-text'

    return 'lucide:file'
  }

  const loadAttachmentsForForward = async (emailId: string): Promise<AttachmentData[]> => {
    try {
      // Get attachment list
      const attachmentList = await invoke<AttachmentInfo[]>('get_email_attachments', { emailId })

      // Load each attachment's content
      const attachmentDataPromises = attachmentList.map(async (att) => {
        try {
          const data = await invoke<AttachmentData>('read_attachment_for_forward', {
            attachmentId: att.id
          })
          return data
        } catch (err) {
          console.warn(`Failed to load attachment ${att.filename}:`, err)
          return null
        }
      })

      const loadedAttachments = (await Promise.all(attachmentDataPromises))
        .filter((att): att is AttachmentData => att !== null)

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
    saveToDownloads,
    saveToCustomLocation,
    formatFileSize,
    getFileIcon,
    loadAttachmentsForForward,
  }
}
