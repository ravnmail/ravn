<script lang="ts" setup>

defineProps<{
  icon?: string;
  color?: string;
  name?: string;
}>()

</script>

<template>
  <div class="flex items-center gap-1">
    <Icon
      v-if="icon"
      :name="`lucide:${icon}`"
      :style="{ color }"
      class="mr-1"
    />
    <span>{{ name }}</span>
  </div>
</template>