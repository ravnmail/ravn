<script lang="ts" setup>

import { marked } from 'marked'
import { invoke } from '@tauri-apps/api/core'

const props = defineProps<{ content: string }>()

const html = computed(() => {
  return marked.parse(props.content)
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
    class="prose"
    @click="handleClick"
    v-html="html"
  />
</template>