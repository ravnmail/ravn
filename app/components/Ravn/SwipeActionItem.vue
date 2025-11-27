<script setup lang="ts">
import type { SwipeAction } from '~/composables/useSwipeActions'

const props = defineProps<{
  action: SwipeAction
  side: 'left' | 'right'
  isActive: boolean
}>()

const emit = defineEmits<{
  (e: 'click', action: SwipeAction): void
}>()

const handleClick = () => {
  emit('click', props.action)
}
</script>

<template>
  <div
    class="flex flex-col items-center justify-center h-full w-20 text-white transition-colors duration-200"
    :class="[
      action.color,
      isActive ? 'bg-opacity-100' : 'bg-opacity-90',
    ]"
    @click="handleClick"
  >
    <Icon
      :name="action.icon"
      class="text-xl mb-2"
    />
    <span class="text-xs font-semibold">{{ $t(action.label) }}</span>
  </div>
</template>