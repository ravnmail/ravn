import { listen } from '@tauri-apps/api/event'
import type { QueryClient } from '@tanstack/vue-query'

type EventDef = { name: string; invalidateKey: readonly unknown[] | undefined }

type ListenerSubscription = {
  count: number
  unlisten: (() => void) | null
}

const subscriptions: Map<string, ListenerSubscription> = new Map()
const setupPromises: Map<string, Promise<void>> = new Map()

const setupEventListeners = async (
  queryClient: QueryClient,
  events: Array<EventDef>,
  namespace: string,
) => {
  const setupListener = async (eventName: string, callback: () => void) => {
    const key = `${namespace}:${eventName}`

    // If setup is already in progress, wait for it to complete
    if (setupPromises.has(key)) {
      console.log(`[useSetupQueryListeners] Waiting for setup to complete for ${key}`)
      await setupPromises.get(key)
      subscriptions.get(key)!.count++
      console.log(`[useSetupQueryListeners] Incrementing subscription count for ${key}, count:`, subscriptions.get(key)!.count)
      return
    }

    // If already set up, just increment the count
    if (subscriptions.has(key)) {
      subscriptions.get(key)!.count++
      console.log(`[useSetupQueryListeners] Incrementing subscription count for ${key}, count:`, subscriptions.get(key)!.count)
      return
    }

    // Create and track the setup promise to prevent race conditions
    const setupPromise = (async () => {
      try {
        const unlisten = await listen(eventName, callback)
        subscriptions.set(key, { count: 1, unlisten })
        console.log(`[useSetupQueryListeners] Set up listener for ${key}`)
      } catch (error) {
        console.error(`Failed to set up listener for ${eventName}:`, error)
      } finally {
        setupPromises.delete(key)
      }
    })()

    setupPromises.set(key, setupPromise)
    await setupPromise
  }

  const promises = events.map(({ name, invalidateKey }) =>
    setupListener(name, (a) => {
      console.log(`Event received: ${name}, invalidating queries for key:`, invalidateKey, a)
      queryClient.invalidateQueries({ queryKey: invalidateKey })
    }),
  )

  await Promise.allSettled(promises)
}

const cleanupListeners = (namespace: string) => {
  const keysToDelete: string[] = []

  subscriptions.forEach((subscription, key) => {
    if (key.startsWith(`${namespace}:`)) {
      subscription.count--
      console.log(`[useSetupQueryListeners] Decrementing subscription count for ${key}, count:`, subscription.count)

      if (subscription.count === 0) {
        try {
          subscription.unlisten?.()
          console.log(`[useSetupQueryListeners] Cleaned up listener for ${key}`)
        } catch (e) {
          console.error('Error while unlistening:', e)
        }
        keysToDelete.push(key)
      }
    }
  })

  keysToDelete.forEach(key => subscriptions.delete(key))
}

export const useSetupQueryListeners = (
  queryClient: QueryClient,
  events: Array<EventDef>,
  namespace: string = 'global',
) => {
  onMounted(() => {
    void setupEventListeners(queryClient, events, namespace)
  })

  onUnmounted(() => {
    cleanupListeners(namespace)
  })
}