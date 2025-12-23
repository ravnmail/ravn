<script lang="ts" setup>
import type { NumberFieldIncrementProps } from 'reka-ui'
import { cn } from '@/lib/utils'
import { NumberFieldIncrement, useForwardProps } from 'reka-ui'
import { computed, type HTMLAttributes } from 'vue'

const props = defineProps<NumberFieldIncrementProps & { class?: HTMLAttributes['class'] }>()

const delegatedProps = computed(() => {
  const { class: _, ...delegated } = props

  return delegated
})

const forwarded = useForwardProps(delegatedProps)
</script>

<template>
  <NumberFieldIncrement
    :class="cn('flex absolute top-1/2 -translate-y-1/2 right-0 disabled:cursor-not-allowed disabled:opacity-20 py-1.5 px-3', props.class)"
    data-slot="increment"
    v-bind="forwarded"
  >
    <slot>
      <Icon name="lucide:plus"/>
    </slot>
  </NumberFieldIncrement>
</template>
