<script lang="ts" setup>

import { getShortcutKey } from '~/lib/utils/platform'
import { Kbd, KbdGroup } from './'

defineProps<{
  shortcutKeys: string[]
}>()

</script>

<template>
  <KbdGroup>
    <Kbd
      v-for="(shortcutKey, index) in shortcutKeys"
      :key="index"
    >
      {{ getShortcutKey(shortcutKey) }}
    </Kbd>
  </KbdGroup>
</template>