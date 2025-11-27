<script lang="ts" setup>
import type { ButtonViewReturnComponentProps } from '@/type/composer'
import { getShortcutKeys } from '@/lib/utils/platform'
import { Button } from '@/components/ui/button'
import { Tooltip, TooltipContent, TooltipTrigger, TooltipProvider } from '@/components/ui/tooltip'
import { cn } from '@/lib/utils'

interface Props {
  icon?: any
  class?: string
  title?: string
  tooltip?: string
  disabled?: boolean
  shortcutKeys?: string[]
  color?: string
  action?: ButtonViewReturnComponentProps['action']
  isActive?: ButtonViewReturnComponentProps['isActive']
}

const props = withDefaults(defineProps<Props>(), {
  icon: undefined,
  title: undefined,
  tooltip: undefined,
  disabled: false,
  color: undefined,
  shortcutKeys: undefined,
  action: undefined,
  isActive: undefined,
  class: '',
})
</script>

<template>
  <TooltipProvider>
    <Tooltip :delay-duration="0">
      <TooltipTrigger as-child>
        <Button
          :class="cn('h-[32px] !px-1.5 py-0', props.class)"
          :disabled="disabled"
          size="xs"
          variant="ghost"
        >
          <div class="flex items-center h-full justify-between font-normal w-full">
            <div
              v-if="title"
              class="text-left truncate text-sm flex-grow"
            >{{ title }}
            </div>
            <Icon
              v-if="icon"
              :name="icon"
              class="w-[16px] h-[16px]"
            />
            <Icon
              class="w-3 h-3 ml-1 flex-shrink-0"
              name="lucide:chevron-down"
            />
          </div>
        </Button>
      </TooltipTrigger>
      <TooltipContent
        v-if="tooltip || (shortcutKeys && shortcutKeys.length)"
        hide-when-detached
      >
        <div class="max-w-24 text-center flex flex-col items-center">
          <div>{{ tooltip }}</div>
          <div
            v-if="shortcutKeys && shortcutKeys.length"
            class="flex"
          >
            <span>{{ getShortcutKeys(shortcutKeys) }}</span>
          </div>
        </div>
      </TooltipContent>
    </Tooltip>
  </TooltipProvider>
</template>
