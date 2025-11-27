<script lang="ts" setup>

import IconName from '~/components/ui/IconName.vue'
import { Command, CommandGroup, CommandInput, CommandItem, CommandList } from '~/components/ui/command'

const { useNavigationFolders, flatten } = useFolders()

const props = defineProps<{
  accountId: string
  showHidden?: boolean
}>()

const data = useNavigationFolders(props.accountId)

const flattened = computed(() => {
  return flatten(data.value)
})

</script>

<template>
  <Command>
    <CommandInput
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
            v-bind="item"
          />
        </CommandItem>
      </CommandGroup>
    </CommandList>
  </Command>
</template>