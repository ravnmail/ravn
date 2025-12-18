<script lang="ts" setup>

import IconName from '~/components/ui/IconName.vue'
import { Command, CommandGroup, CommandInput, CommandItem, CommandList } from '~/components/ui/command'

const { accounts } = useAccounts()
const { folders, mapFolderTree, flatten } = useFolders()

const props = defineProps<{
  showHidden?: boolean
}>()

const flattened = computed(() => {
  return flatten(mapFolderTree(folders.value, accounts.value))
})

</script>

<template>
  <Command>
    <CommandInput
      class="py-2 text-sm"
      placeholder="Filter folders..."
    />
    <CommandList>
      <CommandGroup>
        <CommandItem
          v-for="item in flattened"
          :key="item.id"
          :value="item.id"
        >
          <IconName
            :style="{ paddingLeft: `${item.level}rem` }"
            class="font-medium"
            v-bind="item"
          />
        </CommandItem>
      </CommandGroup>
    </CommandList>
  </Command>
</template>