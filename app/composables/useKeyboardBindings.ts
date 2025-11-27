import { useMagicKeys, whenever } from '@vueuse/core'
import type { KeyboardBindings } from '~/types/settings'

export type KeyboardAction =
  | 'nextEmail'
  | 'previousEmail'
  | 'nextConversation'
  | 'previousConversation'
  | 'openConversation'
  | 'goBack'
  | 'archive'
  | 'delete'
  | 'reply'
  | 'replyAll'
  | 'forward'
  | 'toggleStar'
  | 'markRead'
  | 'markUnread'
  | 'newEmail'
  | 'send'
  | 'discardDraft'
  | 'toggleDetailsPane'
  | 'toggleSidebar'
  | 'search'
  | 'settings'
  | 'help'
  | 'refresh'

export type KeyboardActionHandler = () => void | Promise<void>

export interface KeyboardBindingOptions {
  enabled?: boolean
  ignoreInputFields?: boolean
}

export function useKeyboardBindings(
  handlers: Partial<Record<KeyboardAction, KeyboardActionHandler>>,
  options: KeyboardBindingOptions = {}
) {
  if (import.meta.server) {
    return {
      isEnabled: ref(false),
      bindings: computed(() => ({})),
      enable: () => {
      },
      disable: () => {
      },
    }
  }

  const isEnabled = ref(options.enabled ?? true)
  const ignoreInputFields = options.ignoreInputFields ?? true
  const stopHandles: Array<() => void> = []

  const settingsComposable = useSettings()
  const settings = settingsComposable.settings as Ref<any>
  const keys = useMagicKeys()

  const bindings = computed<KeyboardBindings>(() => {
    return settings?.value?.keyboard?.bindings ?? {}
  })

  const keyboardEnabled = computed(() => {
    return settings?.value?.keyboard?.enabled ?? true
  })

  const shouldIgnoreEvent = () => {
    if (!ignoreInputFields) return false

    const activeElement = document.activeElement
    if (!activeElement) return false

    const tagName = activeElement.tagName
    const isContentEditable = (activeElement as HTMLElement).isContentEditable

    return (
      tagName === 'INPUT' ||
      tagName === 'TEXTAREA' ||
      tagName === 'SELECT' ||
      isContentEditable
    )
  }

  const normalizeKey = (key: string): string => {
    const replacements: Record<string, string> = {
      'Meta': 'cmd',
      'Ctrl': 'ctrl',
      'Shift': 'shift',
      'Alt': 'alt',
      'Enter': 'enter',
      'Escape': 'escape',
      'Backspace': 'backspace',
      'Delete': 'delete',
      'ArrowUp': 'arrowup',
      'ArrowDown': 'arrowdown',
      'ArrowLeft': 'arrowleft',
      'ArrowRight': 'arrowright',
    }

    const parts = key.split('+').map(part => {
      const replacement = replacements[part]
      return replacement ? replacement : part.toLowerCase()
    })

    return parts.join('+')
  }

  const setupBindings = () => {
    stopHandles.forEach(stop => stop())
    stopHandles.length = 0

    if (!isEnabled.value || !keyboardEnabled.value) return

    Object.entries(handlers).forEach(([action, handler]) => {
      if (!handler) return

      const keyBindings = bindings.value[action]
      if (!keyBindings || keyBindings.length === 0) return

      keyBindings.forEach((keyBinding) => {
        const normalizedKey = normalizeKey(keyBinding)
        const keyRef = keys[normalizedKey]

        if (!keyRef) {
          console.warn(`[KeyboardBindings] Key not found: ${normalizedKey} for action ${action}`)
          return
        }

        const stop = whenever(keyRef, () => {
          if (shouldIgnoreEvent()) return

          console.log(`[KeyboardBindings] Action triggered: ${action} (${keyBinding})`)
          handler()
        })

        stopHandles.push(stop)
      })
    })
  }

  watch([isEnabled, keyboardEnabled, bindings], setupBindings, { immediate: true })

  onUnmounted(() => {
    stopHandles.forEach(stop => stop())
    stopHandles.length = 0
  })

  return {
    isEnabled,
    bindings,
    enable: () => {
      isEnabled.value = true
    },
    disable: () => {
      isEnabled.value = false
    },
  }
}

export function getKeyDisplayName(key: string): string {
  const replacements: Record<string, string> = {
    'Meta': '⌘',
    'Ctrl': 'Ctrl',
    'Shift': '⇧',
    'Alt': '⌥',
    'Enter': '↵',
    'Escape': 'Esc',
    'Backspace': '⌫',
    'Delete': 'Del',
    'ArrowUp': '↑',
    'ArrowDown': '↓',
    'ArrowLeft': '←',
    'ArrowRight': '→',
  }

  return key.split('+').map(part => {
    return replacements[part] || part.toUpperCase()
  }).join('')
}
