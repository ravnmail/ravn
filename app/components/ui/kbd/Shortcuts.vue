<script lang="ts" setup>
import { getShortcutKey } from '~/lib/utils/platform'
import { KbdGroup } from './'

const props = defineProps<{
  keys: string
}>()

function parseShortcuts(input: string): string[][] {
  const groups: string[][] = []
  let currentGroup: string[] = []
  let current = ''
  let i = 0

  while (i < input.length) {
    const char = input[i]

    if (char === '\\' && i + 1 < input.length) {
      current += input[i + 1]
      i += 2
    } else if (char === ' ') {
      if (current) currentGroup.push(current)
      current = ''
      if (currentGroup.length > 0) {
        groups.push(currentGroup)
        currentGroup = []
      }
      i++
    } else if (char === '+') {
      if (current) currentGroup.push(current)
      current = ''
      i++
    } else {
      current += char
      i++
    }
  }

  // Add remaining key
  if (current) currentGroup.push(current)
  if (currentGroup.length > 0) groups.push(currentGroup)

  return groups
}

const parsedGroups = computed(() => parseShortcuts(props.keys))
</script>

<template>
  <div class="flex gap-2">
    <KbdGroup
      v-for="(group, i) in parsedGroups"
      :key="i"
      class="flex gap-px"
    >
      <kbd
        v-for="(key, j) in group"
        :key="`${i}-${j}`"
        class="bg-secondary font-medium px-1 rounded-sm text-xs text-foreground"
      >
        {{ getShortcutKey(key) }}
      </kbd>
    </KbdGroup>
  </div>
</template>