<script lang="ts" setup>
import type { HTMLAttributes } from 'vue'
import { cn } from '@/lib/utils'

const props = defineProps<{
  class?: HTMLAttributes['class']
}>()
</script>

<template>
  <tr :class="cn('border-b border-b-border transition-colors hover:bg-selection/20 data-[state=selected]:bg-selection focus:outline-none focus:ring-1 focus:ring-ring', props.class)">
    <slot />
  </tr>
</template>
