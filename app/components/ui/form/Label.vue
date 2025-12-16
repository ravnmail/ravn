<script lang="ts" setup>

import type { CleanTranslation } from 'nuxt-i18n-micro-types'

defineProps<{
  label: CleanTranslation | string
  hideLabel?: boolean
  required?: boolean
}>()

</script>

<template>
  <label
    :class="['text-sm font-semibold text-primary', hideLabel && 'sr-only']"
  >
    <slot>
      {{ label }}
    </slot>
    <span
      v-if="required"
      class="text-destructive ml-1"
    >*</span>
  </label>
</template>