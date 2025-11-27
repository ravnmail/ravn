<script lang="ts" setup>

import type { EmailListItem } from '~/types/email'
import useFormatting from '~/composables/useFormatting'
import EmailLabel from '~/components/ui/EmailLabel.vue'

interface Props extends EmailListItem {
  isSelected?: boolean
}

const { formatEmailDate } = useFormatting()
defineProps<Props>()

</script>

<template>
  <div
    :class="[ is_read ? '' : 'text-primary', isSelected ? 'border-l-accent bg-accent/30 text-accent-foreground' : 'border-l-transparent']"
    class="flex flex-1 w-full items-center gap-3 py-2 px-6 border-l-4"
  >
    <div class="flex items-center font-semibold w-1/5 max-w-3xs truncate">
      <RavnAvatar
        v-if="from.address"
        :account-id="account_id"
        :email="from.address"
        :name="from.name"
        class="mr-4 pointer-events-none"
        size="sm"
      />
      {{ from.name }}
    </div>
    <div
      v-if="!is_read"
      class="w-2 h-2 bg-accent rounded-full"
    />
    <div class="flex-1 line-clamp-1 text-nowrap flex gap-6 items-center">
      <div
        v-if="labels?.length"
        class="flex mt-2 gap-1 flex-wrap"
      >
        <EmailLabel
          v-for="l in labels"
          :key="l.id"
          v-bind="l"
        />
      </div>
      <b class="font-semibold">{{ subject }}</b>
      <span class="opacity-60 text-sm">{{ snippet }}</span>
    </div>
    <div class="flex gap-2 justify-between items-center">
      <div>
        <Icon
          v-if="has_attachments"
          class="shrink-0 opacity-50"
          name="lucide:paperclip"
        />
      </div>
      <div class="opacity-60 text-nowrap">
        {{ formatEmailDate($props, 2) }}
      </div>
    </div>
  </div>
</template>