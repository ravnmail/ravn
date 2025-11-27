import type { StrikeOptions as TiptapStrikeOptions } from '@tiptap/extension-strike'
import { Strike as TiptapStrike } from '@tiptap/extension-strike'
import ActionButton from '@/components/ActionButton.vue'
import type { GeneralOptions } from '@/types/composer'

export interface StrikeOptions extends TiptapStrikeOptions, GeneralOptions<StrikeOptions> { }

export const Strike = TiptapStrike.extend<StrikeOptions>({
  addOptions() {
    return {
      ...this.parent?.(),
      button: ({ editor, t }) => ({
        component: ActionButton,
        componentProps: {
          action: () => editor.chain().toggleStrike().focus().run(),
          isActive: () => editor.isActive('strike') || false,
          disabled: !editor.isEditable || !editor.can().toggleStrike(),
          icon: 'strikethrough',
          shortcutKeys: ['shift', 'mod', 'S'],
          tooltip: t('composer.strike.tooltip'),
        },
      }),
    }
  },
})
