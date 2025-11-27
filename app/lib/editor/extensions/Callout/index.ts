import { Node, mergeAttributes } from '@tiptap/core'
import { VueNodeViewRenderer } from '@tiptap/vue-3'
import CalloutComponent from './components/Callout.vue'
import ActionButton from '~/components/ActionButton.vue'


export type CalloutType = 'info' | 'success' | 'warning' | 'error' | 'note'

export interface CalloutOptions {
  HTMLAttributes: Record<string, any>
}

export interface CalloutAttributes {
  emoji: string
  type: CalloutType
}

declare module '@tiptap/core' {
  interface Commands<ReturnType> {
    callout: {
      setCallout: (attributes?: Partial<CalloutAttributes>) => ReturnType
      toggleCallout: (attributes?: Partial<CalloutAttributes>) => ReturnType
      updateCalloutEmoji: (emoji: string) => ReturnType
      updateCalloutType: (type: CalloutType) => ReturnType
    }
  }
}

export const Callout = Node.create<CalloutOptions>({
  name: 'callout',
  group: 'block',
  content: 'block+',
  defining: true,
  isolating: true,
  draggable: true,

  addOptions() {
    return {
      HTMLAttributes: {},
      button: ({ editor, t }) => ({
        component: ActionButton,
        componentProps: {
          action: () => editor.commands.toggleCallout(),
          disabled: !editor?.isEditable,
          icon: 'megaphone',
          tooltip: t('composer.callout.tooltip'),
        },
      }),
    }
  },

  parseHTML() {
    return [
      {
        tag: 'div[data-type="callout"]',
      },
    ]
  },

  renderHTML({ HTMLAttributes }) {
    return ['div', mergeAttributes(this.options.HTMLAttributes, HTMLAttributes, { 'data-type': 'callout' }), 0]
  },

  addAttributes() {
    return {
      emoji: {
        default: '💡',
        parseHTML: element => element.getAttribute('data-emoji'),
        renderHTML: attributes => {
          return {
            'data-emoji': attributes.emoji,
          }
        },
      },
      type: {
        default: 'info' as CalloutType,
        parseHTML: element => element.getAttribute('data-callout-type') as CalloutType,
        renderHTML: attributes => {
          return {
            'data-callout-type': attributes.type,
          }
        },
      }
    }
  },

  addNodeView() {
    return VueNodeViewRenderer(CalloutComponent)
  },

  addCommands() {
    return {
      setCallout: (attributes = {}) => ({ commands }) => {
        return commands.setNode(this.name, attributes)
      },
      toggleCallout: (attributes = {}) => ({ commands, chain }) => {
        return commands.toggleWrap(this.name, attributes)
      },
      updateCalloutEmoji: (emoji: string) => ({ tr, state, dispatch }) => {
        const { selection } = state
        const position = selection.$anchor.pos

        // Find the callout node position
        let calloutPos: number | null = null
        let calloutNode = null

        state.doc.nodesBetween(position, position, (node, pos) => {
          if (node.type.name === this.name) {
            calloutPos = pos
            calloutNode = node
            return false // stop searching
          }
          return true
        })

        if (calloutPos === null) return false

        if (dispatch) {
          tr.setNodeMarkup(calloutPos, undefined,
            { ...calloutNode.attrs, emoji }
          )
        }

        return true
      },
      updateCalloutType: (type: CalloutType) => ({ tr, state, dispatch }) => {
        const { selection } = state
        const position = selection.$anchor.pos

        // Find the callout node position
        let calloutPos: number | null = null
        let calloutNode = null

        state.doc.nodesBetween(position, position, (node, pos) => {
          if (node.type.name === this.name) {
            calloutPos = pos
            calloutNode = node
            return false // stop searching
          }
        })

        if (calloutPos === null) return false

        if (dispatch) {
          tr.setNodeMarkup(calloutPos, undefined,
            { ...calloutNode.attrs, type }
          )
        }

        return true
      }
    }
  },
})