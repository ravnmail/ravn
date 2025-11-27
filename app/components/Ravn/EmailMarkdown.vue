<script lang="ts" setup>
import { marked } from 'marked'
import { invoke } from '@tauri-apps/api/core'

const props = defineProps<{
  content: string
  imagesBlocked: boolean
}>()

const stripImageSources = (html: string): string => {
  const doc = new DOMParser().parseFromString(html, 'text/html')

  const images = doc.querySelectorAll('img')
  images.forEach((img) => {
    const src = img.getAttribute('src') || ''
    if (src && !src.startsWith('data:') && !src.startsWith('/')) {
      img.removeAttribute('src')
      img.setAttribute('alt', img.getAttribute('alt') || '[Image]')
    }
  })

  return doc.body.innerHTML
}

const html = computed(() => {
  const rendered = marked.parse(props.content) as string
  return props.imagesBlocked ? stripImageSources(rendered) : rendered
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