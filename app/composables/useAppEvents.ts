import { listen } from '@tauri-apps/api/event'
import { navigateToUrl } from './useUrlNavigation'
import { useAuth } from './useAuth'

export function useAppEvents() {
  let unlistenUrl: (() => void) | null = null
  let unlistenOffice365Auth: (() => void) | null = null

  // Handle URL navigation events from Tauri
  const handleUrlNavigation = async (event: { payload: string }) => {
    console.log('[AppEvents] Received navigate-to-url event:', event.payload)
    try {
      await navigateToUrl(event.payload)
      console.log('[AppEvents] Navigation completed successfully')
    }
    catch (error) {
      console.error('[AppEvents] Navigation failed:', error)
    }
  }

  // Handle Office365 OAuth token refresh failures
  const handleOffice365AuthRequired = async (event: any) => {
    const payload = event.payload
    console.log('[AppEvents] Office365 re-authentication required:', payload)

    const { startOAuth2 } = useAuth()

    // Show notification to user if permissions are granted
    if (typeof window !== 'undefined' && 'Notification' in window && Notification.permission === 'granted') {
      new Notification('Office365 Authentication Required', {
        body: 'Your Office365 session has expired. Please re-authenticate.',
      })
    }

    try {
      // Auto-trigger re-authentication flow
      await startOAuth2('office365', payload.account_id)
      console.log('[AppEvents] Office365 re-authentication flow initiated')
    }
    catch (err) {
      console.error('[AppEvents] Failed to initiate Office365 re-authentication:', err)
    }
  }

  onMounted(async () => {
    console.log('[AppEvents] Setting up event listeners')

    // Listen for URL navigation events from menu/shortcuts
    unlistenUrl = await listen<string>('navigate-to-url', handleUrlNavigation)
    console.log('[AppEvents] Listening for navigate-to-url events')

    // Listen for Office365 auth-required events
    unlistenOffice365Auth = await listen('office365:auth-required', handleOffice365AuthRequired)
    console.log('[AppEvents] Listening for office365:auth-required events')
  })

  onUnmounted(() => {
    console.log('[AppEvents] Cleaning up event listeners')

    // Cleanup event listeners
    if (unlistenUrl) {
      unlistenUrl()
    }
    if (unlistenOffice365Auth) {
      unlistenOffice365Auth()
    }
  })
}
