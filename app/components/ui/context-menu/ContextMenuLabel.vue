<script lang="ts" setup>
import type { ContextMenuLabelProps } from "reka-ui"
import type { HTMLAttributes } from "vue"
import { reactiveOmit } from "@vueuse/core"
import { ContextMenuLabel } from "reka-ui"
import { cn } from "@/lib/utils"

const props = defineProps<ContextMenuLabelProps & { class?: HTMLAttributes["class"], inset?: boolean }>()

const delegatedProps = reactiveOmit(props, "class")
</script>

<template>
  <ContextMenuLabel
    :class="
      cn('px-2 py-1.5 text-sm font-semibold text-muted',
         inset && 'pl-8', props.class,
      )"
    v-bind="delegatedProps"
  >
    <slot />
  </ContextMenuLabel>
</template>
