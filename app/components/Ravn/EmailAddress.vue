<script lang="ts" setup>
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuGroup,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '~/components/ui/dropdown-menu'
import DropdownMenuItemRich from '~/components/ui/dropdown-menu/DropdownMenuItemRich.vue'
import type { EmailAddress } from '~/types/email'

const { copy } = useClipboard()

interface Props extends EmailAddress {
  isLast?: boolean
  showAvatar?: boolean
}

withDefaults(defineProps<Props>(), {
  isLast: false,
  showAvatar: false,
})

const aiNotesOpen = ref(false)
</script>

<template>
  <DropdownMenu v-slot="{ open }">
    <DropdownMenuTrigger
      as="button"
      class="group relative -mx-1 -my-0.5 inline-flex items-center space-x-2 rounded py-0.5 pr-3 pl-1 text-foreground select-text hover:z-10 hover:bg-selection hover:text-selection-foreground data-[state=open]:bg-selection data-[state=open]:text-selection-foreground"
    >
      <RavnAvatar
        v-if="showAvatar"
        :email="address"
        :name="name"
        size="xs"
      />
      <p>
        {{ name || address
        }}<span
          v-if="name"
          class="sr-only"
        >
          &lt;{{ address }}&gt;</span
        >
      </p>
      <Icon
        :class="['absolute right-1 opacity-0 group-hover:opacity-100', open ? 'opacity-100' : '']"
        name="lucide:chevron-down"
      />
    </DropdownMenuTrigger>
    <DropdownMenuContent align="start">
      <DropdownMenuGroup>
        <DropdownMenuItemRich
          :label="`Compose email to ${name || address}`"
          icon="lucide:pen-square"
          @click="$emit('compose', address)"
        />
        <DropdownMenuItemRich
          :label="`Search for emails from ${name || address}`"
          icon="lucide:search"
          @click="() => $router.push({ name: 'search', query: { q: `from:${address}` } })"
        />
        <DropdownMenuSeparator />
        <DropdownMenuItemRich
          label="Edit AI Notes"
          icon="ravn:raven"
          class="text-ai"
          @click="aiNotesOpen = true"
        />
        <DropdownMenuSeparator />
        <DropdownMenuLabel>Copy</DropdownMenuLabel>
        <DropdownMenuItemRich
          :label="`${address}`"
          icon="lucide:copy"
          @click="copy(address)"
        />
        <DropdownMenuItemRich
          v-if="name"
          :label="`${name}`"
          icon="lucide:copy"
          @click="copy(name)"
        />
        <DropdownMenuItemRich
          v-if="name"
          :label="`${name} <${address}>`"
          icon="lucide:copy"
          @click="copy(`${name} <${address}>`)"
        />
      </DropdownMenuGroup>
    </DropdownMenuContent>
  </DropdownMenu>

  <RavnContactAINotesDialog
    v-model:open="aiNotesOpen"
    :email="address"
    :name="name"
  />
</template>
