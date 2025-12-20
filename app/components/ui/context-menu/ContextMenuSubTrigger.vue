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
      'flex cursor-default items-center gap-2 rounded-sm px-2 py-1.5 text-sm font-medium outline-none focus:bg-selection focus:text-selection-foreground data-[state=open]:bg-selection data-[state=open]:text-selection-foreground',
      inset && 'pl-7',
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
