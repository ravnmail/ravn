<script lang="ts" setup>
defineOptions({
  inheritAttrs: false,
})

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

const props = withDefaults(defineProps<Props>(), {
  isLast: false,
  showAvatar: false,
})

const attrs = useAttrs()
const aiNotesOpen = ref(false)
</script>

<template>
  <DropdownMenu v-slot="{ open }">
    <DropdownMenuTrigger
      as="button"
      v-bind="attrs"
      :class="[
        'group relative -mx-1 -my-0.5 inline-flex items-center space-x-2 rounded py-0.5 pr-3 pl-1 text-foreground select-text hover:z-10 hover:bg-selection hover:text-selection-foreground data-[state=open]:bg-selection data-[state=open]:text-selection-foreground',
        attrs.class,
      ]"
    >
      <RavnAvatar
        v-if="showAvatar"
        :email="address"
        :name="name"
        size="xs"
      />
      <p>
        {{ props.name || props.address
        }}<span
          v-if="name"
          class="sr-only"
        >
          &lt;{{ props.address }}&gt;</span
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
          :label="`Compose email to ${props.name || props.address}`"
          icon="lucide:pen-square"
          @click="$emit('compose', props.address)"
        />
        <DropdownMenuItemRich
          :label="`Search for emails from ${props.name || props.address}`"
          icon="lucide:search"
          @click="() => $router.push({ name: 'search', query: { q: `from:${props.address}` } })"
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
          :label="`${props.address}`"
          icon="lucide:copy"
          @click="copy(props.address)"
        />
        <DropdownMenuItemRich
          v-if="props.name"
          :label="`${props.name}`"
          icon="lucide:copy"
          @click="copy(props.name)"
        />
        <DropdownMenuItemRich
          v-if="props.name"
          :label="`${props.name} <${props.address}>`"
          icon="lucide:copy"
          @click="copy(`${props.name} <${props.address}>`)"
        />
      </DropdownMenuGroup>
    </DropdownMenuContent>
  </DropdownMenu>

  <RavnContactAINotesDialog
    v-model:open="aiNotesOpen"
    :email="props.address"
    :name="props.name"
  />
</template>
