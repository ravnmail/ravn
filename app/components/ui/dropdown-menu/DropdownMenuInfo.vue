<script lang="ts" setup>

</script>

<template>
  <div
    class="relative flex items-center rounded-md gap-2 px-2 py-1.5 text-sm font-medium outline-none transition-colors focus:bg-selection focus:text-selection-foreground data-[disabled]:pointer-events-none data-[disabled]:opacity-50 [&>svg]:size-4 [&>svg]:shrink-0"
  >
    <slot/>
  </div>
</template>