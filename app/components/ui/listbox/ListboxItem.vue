<script lang="ts" setup>
import { cn } from '@/lib/utils'
import { ListboxItem, ListboxItemIndicator, type ListboxItemProps, useForwardProps } from 'reka-ui'
import { computed, type HTMLAttributes } from 'vue'

const props = defineProps<ListboxItemProps & { class?: HTMLAttributes['class'], inset?: boolean }>()

const delegatedProps = computed(() => {
  const { class: _, ...delegated } = props

  return delegated
})

const forwardedProps = useForwardProps(delegatedProps)
</script>

<template>
  <ListboxItem
    :class="cn(
      'relative flex items-center rounded-sm py-1.5 pl-2 pr-8 text-sm font-medium outline-none transition-colors data-highlighted:bg-selection focus:bg-selection focus:text-selection-foreground data-[disabled]:pointer-events-none data-[disabled]:opacity-50',
      inset && 'pl-7',
      props.class,
    )"
    v-bind="forwardedProps"
  >
    <span class="absolute right-2 flex h-3.5 w-3.5 items-center justify-center">
      <ListboxItemIndicator>
        <Icon name="lucide:check"/>
      </ListboxItemIndicator>
    </span>
    <slot />
  </ListboxItem>
</template>
