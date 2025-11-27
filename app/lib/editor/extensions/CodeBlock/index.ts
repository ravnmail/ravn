import { mergeAttributes, Node, textblockTypeInputRule  } from '@tiptap/core'
import { VueNodeViewRenderer } from '@tiptap/vue-3'
import type { GeneralOptions } from '@/types/composer'

import NodeView from './components/NodeView.vue'
import ActionButton from '~/components/ActionButton.vue'

export interface CodeBlockOptions extends GeneralOptions<CodeBlockOptions> { }

declare module '@tiptap/core' {
  interface Commands<ReturnType> {
    setCodeBlock: {
      setCodeBlock: (options?: any) => ReturnType
    }
  }
}



export const backtickInputRegex = /^```([a-z]+)?[\s\n]$/

export const tildeInputRegex = /^~~~([a-z]+)?[\s\n]$/


export const CodeBlock = Node.create({
  name: 'codeBlock',
  group: 'block',
  atom: true,
  content: 'text*',
  addAttributes() {
    return {
      vnode: {
        default: true,
      },
      code: {
        default: '',
        parseHTML: (element) => {
          return element.textContent || ''
        }
      },
      language: {
        default: 'plaintext',
      },
      lineNumbers: {
        default: true,
      },
      wordWrap: {
        default: false,
      },
      tabSize: {
        default: 2
      },
      shouldFocus: {
        default: true,
        parseHTML: () => false,
        renderHTML: false
      }
    }
  },
  parseHTML() {
    return [
      {
        tag: 'pre',
        preserveWhitespace: 'full',
        getAttrs: (node: HTMLElement) => {
          return {
            code: node.textContent || ''
          }
        }
      },
      {
        tag: 'pre code',
        preserveWhitespace: 'full',
        getAttrs: (node: HTMLElement) => {
          return {
            code: node.textContent || ''
          }
        }
      }
    ]
  },
  renderHTML({ HTMLAttributes, node }) {
    const code = node.attrs.code || node.content.firstChild?.text || ''
    return [
      'pre',
      mergeAttributes(this.options.HTMLAttributes, HTMLAttributes),
      ['code', {}, code]
    ]
  },
  addNodeView() {
    return VueNodeViewRenderer(NodeView)
  },
  addCommands() {
    return {
      setCodeBlock:
        (options) =>
          ({ commands }) => {
            return commands.insertContent({
              type: this.name,
              attrs: {
                ...options,
                shouldFocus: true
              },
            })
          },
    }
  },
  addOptions() {
    return {
      ...this.parent?.(),
      HTMLAttributes: {
        class: 'code-block',
        spellcheck: 'false',
        autocorrect: 'off',
        autocapitalize: 'off',
      },
      button: ({ editor, t }) => ({
        component: ActionButton,
        componentProps: {
          action: () => editor.commands.setCodeBlock({}),
          isActive: () => editor.isActive('codeBlock') || false,
          disabled: !editor.isEditable || !editor.can().setCodeBlock({}),
          icon: 'code',
          shortcutKeys: ['shift', 'mod', '5'],
          tooltip: t('composer.codeblock.tooltip'),
        },
      }),
    }
  },
  addInputRules() {
    return [
      textblockTypeInputRule({
        find: backtickInputRegex,
        type: this.type,
        getAttributes: match => ({
          language: match[1],
        }),
      }),
      textblockTypeInputRule({
        find: tildeInputRegex,
        type: this.type,
        getAttributes: match => ({
          language: match[1],
        }),
      }),
    ]
  },
})
