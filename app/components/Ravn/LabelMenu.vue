<script lang="ts" setup>

import IconName from '~/components/ui/IconName.vue'
import { Command, CommandGroup, CommandInput, CommandItem, CommandList } from '~/components/ui/command'

const { labels } = useLabels()

defineProps<{
  showHidden?: boolean
}>()

</script>

<template>
  <Command>
    <CommandInput
      class="py-2 text-sm"
      placeholder="Filter labels..."
    />
    <CommandList>
      <CommandGroup>
        <CommandItem
          v-for="item in labels"
          :key="item.id"
          :value="item.id"
        >
          <IconName
            class="font-medium"
            v-bind="item"
          />
        </CommandItem>
      </CommandGroup>
    </CommandList>
  </Command>
</template>