<script lang="ts" setup>

import { getShortcutKey } from '~/lib/utils/platform'
import { KbdGroup } from './'

defineProps<{
  shortcutKeys: string[]
}>()

</script>

<template>
  <KbdGroup>
    <kbd
      v-for="(shortcutKey, index) in shortcutKeys"
      :key="index"
      class="text-muted opacity-80 border-border border px-1 rounded-sm"
    >
      {{ getShortcutKey(shortcutKey) }}
    </kbd>
  </KbdGroup>
</template>