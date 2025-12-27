import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import type { KeyMapFile } from './useActions'
import type { CleanTranslation } from 'nuxt-i18n-micro-types'

export type KeybindingListItem = {
  context: string
  key: string
  namespace: string
  fullName: string
  description?: CleanTranslation | string
  action: string | null
  props?: unknown
  source: 'Default' | 'User'
}

export function useKeybindings() {
  const keybindings = useState<KeyMapFile>('keybindings', () => [])
  const userKeybindings = useState<KeyMapFile>('user-keybindings', () => [])

  async function getKeybindings(): Promise<KeyMapFile> {
    const result = await invoke<KeyMapFile>('get_keybindings')
    keybindings.value = result
    return result
  }

  async function getUserKeybindings(): Promise<KeyMapFile> {
    const result = await invoke<KeyMapFile>('get_user_keybindings')
    userKeybindings.value = result
    return result
  }

  async function setKeybinding(
    context: string,
    key: string,
    action: string | null,
    props?: unknown,
  ): Promise<void> {
    await invoke('set_keybinding', {
      context,
      key,
      action,
      props,
    })
    await reloadKeybindings()
  }

  async function removeKeybinding(context: string, key: string): Promise<void> {
    await invoke('remove_keybinding', {
      context,
      key,
    })
    await reloadKeybindings()
  }

  async function reloadKeybindings(): Promise<void> {
    await Promise.all([
      getKeybindings,
      getUserKeybindings
    ])
  }

  async function onKeybindingsChanged(callback: () => void) {
    return await listen('keybindings-changed', callback)
  }

  if (!keybindings.value.length && !userKeybindings.value.length) {
    getKeybindings().then()
    getUserKeybindings().then()
  }

  const keybindingsList = computed(() => {
    const list: Record<string, KeybindingListItem> = {}
    const { t } = useI18n()

    function processKeymaps(keymaps: KeyMapFile, source: 'Default' | 'User') {
      for (const keymap of keymaps) {
        const context = keymap.context
        for (const [key, actionEntry] of Object.entries(keymap.bindings)) {
          let action: string | null = null
          let props: unknown = undefined
          if (Array.isArray(actionEntry)) {
            action = actionEntry[0]
            props = actionEntry[1]
          } else {
            action = actionEntry
          }
          const listKey = `${context}::${key}`
          const [namespace, actionId] = action ? action.split(':') : ["", null]
          const fullName = actionId ? (namespace + ': ' + t(`actions.${actionId}.name`)) : '<Unbound>'
          list[listKey] = {
            context,
            key,
            action,
            namespace,
            fullName,
            description: t(`actions.${actionId}.description`, {}, null),
            props,
            source,
          }
        }
      }
    }
    processKeymaps(keybindings.value, 'Default')
    processKeymaps(userKeybindings.value, 'User')

    return Object.values(list)
  })

  return {
    keybindings: readonly(keybindings),
    userKeybindings: readonly(userKeybindings),
    keybindingsList: readonly(keybindingsList),

    getKeybindings,
    setKeybinding,
    removeKeybinding,
    onKeybindingsChanged,
  }
}
