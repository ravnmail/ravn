import type { Editor } from '@tiptap/core'
import { Node, mergeAttributes } from '@tiptap/core'
import type { Transaction } from '@tiptap/pm/state'
import { computed, defineComponent, h } from 'vue'
import ActionDropdownButtonSplit from '~/components/ActionDropdownButtonSplit.vue'
import {
  DropdownMenuRadioGroup,
  DropdownMenuRadioItem,
  DropdownMenuSeparator
} from '~/components/ui/dropdown-menu'
import { useSettings } from '~/composables/useSettings'

export type SignatureId = string | null;

export interface EmailSignatureOptions {
  HTMLAttributes: Record<string, unknown>;
  renderHTML: ((signatureId: SignatureId) => string) | null;
}

export interface EmailSignatureAttributes {
  signatureId: SignatureId;
}

declare module '@tiptap/core' {
  interface Commands<ReturnType> {
    emailSignature: {
      insertEmailSignature: (
        attributes?: EmailSignatureAttributes,
      ) => ReturnType;
      toggleEmailSignature: (
        signatureId: SignatureId,
      ) => ReturnType;
    };
  }
}

export const EmailSignature = Node.create<EmailSignatureOptions>({
  name: 'emailSignature',
  group: 'block',
  atom: true,
  draggable: true,
  selectable: true,
  inline: false,
  marks: '',
  isolating: true,
  content: '',

  addOptions() {
    return {
      HTMLAttributes: {},
      renderHTML: null,
      button: ({ editor, t }: { editor: Editor; t: (key: string) => string }) => {
        const { settings } = useSettings()
        const signatures = computed(() => settings.value?.signatures?.items || [])
        const globalDefault = computed(() => settings.value?.signatures?.globalDefault)

        const findSignaturePosition = (doc: {
          descendants: (fn: (node: { type: { name: string } }, pos: number) => boolean) => void
        }): number | null => {
          let position: number | null = null
          doc.descendants((node, pos) => {
            if (node.type.name === 'emailSignature') {
              position = pos
              return false
            }
            return true
          })
          return position
        }

        const getCurrentSignatureId = (): SignatureId => {
          let currentId: SignatureId = null
          editor.state.doc.descendants((node) => {
            if (node.type.name === 'emailSignature') {
              currentId = node.attrs.signatureId
              return false
            }
            return true
          })
          return currentId
        }

        const hasSignature = () => findSignaturePosition(editor.state.doc) !== null

        const removeSignature = () => {
          const pos = findSignaturePosition(editor.state.doc)
          if (pos !== null) {
            editor.chain().focus().deleteRange({ from: pos, to: pos + 1 }).run()
          }
        }

        const toggleSignature = (signatureId?: string) => {
          const idToUse = signatureId || globalDefault.value
          const currentId = getCurrentSignatureId()

          if (currentId === idToUse) {
            removeSignature()
          } else if (idToUse) {
            editor.chain().focus().toggleEmailSignature(idToUse).run()
          } else if (hasSignature()) {
            removeSignature()
          }
        }

        const selectSignature = (signatureId: SignatureId) => {
          const currentId = getCurrentSignatureId()

          if (signatureId === null || currentId === signatureId) {
            removeSignature()
          } else {
            editor.chain().focus().toggleEmailSignature(signatureId).run()
          }
        }

        return {
          component: ActionDropdownButtonSplit,
          componentProps: {
            action: () => toggleSignature(),
            disabled: !editor?.isEditable,
            icon: 'signature',
            shortcutKeys: ['mod', 'Shift', 'S'],
            tooltip: t('composer.signature.tooltip'),
          },
          componentSlots: {
            default: defineComponent({
              setup() {
                const currentSignatureId = computed(() => getCurrentSignatureId())

                return () => h(DropdownMenuRadioGroup, {
                  modelValue: currentSignatureId.value,
                  'onUpdate:modelValue': (value: SignatureId) => {
                    selectSignature(value)
                  },
                }, [
                  ...signatures.value.map((sig) =>
                    h(DropdownMenuRadioItem, {
                      key: sig.id,
                      value: sig.id,
                    }, {
                      default: () => [
                        sig.title,
                      ]
                    })
                  ),
                  h(DropdownMenuSeparator),
                  h(DropdownMenuRadioItem, {
                    value: null,
                  }, {
                    default: () => [
                      t('composer.signature.none'),
                    ]
                  }),
                ])
              },
            }),
          },
        }
      },
    }
  },

  addAttributes() {
    return {
      signatureId: {
        default: null,
      },
    }
  },

  parseHTML() {
    return [
      {
        tag: 'div[data-type="email-signature"]',
      },
    ]
  },

  addCommands() {
    const handleEmailSignature = (attributes?: EmailSignatureAttributes) =>
      ({
         tr,
         dispatch,
         editor,
       }: {
        tr: Transaction;
        dispatch: ((transaction: Transaction) => void) | undefined;
        editor: Editor;
      }) => {
        if (!dispatch) {
          return false
        }

        const signatureNode = editor.schema.nodes[this.name]?.create(attributes)
        if (!signatureNode) return false

        let signaturePos: number | null = null
        let quotedContentPos: number | null = null

        tr.doc.descendants((node, pos) => {
          if (node.type.name === this.name) {
            signaturePos = pos
          }
          if (node.type.name === 'quotedContent') {
            quotedContentPos = pos
          }
          return true
        })

        if (signaturePos !== null) {
          tr.delete(signaturePos, signaturePos + 1)
          if (quotedContentPos !== null && quotedContentPos > signaturePos) {
            quotedContentPos = quotedContentPos - 1
          }
        }

        const insertPos = quotedContentPos ?? tr.doc.content.size
        tr.insert(insertPos, signatureNode)

        return true
      }

    return {
      insertEmailSignature: (attributes?: EmailSignatureAttributes) =>
        handleEmailSignature(attributes || { signatureId: null }),
      toggleEmailSignature: (signatureId: SignatureId) =>
        handleEmailSignature({ signatureId }),
    }
  },

  renderHTML({ HTMLAttributes }) {
    const { renderHTML } = this.options
    const { signatureId } = HTMLAttributes

    if (typeof renderHTML === 'function') {
      const wrapper = document.createElement('div')
      wrapper.setAttribute('data-type', 'email-signature')

      Object.entries(HTMLAttributes).forEach(([key, value]) => {
        if (value !== null && value !== undefined) {
          wrapper.setAttribute(key, String(value))
        }
      })

      wrapper.innerHTML = renderHTML(signatureId || null)

      return wrapper
    }

    return [
      'div',
      mergeAttributes({ 'data-type': 'email-signature' }, HTMLAttributes),
      '',
    ]
  },
})
