<script lang="ts" setup>
import type { HTMLAttributes } from 'vue'
import { cn } from '@/lib/utils'

const props = defineProps<{
  class?: HTMLAttributes['class']
}>()
</script>

<template>
  <thead :class="cn('[&_tr]:border-b [&_tr]:border-b-border bg-background', props.class)">
    <slot />
  </thead>
</template>
