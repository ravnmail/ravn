<script lang="ts" setup>
import { cn } from '@/lib/utils'
import {
  DropdownMenuCheckboxItem,
  type DropdownMenuCheckboxItemEmits,
  type DropdownMenuCheckboxItemProps,
  DropdownMenuItemIndicator,
  useForwardPropsEmits,
} from 'reka-ui'
import { computed, type HTMLAttributes } from 'vue'

const props = defineProps<DropdownMenuCheckboxItemProps & { class?: HTMLAttributes['class'] }>()
const emits = defineEmits<DropdownMenuCheckboxItemEmits>()

const delegatedProps = computed(() => {
  const { class: _, ...delegated } = props

  return delegated
})

const forwarded = useForwardPropsEmits(delegatedProps, emits)
</script>

<template>
  <DropdownMenuCheckboxItem
    :class=" cn(
      'relative flex cursor-default select-none items-center rounded-sm py-1.5 pl-2 pr-8 text-sm font-medium outline-none transition-colors focus:bg-selection focus:text-selection-foreground data-[disabled]:pointer-events-none data-[disabled]:opacity-50',
      props.class,
    )"
    v-bind="forwarded"
  >
    <span class="absolute right-2 flex h-3.5 w-3.5 items-center justify-center">
      <DropdownMenuItemIndicator>
        <Icon name="lucide:check"/>
      </DropdownMenuItemIndicator>
    </span>
    <slot/>
  </DropdownMenuCheckboxItem>
</template>
