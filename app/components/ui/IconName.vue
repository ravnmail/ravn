<script lang="ts" setup>

defineProps<{
  icon?: string;
  color?: string;
  name?: string;
}>()

</script>

<template>
  <div class="flex items-center gap-1.5">
    <Icon
      v-if="icon"
      :name="`lucide:${icon}`"
      :size="18"
      :style="{ color }"
    />
    <span class="grow font-medium">{{ name }}</span>
  </div>
</template>