<script lang="ts" setup>
import { cn } from '@/lib/utils'
import { ScrollAreaScrollbar, type ScrollAreaScrollbarProps, ScrollAreaThumb } from 'reka-ui'
import { computed, type HTMLAttributes } from 'vue'

const props = withDefaults(defineProps<ScrollAreaScrollbarProps & { class?: HTMLAttributes['class'] }>(), {
  orientation: 'vertical',
})

const delegatedProps = computed(() => {
  const { class: _, ...delegated } = props

  return delegated
})
</script>

<template>
  <ScrollAreaScrollbar
    :class="
      cn('flex touch-none select-none transition-colors',
         orientation === 'vertical'
           && 'h-full w-1.5 bg-scrollbar-track/10 rounded-full mr-0.5',
         orientation === 'horizontal'
           && 'h-1.5 flex-col bg-scrollbar-track/10 rounded-full mb-0.5',
         props.class)"
    v-bind="delegatedProps"
  >
    <ScrollAreaThumb class="relative flex-1 rounded-full bg-scrollbar-thumb/20"/>
  </ScrollAreaScrollbar>
</template>
