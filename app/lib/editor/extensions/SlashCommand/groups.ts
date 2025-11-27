import type { Editor } from '@tiptap/core'
import type { Group } from './types'

export function renderGroups() {
  const { t } = useI18n()
  const groups: Group[] = [
    {
      name: 'format',
      title: t('composer.slash.format'),
      commands: [
        {
          name: 'paragraph',
          label: t('composer.paragraph'),
          aliases: ['paragraph'],
          iconName: 'type',
          action: ({ editor, range }) => {
            editor.chain().focus().deleteRange(range).clearNodes().focus().run()
          },
        },
        {
          name: 'heading1',
          label: t('composer.headings.h1'),
          aliases: ['h1', '#'],
          shortcut: '#',
          iconName: 'heading-1',
          action: ({ editor, range }) => {
            editor.chain().focus().deleteRange(range).setHeading({ level: 1 }).run()
          },
        },
        {
          name: 'heading2',
          label: t('composer.headings.h2'),
          aliases: ['h2', '##'],
          shortcut: '##',
          iconName: 'heading-2',
          action: ({ editor, range }) => {
            editor.chain().focus().deleteRange(range).setHeading({ level: 2 }).run()
          },
        },
        {
          name: 'heading3',
          label: t('composer.headings.h3'),
          aliases: ['h3', '###'],
          shortcut: '###',
          iconName: 'heading-3',
          action: ({ editor, range }) => {
            editor.chain().focus().deleteRange(range).setHeading({ level: 3 }).run()
          },
        },
        {
          name: 'bulletList',
          label: t('composer.bulletlist.tooltip'),
          aliases: ['ul', '-'],
          iconName: 'list',
          shortcut: '-',
          action: ({ editor, range }) => {
            editor.chain().focus().deleteRange(range).toggleBulletList().run()
          },
        },
        {
          name: 'numberedList',
          label: t('composer.orderedlist.tooltip'),
          aliases: ['ol', '1.'],
          iconName: 'list-ordered',
          shortcut: '1.',
          action: ({ editor, range }) => {
            editor.chain().focus().deleteRange(range).toggleOrderedList().run()
          },
        },
        {
          name: 'blockquote',
          label: t('composer.blockquote.tooltip'),
          description: '插入引入格式',
          aliases: ['>'],
          shortcut: '>',
          iconName: 'text-quote',
          action: ({ editor, range }) => {
            editor.chain().focus().deleteRange(range).setBlockquote().run()
          },
        },
        {
          name: 'horizontalrule',
          label: t('composer.horizontalrule.tooltip'),
          aliases: ['---', 'horizontalRule'],
          shortcut: '---',
          iconName: 'minus',
          action: ({ editor, range }) => {
            editor.chain().focus().deleteRange(range).setHorizontalRule().run()
          },
        },
        {
          name: 'link',
          label: t('composer.link.tooltip'),
          aliases: ['link', 'a'],
          iconName: 'link',
          action: ({ editor, range }) => {
            editor
              .chain()
              .deleteRange(range)
              .extendMarkRange('link')
              .insertContent({
                type: 'text',
                text: 'link',
                marks: [
                  {
                    type: 'link',
                    attrs: {
                      href: '',
                      target: '_blank',
                    },
                  },
                ],
              })
              .setLink({ href: '' })
              .focus()
              .run()
          },
        },
      ],
    },
    {
      name: 'media',
      title: t('composer.slash.media'),
      commands: [
        {
          name: 'image',
          label: t('composer.image.tooltip'),
          iconName: 'image-up',
          description: 'Insert a image',
          aliases: ['image', 'img'],
          shouldBeHidden: editor => editor.isActive('columns'),
          action: ({ editor, range }) => {
            editor.chain().focus().deleteRange(range).setImageUpload().run()
          },
        },
        {
          name: 'codeBlock',
          label: t('composer.codeblock.tooltip'),
          iconName: 'code-2',
          aliases: ['codeBlock'],
          description: 'Code block with syntax highlighting',
          shouldBeHidden: editor => editor.isActive('columns'),
          action: ({ editor, range }) => {
            editor.chain().focus().deleteRange(range).setCodeBlock().run()
          },
        },
        {
          name: 'callout',
          iconName: 'megaphone',
          aliases: ['callout'],
          label: t('composer.callout.tooltip'),
          description: 'Insert a callout',
          action: ({ editor, range }) => {
            editor.chain().focus().deleteRange(range).toggleCallout().run()
          },
        },
        {
          name: 'signature',
          label: t('composer.signature.tooltip'),
          iconName: 'signature',
          aliases: ['signature'],
          description: 'Insert a signature',
          action: ({ editor, range }) => {
            editor.chain().focus().deleteRange(range).insertEmailSignature().run()
          },
        }
      ],
    },
  ]

  return groups
}
