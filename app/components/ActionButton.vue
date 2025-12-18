<script lang="ts" setup>
import type { ButtonViewReturnComponentProps } from '@/types/composer'
import { Toggle } from '@/components/ui/toggle'
import { Tooltip, TooltipContent, TooltipTrigger, TooltipProvider } from '@/components/ui/tooltip'
import type { TooltipContentProps } from 'reka-ui'
import type { Editor } from '@tiptap/core'
import type { HTMLAttributes } from 'vue'
import { Kbd, KbdGroup } from '~/components/ui/kbd'
import Shortcuts from '~/components/ui/kbd/Shortcuts.vue'

interface Props {
  icon?: string
  title?: string
  tooltip?: string
  disabled?: boolean
  shortcutKeys?: string[]
  customClass?: string
  loading?: boolean
  tooltipOptions?: TooltipContentProps
  color?: string
  action?: ButtonViewReturnComponentProps['action']
  isActive?: ButtonViewReturnComponentProps['isActive']
  editor?: Editor
  class?: HTMLAttributes['class']
}

const props = withDefaults(defineProps<Props>(), {
  icon: undefined,
  title: undefined,
  tooltip: undefined,
  disabled: false,
  customClass: '',
  color: undefined,
  loading: false,
  shortcutKeys: undefined,
  tooltipOptions: undefined,
  action: undefined,
  isActive: undefined,
  class: undefined,
})

const iconName = computed(() => {
  if (props.icon) {
    return props.icon.startsWith('ravn:') ? props.icon : `lucide:${props.icon}`
  }
  return ''
})
</script>

<template>
  <TooltipProvider>
    <Tooltip :delay-duration="0">
      <TooltipTrigger>
        <Toggle
          :class="[customClass, title ? 'w-auto' : 'w-6']"
          :disabled="disabled"
          :model-value="isActive?.() || false"
          size="sm"
          @click="action"
        >
          <div v-if="loading">
            <Icon
              class="animate-spin"
              name="LoaderCircle"
            />
          </div>
          <div
            v-else
            class="flex gap-1 items-center"
          >
            <Icon
              v-if="icon"
              :name="iconName"
            />
            <slot name="icon"/>
          </div>
          <span v-if="title">{{ title }}
          </span>
          <slot/>
        </Toggle>
      </TooltipTrigger>
      <TooltipContent
        v-if="tooltip"
        v-bind="tooltipOptions"
      >
        <div class="max-w-24 flex flex-col gap-0.5">
          <div>{{ tooltip }}</div>
          <Shortcuts :shortcut-keys="shortcutKeys" />
        </div>
      </TooltipContent>
    </Tooltip>
  </TooltipProvider>
</template>
