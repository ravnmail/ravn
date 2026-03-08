import { convertFileSrc, invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { getCurrentWindow } from '@tauri-apps/api/window'
import {
  isPermissionGranted,
  requestPermission,
  sendNotification,
} from '@tauri-apps/plugin-notification'

import { navigateToUrl } from './useUrlNavigation'

interface BadgeCount {
  count: number
  visible?: boolean
  mode?: BadgeType | 'none'
}

interface NotificationEmailPreview {
  id?: string
  accountId?: string
  folderId?: string
  conversationId?: string | null
  senderName?: string | null
  senderAddress?: string | null
  subject?: string | null
  snippet?: string | null
  avatarUrl?: string | null
  remindAt?: string | null
  navigationTarget?: string | null
}

interface NativeNotificationPayload {
  kind?: 'incoming-email' | 'outgoing-email' | 'reminder-email' | 'system'
  title?: string
  body?: string
  email?: NotificationEmailPreview
  playSound?: boolean
  suppressDuringBootstrap?: boolean
  tag?: string
  deepLink?: string | null
}

type BadgeType = 'count' | 'dot' | null

type ReminderNotificationCandidate = {
  id: string
  account_id: string
  folder_id: string
  from: {
    address: string
    name?: string
  }
  subject?: string
  snippet?: string
  remind_at?: string
}

type ReminderCheckResult = {
  due: ReminderNotificationCandidate[]
  nextReminderAt: string | null
}

type ReminderNotificationRecord = {
  notifiedAt: string
  remindAt: string
}

const REMINDER_DEDUP_STORAGE_KEY = 'ravn.notifications.reminders.sent'
const MAX_REMINDER_RECORDS = 500
let notificationSetupPromise: Promise<void> | null = null

export const useNotifications = () => {
  const initialized = useState<boolean>('notifications-initialized', () => false)
  const badgeCount = useState<number>('notifications-badge-count', () => 0)
  const audioContext = useState<AudioContext | null>('notifications-audio-context', () => null)
  const settings = useState<any | null>('settings', () => null)
  const bootstrapSyncInProgress = useState<boolean>('notifications-bootstrap-sync', () => false)
  const reminderCheckInFlight = useState<boolean>(
    'notifications-reminder-check-in-flight',
    () => false
  )
  const audioCache = new Map<string, AudioBuffer>()

  const notificationSettings = computed(() => settings.value?.notifications)
  const badgeType = computed<BadgeType>(() => notificationSettings.value?.badgeType ?? 'count')

  const isClient = typeof window !== 'undefined'
  const isProductionBuild = import.meta.env.PROD
  const supportsImageNotifications = computed(() => {
    if (!isClient) return false
    const platform = navigator.platform?.toLowerCase?.() || ''
    return platform.includes('mac') || platform.includes('win')
  })

  const initAudioContext = () => {
    if (!audioContext.value) {
      audioContext.value = new AudioContext()
    }
  }

  const normalizeText = (value?: string | null) => value?.replace(/\s+/g, ' ').trim() || undefined

  const truncate = (value?: string | null, max = 160) => {
    const normalized = normalizeText(value)
    if (!normalized) return undefined
    if (normalized.length <= max) return normalized
    return `${normalized.slice(0, max - 1).trimEnd()}…`
  }

  const buildSenderLabel = (email?: NotificationEmailPreview | null) =>
    normalizeText(email?.senderName) || normalizeText(email?.senderAddress) || 'Unknown sender'

  const buildSubjectLabel = (email?: NotificationEmailPreview | null) =>
    normalizeText(email?.subject) || '(no subject)'

  const buildSnippetLabel = (email?: NotificationEmailPreview | null) =>
    truncate(email?.snippet, 180)

  const buildReminderStorageKey = (emailId: string, remindAt: string) => `${emailId}:${remindAt}`

  const readReminderRecords = (): Record<string, ReminderNotificationRecord> => {
    if (!isClient) return {}

    try {
      const raw = window.localStorage.getItem(REMINDER_DEDUP_STORAGE_KEY)
      if (!raw) return {}
      const parsed = JSON.parse(raw)
      if (!parsed || typeof parsed !== 'object') return {}
      return parsed
    } catch (error) {
      console.error('Failed to read reminder notification records:', error)
      return {}
    }
  }

  const writeReminderRecords = (records: Record<string, ReminderNotificationRecord>) => {
    if (!isClient) return

    try {
      const entries = Object.entries(records)
        .sort((a, b) => {
          const aTs = new Date(a[1]?.notifiedAt || 0).getTime()
          const bTs = new Date(b[1]?.notifiedAt || 0).getTime()
          return bTs - aTs
        })
        .slice(0, MAX_REMINDER_RECORDS)

      window.localStorage.setItem(
        REMINDER_DEDUP_STORAGE_KEY,
        JSON.stringify(Object.fromEntries(entries))
      )
    } catch (error) {
      console.error('Failed to persist reminder notification records:', error)
    }
  }

  const pruneReminderRecords = () => {
    const now = Date.now()
    const maxAgeMs = 1000 * 60 * 60 * 24 * 14
    const existing = readReminderRecords()
    const next: Record<string, ReminderNotificationRecord> = {}

    for (const [key, value] of Object.entries(existing)) {
      const remindAtTs = new Date(value.remindAt).getTime()
      const notifiedAtTs = new Date(value.notifiedAt).getTime()

      if (Number.isFinite(remindAtTs) && remindAtTs >= now - maxAgeMs) {
        next[key] = value
        continue
      }

      if (Number.isFinite(notifiedAtTs) && notifiedAtTs >= now - maxAgeMs) {
        next[key] = value
      }
    }

    writeReminderRecords(next)
  }

  const hasReminderNotificationBeenSent = (emailId?: string, remindAt?: string | null) => {
    if (!emailId || !remindAt) return false
    const records = readReminderRecords()
    return !!records[buildReminderStorageKey(emailId, remindAt)]
  }

  const markReminderNotificationSent = (emailId?: string, remindAt?: string | null) => {
    if (!emailId || !remindAt) return

    const records = readReminderRecords()
    records[buildReminderStorageKey(emailId, remindAt)] = {
      remindAt,
      notifiedAt: new Date().toISOString(),
    }
    writeReminderRecords(records)
  }

  const loadSound = async (soundName: string): Promise<AudioBuffer | null> => {
    try {
      if (audioCache.has(soundName)) {
        return audioCache.get(soundName)!
      }

      const soundPath = `/sounds/${soundName}.mp3`
      const response = await fetch(soundPath)

      if (!response.ok) {
        console.warn(`Sound file not found: ${soundPath}`)
        return null
      }

      const arrayBuffer = await response.arrayBuffer()

      initAudioContext()
      if (!audioContext.value) {
        console.error('AudioContext not available')
        return null
      }

      const audioBuffer = await audioContext.value.decodeAudioData(arrayBuffer)
      audioCache.set(soundName, audioBuffer)

      return audioBuffer
    } catch (error) {
      console.error(`Failed to load sound ${soundName}:`, error)
      return null
    }
  }

  const playSound = async (soundName: string) => {
    try {
      initAudioContext()

      if (!audioContext.value) {
        console.error('AudioContext not available')
        return
      }

      if (audioContext.value.state === 'suspended') {
        await audioContext.value.resume()
      }

      const audioBuffer = await loadSound(soundName)

      if (!audioBuffer) {
        console.warn(`Cannot play sound: ${soundName}`)
        return
      }

      const source = audioContext.value.createBufferSource()
      source.buffer = audioBuffer
      source.connect(audioContext.value.destination)
      source.start(0)

      console.debug(`Playing sound: ${soundName}`)
    } catch (error) {
      console.error(`Error playing sound ${soundName}:`, error)
    }
  }

  const applyBadgeCount = async (count: number) => {
    badgeCount.value = count

    try {
      if (badgeType.value === null) {
        await getCurrentWindow().setBadgeCount(undefined)
        return
      }

      if (count <= 0) {
        await getCurrentWindow().setBadgeCount(undefined)
        return
      }

      await getCurrentWindow().setBadgeCount(badgeType.value === 'dot' ? 1 : count)
    } catch (error) {
      console.error('Failed to apply badge count:', error)
    }
  }

  const buildNotificationBody = (payload: NativeNotificationPayload) => {
    if (payload.body?.trim()) {
      return payload.body
    }

    if (payload.kind === 'incoming-email' && payload.email) {
      const sender = buildSenderLabel(payload.email)
      const subject = buildSubjectLabel(payload.email)
      const snippet = buildSnippetLabel(payload.email)

      return snippet ? `${sender} — ${subject}\n${snippet}` : `${sender} — ${subject}`
    }

    if (payload.kind === 'reminder-email' && payload.email) {
      const sender = buildSenderLabel(payload.email)
      const subject = buildSubjectLabel(payload.email)
      const snippet = buildSnippetLabel(payload.email)

      return snippet
        ? `Reminder for ${sender} — ${subject}\n${snippet}`
        : `Reminder for ${sender} — ${subject}`
    }

    return undefined
  }

  const buildNotificationTitle = (payload: NativeNotificationPayload) => {
    if (payload.title?.trim()) {
      return payload.title
    }

    switch (payload.kind) {
      case 'incoming-email':
        return buildSenderLabel(payload.email)
      case 'outgoing-email':
        return 'Email sent'
      case 'reminder-email':
        return `Reminder: ${buildSubjectLabel(payload.email)}`
      default:
        return 'Ravn'
    }
  }

  const buildNotificationOptions = (payload: NativeNotificationPayload) => {
    const body = buildNotificationBody(payload)
    const sender = buildSenderLabel(payload.email)
    const subject = buildSubjectLabel(payload.email)
    const snippet = buildSnippetLabel(payload.email)

    const options: Record<string, any> = {
      body,
      autoCancel: true,
    }

    if (payload.tag?.trim()) {
      options.tag = payload.tag.trim()
    } else if (payload.kind === 'incoming-email' && payload.email?.id) {
      options.tag = `incoming-email:${payload.email.id}`
    } else if (payload.kind === 'reminder-email' && payload.email?.id && payload.email?.remindAt) {
      options.tag = `reminder-email:${payload.email.id}:${payload.email.remindAt}`
    }

    const targetUrl = buildNotificationDeepLink(payload)
    if (targetUrl) {
      options.extra = {
        targetUrl,
      }
    }

    const avatarPath = payload.email?.avatarUrl?.trim()
    if (avatarPath) {
      const attachmentUrl =
        avatarPath.startsWith('asset:') || avatarPath.startsWith('file:')
          ? avatarPath
          : convertFileSrc(avatarPath)

      options.icon = attachmentUrl
      options.attachments = [
        {
          id: `avatar:${payload.email?.id || payload.tag || payload.kind || 'notification'}`,
          url: attachmentUrl,
        },
      ]

      if (supportsImageNotifications.value) {
        options.largeBody = snippet
        options.summary = `${sender} • ${subject}`
      }
    }

    if (payload.kind === 'reminder-email') {
      options.title = buildNotificationTitle(payload)
      options.subtitle = sender
    }

    return options
  }

  const ensureNotificationPermission = async () => {
    try {
      let permissionGranted = await isPermissionGranted()
      if (!permissionGranted) {
        const permission = await requestPermission()
        permissionGranted = permission === 'granted'
      }
      return permissionGranted
    } catch (error) {
      console.error('Failed to resolve notification permission:', error)
      return false
    }
  }

  const buildNotificationDeepLink = (payload: NativeNotificationPayload) => {
    if (payload.deepLink?.trim()) {
      return payload.deepLink.trim()
    }

    const navigationTarget = payload.email?.navigationTarget?.trim()
    if (navigationTarget) {
      return navigationTarget
    }

    const accountId = payload.email?.accountId?.trim()
    const folderId = payload.email?.folderId?.trim()
    const conversationId = payload.email?.conversationId?.trim()
    const emailId = payload.email?.id?.trim()

    if (accountId && folderId && conversationId) {
      const query = emailId ? `?email=${encodeURIComponent(emailId)}` : ''
      return `ravn://mail/${accountId}/folders/${folderId}/conversations/${conversationId}${query}`
    }

    if (accountId && folderId && emailId) {
      return `ravn://mail/${accountId}/folders/${folderId}/emails/${emailId}`
    }

    return null
  }

  const showNativeNotification = async (payload: NativeNotificationPayload) => {
    if (!notificationSettings.value?.enabled) {
      return
    }

    if (payload.suppressDuringBootstrap !== false && bootstrapSyncInProgress.value) {
      return
    }

    const hasPermission = await ensureNotificationPermission()
    if (!hasPermission) {
      return
    }

    try {
      const title = buildNotificationTitle(payload)
      const options = buildNotificationOptions(payload)

      sendNotification({
        title,
        ...options,
      })
    } catch (error) {
      console.error('Failed to send native notification:', error)
    }
  }

  const updateBadgeCount = async () => {
    try {
      const result = await invoke<BadgeCount>('update_badge_count')
      await applyBadgeCount(result.count)
    } catch (error) {
      console.error('Failed to update badge count:', error)
    }
  }

  const getBadgeCount = async () => {
    try {
      const result = await invoke<BadgeCount>('get_badge_count')
      await applyBadgeCount(result.count)
      return result.count
    } catch (error) {
      console.error('Failed to get badge count:', error)
      return 0
    }
  }

  const testNotificationSound = async (soundName: string) => {
    try {
      await invoke('test_notification_sound', { soundName })
    } catch (error) {
      console.error('Failed to test notification sound:', error)
      throw error
    }
  }

  const checkDueReminderNotifications = async () => {
    if (!notificationSettings.value?.enabled || reminderCheckInFlight.value) {
      return
    }

    reminderCheckInFlight.value = true

    try {
      pruneReminderRecords()

      const result = await invoke<ReminderCheckResult>('get_due_reminder_notifications')
      const due = result?.due || []

      for (const email of due) {
        const remindAt = email.remind_at || null
        if (!email.id || !remindAt || hasReminderNotificationBeenSent(email.id, remindAt)) {
          continue
        }

        const payload: NativeNotificationPayload = {
          kind: 'reminder-email',
          email: {
            id: email.id,
            accountId: email.account_id,
            folderId: email.folder_id,
            senderName: email.from?.name || null,
            senderAddress: email.from?.address || null,
            subject: email.subject || null,
            snippet: email.snippet || null,
            remindAt,
          },
          playSound: true,
          suppressDuringBootstrap: false,
          tag: `reminder-email:${email.id}:${remindAt}`,
        }

        await showNativeNotification(payload)

        const reminderSound = notificationSettings.value?.reminderSound
        if (reminderSound) {
          await playSound(reminderSound)
        }

        markReminderNotificationSent(email.id, remindAt)
      }
    } catch (error) {
      console.error('Failed to check reminder notifications:', error)
    } finally {
      reminderCheckInFlight.value = false
    }
  }

  const setupListeners = () => {
    if (initialized.value) {
      return
    }

    initialized.value = true
  }

  const ensureSetup = async () => {
    if (!isClient) {
      return
    }

    if (initialized.value) {
      return
    }

    if (!notificationSetupPromise) {
      notificationSetupPromise = (async () => {
        await listen<string>('play-sound', (event) => {
          const soundName = event.payload
          console.debug(`Received play-sound event: ${soundName}`)
          void playSound(soundName)
        })

        await listen<BadgeCount>('badge-count-updated', async (event) => {
          await applyBadgeCount(event.payload.count)
          console.debug(`Badge count updated: ${event.payload.count}`)
        })

        await listen<NativeNotificationPayload>('native-notification', async (event) => {
          await showNativeNotification(event.payload)
        })

        await listen<boolean>('notifications:bootstrap-sync-state', (event) => {
          bootstrapSyncInProgress.value = !!event.payload
        })

        await listen<string>('notification:navigate', async (event) => {
          const targetUrl = event.payload?.trim()
          if (!targetUrl) {
            return
          }

          try {
            await navigateToUrl(targetUrl)
          } catch (error) {
            console.error('Failed to navigate from notification click:', error)
          }
        })

        setupListeners()
        await getBadgeCount()
      })()

      try {
        await notificationSetupPromise
      } finally {
        notificationSetupPromise = null
      }
      return
    }

    await notificationSetupPromise
  }

  watch(
    () => settings.value?.notifications,
    async () => {
      await applyBadgeCount(badgeCount.value)
    },
    { deep: true }
  )

  void ensureSetup()

  return {
    badgeCount,
    bootstrapSyncInProgress,
    isProductionBuild,
    playSound,
    updateBadgeCount,
    getBadgeCount,
    testNotificationSound,
    showNativeNotification,
    checkDueReminderNotifications,
  }
}
