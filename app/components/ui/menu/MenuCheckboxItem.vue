<script lang="ts" setup>
import { cn } from '@/lib/utils'
import type { HTMLAttributes } from 'vue'

interface MenuCheckboxItemProps {
  modelValue?: boolean
}

const props = defineProps<MenuCheckboxItemProps & { class?: HTMLAttributes['class'] }>()

const emit = defineEmits<{
  select: []
}>()
</script>

<template>
  <div
    :class="
      cn(
        'relative flex cursor-default select-none items-center rounded-sm py-1.5 pr-8 pl-1.5 text-sm outline-none transition-colors focus:bg-accent focus:text-accent-foreground data-[disabled]:pointer-events-none data-[disabled]:opacity-50 hover:bg-accent',
        props.class
      )
    "
    @click="emit('select')"
  >
    <slot/>
    <span class="absolute right-2 flex h-3.5 w-3.5 items-center justify-center">
      <span v-if="modelValue">
        <Icon
          class="h-4 w-4"
          name="lucide:check"
        />
      </span>
    </span>
  </div>
</template>
