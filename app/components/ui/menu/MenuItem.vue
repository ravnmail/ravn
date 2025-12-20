<script lang="ts" setup>
import { cn } from '@/lib/utils'
import { type DropdownMenuItemProps, useForwardProps } from 'reka-ui'
import { computed, type HTMLAttributes } from 'vue'

const props = defineProps<DropdownMenuItemProps & { class?: HTMLAttributes['class']; inset?: boolean }>()

const delegatedProps = computed(() => {
  const { class: _, ...delegated } = props

  return delegated
})

const forwardedProps = useForwardProps(delegatedProps)
</script>

<template>
  <div
    :class="
      cn(
        'relative flex items-center p-1.5 text-sm outline-none transition-colors focus:bg-selection focus:text-selection-foreground data-[disabled]:pointer-events-none data-[disabled]:opacity-50 hover:bg-selection rounded-sm gap-2',
        inset && 'pl-7',
        props.class
      )
    "
    v-bind="forwardedProps"
  >
    <slot />
  </div>
</template>
