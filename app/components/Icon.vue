<script lang="ts" setup>

defineProps<{
  name: string;
  size?: number;
}>()

</script>

<template>
  <span
    :class="['i', `i-${name}`]"
    :style="{ fontSize: size ? size + 'px' : undefined }"
    aria-hidden="true"
  />
</template>