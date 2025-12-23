<script lang="ts" setup>

import {
  NumberField,
  NumberFieldContent,
  NumberFieldDecrement,
  NumberFieldIncrement,
  NumberFieldInput,
} from '@/components/ui/number-field'

</script>

<template>
  <NumberField>
    <NumberFieldContent>
      <NumberFieldDecrement />
      <NumberFieldInput class="text-center" />
      <NumberFieldIncrement />
    </NumberFieldContent>
  </NumberField>
</template>