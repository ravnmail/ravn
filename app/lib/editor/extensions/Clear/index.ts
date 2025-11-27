import { Node } from '@tiptap/core'

import ActionButton from '@/components/ActionButton.vue'

import type { GeneralOptions } from '@/types/composer'

export type ClearOptions = GeneralOptions<ClearOptions>

export const Clear = Node.create<ClearOptions>({
  name: 'clear',
  addOptions() {
    return {
      ...this.parent?.(),
      button: ({ editor, t }) => ({
        component: ActionButton,
        componentProps: {
          action: () => editor.chain().focus().clearNodes().unsetAllMarks().run(),
          disabled: !editor.can().chain().focus().clearNodes().unsetAllMarks().run(),
          icon: 'eraser',
          tooltip: t('editor.clear.tooltip'),
        },
      }),
    }
  },
})
