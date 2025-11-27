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
    class="gap-2 inline-flex items-center rounded-md font-semibold text-xs px-2 py-1"
  >
    <Icon
      v-if="icon"
      :name="`lucide:${icon}`"
    />
    <span>{{ name }}</span>
  </div>
</template>