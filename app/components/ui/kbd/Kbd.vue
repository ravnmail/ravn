<script lang="ts" setup>
import type { HTMLAttributes } from "vue"
import { cn } from "@/lib/utils"

const props = defineProps<{
  class?: HTMLAttributes["class"]
}>()
</script>

<template>
  <kbd
    :class="cn(
      'bg-secondary font-medium px-1 rounded-sm text-sm text-foreground',
      '[&_svg:not([class*=\'size-\'])]:size-3',
      props.class,
    )"
  >
    <slot />
  </kbd>
</template>
