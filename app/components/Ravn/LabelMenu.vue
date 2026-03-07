<script lang="ts" setup>
import { computed } from 'vue'

import {
  Command,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
} from '~/components/ui/command'
import EmailLabel from '~/components/ui/EmailLabel.vue'
import type { EmailListItem } from '~/types/email'

const { labels } = useLabels()

const props = withDefaults(
  defineProps<{
    email?: EmailListItem | null
    selectedLabels?: string[]
  }>(),
  {
    email: null,
    selectedLabels: undefined,
  }
)

const emit = defineEmits<{
  (e: 'toggle', value: { labelId: string; selected: boolean }): void
}>()

const selectedLabelIds = computed(() => {
  if (props.selectedLabels) {
    return [...props.selectedLabels]
  }

  return props.email?.labels?.map((label) => label.id) ?? []
})

const isSelected = (labelId: string) => {
  return selectedLabelIds.value.includes(labelId)
}

const handleSelect = (labelId: string) => {
  emit('toggle', {
    labelId,
    selected: !isSelected(labelId),
  })
}
</script>

<template>
  <Command :model-value="selectedLabelIds">
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
          @select.prevent="handleSelect(item.id)"
        >
          <EmailLabel
            class="font-medium"
            v-bind="item"
          />
          <Icon
            v-if="isSelected(item.id)"
            class="ml-auto"
            name="lucide:check"
          />
        </CommandItem>
      </CommandGroup>
    </CommandList>
  </Command>
</template>
