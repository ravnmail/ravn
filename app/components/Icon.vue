<script lang="ts" setup>

withDefaults(defineProps<{
  name: string;
  mode?: 'i' | 'ib';
  size?: number;
}>(), {
  mode: 'i',
  size: undefined
})

</script>

<template>
  <span
    :class="[mode, `${mode}-${name}`]"
    :style="{ fontSize: size ? size + 'px' : undefined }"
    aria-hidden="true"
  />
</template>