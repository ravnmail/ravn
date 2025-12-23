<script lang="ts" setup>
import type { NumberFieldDecrementProps } from 'reka-ui'
import { cn } from '@/lib/utils'
import { NumberFieldDecrement, useForwardProps } from 'reka-ui'
import { computed, type HTMLAttributes } from 'vue'

const props = defineProps<NumberFieldDecrementProps & { class?: HTMLAttributes['class'] }>()

const delegatedProps = computed(() => {
  const { class: _, ...delegated } = props

  return delegated
})

const forwarded = useForwardProps(delegatedProps)
</script>

<template>
  <NumberFieldDecrement
    :class="cn('absolute flex top-1/2 -translate-y-1/2 left-0 px-3 py-1.5 disabled:cursor-not-allowed disabled:opacity-20', props.class)"
    data-slot="decrement"
    v-bind="forwarded"
  >
    <slot>
      <Icon name="lucide:minus"/>
    </slot>
  </NumberFieldDecrement>
</template>
