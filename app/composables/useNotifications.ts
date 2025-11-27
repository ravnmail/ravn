import { getCurrentWindow } from '@tauri-apps/api/window'
import { listen } from '@tauri-apps/api/event'
import { invoke } from '@tauri-apps/api/core'

interface BadgeCount {
  count: number
}


const initialized = useState<boolean>('notifications-initialized', () => false)
export const useNotifications = () => {
  const badgeCount = ref<number>(0)
  const audioContext = ref<AudioContext | null>(null)
  const audioCache = new Map<string, AudioBuffer>()

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

  // Update badge count
  const updateBadgeCount = async () => {
    try {
      const result = await invoke<BadgeCount>('update_badge_count')
      badgeCount.value = result.count
    } catch (error) {
      console.error('Failed to update badge count:', error)
    }
  }

  // Get current badge count
  const getBadgeCount = async () => {
    try {
      const result = await invoke<BadgeCount>('get_badge_count')
      badgeCount.value = result.count
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
    // Listen for play-sound events from backend
    listen<string>('play-sound', (event) => {
      const soundName = event.payload
      console.debug(`Received play-sound event: ${soundName}`)
      playSound(soundName)
    })

    // Listen for badge-count-updated events from backend
    listen<BadgeCount>('badge-count-updated', async  (event) => {
      await getCurrentWindow().setBadgeCount(event.payload.count > 0 ? event.payload.count : null)
      console.debug(`Badge count updated: ${event.payload.count}`)
    })
    initialized.value = true
  }

  // Initialize on mount
  onMounted(() => {
    setupListeners()
  })

  // Cleanup on unmount
  onUnmounted(() => {
    if (audioContext.value) {
      audioContext.value.close()
      audioContext.value = null
    }
  })

  return {
    badgeCount,
    playSound,
    updateBadgeCount,
    getBadgeCount,
    testNotificationSound,
  }
}
