<script lang="ts" setup>
import type { ContextMenuSubTriggerProps } from "reka-ui"
import type { HTMLAttributes } from "vue"
import { reactiveOmit } from "@vueuse/core"
import {
  ContextMenuSubTrigger,
  useForwardProps,
} from "reka-ui"
import { cn } from "@/lib/utils"

const props = defineProps<ContextMenuSubTriggerProps & { class?: HTMLAttributes["class"], inset?: boolean }>()

const delegatedProps = reactiveOmit(props, "class")

const forwardedProps = useForwardProps(delegatedProps)
</script>

<template>
  <ContextMenuSubTrigger
    :class="cn(
      'flex cursor-default select-none items-center rounded-sm px-2 py-1.5 text-sm font-medium outline-none focus:bg-accent focus:text-accent-foreground data-[state=open]:bg-accent data-[state=open]:text-accent-foreground',
      inset && 'pl-8',
      props.class,
    )"
    v-bind="forwardedProps"
  >
    <slot />
    <Icon
      class="ml-auto"
      name="lucide:chevron-right"
    />
  </ContextMenuSubTrigger>
</template>
