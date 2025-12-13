<script lang="ts" setup>
import type { ContextMenuRadioItemEmits, ContextMenuRadioItemProps } from 'reka-ui'
import type { HTMLAttributes } from 'vue'
import { reactiveOmit } from '@vueuse/core'
import {
  ContextMenuItemIndicator,
  ContextMenuRadioItem,
  useForwardPropsEmits,
} from 'reka-ui'
import { cn } from '@/lib/utils'

const props = defineProps<ContextMenuRadioItemProps & { class?: HTMLAttributes['class'] }>()
const emits = defineEmits<ContextMenuRadioItemEmits>()

const delegatedProps = reactiveOmit(props, 'class')

const forwarded = useForwardPropsEmits(delegatedProps, emits)
</script>

<template>
  <ContextMenuRadioItem
    :class="cn(
      'relative flex cursor-default select-none items-center rounded-sm py-1.5 pl-8 pr-2 text-sm font-medium outline-none focus:bg-accent focus:text-accent-foreground data-[disabled]:pointer-events-none data-[disabled]:opacity-50',
      props.class,
    )"
    v-bind="forwarded"
  >
    <span class="absolute left-2 flex h-3.5 w-3.5 items-center justify-center">
      <ContextMenuItemIndicator>
        <Icon name="lucide:circle-check"/>
      </ContextMenuItemIndicator>
    </span>
    <slot/>
  </ContextMenuRadioItem>
</template>
