<script lang="ts" setup>
import {
  Command,
  CommandDialog,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList, CommandShortcut,
} from '~/components/ui/command'
import IconName from '~/components/ui/IconName.vue'
import { Button } from '~/components/ui/button'
import { Kbd } from '~/components/ui/kbd'
import Shortcuts from '~/components/ui/kbd/Shortcuts.vue'

const router = useRouter()
const isOpen = ref(false)

const { register, unregister, executeAction, possibleActions } = useActions()
const { accounts } = useAccounts()
const { folders, mapFolderTree, flattenAccountFolders } = useFolders()
const { views } = useViews()
const highlightedAction = ref<string | null>(null)

const accountFolders = computed(() => flattenAccountFolders(mapFolderTree(folders.value, accounts.value)))

onMounted(() => {
  register({
    id: 'openCommandPalette',
    namespace: 'global',
    handler: () => {
      isOpen.value = true
    }
  })
  register({
    id: 'closeCommandPalette',
    namespace: 'global',
    handler: () => {
      isOpen.value = false
    }
  })
  register({
    id: 'toggleCommandPalette',
    namespace: 'global',
    handler: () => {
      isOpen.value = !isOpen.value
    }
  })
})

onUnmounted(() => {
  unregister('global', 'openCommandPalette')
  unregister('global', 'closeCommandPalette')
  unregister('global', 'toggleCommandPalette')
})


const handleSelect = (value: string) => {
  if (value.startsWith('ravn://')) {
    router.push(value.replace('ravn://', '/'))
  } else {
    executeAction(value)
  }
  isOpen.value = false
}

</script>

<template>
  <CommandDialog
    :open="isOpen"
    @update:open="isOpen = $event"
  >
    <Command
      @highlight="({ value }) => value && (highlightedAction = value as string)"
      @update:model-value="(v) => handleSelect(v as string)"
    >
      <CommandInput placeholder="Type a command or search..."/>
      <CommandList>
        <CommandGroup>
          <CommandItem
            v-for="item in possibleActions"
            :key="item.key"
            :value="item.key"
          >
            <span>{{ item.name }}</span>
            <Shortcuts
              v-if="item.shortcut"
              :keys="item.shortcut"
              class="ml-auto text-xs"
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
        <CommandGroup
          v-for="account in accountFolders"
          :key="account.id"
          :heading="account.title"
        >
          <CommandItem
            v-for="item in account.children"
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
    <div class="px-2 py-1 gap-2 flex items-center justify-end border-t border-popover-border">
      <Button
        size="xs"
        variant="ghost"

      >
        Change Keybinding...
        <Shortcuts keys="Meta + Enter"/>
      </Button>
      <Button
        size="xs"
        variant="ghost"
        @click="handleSelect(highlightedAction!)"
      >Run
        <Shortcuts keys="Enter"/>
      </Button>
    </div>
  </CommandDialog>
</template>