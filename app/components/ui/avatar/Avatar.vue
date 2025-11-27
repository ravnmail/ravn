<script setup lang="ts">

import type { HTMLAttributes } from 'vue'
import { cn } from '@/lib/utils'
import { type AvatarVariants, avatarVariants } from '.'

const props = defineProps<{
  name: string,
  avatar?: string,
  size?: AvatarVariants['size'],
  class?: HTMLAttributes['class']
}>()

const initials = computed(() => {
  const name = props.name
  if (!name) return ''
  const names = name.split(' ')
  if (names.length === 1) return names[0].charAt(0).toUpperCase()
  return names[0].charAt(0).toUpperCase() + names[1].charAt(0).toUpperCase()
})

</script>

<template>
  <div :class="cn(avatarVariants({ size }), props.class)">
    <img
      v-if="props.avatar"
      :src="props.avatar"
      :alt="initials"
      class="w-full h-full object-cover"
    >
    <span
      v-else
      class="text-xs font-bold"
    >{{ initials }}</span>
  </div>
</template>