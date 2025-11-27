import type { AnyExtension } from '@tiptap/core'
import { Extension } from '@tiptap/core'
import { invoke } from '@tauri-apps/api/core'

import { Highlight } from '@tiptap/extension-highlight'
import AutoJoiner from 'tiptap-extension-auto-joiner'
import { Text } from '@tiptap/extension-text'
import { Paragraph } from '@tiptap/extension-paragraph'
import { Gapcursor } from '@tiptap/extension-gapcursor'
import { Dropcursor } from '@tiptap/extension-dropcursor'
import { HardBreak } from '@tiptap/extension-hard-break'
import { ListItem } from '@tiptap/extension-list-item'

import { TrailingNode } from './TrailingNode'

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
import { Selection } from './Selection'
import { SlashCommand } from './SlashCommand'
import { Strike } from './Strike'
import { TaskList } from './TaskList'
import { TextBubble } from './TextBubble'
import { Underline } from './UnderLine'

import { defaultBubbleList, generateBubbleTypeMenu } from '../menus/BasicBubble'

export type MailKitOptions = {}

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
            'link'
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
          if (nodeTypeName === 'table' || nodeTypeName === 'bulletList' || nodeTypeName === 'orderedList' || nodeTypeName === 'taskList' || nodeTypeName === 'listItem') {
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
          console.log('AI completions called with history:', {...history})
          const result = await invoke('ask_ai', {
            context: { history },
          })

          console.log('AI completions result:', result)

          return result
        },
        shortcuts: []
      }),
      // Autocomplete,
      AutoJoiner.configure({
        elementsToJoin: [
          'blockquote',
          'codeBlock',
          'bulletList',
          'orderedList',
        ],
      }),
      Callout.configure(),
      EmailSignature.configure({
        renderHTML: () => {
          return `<br><table cellpadding="0"cellspacing="0"style="border-collapse:collapse;line-height:1.15"><tr><td style="vertical-align:top;padding:.01px 14px .01px 1px;width:65px;text-align:center"><p style="margin:1px"><img alt="Michael Wallner"border="0"height="48"src="data:image/png;base64,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"style="width:48px;vertical-align:middle;border-radius:50%;height:48px;border:0;display:block"width="48"><td style="padding:.01px .01px .01px 14px;vertical-align:top;border-left:solid 1px #c2c9d6"valign="top"><table cellpadding="0"cellspacing="0"style="border-collapse:collapse"><tr><td style="padding:.01px"><p style="margin:.1px;line-height:120%;font-size:16px"><span style="font-family:Trebuchet MS;font-size:16px;font-weight:700;color:#646464;letter-spacing:0;white-space:nowrap">Michael Wallner</span><br><span style="font-family:Trebuchet MS;font-size:13px;font-weight:700;color:#646464;white-space:nowrap">Founder, Coder's Cantina</span><tr><td><table cellpadding="0"cellspacing="0"style="border-collapse:collapse"><tr><td style="padding-top:14px;white-space:nowrap;width:265px;font-family:Trebuchet MS"nowrap width="265"><p style="margin:1px;line-height:99%;font-size:11px"><span style="color:#ff1744;font-family:Trebuchet MS;font-size:11px;font-weight:700">P</span>  <a href="tel:+4369919166275"target="_blank"rel="nofollow noreferrer"style="font-family:Trebuchet MS;text-decoration:unset"><span style="line-height:120%;font-family:Trebuchet MS;font-size:11px;color-scheme:light only;color:#121314;white-space:nowrap">+43 699 19166275</span> </a>   <span style="color:#ff1744;font-family:Trebuchet MS;font-size:11px;font-weight:700">W</span>  <a href="https://www.coderscantina.com"target="_blank"rel="nofollow noreferrer"style="font-family:Trebuchet MS;text-decoration:unset"><span style="line-height:120%;font-family:Trebuchet MS;font-size:11px;color-scheme:light only;color:#121314;white-space:nowrap">coderscantina.com</span></a><tr><td style="padding-top:8px;white-space:nowrap;width:152px;font-family:Trebuchet MS"nowrap width="152"><p style="margin:1px;line-height:99%;font-size:11px"><span style="color:#ff1744;font-family:Trebuchet MS;font-size:11px;font-weight:700">E</span>  <a href="mailto:m.wallner@coderscantina.com"target="_blank"rel="nofollow noreferrer"style="font-family:Trebuchet MS;text-decoration:unset"><span style="line-height:120%;font-family:Trebuchet MS;font-size:11px;color-scheme:light only;color:#121314;white-space:nowrap">m.wallner@coderscantina.com</span></a><tr><td style="color:#121314;font-size:11px;mso-line-height-rule:exactly;line-height:15px;padding-top:10px;font-family:Trebuchet MS,sans-serif"><p style="margin:.1px"><span style="color:#121314;font-size:11px">Coder's Cantina e.U.<br></span><span style="color:#121314;font-size:11px">Wehlistraße 291/1/47, 1020 Vienna, Austria<br></span><span style="color:#121314;font-size:11px"><a href="mailto:info@neonblack.at"target="_blank"style="color:#a2a4a5;text-decoration:none"><span style="color:#121314;text-decoration:none">hello@coderscantina.com</span> </a><span>•</span> </span><span style="color:#121314;font-size:11px"><a href="https://www.coderscantina.com/?mtm_campaign=email-signature&mtm_kwd=website"target="_blank"style="color:#121314;text-decoration:none"><span style="color:#121314;text-decoration:none">coderscantina.com</span></a><br></span><span style="color:#121314;font-size:11px">LG Wien <span>•</span> </span><span style="color:#121314;font-size:11px">FN 392184t</span></table><tr><td style="padding:14px .01px .01px .01px"><table cellpadding="0"cellspacing="0"border="0"><tr><td style="padding-right:6px;text-align:center;padding-top:0"align="left"><p style="margin:1px"><a href="https://github.com/badmike"target="_blank"rel="nofollow noreferrer"><img alt="github"border="0"height="24"src="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADAAAAAwCAAAAAByaaZbAAAAcElEQVR42u3UMQ7AIAwDQH/Fr/Pv/EEv3arShaSiEgM3sViJEhQcx0bs4T2XGJIl0gnm8mTMuRlQBqwU6JXIy/IA8/J7S+6OVasXx7GGal+VcpJYKIjvtowKxgBTD8CJhxU0poQqshkAPgbOQTy2cgEpt/K3WSCnlQAAAABJRU5ErkJggg=="style="float:left;border:none"width="24"></a><td style="padding-right:6px;text-align:center;padding-top:0"align="left"><p style="margin:1px"><a href="https://www.youtube.com/@coderscantina"target="_blank"rel="nofollow noreferrer"><img alt="youtube"border="0"height="24"src="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADAAAAAwCAMAAABg3Am1AAAAD1BMVEXNIB/WSknnlJT10ND//f1UxIetAAAAW0lEQVR42u3SQQoAMQhDUZN4/zMPncWATCvN3rf2U8HGGOMEpJZU6kWinVf+qCuYG2wC5YZuAl0G30xZ7ioIyAxW4gWg9wK9lcoF7TvYgf81vILRAlgAMcY4eQDMJwS2lw27BAAAAABJRU5ErkJggg=="style="float:left;border:none"width="24"></a><td style="padding-right:6px;text-align:center;padding-top:0"align="left"><p style="margin:1px"><a href="https://www.linkedin.com/in/neonblack-mwallner/"target="_blank"rel="nofollow noreferrer"><img alt="linkedin"border="0"height="24"src="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADAAAAAwCAMAAABg3Am1AAAAD1BMVEUAd7WDvdsiib/3+v1PocxIfhJ9AAAAWElEQVR42u3QMQrAMAxDUUnW/c/chKZktaGBFvwWefmL0dpfkRQKGAMKYtLJwIwwTpKHNYJIKvGlNWZMVDLYyoGzAeW7SAYEoEpgDB28FbASQMOe52rtEy5PagLw81B0VAAAAABJRU5ErkJggg=="style="float:left;border:none"width="24"></a><td style="padding-right:6px;text-align:center;padding-top:0"align="left"><p style="margin:1px"><a href="https://twitter.com/_mwallner"target="_blank"rel="nofollow noreferrer"><img alt="twitter"border="0"height="24"src="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADAAAAAwCAAAAAByaaZbAAAAAXNSR0IB2cksfwAAAAlwSFlzAAALEwAACxMBAJqcGAAAAIpJREFUeNrtlDEOxTAIQ7kTl/PBfDevX+JL0KqqgK1D3sDkED05ih0O30ABrHDJ7R2KAK8RypsrMjTKm4sxMg9rgBAj8y2U/0eeSxqNYb40kCZDjcyvNIpxGyuNxYFsY4a005B8pUH5XKPWYqpBYdUGhVKJ0T6LexsmdvkCIsB3jec6Kjif4+Eb/AD67n6HtlcR3wAAAABJRU5ErkJggg=="style="float:left;border:none"width="24"></a></table><tr><td style="padding:14px .01px .01px .01px"><p style="margin:.1px;width:350px;font-size:13px;color:#121314"><span style="font-family:Trebuchet MS"><b>Ready to elevate your digital project?</b><br><a href="https://outlook.office.com/bookwithme/user/7bdfe44fc61740fa9378c7fecbc2a7ab@coderscantina.com?anonymous&ep=plink"target="_blank"style="color:#ff1744">Book a free conversation with me</a> — I'm just a click away to explore possibilities and solve challenges together. Book your spot and let's create something exceptional!</span></table></table>`
        },
      }),
    ]

    return extensions
  },
})
