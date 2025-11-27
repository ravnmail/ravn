<script lang="ts" setup>
import { useMagicKeys, whenever } from '@vueuse/core'
import {
  Command,
  CommandDialog,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
  CommandSeparator
} from '~/components/ui/command'
import IconName from '~/components/ui/IconName.vue'

const router = useRouter()
const isOpen = ref(false)
const keys = useMagicKeys()
const shortcut = keys['Meta+P']

const { accounts } = useAccounts()
const { useNavigationFolders, flatten } = useFolders()
const { views } = useViews()

whenever(shortcut, () => {
  isOpen.value = !isOpen.value
})

const menu = computed(() => [
  {
    id: 'search',
    name: 'Search',
    icon: 'search',
    href: '/search'
  },
  {
    id: 'settings',
    name: 'Settings',
    icon: 'settings',
    href: '/settings'
  }
])

const handleSelect = (value: string) => {
  if (value.startsWith('ravn://')) {
    router.push(value.replace('ravn://', '/'))
  }
  isOpen.value = false
}

</script>

<template>
  <CommandDialog
    :open="isOpen"
    @update:open="isOpen = $event"
  >
    <Command @update:model-value="(v) => handleSelect(v as string)">
      <CommandInput placeholder="Type a command or search..."/>
      <CommandList>
        <CommandGroup
          heading="General"
        >
          <CommandItem
            v-for="item in menu"
            :key="item.id"
            :value="`ravn://${item.id}`"
          >
            <IconName
              :icon="item.icon"
              :name="item.name"
            />
          </CommandItem>
        </CommandGroup>
        <CommandGroup
          v-if="views.length"
          heading="Views"
        >
          <CommandItem
            v-for="view in views"
            :key="view.id"
            :value="`ravn://views/${view.id}`"
          >
            <IconName
              v-bind="view"
            />
          </CommandItem>
        </CommandGroup>
        <CommandSeparator v-if="views.length"/>
        <CommandGroup
          v-for="account in accounts"
          :key="account.id"
          :heading="account.name"
        >
          <CommandItem
            v-for="item in flatten(useNavigationFolders(account.id).value)"
            :key="item.id"
            :value="`ravn://mail/${account.id}/folders/${item.id}`"
          >
            <IconName
              :color="item.color"
              :icon="item.icon"
              :name="item.name"
              :style="{ paddingLeft: `${item.level}rem` }"
            />
          </CommandItem>
        </CommandGroup>
      </CommandList>
    </Command>
  </CommandDialog>
</template>