<script lang="ts" setup>

import IconName from '~/components/ui/IconName.vue'
import { Command, CommandGroup, CommandInput, CommandItem, CommandList } from '~/components/ui/command'
import EmailLabel from '~/components/ui/EmailLabel.vue'

const { labels } = useLabels()

defineProps<{
  showHidden?: boolean
  multiple?: boolean
  selectedLabels?: string[]
}>()

defineEmits<{
  (e: 'update:selected-labels', value: string[]): void
}>()

</script>

<template>
  <Command
    :model-value="selectedLabels"
    :multiple="multiple"
    @update:model-value="$emit('update:selected-labels', $event)"
  >
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
          <EmailLabel
            class="font-medium"
            v-bind="item"
          />
          <Icon
            v-if="selectedLabels?.includes(item.id)"
            class="ml-auto"
            name="lucide:check"
          />
        </CommandItem>
      </CommandGroup>
    </CommandList>
  </Command>
</template>