import { Node, mergeAttributes } from '@tiptap/core'
import type { Node as ProseMirrorNode } from '@tiptap/pm/model'
import { TextSelection } from '@tiptap/pm/state'
import ActionButton from '@/components/ActionButton.vue'
import type { GeneralOptions, ButtonViewParams } from '@/types/composer'

export interface QuotedContentAttributes {
  type: 'reply' | 'forward'
  originalFrom: string
  originalDate: string
  originalSubject?: string
  originalTo?: string
}

export interface QuotedContentOptions extends GeneralOptions<QuotedContentOptions> {
  HTMLAttributes: Record<string, unknown>
  replyLabel?: string
  forwardedMessageLabel?: string
  fromLabel?: string
  dateLabel?: string
  subjectLabel?: string
  toLabel?: string
  wroteLabel?: string
}

declare module '@tiptap/core' {
  interface Commands<ReturnType> {
    quotedContent: {
      setQuotedContent: (attributes: QuotedContentAttributes, content?: string) => ReturnType
      toggleQuotedContent: () => ReturnType
    }
  }
}

export const QuotedContent = Node.create<QuotedContentOptions>({
  name: 'quotedContent',
  group: 'block',
  content: 'block+',
  atom: false,
  draggable: false,
  selectable: false,
  isolating: true,

  addOptions() {
    return {
      ...this.parent?.(),
      HTMLAttributes: {},
      sort: 50,
      replyLabel: 'Reply',
      forwardedMessageLabel: 'Forwarded message',
      fromLabel: 'From',
      dateLabel: 'Date',
      subjectLabel: 'Subject',
      toLabel: 'To',
      wroteLabel: 'wrote',
      button: ({ editor, t }: ButtonViewParams<QuotedContentOptions>) => ({
        component: ActionButton,
        componentProps: {
          action: () => editor.chain().focus().toggleQuotedContent().run(),
          isActive: () => {
            const { state } = editor
            let foundQuoted = false

            state.doc.descendants((node: ProseMirrorNode) => {
              if (node.type.name === 'quotedContent') {
                foundQuoted = true
                return false
              }
              return true
            })

            return foundQuoted
          },
          disabled: !editor?.isEditable,
          icon: 'quote',
          tooltip: t('composer.quotedContent.tooltip'),
        },
      }),
      toolbar: true,
      divider: false,
      spacer: false,
    }
  },

  addAttributes() {
    return {
      type: {
        default: 'reply',
        parseHTML: element => element.getAttribute('data-type') || 'reply',
        renderHTML: attributes => ({
          'data-type': attributes.type,
        }),
      },
      originalFrom: {
        default: '',
        parseHTML: element => element.getAttribute('data-original-from') || '',
        renderHTML: attributes => ({
          'data-original-from': attributes.originalFrom,
        }),
      },
      originalDate: {
        default: '',
        parseHTML: element => element.getAttribute('data-original-date') || '',
        renderHTML: attributes => ({
          'data-original-date': attributes.originalDate,
        }),
      },
      originalSubject: {
        default: null,
        parseHTML: element => element.getAttribute('data-original-subject') || null,
        renderHTML: attributes => {
          if (attributes.originalSubject) {
            return { 'data-original-subject': attributes.originalSubject }
          }
          return {}
        },
      },
      originalTo: {
        default: null,
        parseHTML: element => element.getAttribute('data-original-to') || null,
        renderHTML: attributes => {
          if (attributes.originalTo) {
            return { 'data-original-to': attributes.originalTo }
          }
          return {}
        },
      },
    }
  },

  parseHTML() {
    return [
      { tag: 'div[data-quoted-content]' },
    ]
  },

  renderHTML({ HTMLAttributes, node }) {
    const attrs = mergeAttributes(
      { 'data-quoted-content': 'true' },
      this.options.HTMLAttributes,
      HTMLAttributes
    )

    const { type, originalFrom, originalDate, originalSubject, originalTo } = node.attrs

    if (type === 'reply') {
      return [
        'div',
        attrs,
        [
          'blockquote', { class: 'quoted-content-wrapper' },
          [
            'div', { class: 'quoted-content-header' },
            ['strong', {}, `${this.options.dateLabel}: ${originalDate}, ${originalFrom} ${this.options.wroteLabel}:`],
          ],
          ['div', { class: 'quoted-content-body' }, 0],
        ],
      ]
    } else {
      const headerContent = [
        ['div', { style: 'font-weight: bold; margin-bottom: 0.5em;' }, `---------- ${this.options.forwardedMessageLabel} ----------`],
        ['div', {}, ['strong', {}, `${this.options.fromLabel}: `], originalFrom],
        ['div', {}, ['strong', {}, `${this.options.dateLabel}: `], originalDate],
      ]

      if (originalSubject) {
        headerContent.push(['div', {}, ['strong', {}, `${this.options.subjectLabel}: `], originalSubject])
      }
      if (originalTo) {
        headerContent.push(['div', {}, ['strong', {}, `${this.options.toLabel}: `], originalTo])
      }

      return [
        'div',
        attrs,
        [
          'blockquote', { class: 'quoted-content-wrapper' },
          [
            'div', { class: 'quoted-content-header' },
            ...headerContent,
          ],
          ['div', { class: 'quoted-content-body' }, 0],
        ],
      ]
    }
  },

  addCommands() {
    return {
      setQuotedContent: (attributes: QuotedContentAttributes, content?: string) =>
        ({ commands }) => {
          if (!content) {
            return commands.insertContent({
              type: this.name,
              attrs: attributes,
              content: [{ type: 'paragraph' }],
            })
          }

          return commands.insertContent(`
            <div data-quoted-content="true" data-type="${attributes.type}" data-original-from="${attributes.originalFrom}" data-original-date="${attributes.originalDate}" ${attributes.originalSubject ? `data-original-subject="${attributes.originalSubject}"` : ''} ${attributes.originalTo ? `data-original-to="${attributes.originalTo}"` : ''}>
              <div class="quoted-content-wrapper">
                ${content.trim()}
              </div>
            </div>
          `)
        },

      toggleQuotedContent: () =>
        ({ state, chain, editor }) => {
          let hasQuoted = false
          let quotedPos: number | null = null
          let quotedSize: number | null = null
          let signaturePos: number | null = null

          state.doc.descendants((node: ProseMirrorNode, pos: number) => {
            if (node.type.name === this.name) {
              hasQuoted = true
              quotedPos = pos
              quotedSize = node.nodeSize
            }
            if (node.type.name === 'emailSignature') {
              signaturePos = pos
            }
            return true
          })

          if (hasQuoted && quotedPos !== null && quotedSize !== null) {
            return chain()
              .focus()
              .deleteRange({ from: quotedPos, to: quotedPos + quotedSize })
              .run()
          }

          const insertBeforeSignature = signaturePos !== null

          return chain()
            .command(({ tr, dispatch }) => {
              if (!dispatch) return false

              const paragraphNode = editor.schema.nodes.paragraph?.create()
              if (!paragraphNode) return false

              const quotedNode = editor.schema.nodes[this.name]?.create({
                type: 'reply',
                originalFrom: '',
                originalDate: new Date().toLocaleString(),
              }, [paragraphNode])

              if (!quotedNode) return false

              const insertPos = insertBeforeSignature ? signaturePos! : tr.doc.content.size
              tr.insert(insertPos, quotedNode)
              const $pos = tr.doc.resolve(insertPos)
              tr.setSelection(TextSelection.near($pos))

              return true
            })
            .run()
        },
    }
  },
})

