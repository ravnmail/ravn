import { invoke } from '@tauri-apps/api/core'
import type { AnyExtension } from '@tiptap/core'
import { Extension } from '@tiptap/core'
import { Dropcursor } from '@tiptap/extension-dropcursor'
import { Gapcursor } from '@tiptap/extension-gapcursor'
import { HardBreak } from '@tiptap/extension-hard-break'
import { Highlight } from '@tiptap/extension-highlight'
import { ListItem } from '@tiptap/extension-list-item'
import { Paragraph } from '@tiptap/extension-paragraph'
import { Text } from '@tiptap/extension-text'
import AutoJoiner from 'tiptap-extension-auto-joiner'

import { defaultBubbleList, generateBubbleTypeMenu } from '../menus/BasicBubble'
import { AI } from './AI'
import { Autocomplete } from './Autocomplete'
import { Blockquote } from './Blockquote'
import { Bold } from './Bold'
import { BulletList } from './BulletList'
import { Callout } from './Callout'
import { Code } from './Code'
import { CodeBlock } from './CodeBlock'
import { Document } from './Document'
import { EmailSignature } from './EmailSignature'
import { Heading } from './Heading'
import { HighlightParagraph } from './HighlightParagraph'
import { History } from './History'
import { HorizontalRule } from './HorizontalRule'
import { Indent } from './Indent'
import { Italic } from './Italic'
import { Link } from './Link'
import { MarkdownPaste } from './MarkdownPaste'
import { OrderedList } from './OrderedList'
import { Placeholder } from './Placeholder'
import { QuotedContent } from './QuotedContent'
import { Selection } from './Selection'
import { SlashCommand } from './SlashCommand'
import { Strike } from './Strike'
import { TaskList } from './TaskList'
import { TextBubble } from './TextBubble'
import { TrailingNode } from './TrailingNode'
import { Underline } from './UnderLine'

const { settings } = useSettings()

export interface MailKitEmailContext {
  sender: () => string
  subject: () => string
  isReply: () => boolean
  recipients: () => string[]
  priorEmail: () => string | undefined
  contactNotes: () => Array<{ email: string; display_name?: string | null; notes: string }>
}

export type MailKitOptions = {
  emailContext?: MailKitEmailContext
}

export const MailKit = Extension.create({
  name: 'mail-kit',
  addOptions() {
    return {
      ...this.parent?.(),
      bubble: {
        list: {
          text: [
            'AI',
            'divider',
            'text-bubble',
            'divider',
            'bold',
            'italic',
            'underline',
            'strike',
            'code',
            'link',
          ],
        },
        defaultBubbleList,
        button: ({ editor, extension, t }) => {
          const { list = {}, defaultBubbleList } = extension.options?.bubble ?? {}
          const defaultList = defaultBubbleList?.(editor) ?? []

          return generateBubbleTypeMenu(list, defaultList, {
            editor,
            extension,
            t,
          })
        },
      },
      link: {
        HTMLAttributes: {
          target: '_blank',
          rel: 'noopener noreferrer nofollow',
        },
        openOnClick: false,
      },
    }
  },

  addExtensions() {
    const { t } = useI18n()

    const extensions: AnyExtension[] = [
      Placeholder.configure({
        placeholder: ({ node }) => {
          const nodeTypeName = node?.type?.name
          if (nodeTypeName === 'heading') {
            return t(`composer.placeholders.h${node.attrs.level}`)
          }
          if (node.type.name === 'codeBlock') {
            return t('composer.placeholders.code')
          }
          if (
            nodeTypeName === 'table' ||
            nodeTypeName === 'bulletList' ||
            nodeTypeName === 'orderedList' ||
            nodeTypeName === 'taskList' ||
            nodeTypeName === 'listItem'
          ) {
            return ''
          }

          return t('composer.placeholders.default')
        },
        ...this.options.placeholder,
      }),
      Document,
      Text,
      Gapcursor,
      Dropcursor.configure({
        width: 2,
        color: '#99B9B9B',
        class: 'ProseMirror-dropcursor border-black',
      }),
      Paragraph,
      HardBreak,
      ListItem,
      TrailingNode,
      History,

      Bold,
      Italic,
      Underline,
      Strike,
      Code,

      Heading,
      TextBubble,

      Link,

      BulletList,
      OrderedList,
      TaskList,

      Blockquote,
      CodeBlock,
      QuotedContent.configure({
        replyLabel: t('composer.quotedContent.reply'),
        forwardedMessageLabel: t('composer.quotedContent.forwardedMessage'),
        fromLabel: t('composer.quotedContent.from'),
        dateLabel: t('composer.quotedContent.date'),
        subjectLabel: t('composer.quotedContent.subject'),
        toLabel: t('composer.quotedContent.to'),
        wroteLabel: t('composer.quotedContent.wrote'),
      }),

      HighlightParagraph,
      SlashCommand,
      MarkdownPaste,

      Selection,
      // Markdown.configure({
      //   html: false,
      //   transformCopiedText: true,
      // }),
      Highlight,
      HorizontalRule,
      Indent,
      AI.configure({
        completions: async (history, signal) => {
          console.log('AI completions called with history:', { ...history })
          const result = await invoke('ask_ai', {
            context: { history },
          })

          console.log('AI completions result:', result)

          return result
        },
        shortcuts: [],
      }),
      AutoJoiner.configure({
        elementsToJoin: ['blockquote', 'codeBlock', 'bulletList', 'orderedList'],
      }),
      Callout.configure(),
      EmailSignature.configure({
        renderHTML: (signatureId) => {
          if (!signatureId) {
            signatureId = settings.value?.signatures.globalDefault || null
          }
          if (signatureId) {
            return (
              settings.value?.signatures.items.find(({ id }) => id === signatureId)?.content || ''
            )
          }
          return ''
        },
      }),
    ]

    if (settings.value.ai.autoCompletion.enabled) {
      const emailCtx = this.options.emailContext
      extensions.push(
        Autocomplete.configure({
          ...settings.value.ai.autoCompletion,
          ...(emailCtx
            ? {
                emailMetadata: () => ({
                  sender: emailCtx.sender(),
                  subject: emailCtx.subject(),
                  is_reply: emailCtx.isReply(),
                  recipients: emailCtx.recipients(),
                }),
                contactNotes: emailCtx.contactNotes,
                priorEmail: emailCtx.priorEmail,
              }
            : {}),
        })
      )
    }

    return extensions
  },
})
