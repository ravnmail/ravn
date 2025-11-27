<script lang="ts" setup>
import { Popover, PopoverContent, PopoverTrigger } from '@/components/ui/popover'
import ActionButton from './ActionButton.vue'
import type { ButtonViewReturnComponentProps } from '@/types/composer'
import { cn } from '@/lib/utils'
import { computed } from 'vue'
import { buttonVariants } from '~/components/ui/button'

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
  size: 'sm',
  class: '',
  btn_class: '',
})

const triggerButtonClasses = computed(() => {
  return [
    buttonVariants({ variant: props.variant, size: props.size }),
    'rounded-l-none border-l border-l-surface !px-0'
  ].filter(Boolean)
})

</script>

<template>
  <div
    :class="[isActive?.() && 'bg-muted']"
    class="flex items-center hover:bg-muted rounded-md"
  >
    <ActionButton
      :action="action"
      :class="btn_class"
      :disabled="disabled"
      :icon="icon"
      :shortcut-keys="shortcutKeys"
      :title="title"
      :tooltip="tooltip"
    />
    <Popover>
      <PopoverTrigger
        :class="triggerButtonClasses"
        :disabled="disabled"
      >
        <Icon
          class="w-3 h-3 ml-1 flex-shrink-0"
          name="lucide:chevron-down"
        />
      </PopoverTrigger>
      <PopoverContent
        :class="cn('min-w-32 p-1 w-full', props.class)"
        align="start"
        side="bottom"
        v-bind="$attrs"
      >
        <slot/>
      </PopoverContent>
    </Popover>
  </div>
</template>
