<script lang="ts" setup>
import { cn } from '@/lib/utils'
import { SelectIcon, SelectTrigger, type SelectTriggerProps, useForwardProps } from 'reka-ui'
import { computed, type HTMLAttributes } from 'vue'

const props = defineProps<SelectTriggerProps & { class?: HTMLAttributes['class'] }>()

const delegatedProps = computed(() => {
  const { class: _, ...delegated } = props

  return delegated
})

const forwardedProps = useForwardProps(delegatedProps)
</script>

<template>
  <SelectTrigger
    :class="cn(
      'gap-1 flex w-full items-center justify-between whitespace-nowrap border border-input-border text-primary rounded-md bg-input px-3 py-1.5 text-sm font-medium shadow-sm ring-offset-background data-[placeholder]:text-muted focus:outline-none focus:ring-1 focus:ring-ring disabled:cursor-not-allowed disabled:opacity-50 [&>span]:truncate text-start',
      props.class,
    )"
    v-bind="forwardedProps"
  >
    <slot/>
    <SelectIcon as-child>
      <Icon
        class="shrink-0"
        name="lucide:chevrons-up-down"
      />
    </SelectIcon>
  </SelectTrigger>
</template>
