<script lang="ts" setup>
import { cn } from '@/lib/utils'
import { SelectItem, SelectItemIndicator, type SelectItemProps, SelectItemText, useForwardProps, } from 'reka-ui'
import { computed, type HTMLAttributes } from 'vue'

const props = defineProps<SelectItemProps & { class?: HTMLAttributes['class'] }>()

const delegatedProps = computed(() => {
  const { class: _, ...delegated } = props

  return delegated
})

const forwardedProps = useForwardProps(delegatedProps)
</script>

<template>
  <SelectItem
    :class="
      cn(
        'relative flex w-full cursor-default select-none items-center rounded-sm py-1.5 pl-2 pr-8 text-sm outline-none focus:bg-selection focus:text-selection-foreground data-[disabled]:pointer-events-none data-[disabled]:opacity-50',
        props.class,
      )
    "
    v-bind="forwardedProps"
  >
    <span class="absolute right-2 flex size-3.5 items-center justify-center">
      <SelectItemIndicator>
        <Icon name="lucide:check"/>
      </SelectItemIndicator>
    </span>

    <SelectItemText>
      <slot/>
    </SelectItemText>
  </SelectItem>
</template>
