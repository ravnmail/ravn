<script lang="ts" setup>
import ActionButton from './ActionButton.vue'
import type { ButtonViewReturnComponentProps } from '@/types/composer'
import { cn } from '@/lib/utils'
import { computed } from 'vue'
import { buttonVariants } from '~/components/ui/button'
import { DropdownMenu, DropdownMenuContent, DropdownMenuTrigger } from '~/components/ui/dropdown-menu'

interface Props extends ButtonViewReturnComponentProps {
  icon?: string
  title?: string
  tooltip?: string
  disabled?: boolean
  class?: string
  shortcutKeys?: string[]
  btn_class?: string
  variant?: ButtonViewReturnComponentProps['variant']
  size?: ButtonViewReturnComponentProps['size']
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
  shortcutKeys: undefined,
  variant: 'ghost',
  size: 'bar',
  class: '',
  btn_class: 'rounded-r-none',
})

const triggerButtonClasses = computed(() => {
  return [
    buttonVariants({ variant: props.variant, size: props.size }),
    'rounded-l-none !px-0'
  ].filter(Boolean)
})

</script>

<template>
  <DropdownMenu
    v-slot="{ open }"
    as-child
  >
    <ActionButton
      :action="action"
      :custom-class="btn_class"
      :disabled="disabled"
      :icon="icon"
      :shortcut-keys="shortcutKeys"
      :title="title"
      :tooltip="tooltip"
    />
    <DropdownMenuTrigger
      :class="triggerButtonClasses"
      :disabled="disabled"
    >
      <Icon
        class="w-3 h-3 ml-1 flex-shrink-0"
        name="lucide:chevron-down"
      />
    </DropdownMenuTrigger>
    <DropdownMenuContent
      :align-offset="-32"
      :class="cn('min-w-32 p-1 w-full', props.class)"
      align="start"
      side="bottom"
      v-bind="$attrs"
    >
      <slot/>
    </DropdownMenuContent>
  </DropdownMenu>
</template>
