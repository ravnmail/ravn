<script lang="ts" setup>
import {
  Command,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
} from '~/components/ui/command'
import IconName from '~/components/ui/IconName.vue'

const { accounts } = useAccounts()
const { folders, mapFolderTree, flattenAccountFolders } = useFolders()

defineProps<{
  showHidden?: boolean
  multiple?: boolean
  selectedFolders?: string[]
}>()

const emit = defineEmits<{
  (e: 'update:selected-folders', value: string[]): void
  (e: 'select', value: string): void
}>()

const handleSelectionChange = (value: string | string[]) => {
  const selectedFolders = Array.isArray(value) ? value : [value]
  emit('update:selected-folders', selectedFolders)

  const selectedFolderId = selectedFolders[0]
  if (selectedFolderId) {
    emit('select', selectedFolderId)
  }
}

const accountFolders = computed(() => {
  return flattenAccountFolders(mapFolderTree(folders.value, accounts.value))
})
</script>

<template>
  <Command
    :model-value="selectedFolders"
    :multiple="multiple"
    @update:model-value="handleSelectionChange"
  >
    <CommandInput
      class="py-2 text-sm"
      placeholder="Filter folders..."
    />
    <CommandList>
      <CommandGroup
        v-for="account in accountFolders"
        :key="account.id"
        :heading="account.title"
      >
        <CommandItem
          v-for="item in account.children"
          :key="item.id"
          :value="item.id"
        >
          <IconName
            :color="item.color"
            :icon="item.icon"
            :name="item.name"
            :style="{ paddingLeft: `${item.level}rem` }"
            class="font-medium"
          />
          <Icon
            v-if="selectedFolders?.includes(item.id)"
            class="ml-auto"
            name="lucide:check"
          />
        </CommandItem>
      </CommandGroup>
    </CommandList>
  </Command>
</template>
