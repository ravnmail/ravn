<script lang="ts" setup>
import { cn } from '@/lib/utils'
import { DropdownMenuSubTrigger, type DropdownMenuSubTriggerProps, useForwardProps, } from 'reka-ui'
import { computed, type HTMLAttributes } from 'vue'

const props = defineProps<DropdownMenuSubTriggerProps & { class?: HTMLAttributes['class'], inset?: boolean }>()

const delegatedProps = computed(() => {
  const { class: _, ...delegated } = props

  return delegated
})

const forwardedProps = useForwardProps(delegatedProps)
</script>

<template>
  <DropdownMenuSubTrigger
    :class="cn(
      'flex cursor-default items-center rounded-sm gap-2 px-2 py-1.5 text-sm font-medium outline-none focus:bg-selection focus:text-selection-foreground data-[state=open]:bg-selection data-[state=open]:text-selection-foreground',
      inset && 'pl-8',
      props.class,
    )"
    v-bind="forwardedProps"
  >
    <slot/>
    <Icon
      class="ml-auto"
      name="lucide:chevron-right"
    />
  </DropdownMenuSubTrigger>
</template>
