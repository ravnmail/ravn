<script lang="ts" setup>

import type { LabelInfo } from '~/types/email'

const props = defineProps<LabelInfo>()

const backgroundColor = computed(() => {
  if (!props.color) return 'transparent'

  return `${props.color}30`
})

</script>

<template>
  <div
    :style="{ backgroundColor, color }"
    class="gap-1.5 inline-flex items-center rounded-md font-semibold text-xs px-2 py-1"
  >
    <Icon
      v-if="icon"
      :name="`lucide:${icon}`"
      :size="14"
    />
    <span>{{ name }}</span>
    <slot/>
  </div>
</template>