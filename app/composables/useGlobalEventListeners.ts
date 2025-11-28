import { listen } from '@tauri-apps/api/event'
import { getCurrentWindow } from '@tauri-apps/api/window'
import type { QueryClient } from '@tanstack/vue-query'

type EventDef =
  | { type: 'query-invalidation'; name: string; invalidateKey: readonly unknown[] }
  | { type: 'custom'; name: string; handler: (event: any) => void | Promise<void> }

type ListenerSubscription = {
  unlisten: (() => void) | null
}

const subscriptions: Map<string, ListenerSubscription> = new Map()
let isInitialized = false

export const useGlobalEventListeners = (queryClient: QueryClient) => {
  if (isInitialized) {
    console.log('[useGlobalEventListeners] Already initialized, skipping')
    return
  }

  isInitialized = true

  const setupListener = async (eventDef: EventDef) => {
    const { name } = eventDef

    if (subscriptions.has(name)) {
      console.log(`[useGlobalEventListeners] Listener already registered for ${name}`)
      return
    }

    try {
      let callback: (event: any) => void | Promise<void>

      if (eventDef.type === 'query-invalidation') {
        callback = () => {
          console.log(`Event received: ${name}, invalidating queries for key:`, eventDef.invalidateKey)
          queryClient.invalidateQueries({ queryKey: eventDef.invalidateKey })
        }
      } else {
        callback = eventDef.handler
      }

      const unlisten = await listen(name, (event) => {
        void Promise.resolve(callback(event))
      })
      subscriptions.set(name, { unlisten })
      console.log(`[useGlobalEventListeners] Set up listener for ${name}`)
    } catch (error) {
      console.error(`Failed to set up listener for ${name}:`, error)
    }
  }

  const registerEvents = async (events: Array<EventDef>) => {
    const promises = events.map((eventDef) => setupListener(eventDef))
    await Promise.allSettled(promises)
  }

  // Register all events from all composables
  const allEvents: Array<EventDef> = [
    // Folders
    { type: 'query-invalidation', name: 'folder:created', invalidateKey: ['folders', 'list'] as const },
    { type: 'query-invalidation', name: 'folder:updated', invalidateKey: ['folders', 'list'] as const },
    { type: 'query-invalidation', name: 'folder:deleted', invalidateKey: ['folders', 'list'] as const },
    { type: 'query-invalidation', name: 'folder:updated', invalidateKey: ['conversations', 'list'] as const },
    // Labels
    { type: 'query-invalidation', name: 'label:created', invalidateKey: ['labels', 'list'] as const },
    { type: 'query-invalidation', name: 'label:updated', invalidateKey: ['labels', 'list'] as const },
    { type: 'query-invalidation', name: 'label:deleted', invalidateKey: ['labels', 'list'] as const },
    // Accounts
    { type: 'query-invalidation', name: 'account:created', invalidateKey: ['accounts', 'list'] as const },
    { type: 'query-invalidation', name: 'account:updated', invalidateKey: ['accounts', 'list'] as const },
    { type: 'query-invalidation', name: 'account:deleted', invalidateKey: ['accounts', 'list'] as const },
    // Emails
    { type: 'query-invalidation', name: 'email:updated', invalidateKey: ['emails', 'list'] as const },
    { type: 'query-invalidation', name: 'email:deleted', invalidateKey: ['emails', 'list'] as const },
    // AI Analysis
    { type: 'query-invalidation', name: 'email:ai-analysis-complete', invalidateKey: ['email-ai'] as const },
    // Contacts
    { type: 'query-invalidation', name: 'contact:created', invalidateKey: ['contacts', 'list'] as const },
    { type: 'query-invalidation', name: 'contact:updated', invalidateKey: ['contacts', 'list'] as const },
    { type: 'query-invalidation', name: 'contact:deleted', invalidateKey: ['contacts', 'list'] as const },
    // Conversations
    { type: 'query-invalidation', name: 'conversation:created', invalidateKey: ['conversations', 'list'] as const },
    { type: 'query-invalidation', name: 'conversation:updated', invalidateKey: ['conversations', 'list'] as const },
    { type: 'query-invalidation', name: 'conversation:deleted', invalidateKey: ['conversations', 'list'] as const },
    // Views
    { type: 'query-invalidation', name: 'view:created', invalidateKey: ['views', 'list'] as const },
    { type: 'query-invalidation', name: 'view:updated', invalidateKey: ['views', 'list'] as const },
    { type: 'query-invalidation', name: 'view:deleted', invalidateKey: ['views', 'list'] as const },
  ]

  void registerEvents(allEvents)
}

export const cleanupGlobalEventListeners = () => {
  subscriptions.forEach(({ unlisten }) => {
    try {
      unlisten?.()
    } catch (e) {
      console.error('Error while unlistening:', e)
    }
  })
  subscriptions.clear()
  isInitialized = false
  console.log('[useGlobalEventListeners] Cleaned up all listeners')
}
