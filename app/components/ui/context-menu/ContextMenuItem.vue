<script lang="ts" setup>
import type { ContextMenuItemEmits, ContextMenuItemProps } from "reka-ui"
import type { HTMLAttributes } from "vue"
import { reactiveOmit } from "@vueuse/core"
import {
  ContextMenuItem,
  useForwardPropsEmits,
} from "reka-ui"
import { cn } from "@/lib/utils"

const props = defineProps<ContextMenuItemProps & { class?: HTMLAttributes["class"], inset?: boolean }>()
const emits = defineEmits<ContextMenuItemEmits>()

const delegatedProps = reactiveOmit(props, "class")

const forwarded = useForwardPropsEmits(delegatedProps, emits)
</script>

<template>
  <ContextMenuItem
    :class="cn(
      'relative flex items-center rounded-sm gap-2 px-2 py-1.5 text-sm font-medium outline-none transition-colors focus:bg-selection focus:text-selection-foreground data-[disabled]:pointer-events-none data-[disabled]:opacity-50 [&>svg]:size-4 [&>svg]:shrink-0',
      inset && 'pl-7',
      props.class,
    )"
    v-bind="forwarded"
  >
    <slot />
  </ContextMenuItem>
</template>
