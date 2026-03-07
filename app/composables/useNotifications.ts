import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { getCurrentWindow } from '@tauri-apps/api/window'
import {
  isPermissionGranted,
  requestPermission,
  sendNotification,
} from '@tauri-apps/plugin-notification'

interface BadgeCount {
  count: number
  visible?: boolean
  mode?: BadgeType | 'none'
}

interface NotificationEmailPreview {
  id?: string
  accountId?: string
  folderId?: string
  senderName?: string | null
  senderAddress?: string | null
  subject?: string | null
  snippet?: string | null
}

interface NativeNotificationPayload {
  kind?: 'incoming-email' | 'outgoing-email' | 'system'
  title?: string
  body?: string
  email?: NotificationEmailPreview
  playSound?: boolean
  suppressDuringBootstrap?: boolean
}

type BadgeType = 'count' | 'dot' | null

export const useNotifications = () => {
  const initialized = useState<boolean>('notifications-initialized', () => false)
  const badgeCount = useState<number>('notifications-badge-count', () => 0)
  const audioContext = useState<AudioContext | null>('notifications-audio-context', () => null)
  const settings = useState<any | null>('settings', () => null)
  const audioCache = new Map<string, AudioBuffer>()

  const notificationSettings = computed(() => settings.value?.notifications)
  const badgeType = computed<BadgeType>(() => notificationSettings.value?.badgeType ?? 'count')
  const bootstrapSyncInProgress = useState<boolean>('notifications-bootstrap-sync', () => false)

  // Initialize audio context on first interaction (for browser autoplay policy)
  const initAudioContext = () => {
    if (!audioContext.value) {
      audioContext.value = new AudioContext()
    }
  }

  // Load and cache audio file
  const loadSound = async (soundName: string): Promise<AudioBuffer | null> => {
    try {
      // Check cache first
      if (audioCache.has(soundName)) {
        return audioCache.get(soundName)!
      }

      // Load from resources/sounds/
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

      // Cache the audio buffer
      audioCache.set(soundName, audioBuffer)

      return audioBuffer
    } catch (error) {
      console.error(`Failed to load sound ${soundName}:`, error)
      return null
    }
  }

  // Play a sound
  const playSound = async (soundName: string) => {
    try {
      initAudioContext()

      if (!audioContext.value) {
        console.error('AudioContext not available')
        return
      }

      // Resume audio context if suspended (browser autoplay policy)
      if (audioContext.value.state === 'suspended') {
        await audioContext.value.resume()
      }

      const audioBuffer = await loadSound(soundName)

      if (!audioBuffer) {
        console.warn(`Cannot play sound: ${soundName}`)
        return
      }

      // Create buffer source
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
      const sender =
        payload.email.senderName?.trim() || payload.email.senderAddress?.trim() || 'Unknown sender'
      const subject = payload.email.subject?.trim() || '(no subject)'
      const snippet = payload.email.snippet?.trim()

      return snippet ? `${sender} — ${subject}\n${snippet}` : `${sender} — ${subject}`
    }

    return undefined
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
      await sendNotification({
        title:
          payload.title ??
          (payload.kind === 'incoming-email'
            ? 'New email'
            : payload.kind === 'outgoing-email'
              ? 'Email sent'
              : 'Ravn'),
        body: buildNotificationBody(payload),
      })
    } catch (error) {
      console.error('Failed to send native notification:', error)
    }
  }

  // Update badge count
  const updateBadgeCount = async () => {
    try {
      const result = await invoke<BadgeCount>('update_badge_count')
      await applyBadgeCount(result.count)
    } catch (error) {
      console.error('Failed to update badge count:', error)
    }
  }

  // Get current badge count
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

  // Test a notification sound
  const testNotificationSound = async (soundName: string) => {
    try {
      await invoke('test_notification_sound', { soundName })
    } catch (error) {
      console.error('Failed to test notification sound:', error)
      throw error
    }
  }

  // Setup event listeners
  const setupListeners = () => {
    if (initialized.value) {
      return
    }

    listen<string>('play-sound', (event) => {
      const soundName = event.payload
      console.debug(`Received play-sound event: ${soundName}`)
      void playSound(soundName)
    })

    listen<BadgeCount>('badge-count-updated', async (event) => {
      await applyBadgeCount(event.payload.count)
      console.debug(`Badge count updated: ${event.payload.count}`)
    })

    listen<NativeNotificationPayload>('native-notification', async (event) => {
      await showNativeNotification(event.payload)

      if (event.payload.playSound && !bootstrapSyncInProgress.value) {
        const soundName = notificationSettings.value?.incomingSound
        if (soundName) {
          await playSound(soundName)
        }
      }
    })

    listen<boolean>('notifications:bootstrap-sync-state', (event) => {
      bootstrapSyncInProgress.value = !!event.payload
    })

    initialized.value = true
  }

  watch(
    () => settings.value?.notifications,
    async () => {
      await applyBadgeCount(badgeCount.value)
    },
    { deep: true }
  )

  onMounted(async () => {
    setupListeners()
    await getBadgeCount()
  })

  onUnmounted(() => {
    if (audioContext.value) {
      audioContext.value.close()
      audioContext.value = null
    }
  })

  return {
    badgeCount,
    bootstrapSyncInProgress,
    playSound,
    updateBadgeCount,
    getBadgeCount,
    testNotificationSound,
    showNativeNotification,
  }
}
