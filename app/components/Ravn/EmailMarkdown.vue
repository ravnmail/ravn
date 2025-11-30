<script lang="ts" setup>
import { marked } from 'marked'
import { invoke } from '@tauri-apps/api/core'

const props = defineProps<{
  content: string
  imagesBlocked: boolean
}>()

const stripImageSources = (markdown: string): string => {
  return markdown.replace(/!\s*\[([^\]]*)]\((?!cid:)[^)]+\)\s*/g, '')
}

const html = computed(() => {
  const source = props.imagesBlocked ? stripImageSources (props.content) : props.content
  return marked.parse(source) as string
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
    class="ProseMirror select-auto"
    @click="handleClick"
    v-html="html"
  />
</template>