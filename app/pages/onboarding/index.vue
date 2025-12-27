<script lang="ts" setup>

import { Button } from '~/components/ui/button'

definePageMeta({
  layout: 'empty'
})

</script>

<template>
  <div class="flex items-center justify-center h-full">
    <div class="flex flex-col items-center gap-4">
      <img
        alt="Ravn Mail Logo"
        class="w-24 h-24 mb-4"
        src="/AppIcon.svg"
      >
      <h1 class="text-primary">Welcome to Ravn Mail!</h1>
      <Button @click="() => $router.push('/')">to Home</Button>
    </div>
  </div>
</template>