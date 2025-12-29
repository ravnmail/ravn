<script lang="ts" setup>
import { cn } from '@/lib/utils'
import { ContextMenuItem, type ContextMenuItemProps, useForwardProps } from 'reka-ui'
import { computed, type HTMLAttributes } from 'vue'
import type { CleanTranslation } from 'nuxt-i18n-micro-types'
import ContextMenuShortcut from '~/components/ui/context-menu/ContextMenuShortcut.vue'

const props = defineProps<ContextMenuItemProps & {
  class?: HTMLAttributes['class'],
  icon?: string,
  iconColor?: string,
  shortcut?: string[] | string,
  label?: string | CleanTranslation,
  inset?: boolean
}>()

const delegatedProps = computed(() => {
  const { class: _, ...delegated } = props

  return delegated
})

const forwardedProps = useForwardProps(delegatedProps)
</script>

<template>
  <ContextMenuItem
    :class="cn(
      'relative flex items-center rounded-md gap-2 px-2 py-1.5 text-sm font-medium outline-none transition-colors focus:bg-selection focus:text-selection-foreground data-[disabled]:pointer-events-none data-[disabled]:opacity-50 [&>svg]:size-4 [&>svg]:shrink-0',
      inset && 'pl-7',
      props.class,
    )"
    v-bind="forwardedProps"
  >
    <Icon
      v-if="icon"
      :name="icon"
      :size="16"
      :style="{ color: iconColor }"
    />
    <span
      v-if="label"
      class="flex-1"
    >{{ label }}</span>
    <ContextMenuShortcut
      v-if="shortcut"
    >
      {{ Array.isArray(shortcut) ? shortcut.join(", ") : shortcut }}
    </ContextMenuShortcut>
  </ContextMenuItem>
</template>
