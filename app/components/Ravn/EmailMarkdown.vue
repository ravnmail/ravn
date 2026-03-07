<script lang="ts" setup>
import { convertFileSrc, invoke } from '@tauri-apps/api/core'
import { marked } from 'marked'

import type { Attachment } from '~/types/email'

const props = defineProps<{
  content: string
  imagesBlocked: boolean
  inlineAttachments?: Attachment[]
}>()

const stripImageSources = (markdown: string): string => {
  // Keep cid: links (handled by resolveCidInMarkdown) and already-resolved asset: URLs
  return markdown.replace(/!\s*\[([^\]]*)]\((?!cid:)(?!asset:)[^)]+\)\s*/g, '')
}

const resolveCidInMarkdown = (markdown: string): string => {
  if (!props.inlineAttachments?.length) return markdown

  const inlineAttachments = props.inlineAttachments.filter((a) => a.full_path)

  // Pass 1: resolve cid: references
  let result = markdown.replace(/!\[([^\]]*)\]\(cid:([^)]+)\)/gi, (_match, alt, contentId) => {
    const bare = contentId.replace(/^<|>$/g, '')
    const attachment = inlineAttachments.find(
      (a) => a.content_id === bare || a.content_id === `<${bare}>`
    )
    if (attachment?.full_path) {
      return `![${alt}](${convertFileSrc(attachment.full_path)})`
    }
    // No cached attachment found — remove the broken cid image
    return ''
  })

  // Pass 2: resolve bare filename references used by some email clients
  result = result.replace(/!\[([^\]]*)\]\(([^)]+)\)/gi, (_match, alt, src) => {
    // Skip already-resolved URLs and external links
    if (src.startsWith('asset:') || src.startsWith('http') || src.startsWith('data:')) {
      return _match
    }
    const srcFilename = src.split('/').pop()?.split('?')[0] ?? ''
    if (!srcFilename) return _match
    const attachment = inlineAttachments.find(
      (a) => a.filename.toLowerCase() === srcFilename.toLowerCase()
    )
    if (attachment?.full_path) {
      return `![${alt}](${convertFileSrc(attachment.full_path)})`
    }
    return _match
  })

  return result
}

const html = computed(() => {
  marked.use({
    renderer: {
      link: ({ href, text }) => {
        return `<a href="${href}" title="${href}" target="_blank" rel="noopener noreferrer">${text}</a>`
      },
    },
  })
  const cidResolved = resolveCidInMarkdown(props.content)
  const source = props.imagesBlocked ? stripImageSources(cidResolved) : cidResolved
  // Convert markdown images to HTML first, before parsing the rest of the markdown
  // This ensures linked images like [![img](src)](url) are properly rendered
  const withHtmlImages = source.replace(/!\[([^\]]*)\]\(([^)]+)\)/g, '<img src="$2" alt="$1" />')
  return marked.parse(withHtmlImages) as string
})

const handleClick = (event: MouseEvent) => {
  const target = event.target as HTMLElement
  if (target.tagName === 'A') {
    event.preventDefault()
    const url = target.getAttribute('href')
    if (url) {
      invoke('open_external_url', { url }).catch((err) => {
        console.error('Failed to open external URL:', err)
      })
    }
  }
}
</script>

<template>
  <div
    class="EmailView select-auto"
    @click="handleClick"
    v-html="html"
  />
</template>
