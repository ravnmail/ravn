import { listen } from '@tauri-apps/api/event'
import type { QueryClient } from '@tanstack/vue-query'

type EventDef = { name: string; invalidateKey: readonly unknown[] | undefined }

const listeners: Map<string, () => void> = new Map()

export const setupEventListeners = async (
  queryClient: QueryClient,
  events: Array<EventDef>,
) => {
  const setupListener = async (eventName: string, callback: () => void) => {
    if (listeners.has(eventName)) return
    try {
      const unlisten = await listen(eventName, callback)
      listeners.set(eventName, unlisten)
    } catch (error) {
      console.error(`Failed to set up listener for ${eventName}:`, error)
    }
  }

  const promises = events.map(({ name, invalidateKey }) =>
    setupListener(name, (a) => {
      console.log(`Event received: ${name}, invalidating queries for key:`, invalidateKey, a)
      queryClient.invalidateQueries({ queryKey: invalidateKey })
    }),
  )

  await Promise.allSettled(promises)
}

export const cleanupListeners = () => {
  listeners.forEach(unlisten => {
    try {
      unlisten()
    } catch (e) {
      console.error('Error while unlistening:', e)
    }
  })
  listeners.clear()
}

export const useSetupQueryListeners = (
  queryClient: QueryClient,
  events: Array<EventDef>,
) => {
  onMounted(() => {
    void setupEventListeners(queryClient, events)
  })

  onUnmounted(() => {
    cleanupListeners()
  })
}