import { invoke } from '@tauri-apps/api/core'
import type { AnyExtension, Editor } from '@tiptap/core'
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

export interface MailKitEmailContext {
  sender: () => string
  subject: () => string
  isReply: () => boolean
  recipients: () => string[]
  priorEmail: () => string | undefined
  contactNotes: () => Array<{ email: string; display_name?: string | null; notes: string }>
}

export type MailKitSettings = {
  signatures?: {
    globalDefault?: string | null
    items?: Array<{
      id: string
      content: string
    }>
  }
  ai?: {
    autoCompletion?: {
      enabled?: boolean
      [key: string]: any
    }
  }
}

export type MailKitOptions = {
  emailContext?: MailKitEmailContext
  settings?: MailKitSettings
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
        button: ({ editor, extension, t }: { editor: Editor; extension: any; t: any }) => {
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
            return String(t(`composer.placeholders.h${node.attrs.level}`) ?? '')
          }
          if (node.type.name === 'codeBlock') {
            return String(t('composer.placeholders.code') ?? '')
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

          return String(t('composer.placeholders.default') ?? '')
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
        replyLabel: String(t('composer.quotedContent.reply') ?? ''),
        forwardedMessageLabel: String(t('composer.quotedContent.forwardedMessage') ?? ''),
        fromLabel: String(t('composer.quotedContent.from') ?? ''),
        dateLabel: String(t('composer.quotedContent.date') ?? ''),
        subjectLabel: String(t('composer.quotedContent.subject') ?? ''),
        toLabel: String(t('composer.quotedContent.to') ?? ''),
        wroteLabel: String(t('composer.quotedContent.wrote') ?? ''),
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
        completions: async (history: any, signal: AbortSignal | undefined): Promise<any> => {
          console.log('AI completions called with history:', { ...history })
          const result = await invoke<any>('ask_ai', {
            context: { history },
          })

          console.log('AI completions result:', result)

          return result as any
        },
        shortcuts: [],
      } as any),
      AutoJoiner.configure({
        elementsToJoin: ['blockquote', 'codeBlock', 'bulletList', 'orderedList'],
      }),
      Callout.configure(),
      EmailSignature.configure({
        renderHTML: (signatureId) => {
          const mailSettings = this.options.settings
          if (!signatureId) {
            signatureId = mailSettings?.signatures?.globalDefault || null
          }
          if (signatureId) {
            return (
              mailSettings?.signatures?.items?.find(({ id }: { id: string }) => id === signatureId)
                ?.content || ''
            )
          }
          return ''
        },
      }),
    ]

    if (this.options.settings?.ai?.autoCompletion?.enabled) {
      const emailCtx = this.options.emailContext
      extensions.push(
        Autocomplete.configure({
          ...this.options.settings.ai.autoCompletion,
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
