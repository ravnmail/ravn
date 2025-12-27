<script lang="ts" setup>
import type { HTMLAttributes } from "vue"
import { cn } from "@/lib/utils"

const props = defineProps<{
  class?: HTMLAttributes["class"]
}>()
</script>

<template>
  <div
    :class="cn('text-lg font-medium', props.class)"
    data-slot="empty-title"
  >
    <slot />
  </div>
</template>
