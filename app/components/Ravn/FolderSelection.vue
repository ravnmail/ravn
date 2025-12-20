<script lang="ts" setup>

import {
  ListboxRoot,
  ListboxFilter,
  PopoverAnchor,
  useFilter,
  ListboxContent,
  ListboxGroup
} from 'reka-ui'
import { ListboxItem, ListboxGroupLabel } from '~/components/ui/listbox'
import { Popover, PopoverContent, PopoverTrigger } from '~/components/ui/popover'
import { Button } from '~/components/ui/button'
import { TagsInput, TagsInputInput, TagsInputItem, TagsInputItemDelete } from '~/components/ui/tags-input'
import IconName from '~/components/ui/IconName.vue'

const { t } = useI18n()
const { accounts } = useAccounts()
const { folders, flattenAccountFolders, mapFolderTree } = useFolders()

const props = withDefaults(defineProps<{
  modelValue?: string[] | undefined
}>(), {
  modelValue: () => [],
})

const emit = defineEmits<{
  (e: 'update:modelValue', value: string[]): void
}>()

const selectedFolders = computed(() => {
  return props.modelValue.map(id => folders.value?.find(f => f.id === id))
})

const searchTerm = ref('')
const open = ref(false)
const { contains } = useFilter({ sensitivity: 'base' })

const filteredFolders = computed(() => {
  const accountFolders = flattenAccountFolders(mapFolderTree(folders.value, accounts.value))
  if (!searchTerm.value) {
    return accountFolders
  }

  return accountFolders.map(account => {
    return {
      ...account,
      children: account.children?.filter(({ name }) => contains(name, searchTerm.value)) || []
    }
  }).filter(account => account.children.length > 0)
})

watch(searchTerm, (f) => {
  if (f) {
    open.value = true
  }
})
</script>

<template>
  <Popover v-model:open="open">
    <ListboxRoot
      :model-value="modelValue"
      highlight-on-hover
      multiple
      @update:model-value="emit('update:modelValue', $event)"
    >
      <label class="text-sm font-medium">{{ t('components.viewWizard.customize.includeFolders') }}</label>
      <PopoverAnchor class="w-full">
        <TagsInput
          :model-value="modelValue"
          class="pr-0"
          @update:model-value="emit('update:modelValue', $event)"
        >
          <TagsInputItem
            v-for="folder in selectedFolders"
            :key="folder.id"
            :value="folder.id"
            class="pl-1 py-0.5 flex items-center gap-1"
          >
            <IconName
              :color="folder.color"
              :icon="folder.icon"
              :name="folder.name"
            />
            <TagsInputItemDelete/>
          </TagsInputItem>
          <ListboxFilter
            v-model="searchTerm"
            as-child
          >
            <TagsInputInput
              :placeholder="t('components.viewWizard.customize.filterFolders')"
              @keydown.enter.prevent
              @keydown.down="open = true"
            />
          </ListboxFilter>
          <PopoverTrigger as-child>
            <Button variant="ghost">
              <Icon name="lucide:chevron-down"/>
            </Button>
          </PopoverTrigger>
        </TagsInput>
      </PopoverAnchor>
      <PopoverContent
        @open-auto-focus.prevent
      >
        <ListboxContent
          class="max-h-75 scroll-py-1 overflow-x-hidden overflow-y-auto empty:after:content-['No_options'] empty:p-1 empty:after:text-sm empty:after:block"
          tabindex="0"
        >
          <ListboxGroup
            v-for="account in filteredFolders"
            :key="account.id"
          >
            <ListboxGroupLabel>{{ account.title }}</ListboxGroupLabel>
            <ListboxItem
              v-for="folder in account.children"
              :key="folder.id"
              :style="{ paddingLeft: `${folder.level + 0.5}rem` }"
              :value="folder.id"
              @select="searchTerm = ''"
            >
              <IconName
                :color="folder.color"
                :icon="folder.icon"
                :name="folder.name"
              />
            </ListboxItem>
          </ListboxGroup>
        </ListboxContent>
      </PopoverContent>
    </ListboxRoot>
  </Popover>
</template>