<script lang="ts" setup>
import type { EmailAddress } from '~/types/email'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuGroup,
  DropdownMenuItem, DropdownMenuLabel, DropdownMenuSeparator,
  DropdownMenuTrigger
} from '~/components/ui/dropdown-menu'
import DropdownMenuItemRich from '~/components/ui/dropdown-menu/DropdownMenuItemRich.vue'
import { slots } from '@vue/language-core/lib/codegen/names'

const { copy } = useClipboard()

interface Props extends EmailAddress {
  isLast?: boolean
  showAvatar?: boolean
  accountId?: string
}

withDefaults(defineProps<Props>(), {
  isLast: false,
  showAvatar: false,
  accountId: undefined
})

</script>

<template>
  <DropdownMenu v-slot="{ open }">
    <DropdownMenuTrigger
      as="button"
      class="relative inline-flex items-center space-x-2 group text-foreground data-[state=open]:bg-selection hover:bg-selection hover:text-selection-foreground data-[state=open]:text-selection-foreground hover:z-10 pl-1 py-0.5 -mx-1 -my-0.5 rounded pr-3"
    >
      <RavnAvatar
        v-if="showAvatar"
        :account-id="accountId"
        :email="address"
        :name="name"
        size="xs"
      />
      <span>{{ name || address }}</span>
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
        <DropdownMenuSeparator/>
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
</template>