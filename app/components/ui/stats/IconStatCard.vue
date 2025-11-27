<script setup lang="ts">

defineProps<{
  title: string
  icon: string
  description?: string
}>()

</script>

<template>
  <div class="bg-input rounded-xl">
    <div class="gap-y-1.5 p-3 flex flex-row items-center justify-between space-y-0 pb-2">
      <h3 class="tracking-tight text-sm font-medium">{{ title }}</h3>
      <Icon
        :name="icon"
        size="1.25rem"
        class="text-muted"
      />
    </div>
    <div class="p-3 pt-0">
      <div class="text-2xl font-bold text-primary">
        <slot/>
      </div>
      <p
        v-if="description"
        class="text-xs"
      >{{ description }}</p>
    </div>
  </div>
</template>e