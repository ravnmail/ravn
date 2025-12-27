import type { CleanTranslation } from 'nuxt-i18n-micro-types'
import { forEach } from 'lodash'
import { useMousetrap } from '~/lib/moustrap-vue'

export type ActionOption = {
  id: string
  namespace: string
  icon?: string
  handler: (arg?: unknown) => void
}

export type UnregisterActionOption = {
  id: string
  namespace: string
}

export type Action = {
  id: string
  namespace: string
  key: string
  name: CleanTranslation
  description?: CleanTranslation
  tooltip?: CleanTranslation
  shortcut?: string
  icon?: string
  hidden?: boolean
  handler: (arg?: unknown) => void
}

export type Keybinding = {
  action: string
  isContext: (context: string) => boolean
  props?: unknown
}

export type KeyMap = {
  context: string
  bindings: {
    [keyCombo: string]: string | [string, unknown?]
  }
}

export type Context = {
  name: string
  focused: ComputedRef<boolean>
}

export type KeyMapFile = KeyMap[]

export function useActions() {
  const actionsRegistry = useState<Record<string, Action>>('actions', () => ({}))
  const keybindingRegistry = useState<Record<string, string>>('action-keybindings', () => ({}))
  const context = useState<Context[]>('action-context', () => [])

  const mousetrap = useMousetrap()

  const { t } = useI18n()

  function setupContext(contexts: Context[]) {
    context.value = contexts
  }

  function addContext(name: string, focused: ComputedRef) {
    if (!context.value.some(c => c.name === name)) {
      context.value.push({ name, focused })
    }
  }

  function removeContext(name: string) {
    context.value = context.value.filter(c => c.name !== name)
  }

  function isContextFocused(name: string, context: Context[]): boolean {
    const ctx = context.find(c => c.name === name)
    return ctx ? ctx.focused : false
  }

  function buildKey(namespace: string, name: string): string {
    return `${namespace}:${name}`
  }

  function register(action: ActionOption): Action {
    const key = buildKey(action.namespace, action.id)
    if (actionsRegistry.value[key]) {
      console.warn(`[Actions] Action with id '${key}' is already registered. Overwriting.`)
    }

    actionsRegistry.value[key] = {
      id: action.id,
      namespace: action.namespace,
      key,
      name: t(`actions.${action.id}.name`),
      description: t(`actions.${action.id}.description`, {}, undefined),
      tooltip: t(`actions.${action.id}.tooltip`, {}, undefined),
      icon: action.icon,
      handler: action.handler,
    }

    return actionsRegistry.value[key]
  }

  function unregister(namespace: string, id: string) {
    // eslint-disable-next-line @typescript-eslint/no-dynamic-delete
    delete actionsRegistry.value[buildKey(namespace, id)]
  }

  function setupKeybindings(file: KeyMapFile) {
    mousetrap.reset()
    forEach(keybindingRegistry.value, (action, keyCombo) => {
      if (keyCombo) {
        delete keybindingRegistry.value[keyCombo]
      }
    })

    file.forEach(map => {
      forEach(map.bindings, (config, keyCombo) => {
        const [id, props] = Array.isArray(config) ? config : [config, undefined]

        mousetrap.bind(keyCombo, () => {
          const action = actionsRegistry.value[id]
          if (action && isContextFocused(action.namespace, context.value)) {
            action.handler(props)
          }
        })

        keybindingRegistry.value[keyCombo] = id
      })
    })
  }

  function executeAction(namespace: string, id?: string | null, arg?: unknown) {
    const key = id ? buildKey(namespace, id) : namespace
    const action = actionsRegistry.value[key]
    if (action) {
      action.handler(arg)
    }
  }

  const actions = computed(() => {
    const shortcuts = Object.entries(keybindingRegistry.value)
    return Object.values(actionsRegistry.value)
      .filter(action => !action.hidden)
      .map(action => ({
        ...action,
        shortcut: shortcuts.find(([, actionKey]) => actionKey === action.key)?.[0],
      }))
  })

  const possibleActions = computed(() => {
    return actions.value.filter(action => {
      return isContextFocused(action.namespace, context.value)
    })
  })

  return {
    keybindings: readonly(keybindingRegistry),
    actions: readonly(actions),
    possibleActions: readonly(possibleActions),
    context: readonly(context),

    getAction(namespace: string, id: string): ComputedRef<Action | undefined> {
      const key = id ? buildKey(namespace, id) : namespace
      return computed(() => actions.value.find(action => action.key === key))
    },

    setupContext,
    addContext,
    removeContext,

    executeAction,
    setupKeybindings,
    register,
    unregister,
  }
}