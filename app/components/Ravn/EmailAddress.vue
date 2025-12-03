<script lang="ts" setup>
import type { EmailAddress } from '~/types/email'
import { DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuTrigger } from '~/components/ui/dropdown-menu'

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
  <DropdownMenu>
    <DropdownMenuTrigger
      class="relative inline-flex items-center space-x-2 group text-foreground hover:bg-selection hover:text-selection-foreground hover:z-10 pl-1 py-0.5 -mx-1 -my-0.5 rounded pr-5">
      <RavnAvatar
        v-if="showAvatar"
        :account-id="accountId"
        :email="address"
        :name="name"
        size="xs"
      />
      <span>{{ name || address }}</span>
      <Icon
        class="absolute right-1 opacity-0 group-hover:opacity-100"
        name="lucide:chevron-down"
      />
    </DropdownMenuTrigger>
    <DropdownMenuContent>
      <DropdownMenuItem @click="copy(address)">
        <span>{{ address }}</span>
      </DropdownMenuItem>
    </DropdownMenuContent>
  </DropdownMenu>
</template>