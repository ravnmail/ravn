<script lang="ts" setup>
import type { StepperIndicatorProps } from 'reka-ui'
import { cn } from '@/lib/utils'
import { StepperIndicator, useForwardProps } from 'reka-ui'

import { computed, type HTMLAttributes } from 'vue'

const props = defineProps<StepperIndicatorProps & { class?: HTMLAttributes['class'] }>()

const delegatedProps = computed(() => {
  const { class: _, ...delegated } = props

  return delegated
})

const forwarded = useForwardProps(delegatedProps)
</script>

<template>
  <StepperIndicator
    :class="cn(
      'inline-flex items-center justify-center rounded-full text-muted w-6 h-6',
      // Disabled
      'group-data-disabled:text-muted group-data-disabled:opacity-50',
      // Active
      'group-data-[state=active]:bg-selection group-data-[state=active]:text-selection-foreground',
      // Completed
      'group-data-[state=completed]:bg-primary group-data-[state=completed]:text-primary-foreground',
      props.class,
    )"
    v-bind="forwarded"
  >
    <slot />
  </StepperIndicator>
</template>
