<script lang="ts" setup>
import { cn } from '@/lib/utils'
import { DropdownMenuLabel, type DropdownMenuLabelProps, useForwardProps } from 'reka-ui'
import { computed, type HTMLAttributes } from 'vue'

const props = defineProps<DropdownMenuLabelProps & { class?: HTMLAttributes['class'], inset?: boolean }>()

const delegatedProps = computed(() => {
  const { class: _, ...delegated } = props

  return delegated
})

const forwardedProps = useForwardProps(delegatedProps)
</script>

<template>
  <DropdownMenuLabel
    :class="cn('px-2 py-1.5 text-xs font-semibold text-muted', inset && 'pl-7', props.class)"
    v-bind="forwardedProps"
  >
    <slot />
  </DropdownMenuLabel>
</template>
