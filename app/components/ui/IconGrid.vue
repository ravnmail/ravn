<script lang="ts" setup>
import { DropdownMenu, DropdownMenuContent, DropdownMenuTrigger } from '~/components/ui/dropdown-menu'
import { ListboxContent, ListboxGroup, ListboxGroupLabel, ListboxItem, ListboxRoot } from 'reka-ui'

import iconList from './iconlist.json'

const selectedIcon = defineModel<string | null>()
const isDialogOpen = ref(false)

function selectIcon(icon: string) {
  selectedIcon.value = icon
  isDialogOpen.value = false
}

</script>

<template>
  <DropdownMenu v-model="isDialogOpen">
    <DropdownMenuTrigger
      class="flex items-center justify-between rounded-md border border-input px-2 py-1 font-semibold shadow-sm ring-offset-background data-[placeholder]:text-muted focus:outline-none focus:ring-1 focus:ring-ring hover:bg-input disabled:cursor-not-allowed disabled:opacity-50 [&>span]:truncate text-start gap-2"
    >
      <Icon
        v-if="selectedIcon"
        :name="`lucide:${selectedIcon}`"
      />
      <span
        v-else
        class="h-4 w-4 rounded bg-input"
      />
    </DropdownMenuTrigger>
    <DropdownMenuContent class="sm:max-w-[700px] max-h-80 overflow-y-auto">
      <ListboxRoot
        :model-value="selectedIcon"
        @update:model-value="selectIcon"
      >
        <ListboxContent class="">
          <ListboxGroup
            v-for="(group, i) in iconList"
            :key="i"
          >
            <ListboxGroupLabel
              class="py-1.5 text-xs tracking-widest text-muted uppercase select-none font-semibold"
            >{{ group.title }}
            </ListboxGroupLabel>
            <div class="grid grid-cols-8">
              <ListboxItem
                v-for="option in group.items"
                :key="`${i}-${option}`"
                :title="option"
                :value="option"
                class="p-1.5"
              >
                <Icon
                  :name="`lucide:${option}`"
                />
              </ListboxItem>
            </div>
          </ListboxGroup>
        </ListboxContent>
      </ListboxRoot>
    </DropdownMenuContent>
  </DropdownMenu>
</template>