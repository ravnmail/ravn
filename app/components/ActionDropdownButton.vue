<script lang="ts" setup>
import ActionMenuButton from '@/components/ActionMenuButton.vue'
import type { ButtonViewReturnComponentProps } from '@/types/composer'
import { cn } from '@/lib/utils'
import { Popover, PopoverContent, PopoverTrigger } from '@/components/ui/popover'
import { DropdownMenu, DropdownMenuContent, DropdownMenuTrigger } from '~/components/ui/dropdown-menu'

interface Props {
  icon?: any
  title?: string
  tooltip?: string
  disabled?: boolean
  class?: string
  btn_class?: string
  action?: ButtonViewReturnComponentProps['action']
  isActive?: ButtonViewReturnComponentProps['isActive']
}

const props = withDefaults(defineProps<Props>(), {
  icon: undefined,
  title: undefined,
  tooltip: undefined,
  disabled: false,
  action: undefined,
  isActive: undefined,
  class: '',
  btn_class: '',
})
</script>

<template>
  <DropdownMenu>
    <DropdownMenuTrigger :disabled="disabled">
      <ActionMenuButton
        :class="btn_class"
        :disabled="disabled"
        :icon="icon"
        :is-active="isActive"
        :title="title"
        :tooltip="tooltip"
      />
    </DropdownMenuTrigger>
    <DropdownMenuContent
      :class="cn('p-1 min-w-32 w-full', props.class)"
      align="start"
      side="bottom"
    >
      <slot/>
    </DropdownMenuContent>
  </DropdownMenu>
</template>
