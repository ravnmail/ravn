import type { BoldOptions as TiptapImageOptions } from '@tiptap/extension-bold'
import { Bold as TiptapBold } from '@tiptap/extension-bold'
import ActionButton from '@/components/ActionButton.vue'

import type { GeneralOptions } from '@/types/composer'

export interface BoldOptions extends TiptapImageOptions, GeneralOptions<BoldOptions> {
}

export const Bold = TiptapBold.extend<BoldOptions>({
  addOptions(): BoldOptions {
    return {
      ...this.parent?.(),
      button: ({ editor, t }) => ({
        component: ActionButton,
        componentProps: {
          action: () => editor?.chain().focus().toggleBold().run(),
          isActive: () => editor.isActive('bold') || false,
          disabled: !editor?.isEditable || !editor.can().toggleBold(),
          icon: 'bold',
          shortcutKeys: ['mod', 'B'],
          tooltip: t('composer.bold.tooltip'),
        },
      }),
    }
  },
})
