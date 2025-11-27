import { Node } from '@tiptap/core'
import ActionButton from './components/AIButton.vue'
import type { GeneralOptions } from '@/types/composer'

export interface MenuItem {
  label: string
  prompt?: string
  children?: MenuItem[]
}

export interface AIOptions extends GeneralOptions<AIOptions> {
  completions: (
    history: Array<{ prompt: string; content: string }>,
    signal?: AbortSignal
  ) => Promise<never>
  shortcuts: MenuItem[]
}

export const AI = Node.create<AIOptions>({
  name: 'AI',
  group: 'block',
  addOptions(): AIOptions {
    return {
      ...this.parent?.(),
      toolbar: false,
      button: ({ t }) => ({
        component: ActionButton,
        componentProps: {
          customClass: 'text-purple-500',
          icon: 'ravn:raven',
          tooltip: t('composer.AI.ask'),
        },
      }),
    }
  },
})
