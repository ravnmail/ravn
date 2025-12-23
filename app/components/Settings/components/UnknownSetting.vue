<script lang="ts" setup>
const props = defineProps<{
  modelValue?: any
}>()

const emit = defineEmits<{
  'update:modelValue': [value: any]
}>()

// Display the raw value as JSON
const displayValue = computed(() => {
  try {
    return JSON.stringify(props.modelValue, null, 2)
  }
  catch {
    return String(props.modelValue)
  }
})
</script>

<template>
  <div class="rounded-md border border-destructive/50 bg-destructive/10 p-4">
    <div class="flex items-start gap-3">
      <Icon
        name="lucide:alert-triangle"
        class="size-5 text-destructive"
      />
      <div class="flex-1 space-y-2">
        <p class="text-sm font-medium text-destructive">
          Unknown Setting Component
        </p>
        <p class="text-xs text-muted-foreground">
          This setting type is not yet implemented. Please implement the corresponding component.
        </p>
        <div class="mt-2 rounded bg-muted p-2">
          <pre class="text-xs">{{ displayValue }}</pre>
        </div>
      </div>
    </div>
  </div>
</template>
