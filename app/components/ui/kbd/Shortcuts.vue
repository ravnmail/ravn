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
      class="bg-secondary font-medium px-1 rounded-sm text-sm text-foreground"
    >
      {{ getShortcutKey(shortcutKey) }}
    </kbd>
  </KbdGroup>
</template>