<script lang="ts" setup>

withDefaults(defineProps<{
  name: string;
  type?: 'i' | 'ib';
  size?: number;
}>(), {
  type: 'i',
  size: undefined
})

</script>

<template>
  <span
    :class="[type, `${type}-${name}`]"
    :style="{ fontSize: size ? size + 'px' : undefined }"
    aria-hidden="true"
  />
</template>