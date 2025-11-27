import type { BulletListOptions as TiptapBulletListOptions } from '@tiptap/extension-bullet-list'
import { BulletList as TiptapBulletList } from '@tiptap/extension-bullet-list'
import BulletListMenuButton from './components/BulletListMenuButton.vue'
import type { GeneralOptions } from '@/types/composer'

export interface BulletListOptions extends TiptapBulletListOptions, GeneralOptions<BulletListOptions> { }

export const BulletList = TiptapBulletList.extend<BulletListOptions>({
  addAttributes() {
    return {
      ...this.parent?.(),
      listStyleType: {
        default: 'disc',
        parseHTML: (element: HTMLElement) => {
          const listStyleType = element.style['list-style-type' as keyof CSSStyleDeclaration] ?? 'disc'
          return { listStyleType }
        },
        renderHTML: ({ listStyleType }) => {
          return {
            style: `list-style-type: ${listStyleType?.listStyleType || listStyleType}`,
          }
        },
      },
    }
  },
  addOptions(): BulletListOptions {
    return {
      ...this.parent?.(),
      button: ({ editor, t }) => ({
        component: BulletListMenuButton,
        componentProps: {
          action: () => editor?.chain().focus().toggleBulletList().run(),
          isActive: () => editor.isActive('bulletList') || false,
          disabled: !editor?.isEditable || !editor.can().toggleBulletList(),
          shortcutKeys: ['shift', 'mod', '5'],
          icon: 'list',
          tooltip: t('composer.bulletlist.tooltip'),
        },
      }),
    }
  },
})
