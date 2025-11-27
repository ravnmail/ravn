<script lang="ts" setup>
import { cn } from '@/lib/utils'
import { DropdownMenuSubTrigger, type DropdownMenuSubTriggerProps, useForwardProps, } from 'reka-ui'
import { computed, type HTMLAttributes } from 'vue'

const props = defineProps<DropdownMenuSubTriggerProps & { class?: HTMLAttributes['class'] }>()

const delegatedProps = computed(() => {
  const { class: _, ...delegated } = props

  return delegated
})

const forwardedProps = useForwardProps(delegatedProps)
</script>

<template>
  <DropdownMenuSubTrigger
    :class="cn(
      'flex cursor-default font-semibold select-none items-center rounded-sm gap-2 px-2 py-1.5 text-sm outline-none focus:bg-accent focus:text-accent-foreground data-[state=open]:bg-accent data-[state=open]:text-accent-foreground',
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
