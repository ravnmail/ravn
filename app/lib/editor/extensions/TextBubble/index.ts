import { Extension } from '@tiptap/core'

import TextDropdown from './components/TextDropdown.vue'

import type { GeneralOptions } from '@/types/composer'

export type TextBubbleOptions = GeneralOptions<TextBubbleOptions>

export const TextBubble = Extension.create<TextBubbleOptions>({
  name: 'text-bubble',
  addOptions() {
    return {
      ...this.parent?.(),
      toolbar: true,
      button: () => ({
        component: TextDropdown,
        componentProps: {},
      }),
    }
  },
})

export default TextBubble
